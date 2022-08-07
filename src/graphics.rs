pub mod gpu_memory;
mod oam_search;
mod picture_generation;
mod ppu;

use self::gpu_memory::*;
use super::io::Io;
use crate::cpu::CPU_PERIOD_NANOS;
use ppu::PpuState;
use ppu::PpuState::{HBlank, OamSearch, PictureGeneration, VBlank};
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::video::Window;
use sdl2::video::WindowContext;
use sdl2::VideoSubsystem;
use std::time::{Duration, Instant};

pub const SCALE: u32 = 3;
pub const NUM_PIXELS_X: u32 = 160;
pub const NUM_PIXELS_Y: u32 = 144;
pub const TOTAL_PIXELS: usize = (NUM_PIXELS_X * NUM_PIXELS_Y) as usize;

pub const SCREEN_WIDTH: u32 = NUM_PIXELS_X * SCALE; // Only used by the window and rect (not in the texture)
pub const SCREEN_HEIGHT: u32 = NUM_PIXELS_Y * SCALE; // Only used by the window and rect (not in the texture)
pub const BYTES_PER_ROW: usize = BYTES_PER_PIXEL * (NUM_PIXELS_X as usize); // :(

pub const NUM_PIXEL_BYTES: usize = TOTAL_PIXELS * BYTES_PER_PIXEL;

pub const BYTES_PER_TILE: usize = 16;
pub const BYTES_PER_TILE_SIGNED: isize = 16;
pub const DMA_SRC_MUL: u16 = 0x0100;

pub struct Graphics {
    state: PpuState,
    gpu_data: GpuMemory,
    frame_ready: bool,
    cycles: usize,
    prev_frame_time: Instant,
}

impl Graphics {
    pub fn new() -> Graphics {
        let mut gpu_mem = GpuMemory::new();
        Graphics {
            state: ppu::init(&mut gpu_mem),
            gpu_data: GpuMemory::new(),
            frame_ready: false,
            cycles: 0,
            prev_frame_time: Instant::now(),
        }
    }

    // When ppu is not enabled we should be in hblank so these read/writes should always work
    pub fn read_byte(self: &Self, addr: u16) -> u8 {
        return match &self.state {
            OamSearch(os) => os.read_byte(&self.gpu_data, addr),
            PictureGeneration(pg) => pg.read_byte(&self.gpu_data, addr),
            HBlank(hb) => hb.read_byte(&self.gpu_data, addr),
            VBlank(vb) => vb.read_byte(&self.gpu_data, addr),
            PpuState::None => panic!("Ppu state should never be None"),
        };
    }

    pub fn write_byte(self: &mut Self, addr: u16, data: u8) {
        match &mut self.state {
            OamSearch(os) => os.write_byte(&mut self.gpu_data, addr, data),
            PictureGeneration(pg) => pg.write_byte(&mut self.gpu_data, addr, data),
            HBlank(hb) => hb.write_byte(&mut self.gpu_data, addr, data),
            VBlank(vb) => vb.write_byte(&mut self.gpu_data, addr, data),
            PpuState::None => panic!("Ppu state should never be None"),
        }
    }

    // I dont think anything stops dma from reading memory ranges above 0xDF9F so...
    pub fn read_byte_for_dma(self: &Self, addr: u16) -> u8 {
        return match addr {
            VRAM_START..=VRAM_END => self.gpu_data.vram[usize::from(addr - VRAM_START)],
            OAM_START..=OAM_END => self.gpu_data.vram[usize::from(addr - OAM_START)],
            0xFEA0..=0xFEFF => 0x00,
            _ => panic!("DMA shouldnt not read from address: {:04X}", addr),
        };
    }

    // addr should be from 0 - 159 inclusive
    pub fn write_byte_for_dma(self: &mut Self, addr: u16, data: u8) {
        self.gpu_data.oam[usize::from(addr)] = data;
    }

    pub fn read_io_byte(self: &Self, addr: u16) -> u8 {
        self.gpu_data.read_ppu_io(addr)
    }

    pub fn write_io_byte(self: &mut Self, addr: u16, data: u8) {
        match addr {
            LCDC_REG => {
                self.gpu_data.write_ppu_io(addr, data);
                if !self.gpu_data.is_ppu_enabled() {
                    self.disable_ppu();
                } // Should I do anything when re-enabling?
            }
            STAT_REG => {
                // For 1 cycle write 0xFF and whatever resulting interrupts
                if self.stat_quirk(data) {
                    self.gpu_data.write_ppu_io(addr, 0xFF);
                }
            }
            _ => self.gpu_data.write_ppu_io(addr, data),
        }
    }

    pub fn adv_cycles(self: &mut Self, io: &mut Io, cycles: usize) {
        if !self.gpu_data.is_ppu_enabled() {
            return;
        }

        self.cycles += cycles;

        let state = std::mem::replace(&mut self.state, PpuState::None);

        self.state = match state {
            OamSearch(os) => os.render(&mut self.gpu_data, cycles),
            PictureGeneration(pg) => pg.render(&mut self.gpu_data, cycles),
            HBlank(hb) => hb.render(&mut self.gpu_data, cycles),
            VBlank(vb) => vb.render(&mut self.gpu_data, cycles),
            PpuState::None => panic!("Ppu state should never be None"),
        };

        // Need to do it this way since no direct access to ifired from gpu_memory.rs
        if self.gpu_data.stat_low_to_high {
            io.request_stat_interrupt();
            self.gpu_data.stat_low_to_high = false;
        }

        // Its okay to check after every cycle since vblank_int is only set on the transition
        // to mode 1 and never afterwards. Thus its not possible that we accidently trigger two
        // vblank interrupts for a single vblank period as set_stat_mode(1) is never called again
        if self.gpu_data.vblank_int {
            io.request_vblank_interrupt();
            self.gpu_data.vblank_int = false;
            self.frame_ready = true;
        }

        // If we have some value in the option, then we had tried to write to stat
        // We should have set the delay to true when setting the option. (stat_quirk function)
        // If the delay is there, this is the adv_cycles call right after writing to STAT_REG
        // so set the delay to false so that on the next adv cycles we can write the val saved
        // within the option to the stat register.
        if let Some(val) = self.gpu_data.dmg_stat_quirk {
            if !self.gpu_data.dmg_stat_quirk_delay {
                self.gpu_data.write_ppu_io(STAT_REG, val);
                self.gpu_data.dmg_stat_quirk = None;
            } else {
                self.gpu_data.dmg_stat_quirk_delay = false;
            }
        }
    }

    pub fn dmg_init(self: &mut Self) {
        self.gpu_data.dmg_init();
    }

    // https://www.reddit.com/r/Gameboy/comments/a1c8h0/what_happens_when_a_gameboy_screen_is_disabled/
    pub fn disable_ppu(self: &mut Self) {
        // Supposed to have an internal clock for LCD that also gets reset to 0?
        // Also clear the physical screen that shows (Display all white or black to SDL)?
        self.state = ppu::reset(&mut self.gpu_data);
        self.gpu_data.set_ly(0);
        self.gpu_data.stat_low_to_high = false; // Just in case
    }

    pub fn stat_quirk(self: &mut Self, data: u8) -> bool {
        match (self.gpu_data.get_lcd_mode(), self.gpu_data.ly_compare()) {
            (_, true) | (2, _) | (1, _) | (0, _) => {
                self.gpu_data.dmg_stat_quirk = Some(data);
                self.gpu_data.dmg_stat_quirk_delay = true;
                return true;
            }
            _ => {
                self.gpu_data.dmg_stat_quirk = None;
                self.gpu_data.dmg_stat_quirk_delay = false;
                return false;
            }
        }
    }

    pub fn dma_transfer_active(self: &Self) -> bool {
        return self.gpu_data.dma_transfer;
    }

    pub fn stop_dma_transfer(self: &mut Self) {
        self.gpu_data.dma_transfer = false;
        self.gpu_data.dma_cycles = 0;
    }

    pub fn get_dma_src(self: &Self) -> u16 {
        return (self.gpu_data.dma as u16) * DMA_SRC_MUL;
    }

    pub fn dma_cycles(self: &Self) -> usize {
        return self.gpu_data.dma_cycles;
    }

    pub fn dma_delay(self: &Self) -> usize {
        return self.gpu_data.dma_delay_cycles;
    }

    pub fn incr_dma_cycles(self: &mut Self) {
        self.gpu_data.dma_cycles += 1;
    }

    pub fn decr_dma_delay(self: &mut Self) {
        self.gpu_data.dma_delay_cycles -= 1;
        if self.gpu_data.dma_delay_cycles == 0 {
            self.gpu_data.dma_transfer = true;
        }
    }

    // Write multiple bytes into memory starting from location
    // This should only be used for tests (How to configure to only compile for tests)
    pub fn write_bytes(self: &mut Self, location: u16, data: &Vec<u8>) {
        for (i, byte) in data.into_iter().enumerate() {
            self.write_byte(location + (i as u16), *byte);
        }
    }

    pub fn update_display(self: &mut Self, texture: &mut Texture) -> bool {
        if self.frame_ready {
            let wait_time = (self.cycles as f64) * CPU_PERIOD_NANOS;
            let elapsed = self.prev_frame_time.elapsed().as_nanos() as f64;
            if elapsed < wait_time {
                std::thread::sleep(Duration::from_nanos((wait_time - elapsed) as u64));
            }

            texture
                .update(None, &self.gpu_data.pixels, BYTES_PER_ROW)
                .expect("updating texture didnt work");

            self.cycles = 0;
            self.frame_ready = false;
            self.prev_frame_time = Instant::now();
            return true;
        }
        return false;
    }
}

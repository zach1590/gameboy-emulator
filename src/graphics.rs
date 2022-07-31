pub mod gpu_memory;
mod oam_search;
mod picture_generation;
mod ppu;

#[cfg(feature = "debug")]
use sdl2::render::Texture;

use self::gpu_memory::{BYTES_PER_PIXEL, OAM_END, OAM_START, STAT_REG, VRAM_END, VRAM_START};
use super::io::Io;
use gpu_memory::GpuMemory;
use gpu_memory::COLORS;
use ppu::PpuState;
use ppu::PpuState::{HBlank, OamSearch, PictureGeneration, VBlank};

pub const SCALE: u32 = 2;
pub const WIDTH: u32 = 16;
pub const HEIGHT: u32 = 24;
pub const TILE_WIDTH_PIXELS: u32 = 8;
pub const TILE_HEIGHT_PIXELS: u32 = 8;
pub const NUM_PIXELS_X: u32 = WIDTH * TILE_WIDTH_PIXELS;
pub const NUM_PIXELS_Y: u32 = HEIGHT * TILE_HEIGHT_PIXELS;
pub const SCREEN_WIDTH: u32 = NUM_PIXELS_X * SCALE;
pub const SCREEN_HEIGHT: u32 = NUM_PIXELS_Y * SCALE;
pub const BYTES_PER_ROW: usize = BYTES_PER_PIXEL * (NUM_PIXELS_X as usize); // :(

pub const BYTES_PER_TILE: usize = 16;
pub const BYTES_PER_TILE_SIGNED: isize = 16;
pub const DMA_SRC_MUL: u16 = 0x0100;

pub struct Graphics {
    state: PpuState,
    gpu_data: GpuMemory,
    pixels: [u8; 98304],
    dirty: bool,
}

impl Graphics {
    pub fn new() -> Graphics {
        Graphics {
            state: ppu::PpuState::OamSearch(ppu::init()),
            gpu_data: GpuMemory::new(),
            pixels: [0; 98304],
            dirty: false,
        }
    }

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
        if addr >= VRAM_START && addr < VRAM_END {
            self.dirty = true;
        }
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
        if self.stat_quirk(addr, data) {
            // For 1 cycle write 0xFF and whatever resulting interrupts
            self.gpu_data.write_ppu_io(addr, 0xFF);
        } else {
            self.gpu_data.write_ppu_io(addr, data);
        }
    }

    pub fn adv_cycles(self: &mut Self, io: &mut Io, cycles: usize) {
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

    pub fn stat_quirk(self: &mut Self, addr: u16, data: u8) -> bool {
        if addr == STAT_REG {
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
        } else {
            return false;
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

    // **(This probably wont be what we need)**
    // Returns a vector of the actual tiles we want to see on screen
    fn weave_tiles_from_map(self: &mut Self) -> Vec<u8> {
        let mut tile_no;
        let mut tile;
        let mut all_tiles = Vec::new();
        let tile_indices = self.gpu_data.get_bg_tile_map();

        for tile_index in (tile_indices.0)..=tile_indices.1 {
            tile_no = self.read_byte(tile_index);
            tile = self.weave_tile_from_index(tile_no);
            all_tiles.append(&mut tile);
        }

        return all_tiles; // all the tiles that were specified by the tile map (256x256 pixels)
    }

    // Takes the index of a tile and returns the
    // result is a vector of 64 bytes (8x8 pixels). Each byte is a pixel represented by a color (0-3)
    fn weave_tile_from_index(self: &mut Self, tile_no: u8) -> Vec<u8> {
        let addr = calculate_addr(tile_no, &self.gpu_data);
        let mut tile: Vec<u8> = Vec::new();

        for i in (0..=15).step_by(2) {
            let byte0 = self.read_byte(addr + i);
            let byte1 = self.read_byte(addr + i + 1);
            let mut tile_row = weave_bytes(byte0, byte1);
            tile.append(&mut tile_row);
        }

        return tile;
    }

    // Write multiple bytes into memory starting from location
    // This should only be used for tests (How to configure to only compile for tests)
    pub fn write_bytes(self: &mut Self, location: u16, data: &Vec<u8>) {
        for (i, byte) in data.into_iter().enumerate() {
            self.write_byte(location + (i as u16), *byte);
        }
    }

    // Later change this to be with a different screen than the main one
    #[cfg(feature = "debug")]
    pub fn update_pixels_with_tiles(self: &mut Self, texture: &mut Texture) {
        if self.dirty {
            let mut xdraw = 0.0; // where should the tile be drawn
            let mut ydraw = 0.0;
            let mut tile_no = 0;

            // Iterate though all 384 tiles, displaying them in a  16 x 24 grid
            for y in 0..24 {
                for x in 0..16 {
                    self.add_tile(tile_no, xdraw, ydraw);
                    xdraw = xdraw + 8.0;
                    tile_no += 1;
                }
                ydraw = ydraw + 8.0;
                xdraw = 0.0;
            }

            self.dirty = false;
            texture
                .update(None, &self.pixels, BYTES_PER_ROW)
                .expect("updating texture didnt work");
        }
    }

    #[cfg(feature = "debug")]
    pub fn add_tile(self: &mut Self, tile_no: usize, xdraw: f32, ydraw: f32) {
        for i in (0..=15).step_by(2) {
            let byte0 = self.gpu_data.vram[(tile_no * BYTES_PER_TILE) + i];
            let byte1 = self.gpu_data.vram[(tile_no * BYTES_PER_TILE) + i + 1];
            let tile_row = weave_bytes(byte0, byte1);

            let y = (ydraw as usize + (i / 2)) * BYTES_PER_ROW;
            for (j, pix) in tile_row.iter().enumerate() {
                let pix_location = y + ((xdraw as usize + j) * 4);

                self.pixels[pix_location] = COLORS[(*pix) as usize][0];
                self.pixels[pix_location + 1] = COLORS[(*pix) as usize][1];
                self.pixels[pix_location + 2] = COLORS[(*pix) as usize][2];
                self.pixels[pix_location + 3] = COLORS[(*pix) as usize][3];
            }
        }
    }
}

// Takes the index of a tile (should be in the tile map) and returns the address
// that the data for this tile is stored in
fn calculate_addr(tile_no: u8, gpu_mem: &GpuMemory) -> u16 {
    let is_sprite = gpu_mem.is_obj_enabled();

    let addr: u16 = match is_sprite {
        true => 0x8000 + (u16::from(tile_no) * BYTES_PER_TILE as u16),
        false => match gpu_mem.get_addr_mode_start() {
            0x8000 => 0x8000 + (u16::from(tile_no) * 16),
            0x9000 => {
                // Wont be a problem: tile_no will be between -0x80 - 0x7F, thus index will be
                // between -0x800 - 0x7F0 and thus final addr: 0x8800 - 0x97F0 (start of last tile)
                let index = isize::from(tile_no as i8) * BYTES_PER_TILE_SIGNED;
                u16::try_from(0x9000 + index).expect("calculated address did not fit within a u16")
            }
            _ => panic!("get_addr_mode only returns 0x9000 or 0x8000"),
        },
    };
    return addr;
}

// Will take 2 bytes and return an array of 8 values that are between 0-3
// weaves the bits together to form the correct output for graphics
// The two bit0s are concatenated to form the last value of the returned array
// The two bit7s are concatenated to form the first value of the returned array
// arg-byte0 should be the byte that is a lower address in memory
fn weave_bytes(byte0: u8, byte1: u8) -> Vec<u8> {
    let mut tile_row = Vec::new();
    for shift in 0..=7 {
        let p1 = (byte1 >> (7 - shift)) & 0x01;
        let p0 = (byte0 >> (7 - shift)) & 0x01;

        match (p1, p0) {
            (0, 0) => tile_row.push(0),
            (0, 1) => tile_row.push(1),
            (1, 0) => tile_row.push(2),
            (1, 1) => tile_row.push(3),
            _ => panic!(
                "Impossible pixel value: {}, {}",
                byte1 & shift,
                byte0 & shift
            ),
        }
    }
    return tile_row;
}

#[cfg(test)]
#[path = "./tests/graphic_tests.rs"]
mod graphic_tests;

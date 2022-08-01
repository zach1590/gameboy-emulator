use super::graphics::Graphics;
use super::io::{Io, IF_REG};
use super::joypad::{Joypad, JOYP_REG};
use super::mbc::Mbc;
use super::memory::Memory;
use super::serial::*;
use super::timer::Timer;
use crate::graphics::gpu_memory::{
    DMA_MAX_CYCLES, OAM_END, OAM_START, PPUIO_END, PPUIO_START, VRAM_END, VRAM_START,
};
use sdl2::EventPump;

#[cfg(feature = "debug")]
use sdl2::render::Texture;

pub struct Bus {
    mem: Memory,
    graphics: Graphics, // 0x8000 - 0x9FFF, 0xFE00 - 0xFE9F, and 0xFF40 - 0xFF4B
    io: Io,             // 0xFF01 - 0xFF7F (But not 0xFF40 - 0xFF4B)
    timer: Timer,
    joypad: Joypad, // 0xFF01
    serial: Serial,
}

impl Bus {
    pub fn new() -> Bus {
        return Bus {
            mem: Memory::new(),
            graphics: Graphics::new(),
            io: Io::new(),
            timer: Timer::new(),
            joypad: Joypad::new(),
            serial: Serial::new(),
        };
    }

    pub fn set_joypad(self: &mut Self, event_pump: EventPump) {
        self.joypad.set_joypad(event_pump);
    }

    pub fn set_mbc(self: &mut Self, cart_mbc: Box<dyn Mbc>) {
        self.mem.set_mbc(cart_mbc);
    }

    pub fn read_byte(self: &Self, addr: u16) -> u8 {
        let byte = match addr {
            VRAM_START..=VRAM_END => self.graphics.read_byte(addr),
            OAM_START..=OAM_END => self.graphics.read_byte(addr),
            0xFEA0..=0xFEFF => self.graphics.read_byte(addr),
            PPUIO_START..=PPUIO_END => self.graphics.read_io_byte(addr),
            JOYP_REG => self.joypad.read_byte(addr),
            SB_REG | SC_REG => self.serial.read_byte(addr),
            0xFF03..=0xFF39 => self.io.read_byte(addr),
            0xFF4C..=0xFF7F => self.io.read_byte(addr),
            _ => self.mem.read_byte(addr),
        };
        return byte;
    }

    pub fn write_byte(self: &mut Self, addr: u16, data: u8) {
        match addr {
            VRAM_START..=VRAM_END => self.graphics.write_byte(addr, data),
            OAM_START..=OAM_END => self.graphics.write_byte(addr, data),
            0xFEA0..=0xFEFF => self.graphics.write_byte(addr, data), // Memory area not usuable
            PPUIO_START..=PPUIO_END => self.graphics.write_io_byte(addr, data),
            JOYP_REG => self.joypad.write_byte(addr, data),
            SB_REG | SC_REG => self.serial.write_byte(addr, data),
            0xFF03..=0xFF39 => self.io.write_byte(addr, data),
            0xFF4C..=0xFF7F => self.io.write_byte(addr, data),
            _ => self.mem.write_byte(addr, data),
        };
    }

    // dma should have access to whatever it wants from 0x0000 - 0xDF00
    // Define extra read_byte functions that bypass any protections
    // What happens if dma tries to read from memory greater than DF9F? (src is E0?)
    pub fn read_byte_for_dma(self: &Self, addr: u16) -> u8 {
        if addr >= 0xE000 {
            // If this never prints ill remove the other reads
            println!("DMA source above *suggested* address range: {}", addr);
        }
        let byte = match addr {
            VRAM_START..=VRAM_END => self.graphics.read_byte_for_dma(addr),
            OAM_START..=OAM_END => self.graphics.read_byte_for_dma(addr),
            0xFEA0..=0xFEFF => self.graphics.read_byte_for_dma(addr),
            JOYP_REG => self.joypad.read_byte(addr),
            SB_REG | SC_REG => self.serial.read_byte(addr),
            0xFF03..=0xFF39 => self.io.read_byte(addr),
            0xFF4C..=0xFF7F => self.io.read_byte(addr),
            _ => self.mem.read_byte_for_dma(addr),
        };
        return byte;
    }

    // dma should be allowed to write to oam regardless of ppu state
    // use this function to bypass any protections
    pub fn write_byte_for_dma(self: &mut Self, addr: u16, data: u8) {
        self.graphics.write_byte_for_dma(addr, data);
    }

    pub fn dmg_init(self: &mut Self) {
        self.mem.dmg_init();
        self.io.dmg_init();
        self.graphics.dmg_init();
        self.serial.dmg_init();
    }

    pub fn adv_cycles(self: &mut Self, cycles: usize) {
        self.timer.adv_cycles(&mut self.io, cycles);
        self.serial.adv_cycles(&mut self.io, cycles);
        // self.graphics.adv_cycles(&mut self.io, cycles);

        if self.graphics.dma_transfer_active() {
            self.dma_transfer();
        }
        if self.graphics.dma_delay() > 0 {
            self.graphics.decr_dma_delay();
        }
    }

    // Full dma transfer takes 160 machine cycles (640 T Cycles)
    // 1 Cycle per sprite entry
    pub fn dma_transfer(self: &mut Self) {
        let src = self.graphics.get_dma_src(); // 0x0000 - 0xDF00
        let dma_cycles = self.graphics.dma_cycles() as u16; // 0x00 - 0x9F

        self.write_byte_for_dma(dma_cycles, self.read_byte_for_dma(src + dma_cycles as u16));

        if dma_cycles + 1 > DMA_MAX_CYCLES {
            self.graphics.stop_dma_transfer();
        } else {
            self.graphics.incr_dma_cycles();
        }
    }

    pub fn interrupt_pending(self: &Self) -> bool {
        (self.mem.i_enable & self.io.read_byte(IF_REG) & 0x1F) != 0
    }

    pub fn update_input(self: &mut Self) -> bool {
        let should_exit = self.joypad.update_input();
        if self.joypad.is_joypad_interrupt() {
            self.io.request_joypad_interrupt();
        }
        return should_exit;
    }

    pub fn write_bytes(self: &mut Self, location: u16, data: &Vec<u8>) {
        self.mem.write_bytes(location, data);
    }

    pub fn get_mem(self: &Self) -> &Memory {
        return &self.mem;
    }

    #[cfg(feature = "debug")]
    pub fn display_tiles(self: &mut Self, texture: &mut Texture) {
        // self.graphics.update_pixels_with_tiles(texture);
    }
}

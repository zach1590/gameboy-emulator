use super::graphics::Graphics;
use super::io::{Io, IF_REG};
use super::mbc::Mbc;
use super::memory::Memory;
use super::timer::Timer;
use crate::graphics::gpu_memory::OAM_START;

#[cfg(feature = "debug")]
use sdl2::render::Texture;

pub struct Bus {
    mem: Memory,
    graphics: Graphics, // 0x8000 - 0x9FFF(VRAM) and 0xFE00 - 0xFE9F(OAM RAM)
    io: Io,             // 0xFF00 - 0xFF7F
    timer: Timer,
}

impl Bus {
    pub fn new() -> Bus {
        return Bus {
            mem: Memory::new(),
            graphics: Graphics::new(),
            io: Io::new(),
            timer: Timer::new(),
        };
    }

    pub fn set_mbc(self: &mut Self, cart_mbc: Box<dyn Mbc>) {
        self.mem.set_mbc(cart_mbc);
    }

    pub fn read_byte(self: &Self, addr: u16) -> u8 {
        let byte = match addr {
            0x8000..=0x9FFF => self.graphics.read_byte(addr),
            0xFE00..=0xFE9F => self.graphics.read_byte(addr),
            0xFEA0..=0xFEFF => self.graphics.read_byte(addr),
            0xFF40..=0xFF4B => self.graphics.read_io_byte(addr),
            0xFF00..=0xFF39 => self.io.read_byte(addr),
            0xFF4C..=0xFF7F => self.io.read_byte(addr),
            _ => self.mem.read_byte(addr),
        };
        return byte;
    }

    pub fn write_byte(self: &mut Self, addr: u16, data: u8) {
        match addr {
            0x8000..=0x9FFF => self.graphics.write_byte(addr, data),
            0xFE00..=0xFE9F => self.graphics.write_byte(addr, data),
            0xFEA0..=0xFEFF => self.graphics.write_byte(addr, data), // Memory area not usuable
            0xFF40..=0xFF4B => self.graphics.write_io_byte(addr, data),
            0xFF00..=0xFF39 => self.io.write_byte(addr, data),
            0xFF4C..=0xFF7F => self.io.write_byte(addr, data),
            _ => self.mem.write_byte(addr, data),
        };
    }

    // dma should have access to whatever it wants from 0x0000 - 0xDF00
    // Define extra read_byte functions that bypass any protections
    pub fn read_byte_for_dma(self: &Self, addr: u16) -> u8 {
        let byte = match addr {
            0x8000..=0x9FFF => self.graphics.read_byte_for_dma(addr),
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
    }

    pub fn adv_cycles(self: &mut Self, cycles: usize) {
        self.timer.adv_cycles(&mut self.io, cycles);
        // self.graphics.adv_cycles(&mut self.io, cycles);

        if self.graphics.dma_delay() > 0 {
            self.graphics.decr_dma_delay();
        } else if self.graphics.dma_transfer_active() {
            self.dma_transfer();
        }
    }

    // Full dma transfer takes 160 machine cycles (640 T Cycles)
    // 1 Cycle per sprite entry
    pub fn dma_transfer(self: &mut Self) {
        let src = self.graphics.get_dma_src(); // 0x0000 - 0xDF00
        let dma_cycles = self.graphics.dma_cycles() as u16; // 0x00 - 0x9F

        self.write_byte_for_dma(dma_cycles, self.read_byte_for_dma(src + dma_cycles as u16));

        if dma_cycles + 1 == 160 {
            self.graphics.stop_dma_transfer();
        } else {
            self.graphics.incr_dma_cycles();
        }
    }

    pub fn interrupt_pending(self: &Self) -> bool {
        (self.mem.i_enable & self.io.read_byte(IF_REG)) != 0
    }

    pub fn write_bytes(self: &mut Self, location: u16, data: &Vec<u8>) {
        self.mem.write_bytes(location, data);
    }

    pub fn get_mem(self: &Self) -> &Memory {
        return &self.mem;
    }

    #[cfg(feature = "debug")]
    pub fn get_io_mut(self: &mut Self) -> &mut Io {
        return &mut self.io;
    }

    #[cfg(feature = "debug")]
    pub fn display_tiles(self: &mut Self, texture: &mut Texture) {
        self.graphics.update_pixels_with_tiles(texture);
    }
}

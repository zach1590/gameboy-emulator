use super::memory::Memory;
use super::graphics::Graphics;
use super::io::{Io, IF_REG};
use super::mbc::{Mbc};
use super::timer::Timer;

pub struct Bus {
    mem: Memory,
    graphics: Graphics,     // 0x8000 - 0x9FFF(VRAM) and 0xFE00 - 0xFE9F(OAM RAM)
    io: Io,                 // 0xFF00 - 0xFF7F
    timer: Timer,
}

impl Bus {
    pub fn new() -> Bus {
        return Bus {
            mem: Memory::new(),
            graphics: Graphics::new(),
            io: Io::new(),
            timer: Timer::new(),
        }
    }

    pub fn set_mbc(self: &mut Self, cart_mbc: Box<dyn Mbc>) {
        self.mem.set_mbc(cart_mbc);
    }

    pub fn read_byte(self: &Self, addr: u16) -> u8 {
        let byte = match addr {
            0x8000..=0x9FFF => self.graphics.read_byte(addr),
            0xFE00..=0xFE9F => self.graphics.read_byte(addr),
            0xFEA0..=0xFEFF => {
                match self.graphics.oam_blocked {
                    true => 0xFF,
                    false => 0x00,
                }
            }
            0xFF00..=0xFF7F => self.io.read_byte(addr),
            _ => self.mem.read_byte(addr),
        };
        return byte;
    }

    // Write a single byte to at the location
    pub fn write_byte(self: &mut Self, addr: u16, data: u8) {
        match addr {
            0x8000..=0x9FFF => self.graphics.write_byte(addr, data),
            0xFE00..=0xFE9F => self.graphics.write_byte(addr, data),
            0xFEA0..=0xFEFF => return, // Memory area not usuable
            0xFF00..=0xFF7F => self.io.write_byte(addr, data),
            _ => self.mem.write_byte(addr, data),
        };
    }
    
    pub fn dmg_init(self: &mut Self) {
        self.mem.dmg_init();
        self.io.dmg_init();
    }

    pub fn adv_cycles(self: &mut Self, cycles: usize) {
        self.timer.adv_cycles(&mut self.io, cycles);
        self.graphics.adv_cycles(&mut self.io, cycles);
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
}
use super::mbc::{Mbc, MbcNone};
use super::io::{Io, IF_REG};
use super::timer::Timer;
use super::render::Render;

pub const LCDC_REG: u16 = 0xFF40;
pub const LY_REG: u16 = 0xFF44;

pub struct Memory {
    mbc: Box<dyn Mbc>,      // MBC will contain ROM and RAM aswell as banks
    render: Render,         // // 0x8000 - 0x9FFF(VRAM) and 0xFE00 - 0xFE9F(OAM RAM)
    wram: [u8; 8_192],      // 0xC000 - 0xDFFF
    echo_wram: [u8; 7_680], // 0xE000 - 0xFDFF (mirror of work ram)
    not_used: [u8; 96],     // 0xFEAO - 0xFEFF
    io: Io,                 // 0xFF00 - 0xFF7F
    hram: [u8; 127],        // 0xFF80 - 0xFFFE
    i_enable: u8,           // 0xFFFF
    timer: Timer,
}

impl Memory {
    pub fn new() -> Memory {
        return Memory {
            mbc: Box::new(MbcNone::new()),  // Swap out mbc once its known
            render: Render::new(),
            wram: [0; 8_192],
            echo_wram: [0; 7_680],
            not_used: [0; 96],
            io: Io::new(),
            hram: [0; 127],
            i_enable: 0,
            timer: Timer::new(),
        };
    }

    pub fn set_mbc(self: &mut Self, cart_mbc: Box<dyn Mbc>) {
        self.mbc = cart_mbc;
    }

    pub fn interrupt_pending(self: &Self) -> bool {
        (self.i_enable & self.io.read_byte(IF_REG)) != 0
    }

    pub fn read_byte(self: &Self, addr: u16) -> u8 {
        let byte = match addr {
            0x0000..=0x7FFF => self.mbc.read_rom_byte(addr),
            0x8000..=0x9FFF => self.render.read_byte(addr),
            0xA000..=0xBFFF => self.mbc.read_ram_byte(addr),
            0xC000..=0xDFFF => self.wram[usize::from(addr - 0xC000)],
            0xE000..=0xFDFF => self.echo_wram[usize::from(addr - 0xE000)],
            0xFE00..=0xFE9F => self.render.read_byte(addr),
            0xFEA0..=0xFEFF => {
                match self.render.oam_blocked {
                    true => 0xFF,
                    false => 0x00,
                }
            }
            0xFF00..=0xFF7F => self.io.read_byte(addr),
            0xFF80..=0xFFFE => self.hram[usize::from(addr - 0xFF80)],
            0xFFFF => self.i_enable,
        };
        return byte;
    }

    // Write a single byte to at the location
    pub fn write_byte(self: &mut Self, addr: u16, data: u8) {
        match addr {
            0x0000..=0x7FFF => self.mbc.write_rom_byte(addr, data),
            0x8000..=0x9FFF => self.render.write_byte(addr, data),
            0xA000..=0xBFFF => self.mbc.write_ram_byte(addr, data),
            0xC000..=0xDFFF => {
                self.wram[usize::from(addr - 0xC000)] = data;
                if addr <= 0xDDFF {
                    self.echo_wram[usize::from(addr - 0xC000)] = data;
                }
            }
            0xE000..=0xFDFF => return,  // Should not write to echo ram
            0xFE00..=0xFE9F => self.render.write_byte(addr, data),
            0xFEA0..=0xFEFF => return, // Memory area not usuable
            0xFF00..=0xFF7F => self.io.write_byte(addr, data),
            0xFF80..=0xFFFE => self.hram[usize::from(addr - 0xFF80)] = data,
            0xFFFF => self.i_enable = data,
        };
    }

    pub fn handle_clocks(self: &mut Self, cycles: usize) {
        self.timer.handle_clocks(&mut self.io, cycles);
    }

    // lcdc can be modified mid scanline (I dont know how??)
    // Maybe better to call this function from each of other helper methods?
    pub fn get_lcdc(self: &Self) -> u8 {
        return self.read_byte(LCDC_REG);
    }

    pub fn get_ly(self: &Self) -> u8 {
        return self.read_byte(LY_REG);
    }

    // Write multiple bytes into memory starting from location
    // This should only be used for tests
    pub fn write_bytes(self: &mut Self, location: u16, data: &Vec<u8>) {
        for (i, byte) in data.into_iter().enumerate() {
            self.write_byte(location + (i as u16), *byte);
        }
    }

    pub fn get_renderer_mut(self: &mut Self) -> &mut Render {
        return &mut self.render;
    }

    #[cfg(feature = "debug")]
    pub fn get_io_mut(self: &mut Self) -> &mut Io {
        return &mut self.io;
    }

    
    pub fn dmg_init(self: &mut Self) {
        self.io.dmg_init();
        self.i_enable = 0x00;
    }
}

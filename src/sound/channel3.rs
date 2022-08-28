#[allow(unused_imports)]
use super::{NR30, NR31, NR32, NR33, NR34, WAVE_RAM_END, WAVE_RAM_START};

pub struct Ch3 {
    nr30: u8,
    nr31: u8,
    nr32: u8,
    nr33: u8,
    nr34: u8,
}

impl Ch3 {
    pub fn new() -> Ch3 {
        Ch3 {
            nr30: 0,
            nr31: 0,
            nr32: 0,
            nr33: 0,
            nr34: 0,
        }
    }

    pub fn read_byte(self: &Self, addr: u16) -> u8 {
        match addr {
            NR30 => self.nr30,
            NR31 => self.nr31 | 0xFF,
            NR32 => self.nr32,
            NR33 => self.nr33 | 0xFF,
            NR34 => self.nr34 | 0x83,
            _ => panic!("ch3 does not handle reads from addr: {}", addr),
        }
    }

    pub fn write_byte(self: &mut Self, addr: u16, data: u8) {
        match addr {
            NR30 => self.nr30 = data | 0x7F,
            NR31 => self.nr31 = data,
            NR32 => self.nr32 = data | 0x9F,
            NR33 => self.nr33 = data,
            NR34 => self.nr34 = data | 0x38,
            _ => panic!("ch3 does not handle writes to addr: {}", addr),
        }
    }

    pub fn dmg_init(self: &mut Self) {
        self.nr30 = 0x7F;
        self.nr31 = 0xFF;
        self.nr32 = 0x9F;
        self.nr33 = 0xFF;
        self.nr34 = 0xBF;
    }
}

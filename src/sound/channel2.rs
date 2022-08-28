use super::{NR21, NR22, NR23, NR24};

pub struct Ch2 {
    nr21: u8,
    nr22: u8,
    nr23: u8,
    nr24: u8,
}

impl Ch2 {
    pub fn new() -> Ch2 {
        Ch2 {
            nr21: 0,
            nr22: 0,
            nr23: 0,
            nr24: 0,
        }
    }

    pub fn read_byte(self: &Self, addr: u16) -> u8 {
        match addr {
            NR21 => self.nr21 | 0x3F,
            NR22 => self.nr22,
            NR23 => self.nr23 | 0xFF,
            NR24 => self.nr24 | 0x83,
            _ => panic!("ch2 does not handle reads from addr: {}", addr),
        }
    }

    pub fn write_byte(self: &mut Self, addr: u16, data: u8) {
        match addr {
            NR21 => self.nr21 = data,
            NR22 => self.nr22 = data,
            NR23 => self.nr23 = data,
            NR24 => self.nr24 = data | 0x38,
            _ => panic!("ch2 does not handle writes to addr: {}", addr),
        }
    }

    pub fn dmg_init(self: &mut Self) {
        self.nr21 = 0x3F;
        self.nr22 = 0x00;
        self.nr23 = 0xFF;
        self.nr24 = 0xBF;
    }
}

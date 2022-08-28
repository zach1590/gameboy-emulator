use super::{NR10, NR11, NR12, NR13, NR14};

pub struct Ch1 {
    nr10: u8,
    nr11: u8,
    nr12: u8,
    nr13: u8,
    nr14: u8,
}

impl Ch1 {
    pub fn new() -> Ch1 {
        Ch1 {
            nr10: 0,
            nr11: 0,
            nr12: 0,
            nr13: 0,
            nr14: 0,
        }
    }

    pub fn read_byte(self: &Self, addr: u16) -> u8 {
        match addr {
            NR10 => self.nr10,
            NR11 => self.nr11 | 0x3F,
            NR12 => self.nr12,
            NR13 => self.nr13,
            NR14 => self.nr14 | 0x83,
            _ => panic!("ch1 does not handle reads from addr: {}", addr),
        }
    }

    pub fn write_byte(self: &mut Self, addr: u16, data: u8) {
        match addr {
            NR10 => self.nr10 = data | 0x80,
            NR11 => self.nr11 = data,
            NR12 => self.nr12 = data,
            NR13 => self.nr13 = data,
            NR14 => self.nr14 = data,
            _ => panic!("ch1 does not handle writes to addr: {}", addr),
        }
    }

    pub fn dmg_init(self: &mut Self) {
        self.nr10 = 0x80;
        self.nr11 = 0xBF;
        self.nr12 = 0xF3;
        self.nr13 = 0xFF;
        self.nr14 = 0xBF;
    }
}

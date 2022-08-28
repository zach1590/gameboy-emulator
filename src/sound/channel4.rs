use super::{NR41, NR42, NR43, NR44};

pub struct Ch4 {
    nr41: u8,
    nr42: u8,
    nr43: u8,
    nr44: u8,
}

impl Ch4 {
    pub fn new() -> Ch4 {
        Ch4 {
            nr41: 0,
            nr42: 0,
            nr43: 0,
            nr44: 0,
        }
    }

    pub fn read_byte(self: &Self, addr: u16) -> u8 {
        match addr {
            NR41 => self.nr41 | 0x3F,
            NR42 => self.nr42,
            NR43 => self.nr43,
            NR44 => self.nr44 | 0x80,
            _ => panic!("ch4 does not handle reads from addr: {}", addr),
        }
    }

    pub fn write_byte(self: &mut Self, addr: u16, data: u8) {
        match addr {
            NR41 => self.nr41 = data | 0xC0,
            NR42 => self.nr42 = data,
            NR43 => self.nr43 = data,
            NR44 => self.nr44 = data | 0x3F,
            _ => panic!("ch4 does not handle writes to addr: {}", addr),
        }
    }

    pub fn dmg_init(self: &mut Self) {
        self.nr41 = 0xFF;
        self.nr42 = 0x00;
        self.nr43 = 0x00;
        self.nr44 = 0xBF;
    }
}

use super::VolEnv;
use super::{NR41, NR42, NR43, NR44};

pub struct Ch4 {
    len: u8,            // NR41 (Only the bottom 5 bits)
    vol_env: VolEnv,    // NR42
    pcntr: PolyCounter, // NR43
    cntr: Counter,      // NR44 (Only the bottom two bits)
}

impl Ch4 {
    pub fn new() -> Ch4 {
        Ch4 {
            len: 0,
            vol_env: VolEnv::new(),
            pcntr: PolyCounter::new(),
            cntr: Counter::new(),
        }
    }

    pub fn read_byte(self: &Self, addr: u16) -> u8 {
        match addr {
            NR41 => self.len | 0xFF,
            NR42 => self.vol_env.get(),
            NR43 => self.pcntr.get(),
            NR44 => self.cntr.get(),
            _ => panic!("ch4 does not handle reads from addr: {}", addr),
        }
    }

    pub fn write_byte(self: &mut Self, addr: u16, data: u8) {
        match addr {
            NR41 => self.len = data & 0x3F,
            NR42 => self.vol_env.set(data),
            NR43 => self.pcntr.set(data),
            NR44 => self.cntr.set(data),
            _ => panic!("ch4 does not handle writes to addr: {}", addr),
        }
    }

    pub fn calc_len(self: &Self) -> f32 {
        return ((64 - self.len) as f32) / 256.;
    }

    pub fn dmg_init(self: &mut Self) {
        self.len = 0xFF;
        self.vol_env.set(0x00);
        self.pcntr.set(0x00);
        self.cntr.set(0xBF);
    }
}

struct PolyCounter {
    pub shift_freq: u8, // Bit 4-7
    pub width: u8,      // Bit 3 (0 is 15 bits and 1 is 7 bits)
    pub ratio: u8,      // Bit 0-2
}

impl PolyCounter {
    pub fn new() -> PolyCounter {
        return PolyCounter {
            shift_freq: 0,
            width: 0,
            ratio: 0,
        };
    }
    pub fn set(self: &mut Self, data: u8) {
        self.shift_freq = (data >> 4) & 0x0F;
        self.width = (data >> 3) & 0x01;
        self.ratio = data & 0x07;
    }
    pub fn get(self: &Self) -> u8 {
        return self.shift_freq << 4 | self.width << 3 | self.ratio;
    }
}

struct Counter {
    restart: bool, // Bit 7 (1 = restart sound)
    counter: bool, // Bit 6 (1 = stop output when len in nr41 expires)
}

impl Counter {
    const MASK: u8 = 0x3F;
    pub fn new() -> Counter {
        return Counter {
            restart: false,
            counter: false,
        };
    }
    pub fn set(self: &mut Self, data: u8) {
        self.restart = (data >> 7) & 0x01 == 0x01;
        self.counter = (data >> 6) & 0x01 == 0x01;
    }
    pub fn get(self: &Self) -> u8 {
        return Self::MASK | ((self.restart as u8) << 7) | ((self.counter as u8) << 6);
    }
    pub fn should_restart(self: &Self) -> bool {
        return self.restart;
    }
    pub fn should_stop(self: &Self) -> bool {
        return self.counter;
    }
}

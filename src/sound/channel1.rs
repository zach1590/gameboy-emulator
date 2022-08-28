use super::{Freq, LenPat, VolEnv};
use super::{NR10, NR11, NR12, NR13, NR14};

pub struct Ch1 {
    sweep: Sweep,   // NR10
    lenpat: LenPat, // NR11
    volenv: VolEnv, // NR12
    freq: Freq,     // NR13 and NR14
}

impl Ch1 {
    pub fn new() -> Ch1 {
        Ch1 {
            sweep: Sweep::new(),
            lenpat: LenPat::new(),
            volenv: VolEnv::new(),
            freq: Freq::new(),
        }
    }

    pub fn read_byte(self: &Self, addr: u16) -> u8 {
        match addr {
            NR10 => self.sweep.get(),
            NR11 => self.lenpat.get(),
            NR12 => self.volenv.get(),
            NR13 => self.freq.get_lo(),
            NR14 => self.freq.get_hi(),
            _ => panic!("ch1 does not handle reads from addr: {}", addr),
        }
    }

    pub fn write_byte(self: &mut Self, addr: u16, data: u8) {
        match addr {
            NR10 => self.sweep.set(data),
            NR11 => self.lenpat.set(data),
            NR12 => self.volenv.set(data),
            NR13 => self.freq.set_lo(data),
            NR14 => self.freq.set_hi(data),
            _ => panic!("ch1 does not handle writes to addr: {}", addr),
        }
    }

    pub fn dmg_init(self: &mut Self) {
        self.sweep.set(0x80);
        self.lenpat.set(0xBF);
        self.volenv.set(0xF3);
        self.freq.set_lo(0xFF);
        self.freq.set_hi(0xBF);
    }
}

// NR10
struct Sweep {
    pub time: u8,      // Bit 4-6
    pub swp_dir: bool, // Bit 3 (1 is decr)
    pub shift: u8,     // Bit 0-2
}

impl Sweep {
    const MASK: u8 = 0x80;

    pub fn new() -> Sweep {
        return Sweep {
            time: 0,
            swp_dir: false,
            shift: 0,
        };
    }
    pub fn set(self: &mut Self, data: u8) {
        self.time = (data >> 4) & 0x07;
        self.swp_dir = (data >> 3) & 0x01 == 0x01;
        self.shift = data & 0x07;
    }
    pub fn get(self: &Self) -> u8 {
        return Self::MASK | (self.time << 4) | ((self.swp_dir as u8) << 3) | self.shift;
    }
}

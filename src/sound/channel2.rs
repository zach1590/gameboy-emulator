use super::{Freq, LenPat, VolEnv};
use super::{NR21, NR22, NR23, NR24};

pub struct Ch2 {
    lenpat: LenPat,  // NR21
    vol_env: VolEnv, // NR22
    freq: Freq,      // NR23 and NR24
}

impl Ch2 {
    pub fn new() -> Ch2 {
        Ch2 {
            lenpat: LenPat::new(),
            vol_env: VolEnv::new(),
            freq: Freq::new(),
        }
    }

    pub fn read_byte(self: &Self, addr: u16) -> u8 {
        match addr {
            NR21 => self.lenpat.get(),
            NR22 => self.vol_env.get(),
            NR23 => self.freq.get_lo(),
            NR24 => self.freq.get_hi(),
            _ => panic!("ch2 does not handle reads from addr: {}", addr),
        }
    }

    pub fn write_byte(self: &mut Self, addr: u16, data: u8) {
        match addr {
            NR21 => self.lenpat.set(data),
            NR22 => self.vol_env.set(data),
            NR23 => self.freq.set_lo(data),
            NR24 => self.freq.set_hi(data),
            _ => panic!("ch2 does not handle writes to addr: {}", addr),
        }
    }

    pub fn adv_cycles(self: &mut Self, _cycles: usize) {}

    pub fn dmg_init(self: &mut Self) {
        self.lenpat.set(0x3F);
        self.vol_env.set(0x00);
        self.freq.dmg_init();
    }
}

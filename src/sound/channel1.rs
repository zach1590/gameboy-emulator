use super::{Freq, LenPat, VolEnv};
use super::{NR10, NR11, NR12, NR13, NR14};

pub struct Ch1 {
    sweep: Sweep,   // NR10
    lenpat: LenPat, // NR11
    volenv: VolEnv, // NR12
    freq: Freq,     // NR13 and NR14
    frame_seq: u8,  // dictates which channel gets clocked
}

impl Ch1 {
    pub fn new() -> Ch1 {
        Ch1 {
            sweep: Sweep::new(),
            lenpat: LenPat::new(),
            volenv: VolEnv::new(),
            freq: Freq::new(),
            frame_seq: 0,
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

    pub fn adv_cycles(self: &mut Self, cycles: usize) {
        let was_reset = self.freq.decr_clock(cycles);

        // TODO: Make sure the frame_sequencer is only clocked by the frequency and not
        // and internal counter that counts to 8192 cycles (512Hz).
        if was_reset {
            self.frame_seq += 1; // Currently this will skip the first length clock (Probably shouldnt)
            self.frame_seq = self.frame_seq % 8;

            match self.frame_seq {
                0 | 4 => {
                    // Clock only len ctr
                    self.clock_length();
                }
                2 | 6 => {
                    // Clock len ctr and sweep
                    self.clock_length();
                }
                7 => {
                    // Clock only vol env
                }
                1 | 5 => { /* Do Nothing */ }
                _ => panic!(
                    "frame sequencer should not be higher than 7: {}",
                    self.frame_seq
                ),
            }
        }
    }

    pub fn get_output(self: &Self) -> u8 {
        if !self.is_ch_enabled() {
            return 0;
        }
        return 0xFF; // For now
    }

    pub fn clock_length(self: &mut Self) {
        if self.freq.counter && self.is_ch_enabled() {
            // Should this decrement regardless or only if the counter is set?
            self.lenpat.decr_len();
        }
    }

    pub fn is_ch_enabled(self: &Self) -> bool {
        return self.lenpat.internal_enable;
    }

    pub fn dmg_init(self: &mut Self) {
        self.sweep.set(0x80);
        self.lenpat.set(0xBF);
        self.volenv.set(0xF3);
        self.freq.dmg_init();
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

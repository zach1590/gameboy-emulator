use super::DUTY_WAVES;
use super::{Freq, LenPat, VolEnv};
use super::{NR21, NR22, NR23, NR24};

pub struct Ch2 {
    lenpat: LenPat,  // NR21
    vol_env: VolEnv, // NR22
    freq: Freq,      // NR23 and NR24
    frame_seq: u8,   // dictates which channel gets clocked
    internal_cycles: usize,
    freq_timer: usize,
    duty_pos: usize,
}

impl Ch2 {
    pub fn new() -> Ch2 {
        Ch2 {
            lenpat: LenPat::new(0x3F),
            vol_env: VolEnv::new(),
            freq: Freq::new(),
            frame_seq: 0,
            internal_cycles: 0,
            freq_timer: 0,
            duty_pos: 0,
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
            NR24 => {
                if self.freq.initial {
                    self.on_trigger();
                }
                self.freq.set_hi(data);
            }
            _ => panic!("ch2 does not handle writes to addr: {}", addr),
        }
    }

    pub fn adv_cycles(self: &mut Self, cycles: usize) {
        self.internal_cycles = self.internal_cycles.wrapping_add(cycles);

        if self.decr_freq_timer(cycles) {
            self.duty_pos = (self.duty_pos + 1) % 8;
        }

        if self.internal_cycles >= 8192 {
            // Currently this will skip the first length clock (Should it?)
            self.frame_seq = (self.frame_seq + 1) % 8;
            self.internal_cycles = self.internal_cycles.wrapping_sub(8192);

            match self.frame_seq {
                0 | 4 => self.clock_length(),
                2 | 6 => {
                    self.clock_length();
                    self.clock_sweep();
                }
                7 => self.clock_volenv(),
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
        return DUTY_WAVES[usize::from(self.lenpat.duty)][self.duty_pos];
    }

    fn clock_length(self: &mut Self) {
        if self.freq.counter && self.lenpat.internal_enable {
            self.lenpat.decr_len();
        }
    }

    fn clock_sweep(self: &mut Self) {}

    fn clock_volenv(self: &mut Self) {}

    // Decrement the internal clock and return if it hit 0
    fn decr_freq_timer(self: &mut Self, cycles: usize) -> bool {
        self.freq_timer = self.freq_timer.wrapping_sub(cycles);

        if self.freq_timer == 0 || self.freq_timer > 8192 {
            self.freq_timer = (2048 - self.freq.get_full() as usize) * 4;
            return true;
        }
        return false;
    }

    fn on_trigger(self: &mut Self) {
        // TODO: Add the other events that occur on trigger
        self.lenpat.reload_timer(); // Should I only reload if equal to zero?
    }

    pub fn is_ch_enabled(self: &Self) -> bool {
        // TODO: Add the other internal enable flags if any
        return self.lenpat.internal_enable;
    }

    pub fn dmg_init(self: &mut Self) {
        self.lenpat.set(0x3F);
        self.vol_env.set(0x00);
        self.freq.set_lo(0xFF);
        self.freq.set_hi(0xBF);
        self.freq_timer = 0; // I think
    }
}

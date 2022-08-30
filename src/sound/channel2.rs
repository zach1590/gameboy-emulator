use super::DUTY_WAVES;
use super::{Freq, LenPat, VolEnv};
use super::{NR21, NR22, NR23, NR24};

pub struct Ch2 {
    lenpat: LenPat, // NR21
    volenv: VolEnv, // NR22
    freq: Freq,     // NR23 and NR24
    frame_seq: u8,  // dictates which channel gets clocked
    internal_cycles: usize,
    duty_pos: usize,
}

impl Ch2 {
    pub fn new() -> Ch2 {
        Ch2 {
            lenpat: LenPat::new(0x3F),
            volenv: VolEnv::new(),
            freq: Freq::new(),
            frame_seq: 0,
            internal_cycles: 0,
            duty_pos: 0,
        }
    }

    pub fn read_byte(self: &Self, addr: u16) -> u8 {
        match addr {
            NR21 => self.lenpat.get(),
            NR22 => self.volenv.get(),
            NR23 => self.freq.get_lo(),
            NR24 => self.freq.get_hi(),
            _ => panic!("ch2 does not handle reads from addr: {}", addr),
        }
    }

    pub fn write_byte(self: &mut Self, addr: u16, data: u8) {
        match addr {
            NR21 => self.lenpat.set(data),
            NR22 => self.volenv.set(data),
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
        if !self.is_ch_enabled() {
            // None of the operations matter since our dac will just return 0
            // anyways. Continuing to increment/decrement the values is also
            // useless since everything is reset on the trigger when it get enabled
            return;
        }

        self.internal_cycles = self.internal_cycles.wrapping_add(cycles);

        if self.freq.decr_timer(cycles, 8192, 2048) {
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

    fn clock_length(self: &mut Self) {
        if self.freq.counter && self.lenpat.enable {
            self.lenpat.decr_len();
        }
    }

    fn clock_sweep(self: &mut Self) {}

    fn clock_volenv(self: &mut Self) {
        if self.volenv.sweep == 0 {
            return;
        }
        if self.volenv.timer != 0 {
            self.volenv.decr_timer();
        }
    }

    // This will be the DAC?
    // TODO: What type is required by SDL Audio?
    pub fn get_output(self: &Self) -> f32 {
        if !self.is_ch_enabled() {
            return 0.0;
        }
        let duty_output = DUTY_WAVES[usize::from(self.lenpat.duty)][self.duty_pos];

        // duty is 0 or 1, and cur_vol is 0-15, so cast to f32 is no problem
        return (f32::from(duty_output * self.volenv.cur_vol) / 7.5) - 1.0;
    }

    fn on_trigger(self: &mut Self) {
        /* Length */
        self.internal_cycles = 0; // Should this happen?
        self.duty_pos = 0;
        self.lenpat.reload_timer();

        /* Frequency */
        self.freq.reload_timer(2048);

        /* Volume Envelope */
        self.volenv.reload_timer();
        self.volenv.reload_vol();
    }

    pub fn is_ch_enabled(self: &Self) -> bool {
        return self.lenpat.enable;
    }

    pub fn dmg_init(self: &mut Self) {
        self.lenpat.set(0x3F);
        self.volenv.set(0x00);
        self.freq.set_lo(0xFF);
        self.freq.set_hi(0xBF);

        self.freq.reload_timer(2048); // I think
        self.volenv.reload_timer();
        self.volenv.reload_vol();
    }
}

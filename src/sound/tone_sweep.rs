use super::DUTY_WAVES;
use super::{Freq, LenPat, VolEnv};
use super::{NR10, NR11, NR12, NR13, NR14};
use super::{NR21, NR22, NR23, NR24};

pub struct Tone {
    sweep: Option<Sweep>, // NR10
    lenpat: LenPat,       // NR11
    volenv: VolEnv,       // NR12
    freq: Freq,           // NR13 and NR14
    frame_seq: u8,        // dictates which channel gets clocked
    internal_cycles: usize,
    duty_pos: usize,
}

impl Tone {
    pub fn new() -> Tone {
        Tone {
            sweep: None,
            lenpat: LenPat::new(0x3F),
            volenv: VolEnv::new(),
            freq: Freq::new(),
            frame_seq: 0,
            internal_cycles: 0,
            duty_pos: 0,
        }
    }

    pub fn with_sweep(mut self) -> Tone {
        self.sweep = Some(Sweep::new());
        return self;
    }

    pub fn read_byte(self: &Self, addr: u16) -> u8 {
        match addr {
            NR10 => {
                if let Some(sweep) = &self.sweep {
                    sweep.get()
                } else {
                    panic!("This channel does not have sweep, check the writes");
                }
            }
            NR11 | NR21 => self.lenpat.get(),
            NR12 | NR22 => self.volenv.get(),
            NR13 | NR23 => self.freq.get_lo(),
            NR14 | NR24 => self.freq.get_hi(),
            _ => panic!("ch1 does not handle reads from addr: {}", addr),
        }
    }

    pub fn write_byte(self: &mut Self, addr: u16, data: u8) {
        match addr {
            NR10 => {
                if let Some(sweep) = &mut self.sweep {
                    sweep.set(data)
                } else {
                    panic!("This channel does not have sweep, check the writes");
                }
            }
            NR11 | NR21 => self.lenpat.set(data),
            NR12 | NR22 => self.volenv.set(data),
            NR13 | NR23 => self.freq.set_lo(data),
            NR14 | NR24 => {
                let prev_len_enable = self.freq.len_enable;
                self.freq.set_hi(data);

                if self.freq.len_enable
                    && !prev_len_enable
                    && self.lenpat.timer != 0
                    && (self.frame_seq % 2) == 0
                {
                    self.lenpat.decr_len();
                }

                // Its possible that the result above causes the channel to disable
                // It should stay disabled if the trigger is clear, but if the trigger
                // is set it'll get renabled below
                if self.freq.initial {
                    self.on_trigger();
                }
            }
            _ => panic!("ch1 does not handle writes to addr: {}", addr),
        }
    }

    pub fn adv_cycles(self: &mut Self, cycles: usize) {
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
                1 | 3 | 5 => { /* Do Nothing */ }
                _ => panic!(
                    "frame sequencer should not be higher than 7: {}",
                    self.frame_seq
                ),
            }
        }
    }

    fn clock_length(self: &mut Self) {
        if self.freq.len_enable && self.lenpat.enable {
            self.lenpat.decr_len();
        }
    }

    fn clock_sweep(self: &mut Self) {
        if let Some(sweep) = &mut self.sweep {
            if sweep.timer == 0 {
                return;
            }
            if sweep.decr_timer() {
                if sweep.enable && sweep.time > 0 {
                    let new_freq = sweep.calc_freq();

                    if new_freq <= 2047 && sweep.shift > 0 {
                        self.freq.set_full(new_freq);
                        sweep.sh_freq = new_freq;

                        /* for overflow check */
                        sweep.calc_freq();
                    }
                }
            }
        }
        // ch2 will do nothing since it has no sweep
    }

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
        self.duty_pos = 0;
        self.lenpat.reload_timer();

        /* Frequency */
        self.freq.reload_timer(2048);

        /* Volume Envelope */
        self.volenv.reload_timer();
        self.volenv.reload_vol();

        /* Sweep */
        if let Some(sweep) = &mut self.sweep {
            sweep.sh_freq = self.freq.get_full();
            sweep.reload_timer();
            sweep.enable = (sweep.time != 0) || (sweep.shift != 0);
            if sweep.shift != 0 {
                // For overflow check (Might disable)
                sweep.calc_freq();
            }
        }

        /* Some of Obscure Behaviour */
        if self.frame_seq == 6 {
            // If the next step clocks the volume envelope
            self.volenv.timer = self.volenv.timer.wrapping_add(1);
        }
        if (self.frame_seq % 2) == 0 {
            // if the next step doesnt clock the length counter and the previous
            // length before reloading it above was 0, instead of 64/256, load with 63/255
            if self.lenpat.timer == u32::from(self.lenpat.mask) + 1 && self.freq.len_enable {
                self.lenpat.timer = u32::from(self.lenpat.mask);
            }
        }
    }

    pub fn is_ch_enabled(self: &Self) -> bool {
        return self.lenpat.enable;
    }

    pub fn dmg_init(self: &mut Self) {
        self.lenpat.set(0xBF);
        self.volenv.set(0xF3);
        self.freq.set_lo(0xFF);
        self.freq.set_hi(0xBF);

        self.lenpat.reload_timer();
        self.freq.reload_timer(2048); // I think
        self.volenv.reload_timer();
        self.volenv.reload_vol();

        if let Some(sweep) = &mut self.sweep {
            sweep.set(0x80);
            sweep.sh_freq = self.freq.get_full();
            sweep.reload_timer();
            sweep.enable = (sweep.time != 0) || (sweep.shift != 0);
        }
    }
}

// NR10
struct Sweep {
    pub time: u8,      // Bit 4-6 (More like period)
    pub swp_dir: bool, // Bit 3 (1 is decr)
    pub shift: u8,     // Bit 0-2
    pub timer: u32,
    pub enable: bool,
    pub sh_freq: u16,
}

impl Sweep {
    const MASK: u8 = 0x80;

    pub fn new() -> Sweep {
        return Sweep {
            time: 0,
            swp_dir: false,
            shift: 0,
            timer: 0,
            enable: false,
            sh_freq: 0,
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

    pub fn decr_timer(self: &mut Self) -> bool {
        self.timer = self.timer.wrapping_sub(1);

        if self.timer == 0 {
            self.reload_timer();
            return true;
        }
        return false;
    }

    pub fn reload_timer(self: &mut Self) {
        self.timer = if self.time == 0 {
            8
        } else {
            u32::from(self.time)
        };
    }

    pub fn calc_freq(self: &mut Self) -> u16 {
        let mut new_freq = self.sh_freq >> self.shift;

        if self.swp_dir {
            new_freq = self.sh_freq - new_freq;
        } else {
            new_freq = self.sh_freq + new_freq;
        }

        /* overflow check */
        if new_freq > 2047 {
            self.enable = false;
        }

        return new_freq;
    }
}

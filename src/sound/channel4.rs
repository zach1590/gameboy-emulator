use super::{LenPat, VolEnv};
use super::{NR41, NR42, NR43, NR44};

pub struct Ch4 {
    len: LenPat,           // NR41 (Doesnt use duty)
    volenv: VolEnv,        // NR42
    pcounter: PolyCounter, // NR43
    counter: Counter,      // NR44 (Only the bottom two bits)
    frame_seq: u8,
    internal_cycles: usize,
    lfsr: u16, // Only 15 bits wide though
}

impl Ch4 {
    pub fn new() -> Ch4 {
        Ch4 {
            len: LenPat::new(0x3F),
            volenv: VolEnv::new(),
            pcounter: PolyCounter::new(),
            counter: Counter::new(),
            frame_seq: 0,
            internal_cycles: 0,
            lfsr: 0,
        }
    }

    pub fn read_byte(self: &Self, addr: u16) -> u8 {
        match addr {
            NR41 => self.len.get(),
            NR42 => self.volenv.get(),
            NR43 => self.pcounter.get(),
            NR44 => self.counter.get(),
            _ => panic!("ch4 does not handle reads from addr: {}", addr),
        }
    }

    pub fn write_byte(self: &mut Self, addr: u16, data: u8) {
        match addr {
            NR41 => self.len.set(data | 0xC0), // Dont use duty (bit 6-7)
            NR42 => self.volenv.set(data),
            NR43 => self.pcounter.set(data),
            NR44 => {
                let prev_len_enable = self.counter.len_enable;
                self.counter.set(data);

                if self.counter.len_enable
                    && !prev_len_enable
                    && self.len.timer != 0
                    && (self.frame_seq % 2) == 0
                {
                    self.len.decr_len();
                }

                if self.counter.restart {
                    self.on_trigger();
                }
            }
            _ => panic!("ch4 does not handle writes to addr: {}", addr),
        }
    }

    pub fn adv_cycles(self: &mut Self, cycles: usize) {
        self.internal_cycles = self.internal_cycles.wrapping_add(cycles);

        self.clock_polycounter(cycles);

        if self.internal_cycles >= 8192 {
            self.frame_seq = (self.frame_seq + 1) % 8;
            self.internal_cycles = self.internal_cycles.wrapping_sub(8192);

            match self.frame_seq {
                0 | 2 | 4 | 6 => self.clock_length(),
                7 => self.clock_volenv(),
                1 | 3 | 5 => { /* Do Nothing */ }
                _ => panic!(
                    "frame sequencer should not be higher than 7: {}",
                    self.frame_seq
                ),
            }
        }
    }

    pub fn clock_length(self: &mut Self) {
        if self.counter.len_enable && self.len.enable {
            self.len.decr_len();
        }
    }

    pub fn clock_volenv(self: &mut Self) {
        if self.volenv.sweep == 0 {
            return;
        }
        if self.volenv.timer != 0 {
            self.volenv.decr_timer();
        }
    }

    pub fn clock_polycounter(self: &mut Self, cycles: usize) {
        if self.pcounter.decr_timer(cycles) {
            // Tick LFSR

            // Shift right once and set bit 14 to the xor value
            // Bit 15 should always be 0 since its unused
            let xor = (self.lfsr & 0x01) ^ ((self.lfsr >> 1) & 0x01);
            self.lfsr = ((self.lfsr & 0x7FFF) >> 1) | (xor << 14);

            if self.pcounter.width {
                // Unset bit 6 and set it to the xor value
                self.lfsr = (self.lfsr & 0xFFBF) | (xor << 6);
            }
        }
    }

    // TODO: What type is required by SDL Audio?
    pub fn get_output(self: &mut Self) -> f32 {
        if !self.is_ch_enabled() {
            return 0.0;
        }
        let value = (!(self.lfsr & 0x01) as u8) * self.volenv.cur_vol;
        return (f32::from(value) / 7.5) - 1.0;
    }

    pub fn is_ch_enabled(self: &Self) -> bool {
        return self.len.enable;
    }

    fn on_trigger(self: &mut Self) {
        /* Length */
        self.len.reload_timer();

        /* Volume Envelope */
        self.volenv.reload_timer();
        self.volenv.reload_vol();

        /* PCounter */
        self.lfsr = 0x7FFF;
        self.pcounter.reload_timer(); // Not sure on this

        /* Some of Obscure Behaviour (tone_sweep.rs is commented) */
        if self.frame_seq == 6 {
            self.volenv.timer = self.volenv.timer.wrapping_add(1);
        }
        if (self.frame_seq % 2) == 0 {
            if self.len.timer == u32::from(self.len.mask) + 1 && self.counter.len_enable {
                self.len.timer = u32::from(self.len.mask);
            }
        }
    }

    pub fn dmg_init(self: &mut Self) {
        self.len.set(0xFF);
        self.volenv.set(0x00);
        self.pcounter.set(0x00);
        self.counter.set(0xBF);

        self.len.reload_timer();
        self.volenv.reload_timer();
        self.volenv.reload_vol();
        self.pcounter.reload_timer();
    }
}

struct PolyCounter {
    pub shift_freq: u8, // Bit 4-7
    pub width: bool,    // Bit 3 (0 is 15 bits and 1 is 7 bits)
    pub ratio: u8,      // Bit 0-2
    pub freq_timer: u16,
}

impl PolyCounter {
    pub fn new() -> PolyCounter {
        return PolyCounter {
            shift_freq: 0,
            width: false,
            ratio: 0,
            freq_timer: 0,
        };
    }

    pub fn set(self: &mut Self, data: u8) {
        self.shift_freq = (data >> 4) & 0x0F;
        self.width = (data >> 3) & 0x01 == 0x01;
        self.ratio = data & 0x07;
    }

    pub fn get(self: &Self) -> u8 {
        return self.shift_freq << 4 | (self.width as u8) << 3 | self.ratio;
    }

    pub fn decr_timer(self: &mut Self, cycles: usize) -> bool {
        let prev_timer = self.freq_timer;

        self.freq_timer = prev_timer.wrapping_sub(cycles as u16);
        if self.freq_timer == 0 || (self.freq_timer > prev_timer) {
            self.reload_timer();

            /* Obscure Behaviour */
            // Using a noise channel clock shift of 14 or 15 results in the
            // LFSR receiving no clocks. That would require min 131072 T-Cycles
            // for the timer to expire (occurs when ratio equals 0) which is
            // greater than the max u16 value - freq_timer is a u16
            return true && (self.freq_timer != 0);
        }
        return false;
    }

    pub fn reload_timer(self: &mut Self) {
        let frequency =
            (1048576 / u32::from(self.ratio) + 1) / (2_u32.pow(u32::from(self.shift_freq) + 1));

        use crate::cpu::CPU_FREQ;
        self.freq_timer = ((CPU_FREQ as u32) / frequency) as u16; // Minimum is 8 T-cycles
    }
}

struct Counter {
    restart: bool,    // Bit 7 (1 = restart sound)
    len_enable: bool, // Bit 6 (1 = stop output when len in nr41 expires)
}

impl Counter {
    const MASK: u8 = 0x3F;
    pub fn new() -> Counter {
        return Counter {
            restart: false,
            len_enable: false,
        };
    }
    pub fn set(self: &mut Self, data: u8) {
        self.restart = (data >> 7) & 0x01 == 0x01;
        self.len_enable = (data >> 6) & 0x01 == 0x01;
    }
    pub fn get(self: &Self) -> u8 {
        return Self::MASK | ((self.restart as u8) << 7) | ((self.len_enable as u8) << 6);
    }
    pub fn should_restart(self: &Self) -> bool {
        return self.restart;
    }
    pub fn should_stop(self: &Self) -> bool {
        return self.len_enable;
    }
}

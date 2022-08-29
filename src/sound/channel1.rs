use super::DUTY_WAVES;
use super::{Freq, LenPat, VolEnv};
use super::{NR10, NR11, NR12, NR13, NR14};

pub struct Ch1 {
    sweep: Sweep,   // NR10
    lenpat: LenPat, // NR11
    volenv: VolEnv, // NR12
    freq: Freq,     // NR13 and NR14
    frame_seq: u8,  // dictates which channel gets clocked
    internal_cycles: usize,
    duty_pos: usize,
}

impl Ch1 {
    pub fn new() -> Ch1 {
        Ch1 {
            sweep: Sweep::new(),
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
            NR14 => {
                self.freq.set_hi(data);
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
                1 | 5 => { /* Do Nothing */ }
                _ => panic!(
                    "frame sequencer should not be higher than 7: {}",
                    self.frame_seq
                ),
            }
        }
    }

    fn clock_length(self: &mut Self) {
        if self.freq.counter && self.lenpat.internal_enable {
            self.lenpat.decr_len();
        }
    }

    fn clock_sweep(self: &mut Self) {}

    fn clock_volenv(self: &mut Self) {}

    pub fn get_output(self: &Self) -> u8 {
        if !self.is_ch_enabled() {
            return 0;
        }
        return DUTY_WAVES[usize::from(self.lenpat.duty)][self.duty_pos];
    }

    fn on_trigger(self: &mut Self) {
        // TODO: Add the other events that occur on trigger
        self.lenpat.reload_timer(); // Should I only reload if equal to zero?
        self.duty_pos = 0;
        self.freq.reload_timer(2048);
    }

    pub fn is_ch_enabled(self: &Self) -> bool {
        // TODO: Add the other internal enable flags if any
        return self.lenpat.internal_enable;
    }

    pub fn dmg_init(self: &mut Self) {
        self.sweep.set(0x80);
        self.lenpat.set(0xBF);
        self.volenv.set(0xF3);
        self.freq.set_lo(0xFF);
        self.freq.set_hi(0xBF);
        self.freq.timer = 0; // I think
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

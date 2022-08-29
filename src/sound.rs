/*
    Produce sound in 4 different ways:
        Quadrangular wave patterns with sweep and envelope functions (CH1)
        Quadrangular wave patterns with envelope functions (CH2)
        Voluntary wave patterns from wave RAM (CH3)
        White noise with an envelope function (CH4)
*/

mod channel1;
mod channel2;
mod channel3;
mod channel4;

use self::channel1::Ch1;
use self::channel2::Ch2;
use self::channel3::Ch3;
use self::channel4::Ch4;

// Sound
pub const NR10: u16 = 0xFF10;
pub const NR11: u16 = 0xFF11;
pub const NR12: u16 = 0xFF12;
pub const NR13: u16 = 0xFF13;
pub const NR14: u16 = 0xFF14;
pub const NR21: u16 = 0xFF16;
pub const NR22: u16 = 0xFF17;
pub const NR23: u16 = 0xFF18;
pub const NR24: u16 = 0xFF19;
pub const NR30: u16 = 0xFF1A;
pub const NR31: u16 = 0xFF1B;
pub const NR32: u16 = 0xFF1C;
pub const NR33: u16 = 0xFF1D;
pub const NR34: u16 = 0xFF1E;
pub const NR41: u16 = 0xFF20;
pub const NR42: u16 = 0xFF21;
pub const NR43: u16 = 0xFF22;
pub const NR44: u16 = 0xFF23;
pub const NR50: u16 = 0xFF24;
pub const NR51: u16 = 0xFF25;
pub const NR52: u16 = 0xFF26;
pub const PCM12: u16 = 0xFF76;
pub const PCM34: u16 = 0xFF77;

pub const WAVE_RAM_START: u16 = 0xFF30;
pub const WAVE_RAM_END: u16 = 0xFF3F;

pub struct Sound {
    ch1: Ch1,
    ch2: Ch2,
    ch3: Ch3,
    ch4: Ch4,
    nr50: u8, // The rest of these are control so I'll keep them here
    nr51: u8,
    nr52: u8,
    pcm12: u8,
    pcm34: u8,
    wave_ram: [u8; 0x0F],
}

impl Sound {
    pub fn new() -> Sound {
        return Sound {
            ch1: Ch1::new(),
            ch2: Ch2::new(),
            ch3: Ch3::new(),
            ch4: Ch4::new(),
            nr50: 0,
            nr51: 0,
            nr52: 0,
            pcm12: 0,
            pcm34: 0,
            wave_ram: [0xFF; 0x0F],
        };
    }

    pub fn read_byte(self: &Self, addr: u16) -> u8 {
        return match addr {
            NR10 | NR11 | NR12 | NR13 | NR14 => self.ch1.read_byte(addr),
            NR21 | NR22 | NR23 | NR24 => self.ch2.read_byte(addr),
            NR30 | NR31 | NR32 | NR33 | NR34 => self.ch3.read_byte(addr),
            NR41 | NR42 | NR43 | NR44 => self.ch4.read_byte(addr),
            NR50 => self.nr50,
            NR51 => self.nr51,
            NR52 => self.nr52,
            PCM12 => self.pcm12,
            PCM34 => self.pcm34,
            WAVE_RAM_START..=WAVE_RAM_END => self.wave_ram[usize::from(addr - WAVE_RAM_START)],
            _ => panic!("Sound does not handle reads from addr {}", addr),
        };
    }

    pub fn write_byte(self: &mut Self, addr: u16, data: u8) {
        match addr {
            NR10 | NR11 | NR12 | NR13 | NR14 => self.ch1.write_byte(addr, data),
            NR21 | NR22 | NR23 | NR24 => self.ch2.write_byte(addr, data),
            NR30 | NR31 | NR32 | NR33 | NR34 => self.ch3.write_byte(addr, data),
            NR41 | NR42 | NR43 | NR44 => self.ch4.write_byte(addr, data),
            NR50 => self.nr50 = data,
            NR51 => self.nr51 = data,
            NR52 => self.nr52 = (data & 0x80) | 0x70 | (self.nr52 & 0x0F),
            PCM12 => return,
            PCM34 => return,
            WAVE_RAM_START..=WAVE_RAM_END => {
                self.wave_ram[usize::from(addr - WAVE_RAM_START)] = data
            }
            _ => panic!("Sound does not handle writes to addr {}", addr),
        };
    }

    pub fn adv_cycles(self: &mut Self, cycles: usize) {
        /*
            A timer generates an output clock every N input clocks,
            where N is the timer's period. If a timer's rate is given
            as a frequency, its period is 4194304/frequency in Hz.
            Each timer has an internal counter that is decremented on
            each input clock. When the counter becomes zero, it is
            reloaded with the period and an output clock is generated.
        */
        self.ch1.adv_cycles(cycles);
        self.ch2.adv_cycles(cycles);
        self.ch3.adv_cycles(cycles);
        self.ch4.adv_cycles(cycles);
    }

    pub fn dmg_init(self: &mut Self) {
        // Sound
        self.ch1.dmg_init();
        self.ch2.dmg_init();
        self.ch3.dmg_init();
        self.ch4.dmg_init();
        self.nr50 = 0x77;
        self.nr51 = 0xF3;
        self.nr52 = 0xF1;
    }
}

struct LenPat {
    pub duty: u8,   // Bit 6-7
    pub length: u8, // Bit 0-5
    pub timer: u8,
    pub internal_enable: bool,
}

impl LenPat {
    const MASK: u8 = 0x3F;

    pub fn new() -> LenPat {
        return LenPat {
            duty: 0,
            length: 0,
            timer: 0,
            internal_enable: false,
        };
    }

    pub fn dmg_init(self: &mut Self) {
        self.duty = (0xBF >> 6) & 0x03;
        self.length = 0xBF & 0x3F;
    }

    pub fn set(self: &mut Self, data: u8) {
        self.duty = (data >> 6) & 0x03;
        self.length = data & 0x3F;
        if self.timer == 0 {
            // TODO: Find out if I should I reload only if it equals 0
            // or reload on every write to the register.
            self.timer = 64 - self.length;
            self.internal_enable = true;
        }
    }

    pub fn get(self: &Self) -> u8 {
        return Self::MASK | self.duty << 6 | self.length;
    }

    pub fn decr_len(self: &mut Self) {
        self.length = self.length.wrapping_sub(1);
        if self.length == 0x00 || self.length > 64 {
            self.length = 0;
            self.internal_enable = false;
        }
    }
}

struct VolEnv {
    pub initial_vol: u8, // Bit 4-7 (0 is no sound)
    pub env_dir: bool,   // Bit 3 (1 is incr)
    pub env_swp: u8,     // Bit 0-2
}

impl VolEnv {
    pub fn new() -> VolEnv {
        return VolEnv {
            initial_vol: 0,
            env_dir: false,
            env_swp: 0,
        };
    }
    pub fn set(self: &mut Self, data: u8) {
        self.initial_vol = (data >> 4) & 0x0F;
        self.env_dir = (data >> 3) & 0x01 == 0x01;
        self.env_swp = data & 0x07;
    }
    pub fn get(self: &Self) -> u8 {
        return self.initial_vol << 4 | (self.env_dir as u8) << 3 | self.env_swp;
    }
    pub fn is_silent(self: &Self) -> bool {
        return self.initial_vol == 0;
    }
    pub fn calc_step(self: &Self) -> f32 {
        return (self.env_swp as f32) / 64.;
    }
    pub fn should_stop(self: &Self) -> bool {
        return self.env_swp == 0;
    }
}

// Frequency but not really
struct Freq {
    pub initial: bool, // Bit 7 (1 = restart)
    pub counter: bool, // Bit 6 (1 = Stop output when length in NR11 expires)
    pub hi: u8,        // Bit 0-2
    pub lo: u8,        // Bit 0-7
    pub internal_clock: usize,
}

impl Freq {
    const MASK_LO: u8 = 0xFF;
    const MASK_HI: u8 = 0xBF;

    pub fn new() -> Freq {
        return Freq {
            initial: false,
            counter: false,
            hi: 0,
            lo: 0,
            internal_clock: 0,
        };
    }

    pub fn dmg_init(self: &mut Self) {
        self.set_lo(0xFF);
        self.set_hi(0xBF);
        self.internal_clock = (2048 - self.get_full() as usize) * 4;
    }

    pub fn set_lo(self: &mut Self, data: u8) {
        self.lo = data;
        // Should internal_clock update when lo is written to
    }

    pub fn set_hi(self: &mut Self, data: u8) {
        self.initial = (data >> 7) & 0x01 == 0x01;
        self.counter = (data >> 6) & 0x01 == 0x01;
        self.hi = data & 0x07;
        // Should internal_clock update when hi is written to
    }

    pub fn get_lo(self: &Self) -> u8 {
        return Self::MASK_LO | self.lo;
    }

    pub fn get_hi(self: &Self) -> u8 {
        return Self::MASK_HI | (self.initial as u8) << 7 | (self.counter as u8) << 6 | self.hi;
    }

    pub fn get_full(self: &Self) -> u32 {
        return (u32::from(self.hi) << 8) | u32::from(self.lo);
    }

    // Decrement the internal clock and return if it hit 0
    pub fn decr_clock(self: &mut Self, cycles: usize) -> bool {
        self.internal_clock = self.internal_clock.wrapping_sub(cycles);

        if self.internal_clock == 0 || self.internal_clock > 8192 {
            // TODO: Increment the wave duty position by 1
            self.internal_clock = (2048 - self.get_full() as usize) * 4;
            return true;
        }
        return false;
    }
}

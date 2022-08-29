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

pub const DUTY_WAVES: [[u8; 8]; 4] = [
    [0, 0, 0, 0, 0, 0, 0, 1],
    [0, 0, 0, 0, 0, 0, 1, 1],
    [0, 0, 0, 0, 1, 1, 1, 1],
    [1, 1, 1, 1, 1, 1, 0, 0],
];

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
    pub timer: u32,
    pub internal_enable: bool,
    mask: u8,
}

impl LenPat {
    pub fn new(mask: u8) -> LenPat {
        return LenPat {
            duty: 0, // Not used by ch3 and ch4
            length: 0,
            timer: 0,
            internal_enable: false,
            mask: mask, // 0x3F for ch1, ch2, and ch4, 0xFF for ch3
        };
    }

    pub fn set(self: &mut Self, data: u8) {
        self.duty = (data >> 6) & 0x03;
        self.length = data & self.mask;
        // Should I reload the timer here or only on the trigger event?
    }

    pub fn get(self: &Self) -> u8 {
        return self.mask | self.duty << 6 | self.length;
    }

    pub fn decr_len(self: &mut Self) {
        self.timer = self.timer.wrapping_sub(1);
        if self.timer == 0x00 || self.timer > u32::from(self.mask) + 1 {
            self.timer = 0;
            self.internal_enable = false;
        }
    }

    pub fn reload_timer(self: &mut Self) {
        if self.timer == 0 {
            // TODO: Find out if I should reload only if it equals 0
            self.timer = u32::from(self.mask - self.length) + 1;
            self.internal_enable = true;
        }
    }
}

struct VolEnv {
    pub initial_vol: u8, // Bit 4-7 (0 is no sound)
    pub dir_up: bool,    // Bit 3 (1 is incr)
    pub sweep: u8,       // Bit 0-2
    pub timer: u32,
    pub cur_vol: u8,
}

impl VolEnv {
    pub fn new() -> VolEnv {
        return VolEnv {
            initial_vol: 0,
            dir_up: false,
            sweep: 0,
            timer: 0,
            cur_vol: 0,
        };
    }

    pub fn set(self: &mut Self, data: u8) {
        self.initial_vol = (data >> 4) & 0x0F;
        self.dir_up = (data >> 3) & 0x01 == 0x01;
        self.sweep = data & 0x07;
    }

    pub fn get(self: &Self) -> u8 {
        return self.initial_vol << 4 | (self.dir_up as u8) << 3 | self.sweep;
    }

    pub fn decr_timer(self: &mut Self) -> bool {
        self.timer = self.timer.wrapping_sub(1);

        if self.timer == 0 {
            self.reload_timer();
            self.adjust_vol();
            return true;
        }
        return false;
    }

    pub fn reload_timer(self: &mut Self) {
        self.timer = if self.sweep == 0 {
            8 // Obscure behaviour - https://gbdev.gg8.se/wiki/articles/Gameboy_sound_hardware
        } else {
            u32::from(self.sweep)
        };
    }

    pub fn adjust_vol(self: &mut Self) {
        // This if statement makes sure the value is always between 0 and 15 as we
        // if it equals 15 we will only enter if the dir is downwards. And if it equals
        // 0, it will only enter if the direction is upwards.
        if (self.cur_vol < 0x0F && self.dir_up) || (self.cur_vol > 0 && !self.dir_up) {
            self.cur_vol = if self.dir_up {
                self.cur_vol.wrapping_add(1)
            } else {
                self.cur_vol.wrapping_sub(1)
            }
        }
    }

    pub fn reload_vol(self: &mut Self) {
        self.cur_vol = self.initial_vol;
    }
}

// Frequency but not really
struct Freq {
    pub initial: bool, // Bit 7 (1 = restart)
    pub counter: bool, // Bit 6 (1 = Stop output when length in NR11 expires)
    pub hi: u8,        // Bit 0-2
    pub lo: u8,        // Bit 0-7
    pub timer: usize,
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
            timer: 0,
        };
    }

    pub fn set_lo(self: &mut Self, data: u8) {
        self.lo = data;
    }

    pub fn set_hi(self: &mut Self, data: u8) {
        self.initial = (data >> 7) & 0x01 == 0x01;
        self.counter = (data >> 6) & 0x01 == 0x01;
        self.hi = data & 0x07;
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
    fn decr_timer(self: &mut Self, cycles: usize, max_cycles: usize, max_reload: usize) -> bool {
        self.timer = self.timer.wrapping_sub(cycles);

        if self.timer == 0 || self.timer > max_cycles {
            self.reload_timer(max_reload);
            return true;
        }
        return false;
    }

    // Make sure this will apply for ch3 and ch4 as well
    pub fn reload_timer(self: &mut Self, max_reload: usize) {
        self.timer = (max_reload - self.get_full() as usize) * 4;
    }
}

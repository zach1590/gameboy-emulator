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
            _ => panic!("Sound does not handle writes to addr {}", addr),
        };
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

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
    nr10: u8,
    nr11: u8,
    nr12: u8,
    nr13: u8,
    nr14: u8,
    nr21: u8,
    nr22: u8,
    nr23: u8,
    nr24: u8,
    nr30: u8,
    nr31: u8,
    nr32: u8,
    nr33: u8,
    nr34: u8,
    nr41: u8,
    nr42: u8,
    nr43: u8,
    nr44: u8,
    nr50: u8,
    nr51: u8,
    nr52: u8,
    pcm12: u8,
    pcm34: u8,
}

impl Sound {
    pub fn new() -> Sound {
        return Sound {
            nr10: 0,
            nr11: 0,
            nr12: 0,
            nr13: 0,
            nr14: 0,
            nr21: 0,
            nr22: 0,
            nr23: 0,
            nr24: 0,
            nr30: 0,
            nr31: 0,
            nr32: 0,
            nr33: 0,
            nr34: 0,
            nr41: 0,
            nr42: 0,
            nr43: 0,
            nr44: 0,
            nr50: 0,
            nr51: 0,
            nr52: 0,
            pcm12: 0,
            pcm34: 0,
        };
    }

    pub fn read_byte(self: &Self, addr: u16) -> u8 {
        return match addr {
            NR10 => self.nr10,
            NR11 => self.nr11 | 0x3F,
            NR12 => self.nr12,
            NR13 => self.nr13,
            NR14 => self.nr14 | 0x83,
            NR21 => self.nr21 | 0x3F,
            NR22 => self.nr22,
            NR23 => self.nr23 | 0xFF,
            NR24 => self.nr24 | 0x83,
            NR30 => self.nr30,
            NR31 => self.nr31 | 0xFF,
            NR32 => self.nr32,
            NR33 => self.nr33 | 0xFF,
            NR34 => self.nr34 | 0x83,
            NR41 => self.nr41 | 0x3F,
            NR42 => self.nr42,
            NR43 => self.nr43,
            NR44 => self.nr44 | 0x80,
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
            NR10 => self.nr10 = data | 0x80,
            NR11 => self.nr11 = data,
            NR12 => self.nr12 = data,
            NR13 => self.nr13 = data,
            NR14 => self.nr14 = data,
            NR21 => self.nr21 = data,
            NR22 => self.nr22 = data,
            NR23 => self.nr23 = data,
            NR24 => self.nr24 = data | 0x38,
            NR30 => self.nr30 = data | 0x7F,
            NR31 => self.nr31 = data,
            NR32 => self.nr32 = data | 0x9F,
            NR33 => self.nr33 = data,
            NR34 => self.nr34 = data | 0x38,
            NR41 => self.nr41 = data | 0xC0,
            NR42 => self.nr42 = data,
            NR43 => self.nr43 = data,
            NR44 => self.nr44 = data | 0x3F,
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
        self.nr10 = 0x80;
        self.nr11 = 0xBF;
        self.nr12 = 0xF3;
        self.nr13 = 0xFF;
        self.nr14 = 0xBF;
        self.nr21 = 0x3F;
        self.nr22 = 0x00;
        self.nr23 = 0xFF;
        self.nr24 = 0xBF;
        self.nr30 = 0x7F;
        self.nr31 = 0xFF;
        self.nr32 = 0x9F;
        self.nr33 = 0xFF;
        self.nr34 = 0xBF;
        self.nr41 = 0xFF;
        self.nr42 = 0x00;
        self.nr43 = 0x00;
        self.nr44 = 0xBF;
        self.nr50 = 0x77;
        self.nr51 = 0xF3;
        self.nr52 = 0xF1;
    }
}

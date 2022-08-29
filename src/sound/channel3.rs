use super::Freq;
#[allow(unused_imports)]
use super::{NR30, NR31, NR32, NR33, NR34, WAVE_RAM_END, WAVE_RAM_START};

pub struct Ch3 {
    is_on: bool,      // NR30 (1 is playback)
    len: u8,          // NR31 TODO: Change this to use lenpat struct and just dont use the duty
    output_level: u8, // NR32 (Only bits 5-6 matter)
    freq: Freq,       // NR33 and NR 34
}

impl Ch3 {
    pub fn new() -> Ch3 {
        Ch3 {
            is_on: false,
            len: 0,
            output_level: 0,
            freq: Freq::new(),
        }
    }

    pub fn read_byte(self: &Self, addr: u16) -> u8 {
        match addr {
            NR30 => (self.is_on as u8) << 7 | 0x7F,
            NR31 => self.len | 0xFF,
            NR32 => (self.output_level << 5) | 0x9F,
            NR33 => self.freq.get_lo(),
            NR34 => self.freq.get_hi(),
            _ => panic!("ch3 does not handle reads from addr: {}", addr),
        }
    }

    pub fn write_byte(self: &mut Self, addr: u16, data: u8) {
        match addr {
            NR30 => self.is_on = (data >> 7) & 0x01 == 0x01,
            NR31 => self.len = data,
            NR32 => self.output_level = (data >> 5) & 0x03,
            NR33 => self.freq.set_lo(data),
            NR34 => self.freq.set_hi(data),
            _ => panic!("ch3 does not handle writes to addr: {}", addr),
        }
    }

    pub fn adv_cycles(self: &mut Self, _cycles: usize) {}

    pub fn get_output_as_percent(self: &Self) -> u8 {
        return match self.output_level {
            0x00 => 0,
            0x01 => 100,
            0x02 => 50,
            0x03 => 25,
            _ => panic!(
                "output level should not be higher than 3, curr: {}",
                self.output_level
            ),
        };
    }

    pub fn get_output_as_shift_right(self: &Self) -> u8 {
        return match self.output_level {
            0x00 => 0xFF,
            0x01 => 0,
            0x02 => 1,
            0x03 => 2,
            _ => panic!(
                "output level should not be higher than 3, curr: {}",
                self.output_level
            ),
        };
    }

    pub fn dmg_init(self: &mut Self) {
        self.is_on = false; // (0x7F >> 7) & 0x01 == 0x01
        self.len = 0xFF;
        self.output_level = 0x9F;
        self.freq.set_lo(0xFF);
        self.freq.set_hi(0xBF);
    }
}

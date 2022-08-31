use super::{Freq, LenPat};
#[allow(unused_imports)]
use super::{NR30, NR31, NR32, NR33, NR34, WAVE_RAM_END, WAVE_RAM_START};

pub struct Ch3 {
    is_on: bool,      // NR30 (1 is playback)
    len: LenPat,      // NR31 (Uses full 8 bits for length and no duty)
    output_level: u8, // NR32 (Only bits 5-6 matter)
    freq: Freq,       // NR33 and NR 34
    frame_seq: u8,
    wave_pos: usize,
    internal_cycles: usize,
    wave_ram: [u8; 0x0F], // 4 Bit samples (2 Samples per Byte)
}

impl Ch3 {
    pub fn new() -> Ch3 {
        Ch3 {
            is_on: false,
            len: LenPat::new(0xFF),
            output_level: 0,
            freq: Freq::new(),
            frame_seq: 0,
            wave_pos: 0,
            internal_cycles: 0,
            wave_ram: [0xFF; 0x0F],
        }
    }

    pub fn read_byte(self: &Self, addr: u16) -> u8 {
        match addr {
            NR30 => (self.is_on as u8) << 7 | 0x7F,
            NR31 => 0xFF,
            NR32 => (self.output_level << 5) | 0x9F,
            NR33 => self.freq.get_lo(),
            NR34 => self.freq.get_hi(),
            WAVE_RAM_START..=WAVE_RAM_END => self.wave_ram[usize::from(addr - WAVE_RAM_START)],
            _ => panic!("ch3 does not handle reads from addr: {}", addr),
        }
    }

    pub fn write_byte(self: &mut Self, addr: u16, data: u8) {
        match addr {
            NR30 => self.is_on = (data >> 7) & 0x01 == 0x01,
            NR31 => self.len.set(data),
            NR32 => self.output_level = (data >> 5) & 0x03,
            NR33 => self.freq.set_lo(data),
            NR34 => {
                let prev_len_enable = self.freq.len_enable;
                self.freq.set_hi(data);

                if self.freq.len_enable
                    && !prev_len_enable
                    && self.len.timer != 0
                    && (self.frame_seq % 2) == 0
                {
                    self.len.decr_len();
                }

                if self.freq.initial {
                    self.on_trigger();
                }
            }
            WAVE_RAM_START..=WAVE_RAM_END => {
                if !self.is_on {
                    self.wave_ram[usize::from(addr - WAVE_RAM_START)] = data
                }
            }
            _ => panic!("ch3 does not handle writes to addr: {}", addr),
        }
    }

    pub fn adv_cycles(self: &mut Self, cycles: usize) {
        self.internal_cycles = self.internal_cycles.wrapping_add(cycles);

        if self.freq.decr_timer(cycles) {
            self.wave_pos = (self.wave_pos + 1) % 32; // 0 - 31
        }

        if self.internal_cycles >= 8192 {
            self.frame_seq = (self.frame_seq + 1) % 8;
            self.internal_cycles = self.internal_cycles.wrapping_sub(8192);

            match self.frame_seq {
                0 | 2 | 4 | 6 => self.clock_length(),
                1 | 3 | 5 | 7 => { /* Do Nothing */ }
                _ => panic!(
                    "frame sequencer should not be higher than 7: {}",
                    self.frame_seq
                ),
            }
        }
    }

    pub fn clock_length(self: &mut Self) {
        if self.freq.len_enable && self.len.enable {
            self.len.decr_len();
        }
    }

    pub fn get_output(self: &Self) -> u8 {
        if !self.is_on {
            return 0;
        }

        let vol_shift = self.get_output_as_shift_right();

        // wave ram is 15 bytes, every 4 bits is a sample
        let wave_index = self.wave_pos / 2;

        // The upper 4 bits should be first, so if this calculation
        // returns 0, then we need to take the upper 4 bits of the index
        let sample_num = self.wave_pos % 2; // 1 means the low 4 bits

        return if sample_num == 0 {
            (self.wave_ram[wave_index] & 0xF0 >> 4) >> vol_shift
        } else {
            self.wave_ram[wave_index] & 0x0F >> vol_shift
        };
    }

    pub fn get_output_as_shift_right(self: &Self) -> u8 {
        return match self.output_level {
            0x00 => 4,
            0x01 => 0,
            0x02 => 1,
            0x03 => 2,
            _ => panic!(
                "output level should not be higher than 3, curr: {}",
                self.output_level
            ),
        };
    }

    fn on_trigger(self: &mut Self) {
        /* Length */
        self.len.reload_timer();

        /* Frequency */
        self.wave_pos = 0;
        self.freq.reload_timer();
    }

    pub fn dmg_init(self: &mut Self) {
        self.is_on = false; // (0x7F >> 7) & 0x01 == 0x01
        self.len.set(0xFF);
        self.output_level = 0x9F;
        self.freq.set_lo(0xFF);
        self.freq.set_hi(0xBF);
    }
}

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
    wave_ram: [u8; 16], // 4 Bit samples (2 Samples per Byte)
    sample_buffer: u8,
}

impl Ch3 {
    pub fn new() -> Ch3 {
        Ch3 {
            is_on: false,
            len: LenPat::new(0xFF),
            output_level: 0,
            freq: Freq::new(64),
            frame_seq: 0,
            wave_pos: 0,
            internal_cycles: 0,
            wave_ram: [
                0x84, 0x40, 0x43, 0xAA, 0x2D, 0x78, 0x92, 0x3C, 0x60, 0x59, 0x59, 0xB0, 0x34, 0xB8,
                0x2E, 0xDA,
                // These are somewhat random, this just one possible set
            ],
            sample_buffer: 0,
        }
    }

    pub fn read_byte(self: &Self, addr: u16) -> u8 {
        match addr {
            NR30 => ((self.is_on as u8) << 7) | 0x7F,
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
            NR30 => self.is_on = ((data >> 7) & 0x01) == 0x01,
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

        // Check if channel enabled?

        if self.freq.decr_timer(cycles) {
            self.wave_pos = (self.wave_pos + 1) % 32; // 0 - 31
            self.load_sample_buffer();
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
        if self.freq.len_enable && (self.len.timer != 0) {
            self.len.decr_len();
        }
    }

    fn load_sample_buffer(self: &mut Self) {
        // wave ram is 15 bytes, every 4 bits is a sample
        let wave_index = self.wave_pos / 2;

        // The upper 4 bits should be first, so if this calculation
        // returns 0, then we need to take the upper 4 bits of the index
        let sample_num = self.wave_pos % 2; // 1 means the low 4 bits

        self.sample_buffer = if sample_num == 0 {
            self.wave_ram[wave_index] & 0xF0 >> 4
        } else {
            self.wave_ram[wave_index] & 0x0F
        };
    }

    pub fn get_output(self: &Self) -> f32 {
        if !self.is_dac_enabled() || !self.is_ch_enabled() {
            return 0.0;
        }

        let vol_shift = self.get_output_as_shift_right();

        let value = self.sample_buffer >> vol_shift;

        return (f32::from(value) / 7.5) - 1.0;
    }

    pub fn get_output_as_shift_right(self: &Self) -> u8 {
        return match self.output_level {
            0x00 => 4, // mute
            0x01 => 0, // 100%
            0x02 => 1, // 50%
            0x03 => 2, // 25%
            _ => panic!(
                "output level should not be higher than 3, curr: {}",
                self.output_level
            ),
        };
    }

    pub fn is_dac_enabled(self: &Self) -> bool {
        return self.is_on;
    }
    pub fn is_ch_enabled(self: &Self) -> bool {
        return self.len.timer != 0;
    }
    pub fn is_counter_off(self: &Self) -> bool {
        return !self.freq.len_enable;
    }

    fn on_trigger(self: &mut Self) {
        /* Length */
        self.len.reload_timer();

        /* Frequency */
        self.wave_pos = 0;
        self.freq.reload_timer();
    }

    pub fn restart(self: &mut Self) {
        // From Pandocs:
        // When restarting CH3, it resumes playing the last 4-bit sample
        // it read from wave RAM, or 0 if no sample has been read since
        // APU reset (Value from sample buffer will be maintained).
        // After the latched sample completes, it starts
        // with the second sample in wave RAM (low 4 bits of $FF30).
        self.frame_seq = 7;
        self.wave_pos = 0; // It will increment to the next sample (second)
    }

    pub fn clear(self: &mut Self) {
        self.len.set(0);
        self.is_on = false;
        self.output_level = 0;
        self.freq.set_lo(0);
        self.freq.set_hi(0);
    }

    pub fn dmg_init(self: &mut Self) {
        self.is_on = false; // (0x7F >> 7) & 0x01 == 0x01
        self.len.set(0xFF);
        self.output_level = 0x9F;
        self.freq.set_lo(0xFF);
        self.freq.set_hi(0xBF);

        self.frame_seq = 7;
        self.wave_pos = 31; // So that first increment will load sample 0
    }
}

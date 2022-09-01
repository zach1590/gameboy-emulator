/*
    Produce sound in 4 different ways:
        Quadrangular wave patterns with sweep and envelope functions (CH1)
        Quadrangular wave patterns with envelope functions (CH2)
        Voluntary wave patterns from wave RAM (CH3)
        White noise with an envelope function (CH4)
*/

mod channel3;
mod channel4;
mod tone_sweep;

use self::channel3::Ch3;
use self::channel4::Ch4;
use self::tone_sweep::Tone;

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
    ch1: Tone,
    ch2: Tone,
    ch3: Ch3,
    ch4: Ch4,
    nr50: ChControl,
    nr51: u8,
    nr52: u8,
    pcm12: u8,
    pcm34: u8,
}

impl Sound {
    pub fn new() -> Sound {
        return Sound {
            ch1: Tone::new().with_sweep(),
            ch2: Tone::new(),
            ch3: Ch3::new(),
            ch4: Ch4::new(),
            nr50: ChControl::new(),
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
            NR50 => self.nr50.get(),
            NR51 => self.nr51,
            NR52 => self.nr52,
            PCM12 => self.pcm12,
            PCM34 => self.pcm34,
            WAVE_RAM_START..=WAVE_RAM_END => self.ch3.read_byte(addr),
            _ => panic!("Sound does not handle reads from addr {}", addr),
        };
    }

    pub fn write_byte(self: &mut Self, addr: u16, data: u8) {
        match addr {
            NR10 | NR11 | NR12 | NR13 | NR14 => self.ch1.write_byte(addr, data),
            NR21 | NR22 | NR23 | NR24 => self.ch2.write_byte(addr, data),
            NR30 | NR31 | NR32 | NR33 | NR34 => self.ch3.write_byte(addr, data),
            NR41 | NR42 | NR43 | NR44 => self.ch4.write_byte(addr, data),
            NR50 => self.nr50.set(data),
            NR51 => self.nr51 = data,
            NR52 => {
                self.nr52 = (data & 0x80) | 0x70 | (self.nr52 & 0x0F);
                // Reading
                // from $FF26(NR52) while the audio processing unit is disabled will yield the
                // values last written into the unused bits (0x70), all other bits are 0.
                // If disabled then sound registers $FF10-$FF2F cannot be accessed
            }
            PCM12 => return,
            PCM34 => return,
            WAVE_RAM_START..=WAVE_RAM_END => self.ch3.write_byte(addr, data),
            _ => panic!("Sound does not handle writes to addr {}", addr),
        };
    }

    pub fn adv_cycles(self: &mut Self, cycles: usize) {
        self.ch1.adv_cycles(cycles);
        self.ch2.adv_cycles(cycles);
        self.ch3.adv_cycles(cycles);
        self.ch4.adv_cycles(cycles);
    }

    fn get_channel_outputs(self: &mut Self) {
        // Each output will be a value (representing voltage) from -1.0 to 1.0
        let ch_outputs = [
            self.ch1.get_output(),
            self.ch2.get_output(),
            self.ch3.get_output(),
            self.ch4.get_output(),
        ];
    }

    // Use NR51 to sum the channel dac outputs into 2 outputs for left and right
    fn mixer(self: &Self, ch_outputs: [f32; 4]) -> (f32, f32) {
        let mut left = 0.0; // SO2?
        let mut right = 0.0; // SO1?

        for i in 0..=3 {
            if (self.nr51 >> i) & 0x01 == 0x01 {
                right += ch_outputs[i];
            }
        }

        for i in 0..=3 {
            if (self.nr51 >> (i + 4)) & 0x01 == 0x01 {
                left += ch_outputs[i];
            }
        }

        return (left, right);
    }

    // Mutiply the signals by volume + 1
    // Thus the output cannot be 0
    fn amplifier(self: &Self, left: f32, right: f32) -> (f32, f32) {
        return (
            left * f32::from(self.nr50.so2_output + 1),
            right * f32::from(self.nr50.so1_output + 1),
        );
    }

    pub fn dmg_init(self: &mut Self) {
        // Sound
        self.ch1.dmg_init();
        self.ch2.dmg_init();
        self.ch3.dmg_init();
        self.ch4.dmg_init();
        self.nr50.set(0x77);
        self.nr51 = 0xF3;
        self.nr52 = 0xF1;
    }
}

// NR50
struct ChControl {
    // The vin signal is referring to a signal received from the
    // game cartridge bus.
    // SO2 and SO1 outputs are master volume that get multiplied
    // to the left and right channels
    pub output_so2_vin_enable: bool, // Bit 7
    pub so2_output: u8,              // Bit 4-6
    pub output_so1_vin_enable: bool, // Bit 3
    pub so1_output: u8,              // Bit 0-2
}

impl ChControl {
    pub fn new() -> ChControl {
        return ChControl {
            output_so2_vin_enable: false,
            so2_output: 0x00,
            output_so1_vin_enable: false,
            so1_output: 0x00,
        };
    }
    pub fn set(self: &mut Self, data: u8) {
        self.output_so2_vin_enable = (data >> 7) & 0x01 == 0x01;
        self.so2_output = (data >> 4) & 0x07;
        self.output_so1_vin_enable = (data >> 3) & 0x01 == 0x01;
        self.so1_output = data & 0x07;
    }

    pub fn get(self: &Self) -> u8 {
        return (self.output_so2_vin_enable as u8) << 7
            | self.so2_output << 4
            | (self.output_so1_vin_enable as u8) << 3
            | self.so1_output;
    }
}

struct LenPat {
    pub duty: u8,   // Bit 6-7
    pub length: u8, // Bit 0-5  (This is not a reload value)
    pub timer: u32,
    pub enable: bool,
    mask: u8,
}

impl LenPat {
    pub fn new(mask: u8) -> LenPat {
        return LenPat {
            duty: 0, // Not used by ch3 and ch4
            length: 0,
            timer: 0,
            enable: false,
            mask: mask, // 0x3F for ch1, ch2, and ch4, 0xFF for ch3
        };
    }

    pub fn set(self: &mut Self, data: u8) {
        self.duty = (data >> 6) & 0x03;
        self.length = data & self.mask;
        self.timer = u32::from(self.mask - self.length) + 1;
    }

    pub fn get(self: &Self) -> u8 {
        return self.mask | self.duty << 6 | self.length;
    }

    pub fn decr_len(self: &mut Self) -> bool {
        self.timer = self.timer.wrapping_sub(1);

        if self.timer == 0x00 || self.timer > u32::from(self.mask) + 1 {
            self.timer = 0;
            self.enable = false;
            return true;
        }
        return false;
    }

    // Only reload on trigger
    pub fn reload_timer(self: &mut Self) {
        if self.timer == 0 {
            self.timer = u32::from(self.mask) + 1;
        }
        self.enable = true;
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

    // True, if any of the top 5 bits are set
    pub fn is_dac_enabled(self: &Self) -> bool {
        return (self.cur_vol != 0) || self.dir_up;
    }
}

// Frequency but not really
struct Freq {
    pub initial: bool,    // Bit 7 (1 = restart)
    pub len_enable: bool, // Bit 6 (1 = Stop output when length in NR11 expires)
    pub hi: u8,           // Bit 0-2
    pub lo: u8,           // Bit 0-7
    pub timer: u32,
    cycle_multiplier: u8,
}

impl Freq {
    const MASK_LO: u8 = 0xFF;
    const MASK_HI: u8 = 0xBF;

    pub fn new(multiple: u8) -> Freq {
        return Freq {
            initial: false,
            len_enable: false,
            hi: 0,
            lo: 0,
            timer: 0,
            cycle_multiplier: multiple,
        };
    }

    pub fn set_lo(self: &mut Self, data: u8) {
        self.lo = data;
    }

    pub fn set_hi(self: &mut Self, data: u8) {
        self.initial = (data >> 7) & 0x01 == 0x01;
        self.len_enable = (data >> 6) & 0x01 == 0x01;
        self.hi = data & 0x07;
    }

    pub fn get_lo(self: &Self) -> u8 {
        return Self::MASK_LO | self.lo;
    }

    pub fn get_hi(self: &Self) -> u8 {
        return Self::MASK_HI | (self.initial as u8) << 7 | (self.len_enable as u8) << 6 | self.hi;
    }

    pub fn get_full(self: &Self) -> u16 {
        return (u16::from(self.hi) << 8) | u16::from(self.lo);
    }

    pub fn set_full(self: &mut Self, new_freq: u16) {
        self.lo = new_freq as u8;
        self.hi = ((new_freq >> 8) as u8) & 0x07;
    }

    // Decrement the internal clock and return if it hit 0
    fn decr_timer(self: &mut Self, cycles: usize) -> bool {
        let prev = self.timer;
        self.timer = self.timer.wrapping_sub(cycles as u32);

        if self.timer == 0 || self.timer > prev {
            self.reload_timer();
            return true;
        }
        return false;
    }

    // Calculated differently depending on documentation source
    // From Pan Docs:
    // Channel 1 and 2 uses - Frequency = 131072/(2048-x) Hz
    // Channel 3 uses - Frequency = 65536/(2048-x) Hz
    // Num of cycles can then be calculated from 4194304/frequency
    // Or just calculate 4194304/C (32 or 64) at beginning and multiply by (2048-x)
    // gbdev.gg8.se says  (2048-frequency)*2 for ch3 and *4 for channel 1/2 which doesnt match
    pub fn reload_timer(self: &mut Self) {
        self.timer = (2048 - self.get_full() as u32) * u32::from(self.cycle_multiplier);
    }
}

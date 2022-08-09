const ROM_BANK_SIZE: usize = 16_384;
const RAM_BANK_SIZE: usize = 8_192;
const CPU_CYCLES_PER_RTC_CYCLE: usize = CPU_FREQ / RTC_FREQ;

use super::battery::Battery;
use super::mbc_timer::{MbcTimer, RTC_FREQ};
use crate::cpu::CPU_FREQ;
use crate::mbc::Mbc;

/*
    Max 2MByte ROM  (128 Banks)
    Max 32KByte RAM (4 Banks)

    0x0000 - 0x3FFF Bank 0 non-swappable
    0x4000 - 0x7FFF Bank 1 - 127 swappable
*/

pub struct Mbc3 {
    rom: Vec<u8>, // 0x0000 - 0x7FFF
    ram: Vec<u8>, // 0xA000 - 0xBFFF
    max_rom_banks: usize,
    max_ram_banks: usize,
    ram_and_timer_enable: u8,    // Value of 0xA will enable it
    rom_bank_num: usize,         // 1 - 127 (0x01 - 0x7F)
    ram_bank_num_and_rtc: usize, // 0 - 3
    latch_reg: u8,
    battery: Option<Battery>,
    timer: Option<MbcTimer>,
    latched_timer: Option<MbcTimer>,
}

impl Mbc3 {
    pub fn new() -> Mbc3 {
        Mbc3 {
            rom: Vec::new(),
            ram: Vec::new(),
            max_rom_banks: 0,
            max_ram_banks: 0,
            ram_and_timer_enable: 0x00,
            rom_bank_num: 0x01,
            ram_bank_num_and_rtc: 0x00,
            latch_reg: 0x00,
            battery: None,
            timer: None,
            latched_timer: None,
        }
    }

    fn load_timers(self: &mut Self) {}
}

impl Mbc for Mbc3 {
    fn read_rom_byte(self: &Self, addr: u16) -> u8 {
        return match addr {
            0x0000..=0x3FFF => self.rom[usize::from(addr)],
            0x4000..=0x7FFF => {
                self.rom[usize::from(addr - 0x4000)
                    + (ROM_BANK_SIZE * (self.rom_bank_num & (self.max_rom_banks - 1)))]
            }
            _ => panic!("MBC3 - Invalid read from rom addr: {}", addr),
        };
    }

    fn write_rom_byte(self: &mut Self, addr: u16, val: u8) {
        match addr {
            0x0000..=0x1FFF => self.ram_and_timer_enable = val & 0x0A,
            0x2000..=0x3FFF => self.rom_bank_num = if val == 0x00 { 0x01 } else { val as usize },
            0x4000..=0x5FFF => self.ram_bank_num_and_rtc = val as usize,
            0x6000..=0x7FFF => {
                if self.latch_reg == 0 && val == 1 {
                    if let Some(l_rtc) = &mut self.latched_timer {
                        if let Some(new_rtc) = &self.timer {
                            l_rtc.on_latch_register(new_rtc);
                        }
                    }
                }
                self.latch_reg = val;
            }
            _ => panic!("MBC3 - Invalid write to rom addr: {}, value: {}", addr, val),
        }
    }

    fn read_ram_byte(self: &Self, addr: u16) -> u8 {
        if self.ram_and_timer_enable != 0x0A {
            return 0xFF;
        } else if (0x08..=0x0C).contains(&self.ram_bank_num_and_rtc) && self.latched_timer.is_some()
        {
            if let Some(l_rtc) = &self.latched_timer {
                return match self.ram_bank_num_and_rtc {
                    0x08 => l_rtc.seconds,
                    0x09 => l_rtc.minutes,
                    0x0A => l_rtc.hours,
                    0x0B => l_rtc.days_lo,
                    0x0C => l_rtc.days_hi,
                    _ => panic!("Invalid selection for rtc"), // Not possible
                };
            } else {
                panic!("No real time clock was initialized"); // Not possible due to is_some()
            }
        } else {
            return match addr {
                0xA000..=0xBFFF => {
                    self.ram[usize::from(addr - 0xA000)
                        + (RAM_BANK_SIZE * (self.ram_bank_num_and_rtc & (self.max_ram_banks - 1)))]
                }
                _ => panic!("MBC3 - Invalid read from ram addr: {}", addr),
            };
        }
    }

    fn write_ram_byte(self: &mut Self, addr: u16, val: u8) {
        if self.ram_and_timer_enable != 0x0A {
            return;
        } else if (0x08..=0x0C).contains(&self.ram_bank_num_and_rtc) && self.latched_timer.is_some()
        {
            // If I dont write to the updating data, then that means on the next latch
            // the values will always be synched to real time. Which sort of defeats the purpose
            // of writing to these registers in the first place. I will take the difference between
            // what was written to the latched data and the current latched data, and then add/sub
            // that difference to both the rtc.
            if let Some(l_rtc) = &mut self.latched_timer {
                let diff: i32;
                match self.ram_bank_num_and_rtc {
                    0x08 => diff = val as i32 - l_rtc.seconds as i32,
                    0x09 => diff = 60 * (val as i32 - l_rtc.minutes as i32),
                    0x0A => diff = 3600 * (val as i32 - l_rtc.hours as i32),
                    0x0B => diff = 86400 * (val as i32 - l_rtc.days_lo as i32),
                    0x0C => {
                        // diff will be either 0 or 256 * 86400
                        diff = 86400 * (((val & 0x01) as i32 - (l_rtc.days_hi & 0x01) as i32) << 8);
                        l_rtc.days_hi = (val & 0xC0) | (l_rtc.days_hi & 0x01);
                    }
                    _ => panic!("Invalid selection for rtc"), // Not possible
                };
                if diff != 0 {
                    // If they wrote 70 to seconds and it was previosly 10, minus 60 from the timers
                    l_rtc.update_timer(diff, false);
                    if let Some(updating_rtc) = &mut self.timer {
                        updating_rtc.update_timer(diff, false);
                    }
                }
            }
        } else {
            match addr {
                0xA000..=0xBFFF => {
                    self.ram[usize::from(addr - 0xA000)
                        + (RAM_BANK_SIZE
                            * (self.ram_bank_num_and_rtc & (self.max_ram_banks - 1)))] = val;
                }
                _ => panic!("MBC3 - Invalid write to ram addr: {}", addr),
            };
        }
    }

    fn adv_cycles(self: &mut Self, cycles: usize) {
        if let Some(rtc) = &mut self.timer {
            rtc.cycles = rtc.cycles.wrapping_add(cycles);

            // I dont think the while loops are necessary since
            // cycles is only ever 4 but just in case...
            while rtc.cycles > CPU_CYCLES_PER_RTC_CYCLE {
                rtc.int_cycles = rtc.int_cycles.wrapping_add(1);
                rtc.cycles = rtc.cycles.wrapping_sub(CPU_CYCLES_PER_RTC_CYCLE);

                // 1 second passed
                while rtc.int_cycles > RTC_FREQ {
                    rtc.int_cycles = rtc.int_cycles.wrapping_sub(RTC_FREQ);
                    rtc.update_timer(1, false);
                }
            }
        }
    }

    fn load_game(
        self: &mut Self,
        game_path: &str,
        game_bytes: Vec<u8>,
        features: Vec<&str>,
        rom_size: usize,
        rom_banks: usize,
        ram_size: usize,
        ram_banks: usize,
    ) {
        self.rom = vec![0; rom_size];
        self.max_rom_banks = rom_banks;
        self.rom = game_bytes;

        match features[..] {
            ["MBC3"] => { /* Nothing to do */ }
            ["MBC3", "RAM"] => {
                self.ram = vec![0; ram_size];
                self.max_ram_banks = ram_banks;
            }
            ["MBC3", "RAM", "BATTERY"] => {
                let mut ram_path = String::from(game_path);
                ram_path = ram_path.replace(".gb", ".gbsav");

                let file_size = u64::try_from(ram_size).unwrap();
                let mut battery = Battery::new(ram_path, file_size);

                self.ram = battery.load_ram();
                self.battery = Some(battery);
                self.max_ram_banks = ram_banks;
            }
            ["MBC3", "TIMER", "BATTERY"] => {
                // Will create a second file within MbcTimer for storing the RTC registers
                self.timer = Some(MbcTimer::new());
                self.latched_timer = Some(MbcTimer::new());
            }
            ["MBC3", "TIMER", "RAM", "BATTERY"] => {
                let mut ram_path = String::from(game_path);
                ram_path = ram_path.replace(".gb", ".gbsav");

                let file_size = u64::try_from(ram_size).unwrap();
                let mut battery = Battery::new(ram_path, file_size);

                self.ram = battery.load_ram();
                self.battery = Some(battery);
                self.max_ram_banks = ram_banks;
                self.timer = Some(MbcTimer::new());
                self.latched_timer = Some(MbcTimer::new());
            }
            _ => panic!("Feature array not possible for MBC3"),
        }
    }
}

// When the program ends for whatever reason, if we have battery backed ram
// Dump the current ram vector to save for the next time
impl Drop for Mbc3 {
    fn drop(self: &mut Self) {
        if let Some(battery) = &mut self.battery {
            battery.save_ram(&self.ram);
            // also get a timestamp
            // write the latched rtc, updating rtc and timestamp into a file
            // When reloading the game, read the data in, take a new timestamp
            // new timestamp - old timestamp to determine the values to add to
            // the updating rtc. Latched can stay constant
        }
    }
}

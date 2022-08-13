const ROM_BANK_SIZE: usize = 16_384;
const RAM_BANK_SIZE: usize = 8_192;
const CPU_CYCLES_PER_RTC_CYCLE: usize = CPU_FREQ / RTC_FREQ;

use super::battery::Battery;
use super::mbc_timer::{MbcTimer, COUNTER_MAX_SECONDS, RTC_FREQ};
use crate::cpu::CPU_FREQ;
use crate::mbc::Mbc;

/*
    Max 2MByte ROM  (128 Banks)
    Max 32KByte RAM (4 Banks)

    0x0000 - 0x3FFF Bank 0 non-swappable
    0x4000 - 0x7FFF Bank 1 - 127 swappable
*/

pub struct Mbc3 {
    rom: Vec<u8>,         // 0x0000 - 0x7FFF
    ram: Option<Vec<u8>>, // 0xA000 - 0xBFFF
    max_rom_banks: usize,
    max_ram_banks: usize,
    ram_and_timer_enable: bool, // Value of 0xA will enable it
    rom_bank_num: usize,        // 1 - 127 (0x01 - 0x7F)
    ram_bank_num: usize,        // 0 - 3 (Also selects RTC register)
    latch_reg: u8,
    battery: Option<Battery>,
    timer: Option<MbcTimer>,
    latched_timer: Option<MbcTimer>,
}

impl Mbc3 {
    pub fn new() -> Mbc3 {
        Mbc3 {
            rom: Vec::new(),
            ram: None,
            max_rom_banks: 0,
            max_ram_banks: 0,
            ram_and_timer_enable: false,
            rom_bank_num: 0x01,
            ram_bank_num: 0x00,
            latch_reg: 0x00,
            battery: None,
            timer: None,
            latched_timer: None,
        }
    }

    fn load_and_set_timers(self: &mut Self, battery: &mut Battery) {
        let mut rtc = MbcTimer::new();
        let mut latched_rtc = MbcTimer::new();
        let save_time = battery.load_rtc(&mut latched_rtc, &mut rtc);

        // It should be impossible for the save_time to be earlier than current
        let time_offline = MbcTimer::get_current_time() - save_time;
        let carry = if time_offline > COUNTER_MAX_SECONDS {
            true
        } else {
            false
        };

        // Counter max seconds is way smaller than the max i32 so this okay
        rtc.update_timer((time_offline % (COUNTER_MAX_SECONDS + 1)) as i32, carry);
        self.timer = Some(rtc);
        self.latched_timer = Some(latched_rtc);
    }
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
            0x0000..=0x1FFF => self.ram_and_timer_enable = (val & 0x0F) == 0x0A,
            0x2000..=0x3FFF => self.rom_bank_num = if val == 0x00 { 0x01 } else { val as usize },
            0x4000..=0x5FFF => self.ram_bank_num = val as usize,
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
        if !self.ram_and_timer_enable {
            return 0xFF;
        }

        match (self.ram_bank_num, &self.latched_timer) {
            (0x08, Some(l_rtc)) => return l_rtc.seconds,
            (0x09, Some(l_rtc)) => return l_rtc.minutes,
            (0x0A, Some(l_rtc)) => return l_rtc.hours,
            (0x0B, Some(l_rtc)) => return l_rtc.days_lo,
            (0x0C, Some(l_rtc)) => return l_rtc.days_hi,
            _ => {} // fall through
        }

        match (addr, &self.ram) {
            (0xA000..=0xBFFF, Some(ram)) => {
                return ram[usize::from(addr - 0xA000)
                    + (RAM_BANK_SIZE * (self.ram_bank_num % self.max_ram_banks))]
            }
            _ => {
                println!(
                    "No ram, but you enabled it, and selected a ram bank num that is unrelated to the
                    timer or you selected a bank num related to the timer but have no timer and no ram: {}",
                    self.ram_bank_num
                );
                return 0xFF;
            }
        };
    }

    fn write_ram_byte(self: &mut Self, addr: u16, val: u8) {
        if !self.ram_and_timer_enable {
            return;
        }

        if let Some(l_rtc) = &mut self.latched_timer {
            let mut diff: i32 = 0;
            let mut matched = true;

            match self.ram_bank_num {
                0x08 => diff = val as i32 - l_rtc.seconds as i32,
                0x09 => diff = 60 * (val as i32 - l_rtc.minutes as i32),
                0x0A => diff = 3600 * (val as i32 - l_rtc.hours as i32),
                0x0B => diff = 86400 * (val as i32 - l_rtc.days_lo as i32),
                0x0C => {
                    diff = 86400 * (((val & 0x01) as i32 - (l_rtc.days_hi & 0x01) as i32) << 8);
                    l_rtc.days_hi = (val & 0xC0) | (l_rtc.days_hi & 0x01);
                }
                _ => matched = false, // fall through
            }
            if diff != 0 {
                l_rtc.update_timer(diff, false);
                // if let Some(updating_rtc) = &mut self.timer {
                //     updating_rtc.update_timer(diff, false);
                // }
            }
            if matched {
                return;
            }
        }

        match (addr, &mut self.ram) {
            (0xA000..=0xBFFF, Some(ram)) => {
                ram[usize::from(addr - 0xA000)
                    + (RAM_BANK_SIZE * (self.ram_bank_num % self.max_ram_banks))] = val;
            }
            _ => {
                println!(
                    "No ram, but you enabled it, and selected a ram bank num that is unrelated to the
                    timer or you selected a bank num related to the timer but have no timer and no ram: {}",
                    self.ram_bank_num
                );
                return;
            }
        };
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
                self.ram = Some(vec![0; ram_size]);
                self.max_ram_banks = ram_banks;
            }
            ["MBC3", "RAM", "BATTERY"] => {
                let ram_path = String::from(game_path).replace(".gb", ".gbsav");

                let ram_file_size = u64::try_from(ram_size).unwrap();
                let mut battery = Battery::new().with_ram(ram_path, ram_file_size);

                self.ram = Some(battery.load_ram());
                self.battery = Some(battery);
                self.max_ram_banks = ram_banks;
            }
            ["MBC3", "TIMER", "BATTERY"] => {
                // Will create a second file within MbcTimer for storing the RTC registers
                let rtc_path = String::from(game_path).replace(".gb", ".gbrtc");
                let mut battery = Battery::new().with_rtc(rtc_path);

                self.load_and_set_timers(&mut battery);
                self.battery = Some(battery);
            }
            ["MBC3", "TIMER", "RAM", "BATTERY"] => {
                let ram_path = String::from(game_path).replace(".gb", ".gbsav");
                let rtc_path = String::from(game_path).replace(".gb", ".gbrtc");

                let ram_file_size = u64::try_from(ram_size).unwrap();
                let mut battery = Battery::new()
                    .with_ram(ram_path, ram_file_size)
                    .with_rtc(rtc_path);

                self.ram = Some(battery.load_ram());
                self.max_ram_banks = ram_banks;
                self.load_and_set_timers(&mut battery);
                self.battery = Some(battery);
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
            match &self.ram {
                Some(ram) => battery.save_ram(&ram),
                None => {}
            }
            match (&mut self.latched_timer, &mut self.timer) {
                (Some(l_rtc), Some(rtc)) => match battery.save_rtc(l_rtc, rtc) {
                    Ok(_) => { /* Nice */ }
                    Err(_err) => println!("Failed to save the rtc registers"),
                },
                (_, _) => { /* No timers to save */ }
            }
        }
    }
}

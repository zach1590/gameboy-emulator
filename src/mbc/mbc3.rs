const ROM_BANK_SIZE: usize = 16_384;
const RAM_BANK_SIZE: usize = 8_192;

use super::battery::Battery;
use super::mbc_timer::MbcTimer;
use crate::mbc::Mbc;

/*
    Max 2MByte ROM  (128 Banks)
    Max 32KByte RAM (4 Banks)

    0x0000 - 0x3FFF Bank 0 non-swappable
    0x4000 - 0x7FFF Bank 1 - 127 swappable
*/

pub struct Mbc3 {
    rom: Vec<u8>,                // 0x0000 - 0x7FFF
    ram: Vec<u8>,                // 0xA000 - 0xBFFF
    ram_and_timer_enable: u8,    // Value of 0xA will enable it
    rom_bank_num: usize,         // 1 - 127 (0x01 - 0x7F)
    ram_bank_num_and_rtc: usize, // 0 - 3
    latch_data: u8,
    battery: Option<Battery>,
    timer: Option<MbcTimer>,
}

impl Mbc3 {
    pub fn new() -> Mbc3 {
        Mbc3 {
            rom: Vec::new(),
            ram: Vec::new(),
            ram_and_timer_enable: 0x00,
            rom_bank_num: 0x01,
            ram_bank_num_and_rtc: 0x00,
            latch_data: 0x00,
            battery: None,
            timer: None,
        }
    }
}

impl Mbc for Mbc3 {
    fn read_rom_byte(self: &Self, addr: u16) -> u8 {
        return match addr {
            0x0000..=0x3FFF => self.rom[usize::from(addr)],
            0x4000..=0x7FFF => self.rom[usize::from(addr) + (ROM_BANK_SIZE * self.rom_bank_num)],
            _ => panic!("Invalid read from rom addr: {}", addr),
        };
    }

    fn write_rom_byte(self: &mut Self, addr: u16, val: u8) {
        match addr {
            0x0000..=0x1FFF => self.ram_and_timer_enable = val & 0x0A,
            0x2000..=0x3FFF => self.rom_bank_num = if val == 0x00 { 0x01 } else { val as usize },
            0x4000..=0x5FFF => self.ram_bank_num_and_rtc = val as usize,
            0x6000..=0x7FFF => self.latch_data = val,
            _ => panic!("Invalid write to rom addr: {}, with value: {}", addr, val),
        }
    }

    fn read_ram_byte(self: &Self, addr: u16) -> u8 {
        return 0xFF;
    }

    fn write_ram_byte(self: &mut Self, addr: u16, val: u8) {}

    fn load_game(
        self: &mut Self,
        game_path: &str,
        game_bytes: Vec<u8>,
        features: Vec<&str>,
        rom_size: usize,
        _rom_banks: usize,
        ram_size: usize,
        _ram_banks: usize,
    ) {
        self.rom = vec![0; rom_size];
        // self.max_rom_banks = rom_banks;
        self.rom = game_bytes;

        match features[..] {
            ["MBC3"] => { /* Nothing to do */ }
            ["MBC3", "RAM"] => {
                self.ram = vec![0; ram_size];
                // self.max_ram_banks = ram_banks;
            }
            ["MBC3", "RAM", "BATTERY"] => {
                let mut ram_path = String::from(game_path);
                ram_path = ram_path.replace(".gb", ".gbsav");

                let file_size = u64::try_from(ram_size).unwrap();
                let mut battery = Battery::new(ram_path, file_size);

                self.ram = battery.load_ram();
                self.battery = Some(battery);
                // self.max_ram_banks = ram_banks;
            }
            ["MBC3", "TIMER", "BATTERY"] => {}
            ["MBC3", "TIMER", "RAM", "BATTERY"] => {}
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
        } else {
            // Do Nothing
        }
    }
}

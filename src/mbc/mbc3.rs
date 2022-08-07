const ROM_BANK_SIZE: usize = 16_384;
const RAM_BANK_SIZE: usize = 8_192;

use super::battery::Battery;
use super::mbc_timer::MbcTimer;
use crate::mbc::Mbc;

pub struct Mbc3 {
    rom: Vec<u8>, // bank 0 0x0000 - 0x3FFF(16384) and bank 1 0x4000 - 0x7FFF (bank1 is swappable)
    ram: Vec<u8>, // 0xA000 - 0xBFFF
    rom_offset: usize,
    ram_offset: usize,
    rom_bank: usize, // values of 0 and 1 both select bank 1 to be placed into 0x4000-0x7FFF
    ram_bank: usize,
    ext_bank: usize,
    max_rom_banks: usize,
    max_ram_banks: usize,
    mode: u8,
    ram_enabled: u8,
    battery: Option<Battery>,
    timer: Option<MbcTimer>,
}

impl Mbc3 {
    pub fn new() -> Mbc3 {
        Mbc3 {
            rom: Vec::new(),
            ram: Vec::new(),
            rom_offset: 0x4000,
            ram_offset: 0,
            rom_bank: 1,
            ram_bank: 0,
            ext_bank: 0,
            max_rom_banks: 0x00,
            max_ram_banks: 0x00,
            mode: 0,
            ram_enabled: 0x00,
            battery: None,
            timer: None,
        }
    }
}

impl Mbc for Mbc3 {
    fn read_rom_byte(self: &Self, addr: u16) -> u8 {
        return 0xFF;
    }

    fn write_rom_byte(self: &mut Self, addr: u16, val: u8) {}

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

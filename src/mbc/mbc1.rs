const ROM_BANK_SIZE: usize = 16_384;
const RAM_BANK_SIZE: usize = 8_192;

use super::battery::Battery;
use crate::mbc::Mbc;

pub struct Mbc1 {
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
    ram_enabled: bool,
    battery: Option<Battery>,
}

impl Mbc1 {
    pub fn new() -> Mbc1 {
        Mbc1 {
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
            ram_enabled: false,
            battery: None,
        }
    }

    fn find_rom_offset(self: &mut Self) {
        self.rom_bank = if self.max_rom_banks <= 32 {
            self.rom_bank
        } else {
            ((self.ext_bank << 5) | self.rom_bank) & (self.max_rom_banks - 1)
        };

        self.rom_offset = self.rom_bank * ROM_BANK_SIZE;
    }

    fn find_ram_offset(self: &mut Self) {
        self.ram_bank = if self.mode == 0x01 { self.ext_bank } else { 0 };

        self.ram_offset = (self.ram_bank % self.max_ram_banks) * RAM_BANK_SIZE;
    }
}

impl Mbc for Mbc1 {
    fn read_rom_byte(self: &Self, addr: u16) -> u8 {
        let byte = match addr {
            0x0000..=0x3FFF => {
                if self.mode == 0x00 {
                    self.rom[usize::from(addr)]
                } else {
                    let offset = ((self.ext_bank << 5) & (self.max_rom_banks - 1)) * ROM_BANK_SIZE;
                    self.rom[offset + usize::from(addr)]
                }
            }
            0x4000..=0x7FFF => self.rom[self.rom_offset + usize::from(addr - 0x4000)],
            _ => panic!("MbcNone: rom cannot read from addr {:#04X}", addr),
        };
        return byte;
    }

    fn write_rom_byte(self: &mut Self, addr: u16, val: u8) {
        match addr {
            0x0000..=0x1FFF => self.ram_enabled = (val & 0x0F) == 0x0A,
            0x2000..=0x3FFF => {
                // If just trying to map bank 0 to 0x4000-0x7FFF, wont be possible
                // but if the rom uses less than 5 bits for max banks, then it is possible
                self.rom_bank = usize::from(val & 0x1F);
                if self.rom_bank == 0x00 {
                    self.rom_bank = 0x01;
                } else {
                    self.rom_bank = self.rom_bank & (self.max_rom_banks - 1);
                }
            }
            0x4000..=0x5FFF => self.ext_bank = usize::from(val & 0x03),
            0x6000..=0x7FFF => self.mode = val & 0x01,
            _ => panic!("MbcNone: rom cannot read from addr {:#04X}", addr),
        };

        self.find_rom_offset();
        self.find_ram_offset();
    }

    fn read_ram_byte(self: &Self, addr: u16) -> u8 {
        if self.max_ram_banks == 0 {
            println!("No ram banks");
            return 0xFF;
        }

        if self.ram_enabled {
            let byte = match addr {
                0xA000..=0xBFFF => self.ram[self.ram_offset + usize::from(addr - 0xA000)],
                _ => panic!("MbcNone: ram cannot read from addr {:#04X}", addr),
            };
            return byte;
        } else {
            return 0xFF;
        }
    }

    fn write_ram_byte(self: &mut Self, addr: u16, val: u8) {
        if self.max_ram_banks == 0 {
            return;
        }

        if self.ram_enabled {
            match addr {
                0xA000..=0xBFFF => self.ram[self.ram_offset + usize::from(addr - 0xA000)] = val,
                _ => panic!("MbcNone: ram cannot write to addr {:#04X}", addr),
            };
        }
    }

    fn adv_cycles(self: &mut Self, _cycles: usize) {
        return;
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
            ["MBC1"] => { /* Nothing to do */ }
            ["MBC1", "RAM"] => {
                self.ram = vec![0; ram_size];
                self.max_ram_banks = ram_banks;
            }
            ["MBC1", "RAM", "BATTERY"] => {
                let ram_path = String::from(game_path).replace(".gb", ".gbsav");

                let ram_file_size = u64::try_from(ram_size).unwrap();
                let mut battery = Battery::new().with_ram(ram_path, ram_file_size);

                self.ram = battery.load_ram();
                self.battery = Some(battery);
                self.max_ram_banks = ram_banks;
            }
            _ => panic!("Feature array not possible for MBC1"),
        }
    }
}

// When the program ends for whatever reason, if we have battery backed ram
// Dump the current ram vector to save for the next time
impl Drop for Mbc1 {
    fn drop(self: &mut Self) {
        if let Some(battery) = &mut self.battery {
            battery.save_ram(&self.ram);
        }
    }
}

// Not doing rumble

const ROM_BANK_SIZE: usize = 16_384;
const RAM_BANK_SIZE: usize = 8_192;

use super::battery::Battery;
use crate::mbc::Mbc;

pub struct Mbc5 {
    rom: Vec<u8>, // bank 0 0x0000 - 0x3FFF(16384) and bank 1 0x4000 - 0x7FFF (bank1 is swappable)
    ram: Vec<u8>, // 0xA000 - 0xBFFF
    rom_offset: usize,
    ram_offset: usize,
    rom_bank_lo: usize, // Lower 8 bits of rom bank num
    rom_bank_hi: usize, // Upper 1 bit of rom bank num
    ram_bank: usize,    // 0x00 - 0x0F
    max_rom_banks: usize,
    max_ram_banks: usize,
    ram_enabled: bool,
    battery: Option<Battery>,
}

impl Mbc5 {
    pub fn new() -> Mbc5 {
        Mbc5 {
            rom: Vec::new(),
            ram: Vec::new(),
            rom_offset: 0,
            ram_offset: 0,
            rom_bank_lo: 0,
            rom_bank_hi: 0,
            ram_bank: 0,
            max_rom_banks: 0x00,
            max_ram_banks: 0x00,
            ram_enabled: false,
            battery: None,
        }
    }
}

impl Mbc for Mbc5 {
    fn read_rom_byte(self: &Self, addr: u16) -> u8 {
        return match addr {
            0x0000..=0x3FFF => self.rom[self.rom_offset + usize::from(addr)],
            0x4000..=0x7FFF => self.rom[self.rom_offset + usize::from(addr - 0x4000)],
            _ => panic!("MbcNone: rom cannot read from addr {:#04X}", addr),
        };
    }

    fn write_rom_byte(self: &mut Self, addr: u16, val: u8) {
        match addr {
            0x0000..=0x1FFF => self.ram_enabled = (val & 0x0F) == 0x0A,
            0x2000..=0x2FFF => self.rom_bank_lo = usize::from(val),
            0x3000..=0x3FFF => self.rom_bank_hi = usize::from(val & 0x01),
            0x4000..=0x5FFF => self.ram_bank = usize::from(val & 0x0F),
            0x6000..=0x7FFF => { /* Nothing */ }
            _ => panic!("MbcNone: rom cannot read from addr {:#04X}", addr),
        };

        self.rom_offset = ((self.rom_bank_hi << 8) | self.rom_bank_lo) % self.max_rom_banks;
        self.rom_offset = self.ram_bank % self.max_ram_banks;
    }

    fn read_ram_byte(self: &Self, addr: u16) -> u8 {
        if self.max_ram_banks == 0 {
            println!("read but No ram banks");
            return 0xFF;
        }

        if self.ram_enabled {
            return match addr {
                0xA000..=0xBFFF => self.ram[self.ram_offset + usize::from(addr - 0xA000)],
                _ => panic!("MbcNone: ram cannot read from addr {:#04X}", addr),
            };
        }

        return 0xFF;
    }

    fn write_ram_byte(self: &mut Self, addr: u16, val: u8) {
        if self.max_ram_banks == 0 {
            println!("write but No ram banks");
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
            ["MBC5"] => { /* Nothing to do */ }
            ["MBC5", "RAM"] => {
                self.ram = vec![0; ram_size];
                self.max_ram_banks = ram_banks;
            }
            ["MBC5", "RAM", "BATTERY"] => {
                let ram_path = String::from(game_path).replace(".gb", ".gbsav");

                let ram_file_size = u64::try_from(ram_size).unwrap();
                let mut battery = Battery::new().with_ram(ram_path, ram_file_size);

                self.ram = battery.load_ram();
                self.battery = Some(battery);
                self.max_ram_banks = ram_banks;
            }
            _ => panic!("Feature array not possible for MBC5"),
        }
    }
}

// When the program ends for whatever reason, if we have battery backed ram
// Dump the current ram vector to save for the next time
impl Drop for Mbc5 {
    fn drop(self: &mut Self) {
        if let Some(battery) = &mut self.battery {
            battery.save_ram(&self.ram);
        }
    }
}

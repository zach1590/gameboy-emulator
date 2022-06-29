// MBC1M not supported

use super::mbc::{Mbc};

pub struct Mbc1 {
    rom: Vec<[u8; 16_384]>, // bank 0 0x0000 - 0x3FFF(16384) and bank 1 0x4000 - 0x7FFF (bank1 is swappable)
    ram: Vec<[u8; 8_192]>,  // 0xA000 - 0xBFFF
    rom_bank: usize,        // values of 0 and 1 both select bank 1 to be placed into 0x4000-0x7FFF
    ext_bank: usize,
    max_rom_banks: usize,
    mode: u8,
    ram_enabled: u8,
}

impl Mbc1 {
    pub fn new() -> Mbc1 {
        Mbc1 {
            rom: Vec::new(),
            ram: Vec::new(),
            rom_bank: 1,
            ext_bank: 0,
            max_rom_banks: 0x00,
            mode: 0,
            ram_enabled: 0x00,
        }
    }
}

impl Mbc for Mbc1 {

    fn read_rom_byte(self: &Self, addr: u16) -> u8 {

        let byte = match addr {
            0x0000..=0x3FFF => {
                if self.mode == 0x00 { self.rom[0][usize::from(addr)] }
                else { 
                    self.rom[(self.ext_bank << 5) & (self.max_rom_banks - 1)][usize::from(addr)] 
                }
            },
            0x4000..=0x7FFF => {
                if self.max_rom_banks <= 32 {
                    self.rom[self.rom_bank][usize::from(addr - 0x4000)]
                } else {
                    let b = (self.ext_bank << 5) | self.rom_bank;
                    self.rom[b & (self.max_rom_banks - 1)][usize::from(addr - 0x4000)]
                }
            },
            _ => panic!("MbcNone: rom cannot read from addr {:#04X}", addr),
        };
        return byte;
    }

    fn write_rom_byte(self: &mut Self, addr: u16, val: u8) {
        match addr {
            0x0000..=0x1FFF => self.ram_enabled = val & 0x0A,
            0x2000..=0x3FFF => {
                // If just trying to map bank 0 to 0x4000-0x7FFF, wont be possible
                // but if the rom uses less than 5 bits for max banks, then it is possible
                self.rom_bank = usize::from(val & 0x1F);
                if self.rom_bank == 0x00 { self.rom_bank = 0x01; }
                else { self.rom_bank = self.rom_bank & (self.max_rom_banks - 1); }
            },
            0x4000..=0x5FFF => self.ext_bank = usize::from(val & 0x03),
            0x6000..=0x7FFF => self.mode = val & 0x01,
            _ => panic!("MbcNone: rom cannot read from addr {:#04X}", addr),
        };
    }

    fn read_ram_byte(self: &Self, addr: u16) -> u8 {
        if self.ram_enabled == 0x0A {
            let byte = match addr {
                0xA000..=0xBFFF => self.ram[self.ext_bank][usize::from(addr - 0xA000)],
                _ => panic!("MbcNone: ram cannot read from addr {:#04X}", addr),
            };
            return byte;
        } else {
            return 0xFF;
        }
    }

    fn write_ram_byte(self: &mut Self, addr: u16, val: u8) {
        if self.ram_enabled == 0x0A {
            match addr {
                0xA000..=0xBFFF => self.ram[self.ext_bank][usize::from(addr - 0xA000)] = val,
                _ => panic!("MbcNone: ram cannot write to addr {:#04X}", addr),
            };
        }
    }

    fn load_game(
        self: &mut Self,
        game_bytes: Vec<u8>,
        _features: Vec<&str>,
        _rom_size: usize,
        _rom_banks: usize,
        _ram_size: usize,
        _ram_banks: usize,
    ) {
        // not implemented
    }
}
use super::mbc::Mbc;
use crate::mbc::mbc_none::MbcNone;

pub struct Memory {
    mbc: Box<dyn Mbc>,      // MBC will contain ROM and RAM aswell as banks
    wram: [u8; 8_192],      // 0xC000 - 0xDFFF
    echo_wram: [u8; 7_680], // 0xE000 - 0xFDFF (mirror of work ram)
    not_used: [u8; 96],     // 0xFEAO - 0xFEFF
    hram: [u8; 127],        // 0xFF80 - 0xFFFE
    pub i_enable: u8,       // 0xFFFF
}

impl Memory {
    pub fn new() -> Memory {
        return Memory {
            mbc: Box::new(MbcNone::new()), // Swap out mbc once its known
            wram: [0; 8_192],
            echo_wram: [0; 7_680],
            not_used: [0; 96],
            hram: [0; 127],
            i_enable: 0,
        };
    }

    pub fn set_mbc(self: &mut Self, cart_mbc: Box<dyn Mbc>) {
        self.mbc = cart_mbc;
    }

    pub fn read_byte(self: &Self, addr: u16) -> u8 {
        let byte = match addr {
            0x0000..=0x7FFF => self.mbc.read_rom_byte(addr),
            0xA000..=0xBFFF => self.mbc.read_ram_byte(addr),
            0xC000..=0xDFFF => self.wram[usize::from(addr - 0xC000)],
            0xE000..=0xFDFF => self.echo_wram[usize::from(addr - 0xE000)],
            0xFF80..=0xFFFE => self.hram[usize::from(addr - 0xFF80)],
            0xFFFF => self.i_enable,
            _ => panic!("Memory does not handle reads from: {:04X}", addr),
        };
        return byte;
    }

    pub fn write_byte(self: &mut Self, addr: u16, data: u8) {
        match addr {
            0x0000..=0x7FFF => self.mbc.write_rom_byte(addr, data),
            0xA000..=0xBFFF => self.mbc.write_ram_byte(addr, data),
            0xC000..=0xDFFF => {
                self.wram[usize::from(addr - 0xC000)] = data;
                if addr <= 0xDDFF {
                    self.echo_wram[usize::from(addr - 0xC000)] = data;
                }
            }
            0xE000..=0xFDFF => return, // Should not write to echo ram
            0xFF80..=0xFFFE => self.hram[usize::from(addr - 0xFF80)] = data,
            0xFFFF => self.i_enable = data,
            _ => panic!("Memory does not handle write to: {:04X}", addr),
        };
    }

    pub fn read_byte_for_dma(self: &Self, addr: u16) -> u8 {
        let byte = match addr {
            0x0000..=0x7FFF => self.mbc.read_rom_byte(addr),
            0xA000..=0xBFFF => self.mbc.read_ram_byte(addr),
            0xC000..=0xDFFF => self.wram[usize::from(addr - 0xC000)],
            _ => panic!("DMA should not read from: {:04X}", addr),
        };
        return byte;
    }

    // Write multiple bytes into memory starting from location
    // This should only be used for tests
    pub fn write_bytes(self: &mut Self, location: u16, data: &Vec<u8>) {
        for (i, byte) in data.into_iter().enumerate() {
            self.write_byte(location + (i as u16), *byte);
        }
    }

    pub fn dmg_init(self: &mut Self) {
        self.i_enable = 0x00;
    }
}

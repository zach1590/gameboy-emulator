// Stick with just MbcNone for now
// Get something like tetris working
// Then emulate the other mbc types

pub trait Mbc {
    fn read_ram_byte(self: &Self, addr: u16) -> u8;
    fn write_ram_byte(self: &mut Self, addr: u16, val: u8);
    fn read_rom_byte(self: &Self, addr: u16) -> u8;
    fn write_rom_byte(self: &mut Self, addr: u16, val: u8);
}

pub struct MbcNone {
    rom: [u8; 32_768], // 0x0000 - 0x7FFF
    ram: [u8; 8_192],  // 0xA000 - 0xBFFF
    ram_enabled: bool, // Dont actually know if this should be true/false be default
}

impl MbcNone {
    pub fn new() -> MbcNone {
        MbcNone {
            rom: [0; 32_768],
            ram: [0; 8_192],
            ram_enabled: true,
        }
    }
}

impl Mbc for MbcNone {
    fn read_ram_byte(self: &Self, addr: u16) -> u8 {
        let byte = match addr {
            0xA000..=0xBFFF => self.ram[usize::from(addr - 0xA000)],
            _ => panic!("MbcNone: ram cannot read from addr {:#04X}", addr),
        };
        return byte;
    }

    fn write_ram_byte(self: &mut Self, addr: u16, val: u8) {
        match addr {
            0xA000..=0xBFFF => self.ram[usize::from(addr - 0xA000)] = val,
            _ => panic!("MbcNone: ram cannot write to addr {:#04X}", addr),
        };
    }

    fn read_rom_byte(self: &Self, addr: u16) -> u8 {
        let byte = match addr {
            0x0000..=0x7FFF => self.rom[usize::from(addr)],
            _ => panic!("MbcNone: rom cannot read from addr {:#04X}", addr),
        };
        return byte;
    }

    // Should not write to rom
    // For the other MBC writes to rom are only to control hardware
    fn write_rom_byte(self: &mut Self, _addr: u16, _val: u8) {
        return;
        // match addr {
        //     0x0000..=0x7FFF => self.rom[usize::from(addr)] = val,
        //     _ => panic!("MbcNone: rom cannot write to addr {:#04X}", addr),
        // };
    }
}

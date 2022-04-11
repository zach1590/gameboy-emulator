pub struct Memory {
    mbc: Box<dyn Mbc>,      // MBC will contain ROM and RAM aswell as banks
    vram: [u8; 8_192],      // 0x8000 - 0x9FFF
    wram: [u8; 8_192],      // 0xC000 - 0xDFFF
    echo_wram: [u8; 7_680], // 0xE000 - 0xFDFF (mirror of work ram)
    spr_table: [u8; 160],   // 0xFE00 - 0xFE9F
    not_used: [u8; 96],     // 0xFEAO - 0xFEFF
    io: [u8; 128],          // 0xFF00 - 0xFF7F
    hram: [u8; 127],        // 0xFF80 - 0xFFFE
    i_enable: u8,           // 0xFFFF
}

impl Memory {
    pub fn new() -> Memory {
        return Memory {
            mbc: Box::new(MbcNone::new()),
            vram: [0; 8_192],
            wram: [0; 8_192],
            echo_wram: [0; 7_680],
            spr_table: [0; 160],
            not_used: [0; 96],
            io: [0; 128],
            hram: [0; 127],
            i_enable: 0,
        };
    }

    pub fn interrupt_pending(self: &Self) -> bool {
        (self.i_enable & self.io[0xFF0F]) != 0
    }

    pub fn read_byte(self: &Self, addr: u16) -> u8 {
        let byte = match addr {
            0x0000..=0x7FFF => self.mbc.read_rom_byte(addr),
            0x8000..=0x9FFF => self.vram[usize::from(addr - 0x8000)],
            0xA000..=0xBFFF => self.mbc.read_ram_byte(addr),
            0xC000..=0xDFFF => self.wram[usize::from(addr - 0xC000)],
            0xE000..=0xFDFF => self.echo_wram[usize::from(addr - 0xE000)],
            0xFE00..=0xFE9F => self.spr_table[usize::from(addr - 0xFE00)],
            0xFEA0..=0xFEFF => self.not_used[usize::from(addr - 0xFEA0)],
            0xFF00..=0xFF7F => self.io[usize::from(addr - 0xFF00)],
            0xFF80..=0xFFFE => self.hram[usize::from(addr - 0xFF80)],
            0xFFFF => self.i_enable,
        };
        return byte;
    }

    // Write a single byte to at the location
    pub fn write_byte(self: &mut Self, addr: u16, data: u8) {
        match addr {
            0x0000..=0x7FFF => self.mbc.write_rom_byte(addr, data),
            0x8000..=0x9FFF => self.vram[usize::from(addr - 0x8000)] = data,
            0xA000..=0xBFFF => self.mbc.write_ram_byte(addr, data),
            0xC000..=0xDFFF => {
                self.wram[usize::from(addr - 0xC000)] = data;
                // if addr <= 0xDDFF {
                //     self.echo_wram[usize::from(addr - 0xC000)] = data;
                // }
            }
            0xE000..=0xFDFF => self.echo_wram[usize::from(addr - 0xE000)] = data,
            0xFE00..=0xFE9F => self.spr_table[usize::from(addr - 0xFE00)] = data,
            0xFEA0..=0xFEFF => self.not_used[usize::from(addr - 0xFEA0)] = data,
            0xFF00..=0xFF7F => self.io[usize::from(addr - 0xFF00)] = data,
            0xFF80..=0xFFFE => self.hram[usize::from(addr - 0xFF80)] = data,
            0xFFFF => self.i_enable = data,
        };
    }

    // Write multiple bytes into memory starting from location
    // This should only be used for tests
    pub fn write_bytes(self: &mut Self, location: u16, data: Vec<u8>) {
        for (i, byte) in data.into_iter().enumerate() {
            self.write_byte(location + (i as u16), byte);
        }
    }
}

struct CartridgeMemory {
    rom_data_banks: Vec<[u8; 8_192]>, // Initialize this vector with 0 if the rom_size is only 32KiB
    ram_data_banks: Vec<[u8; 8_192]>, // Initialize this vector with 0 if the ram_size is only 8KiB
}

pub trait Mbc {
    fn read_ram_byte(self: &Self, addr: u16) -> u8;
    fn write_ram_byte(self: &mut Self, addr: u16, val: u8);
    fn read_rom_byte(self: &Self, addr: u16) -> u8;
    fn write_rom_byte(self: &mut Self, addr: u16, val: u8);
}

pub struct MbcNone {
    rom: [u8; 32_768], // 0x0000 - 0x7FFF
    ram: [u8; 8_192],  // 0xA000 - 0xBFFF
}

impl MbcNone {
    pub fn new() -> MbcNone {
        MbcNone {
            rom: [0; 32_768],
            ram: [0; 8_192],
        }
    }
}

impl Mbc for MbcNone {
    fn read_ram_byte(self: &Self, addr: u16) -> u8 {
        let byte = match addr {
            0xA000..=0xBFFF => self.ram[usize::from(addr)],
            _ => panic!("MbcNone: ram cannot read from addr {:#04X}", addr),
        };
        return byte;
    }

    fn write_ram_byte(self: &mut Self, addr: u16, val: u8) {
        match addr {
            0xA000..=0xBFFF => self.ram[usize::from(addr)] = val,
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

    fn write_rom_byte(self: &mut Self, addr: u16, val: u8) {
        match addr {
            0x0000..=0x7FFF => self.rom[usize::from(addr)] = val,
            _ => panic!("MbcNone: rom cannot write to addr {:#04X}", addr),
        };
    }
}

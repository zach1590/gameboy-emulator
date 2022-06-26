use super::mbc::{Mbc, MbcNone};

const I_FIRED: u16 = 0xFF0F;
const DIV_REG: u16 = 0xFF04;    // Writing any value to this register resets it to 0

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
    oam_blocked: bool,
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
            oam_blocked: false,
        };
    }

    pub fn set_mbc(self: &mut Self, cart_mbc: Box<dyn Mbc>) {
        self.mbc = cart_mbc;
    }

    pub fn interrupt_pending(self: &Self) -> bool {
        (self.i_enable & self.io[(I_FIRED as usize) - 0xFF00]) != 0
    }

    pub fn read_byte(self: &Self, addr: u16) -> u8 {
        let byte = match addr {
            0x0000..=0x7FFF => self.mbc.read_rom_byte(addr),
            0x8000..=0x9FFF => self.vram[usize::from(addr - 0x8000)],
            0xA000..=0xBFFF => self.mbc.read_ram_byte(addr),
            0xC000..=0xDFFF => self.wram[usize::from(addr - 0xC000)],
            0xE000..=0xFDFF => self.echo_wram[usize::from(addr - 0xE000)],
            0xFE00..=0xFE9F => self.spr_table[usize::from(addr - 0xFE00)],
            0xFEA0..=0xFEFF => {
                match self.oam_blocked {
                    true => 0xFF,
                    false => 0x00,
                }
                // self.not_used[usize::from(addr - 0xFEA0)]
            }
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
                if addr <= 0xDDFF {
                    self.echo_wram[usize::from(addr - 0xC000)] = data;
                }
            }
            0xE000..=0xFDFF => panic!("Do not write to echo ram"),  // change to return
            0xFE00..=0xFE9F => self.spr_table[usize::from(addr - 0xFE00)] = data,
            0xFEA0..=0xFEFF => panic!("Memory area is not usable"), // change to return
            0xFF00..=0xFF7F => match addr {
                DIV_REG => self.reset_div_reg(),
                _ => self.io[usize::from(addr - 0xFF00)] = data,
            },
            0xFF80..=0xFFFE => self.hram[usize::from(addr - 0xFF80)] = data,
            0xFFFF => self.i_enable = data,
        };
    }

    // Write multiple bytes into memory starting from location
    // This should only be used for tests
    pub fn write_bytes(self: &mut Self, location: u16, data: &Vec<u8>) {
        for (i, byte) in data.into_iter().enumerate() {
            self.write_byte(location + (i as u16), *byte);
        }
    }

    pub fn get_vram(self: &Self) -> &[u8] {
        return &self.vram;
    }
    // pub fn load_game(self: &mut Self, game_bytes: Vec<u8>) {
    //     self.mbc.load_game(game_bytes);
    // }

    pub fn reset_div_reg(self: &mut Self) {
        self.io[usize::from(DIV_REG - 0xFF00)] = 0;
    }
    pub fn incr_div_reg(self: &mut Self, val: u8) {
        self.io[usize::from(DIV_REG - 0xFF00)] = 
            self.io[usize::from(DIV_REG - 0xFF00)].wrapping_add(val);
    }
    
}

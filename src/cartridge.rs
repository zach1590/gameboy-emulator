use super::mbc::Mbc;
use super::memory;
use crate::mbc::mbc1::Mbc1;
use crate::mbc::mbc_none::MbcNone;
use std::fs;

pub struct Cartridge {
    entry_point: [u8; 4], // 0100-0103 - Entry Point, boot jumps here after Nintendo Logo
    logo: [u8; 48],       // Nintendo Logo, On boot verifies the contents of this map or locks up
    title: [u8; 16],      // Title in uppercase ascii, if less than 16 chars, filled with 0x00
    new_lisc_code: [u8; 2],
    cartridge_type: u8,
    rom_size: u8,
    ram_size: u8,
    dest_code: u8,
    old_lisc_code: u8,
    rom_version: u8,
    checksum_val: u8,
}

// Make getters for all this information but no setters

impl Cartridge {
    pub fn new() -> Cartridge {
        return Cartridge {
            entry_point: [0; 4],
            logo: [0; 48],
            title: [0; 16],
            new_lisc_code: [0; 2],
            cartridge_type: 0,
            rom_size: 0,
            ram_size: 0,
            dest_code: 0,
            old_lisc_code: 0,
            rom_version: 0,
            checksum_val: 0,
        };
    }

    // Do this last
    pub fn read_cartridge_header(self: &mut Self, game_path: &str) -> Box<dyn Mbc> {
        let game_bytes = fs::read(game_path).unwrap();

        self.entry_point[..4].clone_from_slice(&game_bytes[0x0100..=0x0103]);
        self.logo[..48].clone_from_slice(&game_bytes[0x0104..=0x0133]);
        self.title[..16].clone_from_slice(&game_bytes[0x0134..=0x0143]);

        self.new_lisc_code[..2].clone_from_slice(&game_bytes[0x0144..=0x0145]);
        self.cartridge_type = game_bytes[0x0147];
        self.rom_size = game_bytes[0x0148];
        self.ram_size = game_bytes[0x0149];
        self.dest_code = game_bytes[0x14A];
        self.old_lisc_code = game_bytes[0x014B];
        self.rom_version = game_bytes[0x014C];
        self.checksum_val = game_bytes[0x014D];

        let (rom_size, rom_banks) = match self.get_rom_size() {
            Some((size, banks)) => (size, banks),
            None => panic!("ROM Size: {} is not supported", self.rom_size),
        };

        let (ram_size, ram_banks) = match self.get_ram_size() {
            Some((size, banks)) => (size, banks),
            None => panic!("ROM Size: {} is not supported", self.ram_size),
        };

        let (mut mbc, features) = match self.get_cartridge_type() {
            (Some(new_mbc), features) => (new_mbc, features),
            (None, features) => panic!("MBC Type with {:?} is not supported", features),
        };

        mbc.load_game(
            game_path, game_bytes, features, rom_size, rom_banks, ram_size, ram_banks,
        );

        return mbc;
    }

    pub fn checksum(self: &Self, mem: &memory::Memory) -> u8 {
        let mut x: u16 = 0;
        for i in 0x0134..=0x014C {
            x = x.wrapping_sub(mem.read_byte(i) as u16).wrapping_sub(1);
        }
        if (x as u8) != self.checksum_val {
            panic!("checksum failed");
        } else {
            println!("checksum passed");
        }

        return self.checksum_val;
    }

    fn get_cartridge_type(self: &Self) -> (Option<Box<dyn Mbc>>, Vec<&str>) {
        // Eventually support MBC3 at least
        match self.cartridge_type {
            0x00 => (Some(Box::new(MbcNone::new())), vec!["ROM_ONLY"]),
            0x01 => (Some(Box::new(Mbc1::new())), vec!["MBC1"]),
            0x02 => (Some(Box::new(Mbc1::new())), vec!["MBC1", "RAM"]),
            0x03 => (Some(Box::new(Mbc1::new())), vec!["MBC1", "RAM", "BATTERY"]),
            0x05 => (None, vec!["MBC2"]),
            0x06 => (None, vec!["MBC2", "BATTERY"]),
            0x08 => (None, vec!["ROM", "RAM"]),
            0x09 => (None, vec!["ROM", "RAM", "BATTERY"]),
            0x0B => (None, vec!["MMM01"]),
            0x0C => (None, vec!["MMM01", "RAM"]),
            0x0D => (None, vec!["MMM01", "RAM", "BATTERY"]),
            0x0F => (None, vec!["MBC3", "TIMER", "BATTERY"]),
            0x10 => (None, vec!["MBC3", "TIMER", "RAM", "BATTERY"]),
            0x11 => (None, vec!["MBC3"]),
            0x12 => (None, vec!["MBC3", "RAM"]),
            0x13 => (None, vec!["MBC3", "RAM", "BATTERY"]),
            0x19 => (None, vec!["MBC5"]),
            0x1A => (None, vec!["MBC5", "RAM"]),
            0x1B => (None, vec!["MBC5", "RAM", "BATTERY"]),
            0x1C => (None, vec!["MBC5", "RUMBLE"]),
            0x1D => (None, vec!["MBC5", "RUMBLE", "RAM"]),
            0x1E => (None, vec!["MBC5", "RUMBLE", "RAM", "BATTERY"]),
            0x20 => (None, vec!["MBC6"]),
            0x22 => (None, vec!["MBC7", "SENSOR", "RUMBLE", "RAM", "BATTERY"]),
            0xFC => (None, vec!["POCKET_CAMERA"]),
            0xFD => (None, vec!["BANDAI_TAMA5"]),
            0xFE => (None, vec!["HuC3"]),
            0xFF => (None, vec!["HuC1", "RAM", "BATTERY"]),
            _ => panic!("Invalid cartridge type byte"),
        }
    }

    fn get_rom_size(self: &Self) -> Option<(usize, usize)> {
        // Each bank is 16Kib in size
        match self.rom_size {
            0x00 => Some((32_768, 2)), // No banking
            0x01 => Some((65_536, 4)),
            0x02 => Some((131_072, 8)),
            0x03 => Some((262_144, 16)),
            0x04 => Some((524_288, 32)),
            0x05 => Some((1_024_000, 64)),
            0x06 => Some((2_048_000, 128)),
            0x07 => Some((4_096_000, 256)),
            0x08 => Some((8_192_000, 512)),
            // 0x52 => Some(1_126_400), // Probably doesnt exist
            // 0x53 => Some(1_228_800), // Probably doesnt exist
            // 0x54 => Some(1_536_000), // Probably doesnt exist
            _ => None,
        }
    }

    fn get_ram_size(self: &Self) -> Option<(usize, usize)> {
        // MBC2 will say 00 but it includes a builtin 512x4 bits
        match self.ram_size {
            0x00 => Some((0, 0)),
            0x02 => Some((8_192, 1)),    // 1 Bank
            0x03 => Some((32_768, 4)),   // 4 Banks of 8KB
            0x04 => Some((131_072, 16)), // 16 Banks of 8KB
            0x05 => Some((65_536, 8)),   // 8 Banks of 8KB
            _ => None,
        }
    }

    fn get_publisher_name(self: &Self) -> Option<String> {
        let switch = if self.old_lisc_code == 0x33 {
            (self.new_lisc_code[0] << 4) | (self.new_lisc_code[1] & 0x0F)
        } else {
            self.old_lisc_code
        };
        match switch {
            0x00 => Some(String::from("None")),
            0x01 => Some(String::from("Nintendo R&D1")),
            0x08 => Some(String::from("Capcom")),
            0x13 => Some(String::from("Electronic Arts")),
            0x18 => Some(String::from("Hudson Soft")),
            0x19 => Some(String::from("b-ai")),
            0x20 => Some(String::from("kss")),
            0x22 => Some(String::from("pow")),
            0x24 => Some(String::from("PCM Complete")),
            0x25 => Some(String::from("san-x")),
            0x28 => Some(String::from("Kemco Japan")),
            0x29 => Some(String::from("seta")),
            0x30 => Some(String::from("Viacom")),
            0x31 => Some(String::from("Nintendo")),
            0x32 => Some(String::from("Bandai")),
            0x33 => Some(String::from("Ocean/Acclaim")),
            0x34 => Some(String::from("Konami")),
            0x35 => Some(String::from("Hector")),
            0x37 => Some(String::from("Taito")),
            0x38 => Some(String::from("Hudson")),
            0x39 => Some(String::from("Banpresto")),
            0x41 => Some(String::from("Ubi Soft")),
            0x42 => Some(String::from("Atlus")),
            0x44 => Some(String::from("Malibu")),
            0x46 => Some(String::from("angel")),
            0x47 => Some(String::from("Bullet-Proof")),
            0x49 => Some(String::from("irem")),
            0x50 => Some(String::from("Absolute")),
            0x51 => Some(String::from("Acclaim")),
            0x52 => Some(String::from("Activision")),
            0x53 => Some(String::from("American sammy")),
            0x54 => Some(String::from("Konami")),
            0x55 => Some(String::from("Hi tech entertainment")),
            0x56 => Some(String::from("LJN")),
            0x57 => Some(String::from("Matchbox")),
            0x58 => Some(String::from("Mattel")),
            0x59 => Some(String::from("Milton Bradley")),
            0x60 => Some(String::from("Titus")),
            0x61 => Some(String::from("Virgin")),
            0x64 => Some(String::from("LucasArts")),
            0x67 => Some(String::from("Ocean")),
            0x69 => Some(String::from("Electronic Arts")),
            0x70 => Some(String::from("Infogrames")),
            0x71 => Some(String::from("Interplay")),
            0x72 => Some(String::from("Broderbund")),
            0x73 => Some(String::from("sculptured")),
            0x75 => Some(String::from("sci")),
            0x78 => Some(String::from("THQ")),
            0x79 => Some(String::from("Accolade")),
            0x80 => Some(String::from("misawa")),
            0x83 => Some(String::from("lozc")),
            0x86 => Some(String::from("Tokuma Shoten Intermedia")),
            0x87 => Some(String::from("Tsukuda Original")),
            0x91 => Some(String::from("Chunsoft")),
            0x92 => Some(String::from("Video system")),
            0x93 => Some(String::from("Ocean/Acclaim")),
            0x95 => Some(String::from("Varie")),
            0x96 => Some(String::from("Yonezawa/s'pal")),
            0x97 => Some(String::from("Kaneko")),
            0x99 => Some(String::from("Pack in soft")),
            0xA4 => Some(String::from("Konami (Yu-Gi-Oh!)")),
            _ => None,
        }
    }

    pub fn get_logo(self: &Self) -> [u8; 48] {
        return self.logo;
    }
    pub fn get_entry_point(self: &Self) -> [u8; 4] {
        return self.entry_point;
    }
}

#[cfg(test)]
#[test]
fn test_read_header() {
    let game_path = "./roms/tetris.gb";
    let mut cart = Cartridge::new();

    // Make sure these get overwritten
    cart.cartridge_type = 0x10;
    cart.rom_size = 0x40;
    cart.ram_size = 0x06;

    let mut cpu = super::cpu::Cpu::new();

    let mbc = cart.read_cartridge_header(game_path);
    cpu.set_mbc(mbc);

    let s = match std::str::from_utf8(&cart.title) {
        Ok(v) => v,
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    };
    assert!(s.contains("TETRIS"));
    assert_eq!(cart.cartridge_type, 0x00);
    assert_eq!(cart.rom_size, 0x00);
    assert_eq!(cart.ram_size, 0x00);

    cart.checksum(cpu.get_memory());
}

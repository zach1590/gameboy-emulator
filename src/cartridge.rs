// When we load the first 16KiB of rom data into Memory this data will also be found there (as bytes)
// This is more for convenience and structuring of the program
struct Cartridge {
    entry_point: [u8; 4],   // 0100-0103 - Entry Point, boot jumps here after Nintendo Logo
    logo: [u8; 48],         // Nintendo Logo, On boot verifies the contents of this map or locks up
    title: [char; 16],      // Title in uppercase ascii, if less than 16 chars, filled with 0x00
    new_lisc_code: u16,     // Need a pattern match for this
    sgb_flag: u8,           // 0x03 if game supports SGB functions otherwise anything (For SuperGameBoy)
    cartridge_type: u8,     // Need a pattern match (specifies memory bank controller)
    rom_size: u8,           // Need a pattern match
    ram_size: u8,           // Some ROMs have ram some dont
    dest_code: u8,          // Destination code
    old_lisc_code: u8,      // ???
    rom_version: u8,
    header_checksum: u8,    // Needs to match computed value or boot ROM locks up
    global_checksum: [u8; 2], 

    // These exist but probably dont need to worry
    // manu_code: [char; 4],// Used to be part of title
    // cgb_flag: u8,        // Used to be part of title (For gameboy color)
}

impl Cartridge{
    // fn load_cartridge_header(filename: &str) -> Cartridge {
    //     Cartridge{
                // Load the Rom and parse it for the wanted information to fill the struct fields
    //     }
    // }
    
    // mem: [u8; 1000] will need to go in favour of either the memory struct or the array that will
    // hold the entire program file data in it (assuming that array is not dropped yet when we do this)
    fn checksum(self: &Self, mem: [u8; 1000]) {
        let mut x: u8 = 0;
        for i in 0x0134..0x014C {
            x = x - mem[i] - 1;
        }
        if (0x0F & x) != mem[0x014D]{
            panic!("checksum failed");
        }
    }

    fn get_cartridge_type(self: &Self) -> Option<String> {      // Change to result because we should error if not valid type
        match self.cartridge_type {
            0x00 => Some(String::from("ROM ONLY")),
            0x01 => Some(String::from("MBC1")),
            0x02 => Some(String::from("MBC1+RAM")),
            0x03 => Some(String::from("MBC1+RAM+BATTERY")),
            0x05 => Some(String::from("MBC2")),
            0x06 => Some(String::from("MBC2+BATTERY")),
            0x08 => Some(String::from("ROM+RAM")),
            0x09 => Some(String::from("ROM+RAM+BATTERY")),
            0x0B => Some(String::from("MMM01")),
            0x0C => Some(String::from("MMM01+RAM")),
            0x0D => Some(String::from("MMM01+RAM+BATTERY")),
            0x0F => Some(String::from("MBC3+TIMER+BATTERY")),
            0x10 => Some(String::from("MBC3+TIMER+RAM+BATTERY")),
            0x11 => Some(String::from("MBC3")),
            0x12 => Some(String::from("MBC3+RAM")),
            0x13 => Some(String::from("MBC3+RAM+BATTERY")),
            0x19 => Some(String::from("MBC5")),
            0x1A => Some(String::from("MBC5+RAM")),
            0x1B => Some(String::from("MBC5+RAM+BATTERY")),
            0x1C => Some(String::from("MBC5+RUMBLE")),
            0x1D => Some(String::from("MBC5+RUMBLE+RAM")),
            0x1E => Some(String::from("MBC5+RUMBLE+RAM+BATTERY")),
            0x20 => Some(String::from("MBC6")),
            0x22 => Some(String::from("MBC7+SENSOR+RUMBLE+RAM+BATTERY")),
            0xFC => Some(String::from("POCKET CAMERA")),
            0xFD => Some(String::from("BANDAI TAMA5")),
            0xFE => Some(String::from("HuC3")),
            0xFF => Some(String::from("HuC1+RAM+BATTERY")),
            _ => None
        }
    }

    fn get_rom_size(self: &Self) -> Option<u32> {       // Change to result because we should error if not valid size
        match self.rom_size {
            0x00 => Some(32_768),           // 2 Banks  (0 and 1 with no banking as they are just fixed)
            0x01 => Some(65_536),           // 4 Banks  (Each bank is 16KiB in all cases)
            0x02 => Some(131_072),          // 8 Banks  (Switch what bank being used at a given time)
            0x03 => Some(262_144),          // 16 Banks
            0x04 => Some(524_288),          // 32 Banks
            0x05 => Some(1_024_000),        // 64 Banks
            0x06 => Some(2_048_000),        // 128 Banks
            0x07 => Some(4_096_000),        // 256 Banks
            0x08 => Some(8_192_000),        // 512 Banks
            0x52 => Some(1_126_400),        // Probably doesnt exist
            0x53 => Some(1_228_800),        // Probably doesnt exist
            0x54 => Some(1_536_000),        // Probably doesnt exist
            _ => None
        }
    }
    
    fn get_ram_size(self: &Self) -> Option<u32> {   // Change to result because we should error if not valid size
        match self.ram_size {
            0x00 => Some(0),                // MBC2 will say 00 but it includes a builtin 512x4 bits 
            0x01 => Some(2_048),            // Source not provided (Replace with None? as no cartridge uses this)
            0x02 => Some(8_192),            // 1 Bank
            0x03 => Some(32_768),           // 4 Banks of 8KB
            0x04 => Some(131_072),          // 16 Banks of 8KB
            0x05 => Some(65_536),           // 8 Banks of 8KB
            _ => None
        }
    }

    fn get_publisher_name(self: &Self) -> Option<String> {
        let switch;
        if self.old_lisc_code == 0x33 {         // 0x33 means use the new liscensee code instead
            if self.new_lisc_code > 0xA4 {
                return None;
            }
            switch = self.new_lisc_code as u8;
        }
        else{
            switch = self.old_lisc_code;
        }
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
}
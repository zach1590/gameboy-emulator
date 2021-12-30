mod cartridge;

use std::fs;

fn main() {
    read_boot_rom();
}

// Try reading the DMG_ROM.bin
fn read_boot_rom() {
    let boot_rom_bytes = fs::read("roms/DMG_ROM.bin").unwrap();
    for (_, byte) in (&boot_rom_bytes).into_iter().enumerate(){
        println!("{:#04X}", byte);
    }
}
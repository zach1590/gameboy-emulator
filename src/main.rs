#![allow(dead_code)]

mod cartridge;
mod cpu;
mod emulator;
mod instruction;
mod mbc;
mod memory;

fn main() {
    let mut gameboy = emulator::Emulator::new();
    let rom = "roms/DMG_ROM.bin"; // Later replace this with command line arguments
    gameboy.load_cartridge(rom);
    //gameboy.run();
}

#![allow(dead_code)]

mod cartridge;
mod cpu;
mod emulator;
mod instruction;
mod mbc;
mod memory;
mod render;
mod timer;
mod io;

#[cfg(feature = "debug")]
mod debug;

fn main() {
    let game_path = "roms/cpu_instrs.gb"; // Later replace this with command line arguments
    let mut gameboy = emulator::Emulator::new();
    gameboy.insert_cartridge(game_path);
    gameboy.run();
}

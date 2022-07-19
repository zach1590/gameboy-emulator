#![allow(dead_code)]

mod cartridge;
mod cpu;
mod bus;
mod emulator;

mod mbc;
mod memory;

mod io;
mod timer;

mod graphics;
mod sprite;

#[cfg(feature = "debug")]
mod debug;

fn main() {
    let game_path = "roms\\tests\\mem_timing\\mem_timing.gb";
    let mut gameboy = emulator::Emulator::new();
    gameboy.insert_cartridge(game_path);
    gameboy.run();
}

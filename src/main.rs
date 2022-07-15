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
mod mbc1;
mod battery;
mod alu;

#[cfg(feature = "debug")]
mod debug;

fn main() {
    let game_path = "roms\\tests\\instr_timing\\instr_timing.gb";
    let mut gameboy = emulator::Emulator::new();
    gameboy.insert_cartridge(game_path);
    gameboy.run();
}

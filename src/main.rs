#![allow(dead_code)]

mod bus;
mod cpu;
mod emulator;

mod mbc;
mod memory;

mod io;
mod joypad;
mod timer;

mod graphics;

extern crate sdl2;

fn main() {
    let game_path = "roms\\tests\\cpu_instrs\\cpu_instrs.gb";
    let mut gameboy = emulator::Emulator::new();
    gameboy.setup_emulator(game_path);
    gameboy.run();
}

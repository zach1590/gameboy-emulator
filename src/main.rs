#![allow(dead_code)]

mod bus;
mod cpu;
mod emulator;

mod mbc;
mod memory;

mod io;
mod joypad;
mod serial;
mod timer;

mod graphics;

extern crate sdl2;

fn main() {
    let game_path = "roms\\dmg-acid2\\dmg-acid2.gb";
    let mut gameboy = emulator::Emulator::new();
    gameboy.setup_emulator(game_path);
    gameboy.run();
}

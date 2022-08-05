#![allow(dead_code)]
use std::env;

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
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        panic!("Not enough arguments! What game do you want to play!");
    }
    if args.len() > 2 {
        panic!("Too many arguments!");
    }
    let game_path = &args[1];
    let mut gameboy = emulator::Emulator::new();
    gameboy.setup_emulator(game_path);
    gameboy.run();
}

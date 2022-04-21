use super::cartridge;
use super::cpu;

use std::time::Instant;

pub struct Emulator {
    cpu: cpu::Cpu,
    cart: cartridge::Cartridge,
}

impl Emulator {
    pub fn new() -> Emulator {
        return Emulator {
            cpu: cpu::Cpu::new(),
            cart: cartridge::Cartridge::new(),
        };
    }

    pub fn insert_cartridge(self: &mut Self, game_path: &str) {
        let cart_mbc = self.cart.read_cartridge_header(game_path);
        self.cpu.set_mbc(cart_mbc);
        self.cart.checksum(self.cpu.get_memory());
    }

    pub fn run(self: &mut Self) {
        let mut wait_time;
        let mut previous_time: Instant = Instant::now();
        // Game loop
        loop {
            self.cpu.update_input();

            wait_time = (self.cpu.curr_cycles as f64) * self.cpu.period_nanos;
            while (previous_time.elapsed().as_nanos() as f64) < wait_time {}

            self.cpu.check_interrupts();

            if self.cpu.is_running {
                previous_time = Instant::now(); // Begin new clock timer
                self.cpu.execute(); // Instruction Decode and Execute
            } else {
                // Halted
                self.cpu.curr_cycles = 1;
            }
        }
    }
}

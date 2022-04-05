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

    pub fn run(self: &mut Self) {
        let mut wait_time: u128;
        let mut previous_time: Instant = Instant::now();
        // Game loop
        loop {
            wait_time = ((self.cpu.curr_cycles as f64) * self.cpu.period_nanos) as u128;
            while previous_time.elapsed().as_nanos() < wait_time {}

            if self.cpu.ime == true {
                self.cpu.handle_interrupt();
            }
            if self.cpu.ime_pending == true {
                self.cpu.ime_pending = false;
                self.cpu.ime = true; // Now interrupts should occur delayed one instruction
            }
            if self.cpu.is_running {
                previous_time = Instant::now(); // Begin new clock timer
                self.cpu.execute(); // Instruction Decode and Execute
            } else {
                self.cpu.curr_cycles = 1;
                self.cpu.wait_for_interrupt(); // ??
            }
        }
    }

    pub fn load_cartridge(self: &mut Self, rom_name: &str) {
        // In here lets read, initialize/load everything required from the cartridge
        self.cpu.load_cartridge(rom_name);
        self.cart.checksum(&self.cpu.get_memory());
    }
}

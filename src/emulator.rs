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

    // We just want the mbc type really, we wont bother with the nintendo logo boot
    pub fn insert_cartridge(self: &mut Self, game_path: &str) {
        let cart_mbc = self.cart.read_cartridge_header(game_path);
        self.cpu.set_mbc(cart_mbc);
        self.cart.checksum(self.cpu.get_memory());
    }

    pub fn run(self: &mut Self) {
        // Game loop
        loop {
            self.cpu.update_input();
            self.cpu.wait_and_sync();

            self.cpu.handle_timer_registers();  //Make sure timer/divider registers are synched
            self.cpu.check_interrupts();

            if self.cpu.is_running {
                self.cpu.reset_clock();
                self.cpu.execute(); // Instruction Decode and Execute
            } else {
                // Halted
                self.cpu.curr_cycles = 1;
            }
        }
    }
}

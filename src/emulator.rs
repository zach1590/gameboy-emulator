use super::cartridge;
use super::cpu;

#[cfg(feature = "debug")]
use super::debug;

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
        let checksum = self.cart.checksum(self.cpu.get_memory());
        self.cpu.dmg_init(checksum);
    }

    pub fn run(self: &mut Self) {

        // Game loop
        loop {
            self.cpu.update_input();

            // Goal is to get rid of this by performing cycle accuracy
            // self.cpu.handle_clocks(self.cpu.curr_cycles);

            self.cpu.check_interrupts();

            if self.cpu.is_running {
                self.cpu.curr_cycles = 0;
                self.cpu.execute();
            } else {
                // Halted
                self.cpu.handle_clocks(4);  // Should this be 1 or 4
                self.cpu.curr_cycles += 4;
            }

            #[cfg(feature = "debug")] {
                let io = self.cpu.get_memory_mut().get_io_mut();
                debug::update_serial_buffer(io);
            }

        }
    }
}

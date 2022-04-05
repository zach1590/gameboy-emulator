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
            self.cpu.update_input();

            wait_time = ((self.cpu.curr_cycles as f64) * self.cpu.period_nanos) as u128;
            while previous_time.elapsed().as_nanos() < wait_time {}

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

    /*
    Program Execution will need to be:
            use Cartridge header data to determine sizes for CartridgeMemory vectors
            create functions in Cartridge that take a reference to memory and find required information
            create CartridgeMemory and load it using the array (skip the rom/ram that will go into memory)
            create Memory struct using the above and reading the array (Only read the first 32KiB and 8KiB)
            gbCPU should keep references to the Cartridge and Memory structs

        The above should return an initilized CPU though we shouldnt need Cartridge anymore
    */
    pub fn load_cartridge(self: &mut Self, rom_name: &str) {
        // In here lets read, initialize/load everything required from the cartridge
        self.cpu.load_cartridge(rom_name);
        self.cart.checksum(&self.cpu.get_memory());
    }
}

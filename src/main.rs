#![allow(dead_code)]

mod cartridge;
mod memory;
mod cpu;
mod instruction;

use cpu::Cpu;


fn main() {
    /*
    Program Execution will need to be:
        Create gbCPU (pass name of file/game/ROM)
            load entire file into an array we will drop this array later (when we go out of scope)
            create Cartridge and load the cartridge header using array
                Attempt the checksum
            use Cartridge header data to determine sizes for CartridgeMemory vectors
            create CartridgeMemory and load it using the array (skip the rom/ram that will go into memory)
            create Memory struct using the above and reading the array (Only read the first 32KiB and 8KiB)
            gbCPU should keep references to the Cartridge and Memory structs

        The above should return an initilized CPU though we shouldnt need Cartridge anymore
    */
    let mut gameboy = Cpu::new();
    let cartridge = "roms/DMG_ROM.bin";         // Later replace this with command line arguments
    gameboy.load_cartridge(cartridge);

    //gameboy.run();

    // println!("{}", 1u8.wrapping_sub(0xF));
}
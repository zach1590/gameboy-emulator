mod cartridge;
mod memory;

use std::fs;

fn main() {
    /*
    Program Execution will need to be:
        Create gbCPU (pass name of file/game/ROM)
            load entire file into an array we will drop this array later (when we go out of scope)
            create Cartridge and load the cartridge header using array
            use Cartridge header data to determine sizes for CartridgeMemory vectors
            create CartridgeMemory and load it using the array (skip the rom/ram that will go into memory)
            create Memory struct using the above and reading the array (Only read the first 32KiB and 8KiB)
            gbCPU should keep references to the Cartridge and Memory structs

        The above should return an initilized CPU though we shouldnt need Cartridge anymore
    */
    read_boot_rom();
}

// Try reading the DMG_ROM.bin
fn read_boot_rom() {
    let boot_rom_bytes = fs::read("roms/DMG_ROM.bin").unwrap();
    for (_, byte) in (&boot_rom_bytes).into_iter().enumerate(){
        println!("{:#04X}", byte);
    }
}
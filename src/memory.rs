struct Memory {
    mem: [u8; 65_536],              // 16 bit address = 64KiB of memory
    cart_mem: CartridgeMemory,      // Extra Memory Banks that may be needed 
}


// rom/ram_data are vectors of 8KiB arrays (Do rom/ram_size divided by 8192 to get the number of arrays)
// rom/ram size will come from the cartridge header and get_rom/ram_size methods
// Each array inside the vector will represent a possible bank to switch to unless no extra exists on cartridge
struct CartridgeMemory {
    rom_data_banks: Vec<[u8; 8_192]>,              // Initialize this vector with 0 if the rom_size is only 32KiB
    ram_data_banks: Vec<[u8; 8_192]>,              // Initialize this vector with 0 if the ram_size is only 8KiB
}
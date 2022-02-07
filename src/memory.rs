pub struct Memory {
    pub onboard: [u8; 65_536],              // 16 bit address = 64KiB of memory
    pub cart_mem: CartridgeMemory,      // Extra Memory Banks that may be needed 
}

impl Memory {
    pub fn new() -> Memory {
        return Memory {
            onboard: [0; 65_536],
            cart_mem: CartridgeMemory {
                rom_data_banks: Vec::new(),     // We will push on the u8 arrays once 
                ram_data_banks: Vec::new(),     // we find out how many are required
            },
        }
    }

    // We only read? No reason for mutable self
    pub fn read_byte(self: &Self, location: u16) -> u8 {
        // Implement switching before this? Look into how all that works
        return self.onboard[location as usize];
    }

    pub fn write_bytes(self: &mut Self, location: u16, data: Vec<u8>){
        // Important to keep track of the indices where something is being placed when we have actual cartridge
        let location = location as usize;
        self.onboard[location..data.len()].copy_from_slice(&data[..]);
    }
}

// rom/ram_data are vectors of 8KiB arrays (Do rom/ram_size divided by 8192 to get the number of arrays)
// rom/ram size will come from the cartridge header and get_rom/ram_size methods
// Each array inside the vector will represent a possible bank to switch to unless no extra exists on cartridge
pub struct CartridgeMemory {
    rom_data_banks: Vec<[u8; 8_192]>,              // Initialize this vector with 0 if the rom_size is only 32KiB
    ram_data_banks: Vec<[u8; 8_192]>,              // Initialize this vector with 0 if the ram_size is only 8KiB
}
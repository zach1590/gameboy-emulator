pub struct Memory {
    onboard: [u8; 65_536],     // 16 bit address = 64KiB of memory
    cart_mem: CartridgeMemory, // Extra Memory Banks that may be needed
    i_enable: u8,
    i_fired: u8,
}

impl Memory {
    pub fn new() -> Memory {
        return Memory {
            onboard: [0; 65_536],
            cart_mem: CartridgeMemory {
                rom_data_banks: Vec::new(), // We will push on the u8 arrays once
                ram_data_banks: Vec::new(), // we find out how many are required
            },
            i_enable: 0,
            i_fired: 0,
        };
    }

    // We only read? No reason for mutable self
    pub fn read_byte(self: &Self, location: u16) -> u8 {
        // Implement switching before this? Look into how all that works
        return match location {
            0xFF0F => self.i_fired,
            0xFFFF => self.i_enable,
            x => self.onboard[x as usize],
        };
    }

    // Write a single byte to at the location
    pub fn write_byte(self: &mut Self, location: u16, data: u8) {
        // Important to keep track of the indices where something is being placed when we have actual cartridge
        let location = location as usize;
        match location {
            0xFF0F => {
                self.i_fired = data;
                self.onboard[0xFF0F] = data;
            }
            0xFFFF => {
                self.i_enable = data;
                self.onboard[0xFFFF] = data;
            }
            x => self.onboard[x as usize] = data,
        }
    }

    // Write multiple bytes into memory starting from location
    // Should probably do something else here
    pub fn write_bytes(self: &mut Self, location: u16, data: Vec<u8>) {
        // Important to keep track of the indices where something is being placed when we have actual cartridge
        let location = location as usize;
        self.onboard[location..location + data.len()].copy_from_slice(&data[..]);
        self.i_fired = self.onboard[0xFF0F];
        self.i_enable = self.onboard[0xFFFF];
    }
}

// rom/ram_data are vectors of 8KiB arrays (Do rom/ram_size divided by 8192 to get the number of arrays)
// rom/ram size will come from the cartridge header and get_rom/ram_size methods
// Each array inside the vector will represent a possible bank to switch to unless no extra exists on cartridge
struct CartridgeMemory {
    rom_data_banks: Vec<[u8; 8_192]>, // Initialize this vector with 0 if the rom_size is only 32KiB
    ram_data_banks: Vec<[u8; 8_192]>, // Initialize this vector with 0 if the ram_size is only 8KiB
}

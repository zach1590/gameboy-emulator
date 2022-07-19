// Each one may also be addressed as just the upper or lower 8 bits
pub struct Registers {
    pub af: u16, // A: accumulator, F: flags as 0bZNHC0000
    pub bc: u16,
    pub de: u16,
    pub hl: u16,
}

impl Registers {
    pub fn new() -> Registers {
        return Registers {
            af: 0x0000,
            bc: 0x0000,
            de: 0x0000,
            hl: 0x0000,
        };
    }

    pub fn dmg_init(self: &mut Self, checksum: u8) {
        if checksum == 0x00 {
            self.af = 0x0180;
        } else {
            self.af = 0x01B0;
        }
        self.bc = 0x0013;
        self.de = 0x00D8;
        self.hl = 0x014D;
    } 

    // returns true if z is set
    pub fn get_z(self: &Self) -> bool {
        ((self.af & 0x0080) >> 7) == 1
    }
    // returns true if n is set
    pub fn get_n(self: &Self) -> bool {
        ((self.af & 0x0040) >> 6) == 1
    }
    // returns true if h is set
    pub fn get_h(self: &Self) -> bool {
        ((self.af & 0x0020) >> 5) == 1
    }
    // returns true if c is set
    pub fn get_c(self: &Self) -> bool {
        ((self.af & 0x0010) >> 4) == 1
    }
    // Registers are stored as big endian so its easier in my head
    // returns the given register as 2 u8s in a tuple as (High, Low)
    pub fn get_hi_lo(xy: u16) -> (u8, u8) {
        return ((xy >> 8) as u8, xy as u8);
    }
    pub fn get_hi(xy: u16) -> u8 {
        return (xy >> 8) as u8;
    }
    pub fn get_lo(xy: u16) -> u8 {
        return xy as u8;
    }

    pub fn set_hi(reg: u16, byte: u8) -> u16 {
        let mut new_reg = reg & 0x00FF;
        new_reg = new_reg | ((byte as u16) << 8);
        return new_reg;
    }
    pub fn set_lo(reg: u16, byte: u8) -> u16 {
        let mut new_reg = reg & 0xFF00;
        new_reg = new_reg | (byte as u16);
        return new_reg;
    }
}

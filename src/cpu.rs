use super::memory::Memory;
use std::fs;

pub struct CPU {
    
    mem: Memory,
    reg: Registers,
    pc: u16,                // Program Counter
    sp: u16,                // Stack Pointer

}

impl CPU {
    pub fn new() -> CPU{
        return CPU {
            mem: Memory::new(),
            reg: Registers::new(),
            pc: 0,
            sp: 0,
        }
    }

    pub fn load_cartridge(self: Self, cartridge: &str) {
        let boot_rom_bytes = fs::read(cartridge).unwrap();
        for (_, byte) in (&boot_rom_bytes).into_iter().enumerate(){
            println!("{:#04X}", byte);
        }
        // Now put this into memory struct instead of just reading it
    }
}

// Each one may also be addressed as just the upper or lower 8 bits
struct Registers {
    af: u16,            // A: accumulator, F: flags
    bc: u16,
    de: u16,
    hl: u16,
}

impl Registers {
    pub fn new() -> Registers{
        return Registers {
            af: 0,
            bc: 0,
            de: 0,
            hl: 0,
        }
    }
    // returns the given register as 2 u8s in a tuple as (High, Low)
    fn get_hi_lo(xy: u16) -> (u8, u8){
        return (
            (xy >> 8) as u8,
            xy as u8
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_register_destructuring(){
        let z1: u16 = 0xABCD;
        let z2: u16 = 0xABC;
        let z3: u16 = 0xAB;
        let z4: u16 = 0xA;

        let (high, low): (u8, u8) = Registers::get_hi_lo(z1);
        assert_eq!(high, 0xAB);
        assert_eq!(low, 0xCD);

        let (high, low): (u8, u8) = Registers::get_hi_lo(z2);
        assert_eq!(high, 0x0A);
        assert_eq!(low, 0xBC);

        let (high, low): (u8, u8) = Registers::get_hi_lo(z3);
        assert_eq!(high, 0x00);
        assert_eq!(low, 0xAB);

        let (high, low): (u8, u8) = Registers::get_hi_lo(z4);
        assert_eq!(high, 0x00);
        assert_eq!(low, 0x0A);            // A: accumulator, F: flags
    }
}
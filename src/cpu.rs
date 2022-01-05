use super::memory::Memory;
use std::fs;

pub struct CPU {
    mem: Memory,
    reg: Registers,
    pc: u16,                // Program Counter
    sp: u16,                // Stack Pointer
    vliw: bool,             // For now keep a flag that says whether the next instruction is 8 or 16 bits
}

impl CPU {
    pub fn new() -> CPU{
        return CPU {
            mem: Memory::new(),
            reg: Registers::new(),
            pc: 0,
            sp: 0,
            vliw: false,
        }
    }

    // In here lets read, initialize/load everything required from the cartridge
    pub fn load_cartridge(self: &mut Self, cartridge: &str) {
        let boot_rom_bytes = fs::read(cartridge).unwrap();
        self.mem.onboard[0..boot_rom_bytes.len()].copy_from_slice(&boot_rom_bytes[..]);
        // Important to keep track of the indices where something is being placed when we have actual cartridge
        // for (_, byte) in (&self.mem.mem[..512]).into_iter().enumerate(){
        //     println!("{:#04X}", byte);
        // }
    }

    // Modify this to take an opcode/instruction - These can be either 8 or 16 bits
    // Dont know how to handle differing size instructions yet
    pub fn execute(self: &mut Self, opcode: u16){
        // Create a method for every instruction
        // Create a match arm here that then calls the correct function based on the instruction
    }

    pub fn run(self: &mut Self){
        
        let mut opcode: u16;
        // Game loop
        loop {
            /*
                Gameboy instructions take differing amounts of clock cycles to complete
                In the functions that decode the instruction, set the clock cycles the instruction should
                be taking as a value in the CPU (add a new field)
                    In main loop get the current time
                    compare the current time with the previous time
                    execute next instruction only if elasped_time >= cycles*clock_speed
            */

            opcode = self.mem.onboard[self.pc as usize] as u16;
            self.pc += 1;
            // Might need a function that sets whether a VLIW is coming up or even change this completely
            if self.vliw {
                opcode = (opcode << 8) + ((self.mem.onboard[self.pc as usize]) as u16);
                self.pc += 1;
            }
            self.execute(opcode);

        }
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
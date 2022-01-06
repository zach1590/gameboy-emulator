use super::memory::Memory;
use super::instruction::Instruction;
use super::instruction;
use std::fs;

pub struct CPU {
    mem: Memory,
    pub reg: Registers,
    pub pc: u16,                // Program Counter
    pub sp: u16,                // Stack Pointer
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

    // In here lets read, initialize/load everything required from the cartridge
    pub fn load_cartridge(self: &mut Self, cartridge: &str) {
        let boot_rom_bytes = fs::read(cartridge).unwrap();
        self.mem.onboard[0..boot_rom_bytes.len()].copy_from_slice(&boot_rom_bytes[..]);
        // Important to keep track of the indices where something is being placed when we have actual cartridge
        // for (_, byte) in (&self.mem.onboard[..512]).into_iter().enumerate(){
        //     println!("{:#04X}", byte);
        // }
    }

    fn execute(self: &mut Self, opcode: u8){
        let i = Instruction::get_instruction(opcode);

        if i.values == (0x0C, 0x0B){
            let opcode = self.mem.onboard[self.pc as usize];
            self.pc += 1;
            let cb_i = Instruction::get_instruction(opcode);
            self.match_prefix_instruction(cb_i);
        } else {
            self.match_instruction(i);
        }
    }

    pub fn run(self: &mut Self){
        let opcode: u8;
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

            opcode = self.mem.onboard[self.pc as usize];
            self.pc += 1;
            self.execute(opcode);
            break;
        }
    }

    pub fn match_prefix_instruction(self: &mut Self, i: Instruction){

    }
    pub fn match_instruction(self: &mut Self, i: Instruction){
        // Create a method for every instruction
        match i.values {
            (0x00, 0x01) => { 
                let hi = self.mem.onboard[self.pc as usize];
                let lo = self.mem.onboard[(self.pc + 1) as usize];
                self.pc = self.pc + 2;
                instruction::load_d16(&mut self.reg.bc, hi, lo);
            },
            (0x01, 0x01) => {
                let hi = self.mem.onboard[self.pc as usize];
                let lo = self.mem.onboard[(self.pc + 1) as usize];
                self.pc = self.pc + 2;
                instruction::load_d16(&mut self.reg.de, hi, lo);
            },
            (0x02, 0x01) => {
                let hi = self.mem.onboard[self.pc as usize];
                let lo = self.mem.onboard[(self.pc + 1) as usize];
                self.pc = self.pc + 2;
                instruction::load_d16(&mut self.reg.hl, hi, lo);
            },
            (0x03, 0x01) => {
                let hi = self.mem.onboard[self.pc as usize];
                let lo = self.mem.onboard[(self.pc + 1) as usize];
                self.pc = self.pc + 2;
                instruction::load_d16(&mut self.sp, hi, lo);
            },
            _ => panic!("Opcode not supported"),
        }
    }
}

// Each one may also be addressed as just the upper or lower 8 bits
pub struct Registers {
    pub af: u16,            // A: accumulator, F: flags
    pub bc: u16,
    pub de: u16,
    pub hl: u16,
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

    #[test]
    fn test_load_d16(){
        let mut cpu = CPU::new();
        instruction::load_d16(&mut cpu.reg.bc, 0xA7, 0xFF);
        instruction::load_d16(&mut cpu.reg.de, 0xF0, 0xFF);
        instruction::load_d16(&mut cpu.reg.hl, 0x01, 0xFF);
        instruction::load_d16(&mut cpu.sp, 0xFF, 0x00);
    
        assert_eq!(cpu.reg.bc, 0xA7FF);
        assert_eq!(cpu.reg.de, 0xF0FF);
        assert_eq!(cpu.reg.hl, 0x01FF);
        assert_eq!(cpu.sp, 0xFF00);
    }
}
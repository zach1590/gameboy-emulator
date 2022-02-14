use super::memory::Memory;
use super::instruction::Instruction;
use super::instruction;
use std::fs;
use std::time::{Instant};

pub struct Cpu {
    mem: Memory,
    period_nanos: f64,          // Time it takes for a clock cycle in nanoseconds
    pub reg: Registers,
    pub pc: u16,                // Program Counter
    pub sp: u16,                // Stack Pointer
    pub curr_cycles: usize,     // The number of cycles the current instruction should take to execute
}

impl Cpu {
    pub fn new() -> Cpu{
        return Cpu {
            mem: Memory::new(),
            period_nanos: 238.418579,
            reg: Registers::new(),
            pc: 0,
            sp: 0,
            curr_cycles: 0,
        }
    }

    // In here lets read, initialize/load everything required from the cartridge
    pub fn load_cartridge(self: &mut Self, cartridge: &str) {
        let boot_rom_bytes = fs::read(cartridge).unwrap();
        self.mem.write_bytes(0, boot_rom_bytes);
        // for i in 0..257 {
        //     println!("{:#04X}", self.mem.read_byte(i as u16));
        // }
    }

    fn execute(self: &mut Self, opcode: u8){
        let i = Instruction::get_instruction(opcode);

        if i.values == (0x0C, 0x0B){
            let opcode = self.mem.read_byte(self.pc);
            self.pc += 1;
            let cb_i = Instruction::get_instruction(opcode);
            self.match_prefix_instruction(cb_i);
        } else {
            self.match_instruction(i);
        }
    }

    pub fn run(self: &mut Self){
        let mut opcode: u8;
        let mut wait_time: u128;
        let mut previous_time: Instant = Instant::now();
        // Game loop
        loop {

            wait_time = ((self.curr_cycles as f64)*self.period_nanos) as u128;
            while previous_time.elapsed().as_nanos() <=  wait_time {
                // Do Nothing
                // Maybe take user input in here
            }

            previous_time = Instant::now();             // Begin new clock timer
            opcode = self.mem.read_byte(self.pc);       // Instruction Fetch
            self.pc += 1;
            self.execute(opcode);                       // Instruction Decode and Execute

            // println!("cycles: {}", self.curr_cycles);
            // println!("stack pointer: {:#04X}", self.sp);
            // println!("program counter location: {:#04X}", self.mem.read_byte(self.pc));
            //break;
        }
    }

    pub fn match_prefix_instruction(self: &mut Self, _i: Instruction){
        
    }
    pub fn match_instruction(self: &mut Self, i: Instruction){
        // Create a method for every instruction
        match i.values {
            (0x00, 0x01) => { 
                // Load 16 bit immediate into BC
                let (hi, lo) = self.two_bytes();
                instruction::load_d16(&mut self.reg.bc, &mut self.curr_cycles, hi, lo);
            },
            (0x01, 0x01) => {
                // Load 16 bit immediate into DE
                let (hi, lo) = self.two_bytes();
                instruction::load_d16(&mut self.reg.de, &mut self.curr_cycles, hi, lo);
            },
            (0x02, 0x01) => {
                // Load 16 bit immediate into HL
                let (hi, lo) = self.two_bytes();
                instruction::load_d16(&mut self.reg.hl, &mut self.curr_cycles, hi, lo);
            },
            (0x03, 0x01) => {
                // Load 16 bit immediate into SP
                let (hi, lo) = self.two_bytes();
                instruction::load_d16(&mut self.sp, &mut self.curr_cycles, hi, lo);
            },
            (0x08, 0x00 | 0x01 | 0x02 | 0x03 | 0x04 | 0x05 | 0x06 |0x07) => {
                // A = A ADD R
                let add_value = self.get_register_value_from_opcode(i.values.1);
                //eprintln!("value: {:#04X}, opcode: {:#04X}", xor_value, i.values.1);
                instruction::a_add_r(&mut self.reg.af, add_value, &mut self.curr_cycles, i.values.1)
            },
            (0x09, 0x00 | 0x01 | 0x02 | 0x03 | 0x04 | 0x05 | 0x06 |0x07) => {
                // A = A SUB R
                let sub_value = self.get_register_value_from_opcode(i.values.1);
                //eprintln!("value: {:#04X}, opcode: {:#04X}", xor_value, i.values.1);
                instruction::a_sub_r(&mut self.reg.af, sub_value, &mut self.curr_cycles, i.values.1)
            },
            (0x0A, 0x00 | 0x01 | 0x02 | 0x03 | 0x04 | 0x05 | 0x06 |0x07) => {
                // A = A AND R
                let and_value = self.get_register_value_from_opcode(i.values.1);
                //eprintln!("value: {:#04X}, opcode: {:#04X}", xor_value, i.values.1);
                instruction::a_and_r(&mut self.reg.af, and_value, &mut self.curr_cycles, i.values.1)
            },
            (0x0A, 0x08 | 0x09 | 0x0A | 0x0B | 0x0C | 0x0D | 0x0E |0x0F) => {
                // A = A XOR R
                let xor_value = self.get_register_value_from_opcode(i.values.1);
                //eprintln!("value: {:#04X}, opcode: {:#04X}", xor_value, i.values.1);
                instruction::a_xor_r(&mut self.reg.af, xor_value, &mut self.curr_cycles, i.values.1)
            },
            (0x0B, 0x00 | 0x01 | 0x02 | 0x03 | 0x04 | 0x05 | 0x06 |0x07) => {
                // A = A OR R
                let or_value = self.get_register_value_from_opcode(i.values.1);
                //eprintln!("value: {:#04X}, opcode: {:#04X}", xor_value, i.values.1);
                instruction::a_or_r(&mut self.reg.af, or_value, &mut self.curr_cycles, i.values.1)
            },
            _ => panic!("Opcode not supported"),
        }
    }
    
    // Instructions that are 3 bytes long will call this method to get the next two bytes required
    fn two_bytes(self: &mut Self) -> (u8, u8) {
        let hi = self.mem.read_byte(self.pc);
        let lo = self.mem.read_byte(self.pc + 1);
        self.pc = self.pc + 2;
        return (hi, lo);
    }

    // Instructions that are 2 bytes long will call this method to get the next byte required
    fn one_byte(self: &mut Self) -> u8 {
        let byte = self.mem.read_byte(self.pc);
        self.pc = self.pc + 1;
        return byte;
    }

    // Takes the lower 8 bits of the opcode and returns the value of the register needed for the instruction
    // Should work for obtaining the second operand of virtually all opcodes
    fn get_register_value_from_opcode(self: &Self, opcode_lo: u8) -> u8 {
        return match opcode_lo {
            0x00 | 0x08 => { 
                Registers::get_hi_lo(self.reg.bc).0
            },
            0x01 | 0x09 => {
                Registers::get_hi_lo(self.reg.bc).1
            },
            0x02 | 0x0A => {
                Registers::get_hi_lo(self.reg.de).0
            },
            0x03 | 0x0B => {
                Registers::get_hi_lo(self.reg.de).1
            },
            0x04 | 0x0C => {
                Registers::get_hi_lo(self.reg.hl).0
            },
            0x05 | 0x0D => {
                Registers::get_hi_lo(self.reg.hl).1
            },
            0x06 | 0x0E => {
                self.mem.read_byte(self.reg.hl)
            }
            0x07 | 0x0F => {
                Registers::get_hi_lo(self.reg.af).0
            },
            _ => panic!("Expected Value between 0x00 and 0x0F")
        };
    }
}

// Each one may also be addressed as just the upper or lower 8 bits
pub struct Registers {
    pub af: u16,            // A: accumulator, F: flags as 0bZNHC0000
    pub bc: u16,
    pub de: u16,
    pub hl: u16,
}

impl Registers {
    fn new() -> Registers{
        return Registers {
            af: 0,
            bc: 0,
            de: 0,
            hl: 0,
        }
    }
    // returns the given register as 2 u8s in a tuple as (High, Low)
    pub fn get_hi_lo(xy: u16) -> (u8, u8){
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
        let mut cpu = Cpu::new();
        instruction::load_d16(&mut cpu.reg.bc, &mut cpu.curr_cycles, 0xA7, 0xFF);
        instruction::load_d16(&mut cpu.reg.de,  &mut cpu.curr_cycles, 0xF0, 0xFF);
        instruction::load_d16(&mut cpu.reg.hl,  &mut cpu.curr_cycles, 0x01, 0xFF);
        instruction::load_d16(&mut cpu.sp,  &mut cpu.curr_cycles, 0xFF, 0x00);
    
        assert_eq!(cpu.reg.bc, 0xA7FF);
        assert_eq!(cpu.reg.de, 0xF0FF);
        assert_eq!(cpu.reg.hl, 0x01FF);
        assert_eq!(cpu.sp, 0xFF00);
    }

    #[test]
    fn test_xor_a(){
        let mut cpu = Cpu::new();

        cpu.reg.af = 0xA800;
        cpu.reg.bc = 0xA800;
        cpu.match_instruction(Instruction::get_instruction(0xA8));
        assert_eq!(cpu.reg.af, 0b0000000010000000);
        assert_eq!(cpu.curr_cycles, 4);

        cpu.reg.af = 0xA800;
        cpu.reg.bc = 0x00A8;
        cpu.match_instruction(Instruction::get_instruction(0xA9));
        assert_eq!(cpu.reg.af, 0b0000000010000000);
        assert_eq!(cpu.curr_cycles, 4);

        cpu.reg.af = 0xA800;
        cpu.reg.de = 0xFE01;
        cpu.match_instruction(Instruction::get_instruction(0xAA));
        assert_eq!(cpu.reg.af, 0x5600);
        assert_eq!(cpu.curr_cycles, 4);

        cpu.reg.af = 0xA800;
        cpu.reg.de = 0x01FE;
        cpu.match_instruction(Instruction::get_instruction(0xAB));
        assert_eq!(cpu.reg.af, 0x5600);
        assert_eq!(cpu.curr_cycles, 4);

        cpu.reg.af = 0xA800;
        cpu.reg.hl = 0xF0FE;
        cpu.match_instruction(Instruction::get_instruction(0xAC));
        assert_eq!(cpu.reg.af, 0x5800);
        assert_eq!(cpu.curr_cycles, 4);

        cpu.reg.af = 0xA800;
        cpu.reg.hl = 0xFEF0;
        cpu.match_instruction(Instruction::get_instruction(0xAD));
        assert_eq!(cpu.reg.af, 0x5800);
        assert_eq!(cpu.curr_cycles, 4);

        cpu.reg.af = 0xA800;
        cpu.reg.hl = 0xFFF0;
        cpu.match_instruction(Instruction::get_instruction(0xAE));
        assert_eq!(cpu.reg.af, 0xA800);
        assert_eq!(cpu.curr_cycles, 8);

        cpu.reg.af = 0xA800;
        cpu.match_instruction(Instruction::get_instruction(0xAF));
        assert_eq!(cpu.reg.af, 0x0080);
        assert_eq!(cpu.curr_cycles, 4);
        
    }

    #[test]
    fn test_and_a(){
        let mut cpu = Cpu::new();

        cpu.reg.af = 0xA800;
        cpu.reg.bc = 0xA800;
        cpu.match_instruction(Instruction::get_instruction(0xA0));
        assert_eq!(cpu.reg.af, 0b1010_1000_0010_0000);
        assert_eq!(cpu.curr_cycles, 4);

        cpu.reg.af = 0xA800;
        cpu.reg.bc = 0x00A8;
        cpu.match_instruction(Instruction::get_instruction(0xA1));
        assert_eq!(cpu.reg.af, 0b1010_1000_0010_0000);
        assert_eq!(cpu.curr_cycles, 4);

        cpu.reg.af = 0xA801;
        cpu.reg.de = 0xFE01;
        cpu.match_instruction(Instruction::get_instruction(0xA2));
        assert_eq!(cpu.reg.af, 0xA821);
        assert_eq!(cpu.curr_cycles, 4);

        cpu.reg.af = 0xA806;
        cpu.reg.de = 0x01FE;
        cpu.match_instruction(Instruction::get_instruction(0xA3));
        assert_eq!(cpu.reg.af, 0xA826);
        assert_eq!(cpu.curr_cycles, 4);

        cpu.reg.af = 0xA8F8;
        cpu.reg.hl = 0x00FE;
        cpu.match_instruction(Instruction::get_instruction(0xA4));
        assert_eq!(cpu.reg.af, 0x00A8);
        assert_eq!(cpu.curr_cycles, 4);

        cpu.reg.af = 0xA8FF;
        cpu.reg.hl = 0xFE00;
        cpu.match_instruction(Instruction::get_instruction(0xA5));
        assert_eq!(cpu.reg.af, 0x00AF);
        assert_eq!(cpu.curr_cycles, 4);

        cpu.reg.af = 0xA8CD;
        cpu.reg.hl = 0xFFF0;
        cpu.mem.write_bytes(cpu.reg.hl, vec!(0x7A));    // 0111 and 1010 = 0010
        cpu.match_instruction(Instruction::get_instruction(0xA6));
        assert_eq!(cpu.reg.af, 0x282D);
        assert_eq!(cpu.curr_cycles, 8);

        cpu.reg.af = 0xA823;
        cpu.match_instruction(Instruction::get_instruction(0xA7));
        assert_eq!(cpu.reg.af, 0xA823);
        assert_eq!(cpu.curr_cycles, 4);
        
    }
    #[test]
    fn test_or_a(){
        let mut cpu = Cpu::new();

        cpu.reg.af = 0xA800;
        cpu.reg.bc = 0xA800;
        cpu.match_instruction(Instruction::get_instruction(0xB0));
        assert_eq!(cpu.reg.af, 0xA800);
        assert_eq!(cpu.curr_cycles, 4);

        cpu.reg.af = 0xA800;
        cpu.reg.bc = 0x00A8;
        cpu.match_instruction(Instruction::get_instruction(0xB1));
        assert_eq!(cpu.reg.af, 0xA800);
        assert_eq!(cpu.curr_cycles, 4);

        cpu.reg.af = 0xA801;
        cpu.reg.de = 0xFE01;
        cpu.match_instruction(Instruction::get_instruction(0xB2));
        assert_eq!(cpu.reg.af, 0xFE01);
        assert_eq!(cpu.curr_cycles, 4);

        cpu.reg.af = 0xA806;
        cpu.reg.de = 0x01FE;
        cpu.match_instruction(Instruction::get_instruction(0xB3));
        assert_eq!(cpu.reg.af, 0xFE06);
        assert_eq!(cpu.curr_cycles, 4);

        cpu.reg.af = 0x00F8;
        cpu.reg.hl = 0x00FE;
        cpu.match_instruction(Instruction::get_instruction(0xB4));
        assert_eq!(cpu.reg.af, 0x0088);
        assert_eq!(cpu.curr_cycles, 4);

        cpu.reg.af = 0xA8FF;
        cpu.reg.hl = 0xFE00;
        cpu.match_instruction(Instruction::get_instruction(0xB5));
        assert_eq!(cpu.reg.af, 0xA80F);
        assert_eq!(cpu.curr_cycles, 4);

        cpu.reg.af = 0xA8CD;
        cpu.reg.hl = 0xFFF0;
        cpu.mem.write_bytes(cpu.reg.hl, vec!(0x7A));    // 0111 and 1010 = 0010
        cpu.match_instruction(Instruction::get_instruction(0xB6));
        assert_eq!(cpu.reg.af, 0xFA0D);
        assert_eq!(cpu.curr_cycles, 8);

        cpu.reg.af = 0xA823;
        cpu.match_instruction(Instruction::get_instruction(0xB7));
        assert_eq!(cpu.reg.af, 0xA803);
        assert_eq!(cpu.curr_cycles, 4);
        
    }
    #[test]
    fn test_add_a(){
        let mut cpu = Cpu::new();

        cpu.reg.af = 0xA800;
        cpu.reg.bc = 0xA800;
        cpu.match_instruction(Instruction::get_instruction(0x80));
        assert_eq!(cpu.reg.af, 0x5030);
        assert_eq!(cpu.curr_cycles, 4);

        cpu.reg.af = 0xA800;
        cpu.reg.bc = 0x00A8;
        cpu.match_instruction(Instruction::get_instruction(0x81));
        assert_eq!(cpu.reg.af, 0x5030);
        assert_eq!(cpu.curr_cycles, 4);

        cpu.reg.af = 0xA801;
        cpu.reg.de = 0xFE01;
        cpu.match_instruction(Instruction::get_instruction(0x82));
        assert_eq!(cpu.reg.af, 0xA631);
        assert_eq!(cpu.curr_cycles, 4);

        cpu.reg.af = 0xA806;
        cpu.reg.de = 0x01FE;
        cpu.match_instruction(Instruction::get_instruction(0x83));
        assert_eq!(cpu.reg.af, 0xA636);
        assert_eq!(cpu.curr_cycles, 4);

        cpu.reg.af = 0x00F8;
        cpu.reg.hl = 0x00FE;
        cpu.match_instruction(Instruction::get_instruction(0x84));
        assert_eq!(cpu.reg.af, 0x0088);
        assert_eq!(cpu.curr_cycles, 4);

        cpu.reg.af = 0xA8FF;
        cpu.reg.hl = 0xFE00;
        cpu.match_instruction(Instruction::get_instruction(0x85));
        assert_eq!(cpu.reg.af, 0xA80F);
        assert_eq!(cpu.curr_cycles, 4);

        cpu.reg.af = 0xA8CD;
        cpu.reg.hl = 0xFFF0;
        cpu.mem.write_bytes(cpu.reg.hl, vec!(0x74));
        cpu.match_instruction(Instruction::get_instruction(0x86));
        assert_eq!(cpu.reg.af, 0x1C1D);
        assert_eq!(cpu.curr_cycles, 8);

        cpu.reg.af = 0xA8CD;
        cpu.reg.hl = 0xFFF0;
        cpu.mem.write_bytes(cpu.reg.hl, vec!(0x49));
        cpu.match_instruction(Instruction::get_instruction(0x86));
        assert_eq!(cpu.reg.af, 0xF12D);
        assert_eq!(cpu.curr_cycles, 8);

        cpu.reg.af = 0xA8CD;
        cpu.reg.hl = 0xFFF0;
        cpu.mem.write_bytes(cpu.reg.hl, vec!(0x44));
        cpu.match_instruction(Instruction::get_instruction(0x86));
        assert_eq!(cpu.reg.af, 0xEC0D);
        assert_eq!(cpu.curr_cycles, 8);

        cpu.reg.af = 0xA823;
        cpu.match_instruction(Instruction::get_instruction(0x87));
        assert_eq!(cpu.reg.af, 0x5033);
        assert_eq!(cpu.curr_cycles, 4);
        
    }
}
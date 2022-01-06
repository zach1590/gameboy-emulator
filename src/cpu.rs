use super::memory::Memory;
use super::instruction::Instruction;
use super::instruction;
use std::fs;
use std::time::{Duration, Instant};

pub struct CPU {
    mem: Memory,
    period_nanos: f64,          // Time it takes for a clock cycle in nanoseconds
    pub reg: Registers,
    pub pc: u16,                // Program Counter
    pub sp: u16,                // Stack Pointer
    pub curr_cycles: usize,     // The number of cycles the current instruction should take to execute
}

impl CPU {
    pub fn new() -> CPU{
        return CPU {
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
        let mut opcode: u8;
        let mut wait_time: u128;
        let mut previous_time: Instant = Instant::now();
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
            wait_time = ((self.curr_cycles as f64)*self.period_nanos) as u128;
            while previous_time.elapsed().as_nanos() <=  wait_time {
                // Do Nothing
                // Maybe take user input in here
            }

            // Begin clock timer
            previous_time = Instant::now();

            // Instruction Fetch
            opcode = self.mem.onboard[self.pc as usize];
            self.pc += 1;

            // Instruction Decode and Execute
            self.execute(opcode);


            // println!("cycles: {}", self.curr_cycles);
            // println!("stack pointer: {:#04X}", self.sp);
            // println!("program counter location: {:#04X}", self.mem.onboard[self.pc as usize]);
            //break;
        }
    }

    pub fn match_prefix_instruction(self: &mut Self, i: Instruction){

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
            _ => panic!("Opcode not supported"),
        }
    }
    
    // Instructions that are 3 bytes long will call this method to get the next two bytes required
    fn two_bytes(self: &mut Self) -> (u8, u8) {
        let hi = self.mem.onboard[self.pc as usize];
        let lo = self.mem.onboard[(self.pc + 1) as usize];
        self.pc = self.pc + 2;
        return (hi, lo);
    }
    // Instructions that are 2 bytes long will call this method to get the next byte required
    fn one_byte(self: &mut Self) -> u8 {
        let byte = self.mem.onboard[self.pc as usize];
        self.pc = self.pc + 1;
        return byte;
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
        instruction::load_d16(&mut cpu.reg.bc, &mut cpu.curr_cycles, 0xA7, 0xFF);
        instruction::load_d16(&mut cpu.reg.de,  &mut cpu.curr_cycles, 0xF0, 0xFF);
        instruction::load_d16(&mut cpu.reg.hl,  &mut cpu.curr_cycles, 0x01, 0xFF);
        instruction::load_d16(&mut cpu.sp,  &mut cpu.curr_cycles, 0xFF, 0x00);
    
        assert_eq!(cpu.reg.bc, 0xA7FF);
        assert_eq!(cpu.reg.de, 0xF0FF);
        assert_eq!(cpu.reg.hl, 0x01FF);
        assert_eq!(cpu.sp, 0xFF00);
    }
}
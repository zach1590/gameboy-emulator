use super::instruction;
use super::instruction::Instruction;
use super::instruction::{FlagMod, Operation};
use super::memory::Memory;
use std::fs;
use std::time::Instant;

pub struct Cpu {
    mem: Memory,
    period_nanos: f64, // Time it takes for a clock cycle in nanoseconds
    pub reg: Registers,
    pub pc: u16,            // Program Counter
    pub sp: u16,            // Stack Pointer
    pub curr_cycles: usize, // The number of cycles the current instruction should take to execute
}

impl Cpu {
    pub fn new() -> Cpu {
        return Cpu {
            mem: Memory::new(),
            period_nanos: 238.418579,
            reg: Registers::new(),
            pc: 0,
            sp: 0,
            curr_cycles: 0,
        };
    }

    // In here lets read, initialize/load everything required from the cartridge
    pub fn load_cartridge(self: &mut Self, cartridge: &str) {
        let boot_rom_bytes = fs::read(cartridge).unwrap();
        self.mem.write_bytes(0, boot_rom_bytes);
        // for i in 0..257 {
        //     println!("{:#04X}", self.mem.read_byte(i as u16));
        // }
    }

    fn execute(self: &mut Self, opcode: u8) {
        let i = Instruction::get_instruction(opcode);

        if i.values == (0x0C, 0x0B) {
            let opcode = self.mem.read_byte(self.pc);
            self.pc += 1;
            let cb_i = Instruction::get_instruction(opcode);
            self.match_prefix_instruction(cb_i);
        } else {
            self.match_instruction(i);
        }
    }

    pub fn run(self: &mut Self) {
        let mut opcode: u8;
        let mut wait_time: u128;
        let mut previous_time: Instant = Instant::now();
        // Game loop
        loop {
            wait_time = ((self.curr_cycles as f64) * self.period_nanos) as u128;
            while previous_time.elapsed().as_nanos() <= wait_time {
                // Do Nothing
                // Maybe take user input in here
            }

            previous_time = Instant::now(); // Begin new clock timer
            opcode = self.mem.read_byte(self.pc); // Instruction Fetch
            self.pc += 1;
            self.execute(opcode); // Instruction Decode and Execute

            // println!("cycles: {}", self.curr_cycles);
            // println!("stack pointer: {:#04X}", self.sp);
            // println!("program counter location: {:#04X}", self.mem.read_byte(self.pc));
            //break;
        }
    }

    pub fn match_prefix_instruction(self: &mut Self, _i: Instruction) {}
    pub fn match_instruction(self: &mut Self, i: Instruction) {
        // Create a method for every instruction
        match i.values {
            (0x00, 0x01) => {
                // Load 16 bit immediate into BC
                let (hi, lo) = self.read_next_two_bytes();
                instruction::load_d16(&mut self.reg.bc, hi, lo);
                self.curr_cycles = 12;
            }
            (0x01, 0x01) => {
                // Load 16 bit immediate into DE
                let (hi, lo) = self.read_next_two_bytes();
                instruction::load_d16(&mut self.reg.de, hi, lo);
                self.curr_cycles = 12;
            }
            (0x02, 0x01) => {
                // Load 16 bit immediate into HL
                let (hi, lo) = self.read_next_two_bytes();
                instruction::load_d16(&mut self.reg.hl, hi, lo);
                self.curr_cycles = 12;
            }
            (0x03, 0x01) => {
                // Load 16 bit immediate into SP
                let (hi, lo) = self.read_next_two_bytes();
                instruction::load_d16(&mut self.sp, hi, lo);
                self.curr_cycles = 12;
            }
            (0x00 | 0x01 | 0x02 | 0x03, 0x02) => {
                // LD (BC)/(DE)/(HL+)/(HL-), A
                let (str_val_a, _) = Registers::get_hi_lo(self.reg.af);
                let location = match i.values.0 {
                    0x00 => self.reg.bc,
                    0x01 => self.reg.de,
                    0x02 => instruction::post_incr(&mut self.reg.hl),
                    0x03 => instruction::post_decr(&mut self.reg.hl),
                    _ => panic!(
                        "Valid opcodes here are 0x02, 0x12, 0x22, 0x32, current opcode is 
                                {:#04X}, {:#04X}",
                        i.values.0, i.values.1
                    ),
                };
                self.mem.write_byte(location, str_val_a);
                self.curr_cycles = 8;
            }
            (0x00 | 0x01 | 0x02 | 0x03, 0x0A) => {
                // LD A, (BC)/(DE)/(HL+)/(HL-)
                let location = match i.values.0 {
                    0x00 => self.reg.bc,
                    0x01 => self.reg.de,
                    0x02 => instruction::post_incr(&mut self.reg.hl),
                    0x03 => instruction::post_decr(&mut self.reg.hl),
                    _ => panic!(
                        "Valid opcodes here are 0x0A, 0x1A, 0x2A, 0x3A, current opcode is 
                                {:#04X}, {:#04X}",
                        i.values.0, i.values.1
                    ),
                };
                let new_a_val = self.mem.read_byte(location);
                self.reg.af = Registers::set_top_byte(self.reg.af, new_a_val);
                self.curr_cycles = 8;
            }
            (0x00 | 0x01 | 0x02, 0x06) => {
                // LD B/D/H, d8
                let ld_value = self.read_next_one_byte();
                match i.values.0 {
                    0x00 => instruction::load_imm_d8(&mut self.reg.bc, ld_value, true),
                    0x01 => instruction::load_imm_d8(&mut self.reg.de, ld_value, true),
                    0x02 => instruction::load_imm_d8(&mut self.reg.hl, ld_value, true),
                    _ => panic!(
                        "Valid opcodes here are 0x06, 0x16, 0x26, current opcode is 
                                {:#04X}, {:#04X}",
                        i.values.0, i.values.1
                    ),
                };
                self.curr_cycles = 8;
            }
            (0x03, 0x06) => {
                // LD (HL), d8
                let ld_value = self.read_next_one_byte();
                self.mem.write_byte(self.reg.hl, ld_value);
                self.curr_cycles = 12;
            }
            (0x00 | 0x01 | 0x02 | 0x03, 0x0E) => {
                // LD C/E/L, d8
                let ld_value = self.read_next_one_byte();
                match i.values.0 {
                    0x00 => instruction::load_imm_d8(&mut self.reg.bc, ld_value, false),
                    0x01 => instruction::load_imm_d8(&mut self.reg.de, ld_value, false),
                    0x02 => instruction::load_imm_d8(&mut self.reg.hl, ld_value, false),
                    0x03 => instruction::load_imm_d8(&mut self.reg.af, ld_value, true),
                    _ => panic!(
                        "Valid opcodes here are 0x0E, 0x1E, 0x2E, 0x3E current opcode is 
                                {:#04X}, {:#04X}",
                        i.values.0, i.values.1
                    ),
                };
                self.curr_cycles = 8;
            }
            (0x04, opcode_lo) => {
                // LD B/C, R
                // B for 0x40 - 0x47    C for 0x48 - 0x4F
                let ld_hi = opcode_lo <= 0x07;
                let ld_value = self.get_register_value_from_opcode(opcode_lo);
                instruction::load_8_bit_into_reg(&mut self.reg.bc, ld_hi, ld_value);
                self.curr_cycles = num_cycles_8bit_arithmetic_loads(opcode_lo);
            }
            (0x05, opcode_lo) => {
                // LD D/E, R
                // D for 0x50 - 0x57    E for 0x58 - 0x5F
                let ld_hi = opcode_lo <= 0x07;
                let ld_value = self.get_register_value_from_opcode(opcode_lo);
                instruction::load_8_bit_into_reg(&mut self.reg.de, ld_hi, ld_value);
                self.curr_cycles = num_cycles_8bit_arithmetic_loads(opcode_lo);
            }
            (0x06, opcode_lo) => {
                // LD H/L, R
                // H for 0x60 - 0x67    L for 0x68 - 0x6F
                let ld_hi = opcode_lo <= 0x07;
                let ld_value = self.get_register_value_from_opcode(opcode_lo);
                instruction::load_8_bit_into_reg(&mut self.reg.hl, ld_hi, ld_value);
                self.curr_cycles = num_cycles_8bit_arithmetic_loads(opcode_lo);
            }
            (0x07, 0x06) => {
                // HALT ** COME BACK TO THIS LATER IN CASE THERE IS MORE TO DO
                // Definitely have more to do here
                self.curr_cycles = 4;
            }
            (0x07, 0x00 | 0x01 | 0x02 | 0x03 | 0x04 | 0x05 | 0x07) => {
                // LD (HL), R
                let ld_value = self.get_register_value_from_opcode(i.values.1);
                self.mem.write_bytes(self.reg.hl, vec![ld_value]);
                self.curr_cycles = 8;
            }
            (0x07, 0x08 | 0x09 | 0x0A | 0x0B | 0x0C | 0x0D | 0x0E | 0x0F) => {
                // LD A, R
                let ld_value = self.get_register_value_from_opcode(i.values.1);
                instruction::load_8_bit_into_reg(&mut self.reg.af, true, ld_value);
                self.curr_cycles = num_cycles_8bit_arithmetic_loads(i.values.1);
            }
            (0x08, 0x00 | 0x01 | 0x02 | 0x03 | 0x04 | 0x05 | 0x06 | 0x07) => {
                // A = A ADD R
                let add_value = self.get_register_value_from_opcode(i.values.1);
                instruction::a_add_r(&mut self.reg.af, add_value);
                self.curr_cycles = num_cycles_8bit_arithmetic_loads(i.values.1);
            }
            (0x08, 0x08 | 0x09 | 0x0A | 0x0B | 0x0C | 0x0D | 0x0E | 0x0F) => {
                // A = A ADC R
                let adc_value = self.get_register_value_from_opcode(i.values.1);
                instruction::a_adc_r(&mut self.reg.af, adc_value);
                self.curr_cycles = num_cycles_8bit_arithmetic_loads(i.values.1);
            }
            (0x09, 0x00 | 0x01 | 0x02 | 0x03 | 0x04 | 0x05 | 0x06 | 0x07) => {
                // A = A SUB R
                let sub_value = self.get_register_value_from_opcode(i.values.1);
                instruction::a_sub_r(&mut self.reg.af, sub_value);
                self.curr_cycles = num_cycles_8bit_arithmetic_loads(i.values.1);
            }
            (0x09, 0x08 | 0x09 | 0x0A | 0x0B | 0x0C | 0x0D | 0x0E | 0x0F) => {
                // A = A SBC R
                let sbc_value = self.get_register_value_from_opcode(i.values.1);
                instruction::a_sbc_r(&mut self.reg.af, sbc_value);
                self.curr_cycles = num_cycles_8bit_arithmetic_loads(i.values.1);
            }
            (0x0A, 0x00 | 0x01 | 0x02 | 0x03 | 0x04 | 0x05 | 0x06 | 0x07) => {
                // A = A AND R
                let and_value = self.get_register_value_from_opcode(i.values.1);
                instruction::a_and_r(&mut self.reg.af, and_value);
                self.curr_cycles = num_cycles_8bit_arithmetic_loads(i.values.1);
            }
            (0x0A, 0x08 | 0x09 | 0x0A | 0x0B | 0x0C | 0x0D | 0x0E | 0x0F) => {
                // A = A XOR R
                let xor_value = self.get_register_value_from_opcode(i.values.1);
                instruction::a_xor_r(&mut self.reg.af, xor_value);
                self.curr_cycles = num_cycles_8bit_arithmetic_loads(i.values.1);
            }
            (0x0B, 0x00 | 0x01 | 0x02 | 0x03 | 0x04 | 0x05 | 0x06 | 0x07) => {
                // A = A OR R
                let or_value = self.get_register_value_from_opcode(i.values.1);
                instruction::a_or_r(&mut self.reg.af, or_value);
                self.curr_cycles = num_cycles_8bit_arithmetic_loads(i.values.1);
            }
            (0x0B, 0x08 | 0x09 | 0x0A | 0x0B | 0x0C | 0x0D | 0x0E | 0x0F) => {
                // A CP R (just update flags, dont store result)
                let cp_value = self.get_register_value_from_opcode(i.values.1);
                instruction::a_cp_r(&mut self.reg.af, cp_value);
                self.curr_cycles = num_cycles_8bit_arithmetic_loads(i.values.1);
            }
            (0x0E | 0x0F, 0x00 | 0x02) => {
                // Read and Write to IO Ports (Need to make some kind of notify thing?)
                // Definitely have more to do here
                let (offset, cycles) = match i.values.1 {
                    0x00 => (self.read_next_one_byte(), 12),
                    0x02 => (Registers::get_lo(self.reg.bc), 8),
                    _ => panic!(
                        "Valid opcodes: 0xE0,E2,F0,F2, Current: {:#04X}, {:#04X}",
                        i.values.0, i.values.1
                    ),
                };
                let location = offset as u16 + 0xFF00;
                if i.values.0 == 0x0E {
                    // Do some kind of notify
                    self.mem
                        .write_byte(location, Registers::get_hi(self.reg.af));
                } else {
                    self.reg.af =
                        Registers::set_top_byte(self.reg.af, self.mem.read_byte(location));
                }
                self.curr_cycles = cycles;
            }
            (0x0E | 0x0F, 0x0A) => {
                //ld (nn), A     and     ld A, (nn)
                let (hi, lo) = self.read_next_two_bytes();
                let location = instruction::combine_bytes(hi, lo);
                if i.values.0 == 0x0E {
                    self.mem
                        .write_byte(location, Registers::get_hi(self.reg.af));
                }
                if i.values.0 == 0x0F {
                    self.reg.af =
                        Registers::set_top_byte(self.reg.af, self.mem.read_byte(location));
                }
                self.curr_cycles = 16;
            }
            (0x0E, 0x08) => {
                let byte = self.read_next_one_byte();
                let (result, new_af) = instruction::sp_add_dd(self.sp, byte, self.reg.af);
                self.curr_cycles = 16;
                self.reg.af = new_af;
                self.sp = result;
            }
            (0x0F, 0x08) => {
                let byte = self.read_next_one_byte();
                let (result, new_af) = instruction::sp_add_dd(self.sp, byte, self.reg.af);
                self.curr_cycles = 12;
                self.reg.af = new_af;
                self.reg.hl = result;
            }
            (0x0F, 0x09) => {
                self.sp = self.reg.hl;
                self.curr_cycles = 8;
            }
            _ => panic!("Opcode not supported"),
        } // End of match statement
    } // match instruction function

    // Instructions that are 3 bytes long will call this method to get the next two bytes required
    fn read_next_two_bytes(self: &mut Self) -> (u8, u8) {
        let hi = self.mem.read_byte(self.pc);
        let lo = self.mem.read_byte(self.pc + 1);
        self.pc = self.pc + 2;
        return (hi, lo);
    }

    // Instructions that are 2 bytes long will call this method to get the next byte required
    fn read_next_one_byte(self: &mut Self) -> u8 {
        let byte = self.mem.read_byte(self.pc);
        self.pc = self.pc + 1;
        return byte;
    }

    // Takes the lower 8 bits of the opcode and returns the value of the register needed for the instruction
    // Should work for obtaining the second operand of virtually all opcodes
    fn get_register_value_from_opcode(self: &Self, opcode_lo: u8) -> u8 {
        return match opcode_lo {
            0x00 | 0x08 => Registers::get_hi_lo(self.reg.bc).0,
            0x01 | 0x09 => Registers::get_hi_lo(self.reg.bc).1,
            0x02 | 0x0A => Registers::get_hi_lo(self.reg.de).0,
            0x03 | 0x0B => Registers::get_hi_lo(self.reg.de).1,
            0x04 | 0x0C => Registers::get_hi_lo(self.reg.hl).0,
            0x05 | 0x0D => Registers::get_hi_lo(self.reg.hl).1,
            0x06 | 0x0E => self.mem.read_byte(self.reg.hl),
            0x07 | 0x0F => Registers::get_hi_lo(self.reg.af).0,
            _ => panic!("Expected Value between 0x00 and 0x0F"),
        };
    }
} // Impl CPU

// Returns the number of cycles required by the instruction
// Intended for instructions where the opcode was between 0x40 and 0xBF
// except for 0x70 -> 0x77
fn num_cycles_8bit_arithmetic_loads(opcode_lo: u8) -> usize {
    if (opcode_lo == 0x06) | (opcode_lo == 0x0E) {
        return 8;
    } else {
        return 4;
    }
}

// Each one may also be addressed as just the upper or lower 8 bits
pub struct Registers {
    pub af: u16, // A: accumulator, F: flags as 0bZNHC0000
    pub bc: u16,
    pub de: u16,
    pub hl: u16,
}

impl Registers {
    fn new() -> Registers {
        return Registers {
            af: 0,
            bc: 0,
            de: 0,
            hl: 0,
        };
    }
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

    pub fn set_top_byte(reg: u16, byte: u8) -> u16 {
        let mut new_reg = reg & 0x00FF;
        new_reg = new_reg | ((byte as u16) << 8);
        return new_reg;
    }
    pub fn set_bottom_byte(reg: u16, byte: u8) -> u16 {
        let mut new_reg = reg & 0xFF00;
        new_reg = new_reg | (byte as u16);
        return new_reg;
    }
}

#[cfg(test)]
#[path = "./tests/cpu_tests.rs"]
mod cpu_tests;

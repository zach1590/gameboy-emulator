use crate::cpu::{Registers};

pub struct Instruction {
    pub values: (u8, u8),
}
impl Instruction {
    pub fn get_instruction(x: u8) -> Instruction {
        return Instruction {
            values: (
            ((x & 0x00F0) >> 4) as u8,
            (x & 0x000F) as u8
        )};
    }
}

// Combines two u8s into a u16 value (hi, lo -> result)
pub fn combine_bytes(hi: u8, lo: u8) -> u16 {
    let mut res = hi as u16;
    res = (res << 8) + (lo as u16);
    return res;
}

// Returns the number of cycles required by the instruction
// Intended for instructions where the opcode was between 0x80 and 0xBF
// It may apply for other instructions but thats unintentional currently
fn num_cycles_reg_hl_0x80_0xbf(opcode_lo: u8) -> usize {
    if (opcode_lo == 0x06) | (opcode_lo == 0x0E) {
        return 8;
    } else {
        return 4;
    }
}

// Load 16 bit immediate into register
pub fn load_d16(register: &mut u16, cycles: &mut usize, hi: u8, lo: u8) {
    let imm_val = combine_bytes(hi, lo);
    *register = imm_val;
    *cycles = 12;
    // Do nothing to flags??
}

pub fn a_xor_r(reg_af: &mut u16, xor_value: u8, cycles: &mut usize, opcode_lo: u8) {
    let (mut reg_a, mut reg_f) = Registers::get_hi_lo(*reg_af);
    reg_a = reg_a ^ xor_value;

    // Unset N, H, C flags in all cases, set Z only if result = 0
    // I dont know if the lower 4 bits of F ever has anything but in case it does, try to preserve the value
    if reg_a == 0x00 {
        reg_f = reg_f | 0b10000000; // Make sure Z flag is set to 1, preserve other bits for now
        reg_f = reg_f & 0b10001111; // Set NHC to 000 and preserve all other bits
    } else {
        reg_f = reg_f & 0b00001111;     
    }

    *reg_af = combine_bytes(reg_a, reg_f);
    *cycles = num_cycles_reg_hl_0x80_0xbf(opcode_lo);
}

#[test]
fn test_instruction_creation(){
    let x1 = 0x12;
    let x2 = 0xCB;
    let x3 = 0x03;
    let x4 = 0x20;

    let i1 = Instruction::get_instruction(x1);
    let i2 = Instruction::get_instruction(x2);
    let i3 = Instruction::get_instruction(x3);
    let i4 = Instruction::get_instruction(x4);

    assert_eq!(i1.values, (0x01, 0x02));
    assert_eq!(i2.values, (0x0C, 0x0B));
    assert_eq!(i3.values, (0x00, 0x03));
    assert_eq!(i4.values, (0x02, 0x00));
}

#[test]
fn test_combine_bytes(){
    let x1 = 0x12;
    let x2 = 0xAB;
    let x3 = 0x12AB;
    assert_eq!(x3, combine_bytes(x1, x2));
}
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

pub enum FlagMod {
    Set,
    Unset,
    Nop,
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
    let (reg_a, mut reg_f) = Registers::get_hi_lo(*reg_af);
    let result = reg_a ^ xor_value;
    reg_f = set_flags(
        set_z_flag(result), FlagMod::Unset, FlagMod::Unset, FlagMod::Unset, reg_f
    );
    *reg_af = combine_bytes(result, reg_f);
    *cycles = num_cycles_reg_hl_0x80_0xbf(opcode_lo);
}

pub fn set_flags(z: FlagMod, n: FlagMod, h: FlagMod, c: FlagMod, reg_f: u8) -> u8 {
    // I dont know if the lower 4 bits of F ever has anything but in case it does, try to preserve the value
    // Make sure only the specific flag is set to 0 or 1, and preserve other bits in each operation
    let mut flags = reg_f;
    match z {
        FlagMod::Set =>   { flags = flags | 0b10000000; },      // Only set the z flag
        FlagMod::Unset => { flags = flags & 0b01111111; },      // Only unset the z flag
        FlagMod::Nop => {},
    }
    match n {
        FlagMod::Set =>   { flags = flags | 0b01000000; },      // Only set the n flag
        FlagMod::Unset => { flags = flags & 0b10111111; },      // Only unset the n flag
        FlagMod::Nop => {},
    }
    match h {
        FlagMod::Set =>   { flags = flags | 0b00100000; },      // Only set the h flag
        FlagMod::Unset => { flags = flags & 0b11011111; },      // Only unset the h flag
        FlagMod::Nop => {},
    }
    match c {
        FlagMod::Set =>   { flags = flags | 0b00010000; },      // Only set the c flag
        FlagMod::Unset => { flags = flags & 0b11101111; },      // Only unset the c flag
        FlagMod::Nop => {},
    }
    return flags;
}

// Determines if z flag needs to be set.
fn set_z_flag(result: u8) -> FlagMod {
    if result == 0x00{
        return FlagMod::Set;
    } else {
        return FlagMod::Unset;
    }
}
// Determines if n flag needs to be set.
// Negative flag is never determined from the calculation result and is
// determined by the opcode itself rather then opcode operation

// Determines if h flag needs to be set.
// Not implemented yet
fn set_h_flag(result: u8, operand: u8) -> FlagMod {
    if result == 0x00{
        return FlagMod::Set;
    } else {
        return FlagMod::Unset;
    }
}
// Determines if c flag needs to be set.
// Not implemented yet
fn set_c_flag(result: u8, operand: u8) -> FlagMod {
    if result == 0x00{
        return FlagMod::Set;
    } else {
        return FlagMod::Unset;
    }
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

#[test]
fn test_set_flags(){
    let reg_1 = 0b1010_1010;
    let flags1 = set_flags(
        FlagMod::Nop, FlagMod::Unset, FlagMod::Unset, FlagMod::Unset, reg_1
    );

    let reg_2 = 0b0011_1110;
    let flags2 = set_flags(
        FlagMod::Set, FlagMod::Unset, FlagMod::Set, FlagMod::Nop, reg_2
    );

    let reg_3 = 0b1010_1010;
    let flags3 = set_flags(
        FlagMod::Nop, FlagMod::Nop, FlagMod::Nop, FlagMod::Nop, reg_3
    );

    let reg_4 = 0b1010_0000;
    let flags4 = set_flags(
        FlagMod::Unset, FlagMod::Set, FlagMod::Unset, FlagMod::Set, reg_4
    );

    assert_eq!(flags1, 0b10001010);
    assert_eq!(flags2, 0b10111110);
    assert_eq!(flags3, 0b10101010);
    assert_eq!(flags4, 0b01010000);
}
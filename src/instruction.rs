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

#[derive(Debug, PartialEq)]
pub enum FlagMod {
    Set,
    Unset,
    Nop,
}

enum Operation {
    Add,
    Sub,
    AddCarry (u8),
    SubCarry (u8),
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

pub fn a_and_r(reg_af: &mut u16, and_value: u8, cycles: &mut usize, opcode_lo: u8) {
    let (reg_a, mut reg_f) = Registers::get_hi_lo(*reg_af);
    let result = reg_a & and_value;

    reg_f = set_flags(
        set_z_flag(result), FlagMod::Unset, FlagMod::Set, FlagMod::Unset, reg_f
    );

    *reg_af = combine_bytes(result, reg_f);
    *cycles = num_cycles_reg_hl_0x80_0xbf(opcode_lo);
}

pub fn a_or_r(reg_af: &mut u16, or_value: u8, cycles: &mut usize, opcode_lo: u8) {
    let (reg_a, mut reg_f) = Registers::get_hi_lo(*reg_af);
    let result = reg_a | or_value;

    reg_f = set_flags(
        set_z_flag(result), FlagMod::Unset, FlagMod::Unset, FlagMod::Unset, reg_f
    );

    *reg_af = combine_bytes(result, reg_f);
    *cycles = num_cycles_reg_hl_0x80_0xbf(opcode_lo);
}

pub fn a_add_r(reg_af: &mut u16, add_value: u8, cycles: &mut usize, opcode_lo: u8) {
    let (reg_a, mut reg_f) = Registers::get_hi_lo(*reg_af);
    let (wrap_result, carry) = reg_a.overflowing_add(add_value);

    reg_f = set_flags(
        set_z_flag(wrap_result),
        FlagMod::Unset,
        set_h_flag(reg_a, add_value, Operation::Add),
        set_c_flag(carry),
        reg_f
    );

    *reg_af = combine_bytes(wrap_result, reg_f);
    *cycles = num_cycles_reg_hl_0x80_0xbf(opcode_lo);
}

pub fn a_sub_r(reg_af: &mut u16, sub_value: u8, cycles: &mut usize, opcode_lo: u8) {
    let (reg_a, mut reg_f) = Registers::get_hi_lo(*reg_af);
    let (wrap_result, carry) = reg_a.overflowing_sub(sub_value);

    reg_f = set_flags(
        set_z_flag(wrap_result),
        FlagMod::Set,
        set_h_flag(reg_a, sub_value, Operation::Sub),
        set_c_flag(carry),
        reg_f
    );

    *reg_af = combine_bytes(wrap_result, reg_f);
    *cycles = num_cycles_reg_hl_0x80_0xbf(opcode_lo);
}

pub fn a_adc_r(reg_af: &mut u16, adc_value: u8, cycles: &mut usize, opcode_lo: u8) {
    let (reg_a, mut reg_f) = Registers::get_hi_lo(*reg_af);
    let c = (reg_f & 0b0001_0000) >> 4;
    // carrying_add is nightly only so do this for now
    let (wrap_result, carry1) = reg_a.overflowing_add(adc_value);
    let (wrap_result, carry2) = wrap_result.overflowing_add(c);

    reg_f = set_flags(
        set_z_flag(wrap_result),
        FlagMod::Unset,
        set_h_flag(reg_a, adc_value, Operation::AddCarry (c)),
        set_c_flag(carry1 | carry2),    // The carry may have occured on either addition
        reg_f
    );

    *reg_af = combine_bytes(wrap_result, reg_f);
    *cycles = num_cycles_reg_hl_0x80_0xbf(opcode_lo);
}

pub fn a_sbc_r(reg_af: &mut u16, sbc_value: u8, cycles: &mut usize, opcode_lo: u8) {
    let (reg_a, mut reg_f) = Registers::get_hi_lo(*reg_af);
    let c = (reg_f & 0b0001_0000) >> 4;
    // carrying_sub is nightly only so do this for now
    let (wrap_result, carry1) = reg_a.overflowing_sub(sbc_value);
    let (wrap_result, carry2) = wrap_result.overflowing_sub(c);

    reg_f = set_flags(
        set_z_flag(wrap_result),
        FlagMod::Set,
        set_h_flag(reg_a, sbc_value, Operation::SubCarry (c)),
        set_c_flag(carry1 | carry2),    // The carry may have occured on either subtraction
        reg_f
    );

    *reg_af = combine_bytes(wrap_result, reg_f);
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

/*
    Determines if h flag needs to be set.
    Occurs when there is a carry from bit 3 to bit 4
    i.e. Result of the lower 4 bits added together is >15
        1. Clear out the first four bits of each argument
        2. Add/Sub the lower four bits of each argument together
        3. Clear out the lower four bits of result to extract bit #4
        4. Shift the result right 4 times
        5. If it equals 1 then we must have had a carry
    Can also replace 4 and 5 with == 0x10
*/
fn set_h_flag(arg1: u8, arg2: u8, op: Operation) -> FlagMod {
    let lo1 = arg1 & 0x0F;
    let lo2 = arg2 & 0x0F;

    match op {
        Operation::Add => {
            if ((lo1 + lo2) & (0x10)) == 0x10 {
                return FlagMod::Set;
            } else {
                return FlagMod::Unset;
            }
        },
        Operation::Sub => {
            if (lo1.wrapping_sub(lo2) & (0x10)) == 0x10 {       // Sub can overflow is reg_a < r
                return FlagMod::Set;
            } else {
                return FlagMod::Unset;
            }
        },
        Operation::AddCarry (c) => {
            if ((lo1 + lo2 + (c & 0x0F)) & (0x10)) == 0x10 {
                return FlagMod::Set;
            } else {
                return FlagMod::Unset;
            }
        },
        Operation::SubCarry (c) => {
            if (lo1.wrapping_sub(lo2).wrapping_sub(c & 0x0F) & (0x10)) == 0x10 {
                return FlagMod::Set;
            } else {
                return FlagMod::Unset;
            }
        },
        _ => panic!("Not Implemented"),
    }
    
}
// Determines if c flag needs to be set.
fn set_c_flag(is_carry: bool) -> FlagMod {
    if is_carry == true {
        return FlagMod::Set;
    } else {
        return FlagMod::Unset;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_half_carry_add(){
        // Any numbers where the bottom four bits added together is over
        // 15 should result in set, and everything else should result in unset
        let reg_1 = 0b1010_1010;
        let reg_2 = 0b0011_1110;
        let h_flag_1 = set_h_flag(reg_1, reg_2, Operation::Add);

        let reg_1 = 0b1010_0000;
        let reg_2 = 0b0011_1111;
        let h_flag_2 = set_h_flag(reg_1, reg_2, Operation::Add);

        let reg_1 = 0b1111_0001;
        let reg_2 = 0b0111_1110;
        let h_flag_3 = set_h_flag(reg_1, reg_2, Operation::Add);

        let reg_1 = 0b1010_1111;
        let reg_2 = 0b0011_0001;
        let h_flag_4 = set_h_flag(reg_1, reg_2, Operation::Add);

        assert_eq!(h_flag_1, FlagMod::Set);
        assert_eq!(h_flag_2, FlagMod::Unset);
        assert_eq!(h_flag_3, FlagMod::Unset);
        assert_eq!(h_flag_4, FlagMod::Set);
    }

    #[test]
    fn test_set_carry_flag() {
        let flag1 = set_c_flag(true);
        let flag2 = set_c_flag(false);

        assert_eq!(flag1, FlagMod::Set);
        assert_eq!(flag2, FlagMod::Unset);
    }

    #[test]
    fn test_half_carry_sub() {
        let flag1 = set_h_flag(0xA9, 0x5C, Operation::Sub);
        let flag2 = set_h_flag(0x5C, 0xA9, Operation::Sub);

        assert_eq!(flag1, FlagMod::Set);
        assert_eq!(flag2, FlagMod::Unset);
    }
}
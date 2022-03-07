use crate::cpu::Registers; //use super::cpu::Registers; (Equivalent?)

pub struct Instruction {
    pub values: (u8, u8),
}
impl Instruction {
    pub fn get_instruction(x: u8) -> Instruction {
        return Instruction {
            values: (((x & 0x00F0) >> 4) as u8, (x & 0x000F) as u8),
        };
    }
}

#[derive(Debug, PartialEq)]
pub enum FlagMod {
    Set,
    Unset,
    Nop,
}

enum Operation {
    Add(u8),
    Sub(u8),
}

// Combines two u8s into a u16 value (hi, lo -> result)
pub fn combine_bytes(hi: u8, lo: u8) -> u16 {
    let mut res = hi as u16;
    res = (res << 8) + (lo as u16);
    return res;
}

// Load 16 bit immediate into register
pub fn load_d16(register: &mut u16, hi: u8, lo: u8) {
    let imm_val = combine_bytes(hi, lo);
    *register = imm_val;
    // Do nothing to flags
}

// Load 8 bit immediate into register
pub fn load_imm_d8(register: &mut u16, ld_val: u8, is_hi: bool) {
    let (hi, lo) = Registers::get_hi_lo(*register);
    let new_reg_val = if is_hi {
        combine_bytes(ld_val, lo)
    } else {
        combine_bytes(hi, ld_val)
    };
    *register = new_reg_val;
    // Do nothing to flags
}

// Used for 0x40 -> 0x6F and for 0x78 -> 0x7F
pub fn load_8_bit_into_reg(register: &mut u16, ld_hi: bool, ld_value: u8) {
    let (reg_hi, reg_lo) = Registers::get_hi_lo(*register);
    let load_result = if ld_hi {
        combine_bytes(ld_value, reg_lo)
    } else {
        combine_bytes(reg_hi, ld_value)
    };

    *register = load_result;
}

pub fn a_xor_r(reg_af: &mut u16, xor_value: u8) {
    let (reg_a, mut reg_f) = Registers::get_hi_lo(*reg_af);
    let result = reg_a ^ xor_value;

    reg_f = set_flags(
        set_z_flag(result),
        FlagMod::Unset,
        FlagMod::Unset,
        FlagMod::Unset,
        reg_f,
    );

    *reg_af = combine_bytes(result, reg_f);
}

pub fn a_and_r(reg_af: &mut u16, and_value: u8) {
    let (reg_a, mut reg_f) = Registers::get_hi_lo(*reg_af);
    let result = reg_a & and_value;

    reg_f = set_flags(
        set_z_flag(result),
        FlagMod::Unset,
        FlagMod::Set,
        FlagMod::Unset,
        reg_f,
    );

    *reg_af = combine_bytes(result, reg_f);
}

pub fn a_or_r(reg_af: &mut u16, or_value: u8) {
    let (reg_a, mut reg_f) = Registers::get_hi_lo(*reg_af);
    let result = reg_a | or_value;

    reg_f = set_flags(
        set_z_flag(result),
        FlagMod::Unset,
        FlagMod::Unset,
        FlagMod::Unset,
        reg_f,
    );

    *reg_af = combine_bytes(result, reg_f);
}

pub fn a_add_r(reg_af: &mut u16, add_value: u8) {
    let (reg_a, mut reg_f) = Registers::get_hi_lo(*reg_af);
    let (wrap_result, carry) = reg_a.overflowing_add(add_value);

    reg_f = set_flags(
        set_z_flag(wrap_result),
        FlagMod::Unset,
        set_h_flag(reg_a, add_value, Operation::Add(0)),
        set_c_flag(carry),
        reg_f,
    );

    *reg_af = combine_bytes(wrap_result, reg_f);
}

pub fn a_sub_r(reg_af: &mut u16, sub_value: u8) {
    let (reg_a, mut reg_f) = Registers::get_hi_lo(*reg_af);
    let (wrap_result, carry) = reg_a.overflowing_sub(sub_value);

    reg_f = set_flags(
        set_z_flag(wrap_result),
        FlagMod::Set,
        set_h_flag(reg_a, sub_value, Operation::Sub(0)),
        set_c_flag(carry),
        reg_f,
    );

    *reg_af = combine_bytes(wrap_result, reg_f);
}

pub fn a_adc_r(reg_af: &mut u16, adc_value: u8) {
    let (reg_a, mut reg_f) = Registers::get_hi_lo(*reg_af);
    let c_flag = (reg_f & 0b0001_0000) >> 4;
    // carrying_add is nightly only so do this for now
    let (wrap_result, carry1) = reg_a.overflowing_add(adc_value);
    let (wrap_result, carry2) = wrap_result.overflowing_add(c_flag);

    reg_f = set_flags(
        set_z_flag(wrap_result),
        FlagMod::Unset,
        set_h_flag(reg_a, adc_value, Operation::Add(c_flag)),
        set_c_flag(carry1 | carry2), // The carry may have occured on either addition
        reg_f,
    );

    *reg_af = combine_bytes(wrap_result, reg_f);
}

pub fn a_sbc_r(reg_af: &mut u16, sbc_value: u8) {
    let (reg_a, mut reg_f) = Registers::get_hi_lo(*reg_af);
    let c_flag = (reg_f & 0b0001_0000) >> 4;
    // carrying_sub is nightly only so do this for now
    let (wrap_result, carry1) = reg_a.overflowing_sub(sbc_value);
    let (wrap_result, carry2) = wrap_result.overflowing_sub(c_flag);

    reg_f = set_flags(
        set_z_flag(wrap_result),
        FlagMod::Set,
        set_h_flag(reg_a, sbc_value, Operation::Sub(c_flag)),
        set_c_flag(carry1 | carry2), // The carry may have occured on either subtraction
        reg_f,
    );

    *reg_af = combine_bytes(wrap_result, reg_f);
}

pub fn a_cp_r(reg_af: &mut u16, cp_value: u8) {
    let (reg_a, mut reg_f) = Registers::get_hi_lo(*reg_af);
    let (wrap_result, carry) = reg_a.overflowing_sub(cp_value);

    reg_f = set_flags(
        set_z_flag(wrap_result),
        FlagMod::Set,
        set_h_flag(reg_a, cp_value, Operation::Sub(0)),
        set_c_flag(carry),
        reg_f,
    );

    *reg_af = combine_bytes(reg_a, reg_f);
}

pub fn set_flags(z: FlagMod, n: FlagMod, h: FlagMod, c: FlagMod, reg_f: u8) -> u8 {
    // I dont know if the lower 4 bits of F ever has anything but in case it does, try to preserve the value
    // Make sure only the specific flag is set to 0 or 1, and preserve other bits in each operation
    let mut flags = reg_f;
    match z {
        FlagMod::Set => {
            flags = flags | 0b10000000;
        } // Only set the z flag
        FlagMod::Unset => {
            flags = flags & 0b01111111;
        } // Only unset the z flag
        FlagMod::Nop => {}
    }
    match n {
        FlagMod::Set => {
            flags = flags | 0b01000000;
        } // Only set the n flag
        FlagMod::Unset => {
            flags = flags & 0b10111111;
        } // Only unset the n flag
        FlagMod::Nop => {}
    }
    match h {
        FlagMod::Set => {
            flags = flags | 0b00100000;
        } // Only set the h flag
        FlagMod::Unset => {
            flags = flags & 0b11011111;
        } // Only unset the h flag
        FlagMod::Nop => {}
    }
    match c {
        FlagMod::Set => {
            flags = flags | 0b00010000;
        } // Only set the c flag
        FlagMod::Unset => {
            flags = flags & 0b11101111;
        } // Only unset the c flag
        FlagMod::Nop => {}
    }
    return flags;
}

// Determines if z flag needs to be set.
fn set_z_flag(result: u8) -> FlagMod {
    if result == 0x00 {
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
        Operation::Add(c) => {
            if ((lo1 + lo2 + (c & 0x0F)) & (0x10)) == 0x10 {
                return FlagMod::Set;
            } else {
                return FlagMod::Unset;
            }
        }
        Operation::Sub(c) => {
            if (lo1.wrapping_sub(lo2).wrapping_sub(c & 0x0F) & (0x10)) == 0x10 {
                return FlagMod::Set;
            } else {
                return FlagMod::Unset;
            }
        }
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

// Wrapping should be the correct behaviour. Seems unlikely
// for the actual hardware to have done anything else but let
// the overflow occur
pub fn post_incr(val: &mut u16) -> u16 {
    *val = val.wrapping_add(1); // Increment the value
    return val.wrapping_sub(1); // Return copy of original
}

pub fn post_decr(val: &mut u16) -> u16 {
    *val = val.wrapping_sub(1); // Decrement the value
    return val.wrapping_add(1); // Return copy of original
}

#[cfg(test)]
#[path = "./tests/instruction_tests.rs"]
mod instruction_tests;

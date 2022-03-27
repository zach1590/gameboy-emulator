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

impl ToString for Instruction {
    fn to_string(&self) -> String {
        let mut opcode = self.values.0.to_string();
        opcode.push_str(", ");
        opcode.push_str(&self.values.1.to_string());
        return opcode;
    }
}

#[derive(Debug, PartialEq)]
pub enum Flag {
    Set,
    Unset,
    Nop,
}

pub enum Operation {
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
        Flag::Unset,
        Flag::Unset,
        Flag::Unset,
        reg_f,
    );

    *reg_af = combine_bytes(result, reg_f);
}

pub fn a_and_r(reg_af: &mut u16, and_value: u8) {
    let (reg_a, mut reg_f) = Registers::get_hi_lo(*reg_af);
    let result = reg_a & and_value;

    reg_f = set_flags(
        set_z_flag(result),
        Flag::Unset,
        Flag::Set,
        Flag::Unset,
        reg_f,
    );

    *reg_af = combine_bytes(result, reg_f);
}

pub fn a_or_r(reg_af: &mut u16, or_value: u8) {
    let (reg_a, mut reg_f) = Registers::get_hi_lo(*reg_af);
    let result = reg_a | or_value;

    reg_f = set_flags(
        set_z_flag(result),
        Flag::Unset,
        Flag::Unset,
        Flag::Unset,
        reg_f,
    );

    *reg_af = combine_bytes(result, reg_f);
}

pub fn a_add_r(reg_af: &mut u16, add_value: u8) {
    let (reg_a, mut reg_f) = Registers::get_hi_lo(*reg_af);
    let (wrap_result, carry) = reg_a.overflowing_add(add_value);

    reg_f = set_flags(
        set_z_flag(wrap_result),
        Flag::Unset,
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
        Flag::Set,
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
        Flag::Unset,
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
        Flag::Set,
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
        Flag::Set,
        set_h_flag(reg_a, cp_value, Operation::Sub(0)),
        set_c_flag(carry),
        reg_f,
    );

    *reg_af = combine_bytes(reg_a, reg_f);
}

pub fn sp_add_dd(sp: u16, imm8: u8, reg_af: u16) -> (u16, u16) {
    let (result, carry) = reg_add_8bit_signed(sp, imm8);
    // This instruction uses the 8 bit definition not 16
    let reg_f = set_flags(
        Flag::Unset,
        Flag::Unset,
        set_h_flag(sp as u8, imm8 as u8, Operation::Add(0)),
        set_c_flag(carry),
        Registers::get_lo(reg_af),
    );
    let new_af = Registers::set_bottom_byte(reg_af, reg_f);
    return (result, new_af);
}

pub fn reg_add_8bit_signed(reg: u16, imm8: u8) -> (u16, bool) {
    let (lo_bytes, carry) = (reg as u8).overflowing_add(imm8);
    let hi_bytes = if imm8.leading_ones() > 0 {
        let temp = ((reg >> 8) as u8).wrapping_add(0xFF); // negative
        temp.wrapping_add(carry as u8)
    } else {
        ((reg >> 8) as u8).wrapping_add(carry as u8) // positive
    };
    let result = combine_bytes(hi_bytes, lo_bytes);
    return (result, carry);
}

pub fn hl_add_rr(hl: &mut u16, add_value: u16, reg_af: &mut u16) {
    let (result, carry) = hl.overflowing_add(add_value);
    let reg_f = set_flags(
        Flag::Nop,
        Flag::Unset,
        set_h_flag_16bit(*hl, add_value),
        set_c_flag(carry),
        Registers::get_lo(*reg_af),
    );
    *reg_af = Registers::set_bottom_byte(*reg_af, reg_f);
    *hl = result;
}

pub fn incr_8bit(incr_value: u8, reg_af: &mut u16) -> u8 {
    let result = incr_value.wrapping_add(1);
    let reg_f = set_flags(
        set_z_flag(result),
        Flag::Unset,
        set_h_flag(incr_value, 1, Operation::Add(0)),
        Flag::Nop,
        Registers::get_lo(*reg_af),
    );
    *reg_af = Registers::set_bottom_byte(*reg_af, reg_f);
    return result;
}

pub fn decr_8bit(decr_value: u8, reg_af: &mut u16) -> u8 {
    let result = decr_value.wrapping_sub(1);
    let reg_f = set_flags(
        set_z_flag(result),
        Flag::Set,
        set_h_flag(decr_value, 1, Operation::Sub(0)),
        Flag::Nop,
        Registers::get_lo(*reg_af),
    );
    *reg_af = Registers::set_bottom_byte(*reg_af, reg_f);
    return result;
}

// RLA is through_carry=true, RLCA if through_carry=false
pub fn rotate_left_a(through_carry: bool, reg: &mut Registers) {
    let (mut reg_a, mut reg_f) = Registers::get_hi_lo(reg.af);
    let old_c = reg.get_c();
    let new_c = (reg_a >> 7) == 1;
    reg_a = reg_a << 1;

    if through_carry {
        reg_a = reg_a | old_c as u8;
    } else {
        reg_a = reg_a | new_c as u8;
    }

    reg_f = set_flags(
        Flag::Unset,
        Flag::Unset,
        Flag::Unset,
        set_c_flag(new_c),
        reg_f,
    );
    (*reg).af = combine_bytes(reg_a, reg_f);
}

// RRA is through_carry=true, RRCA if through_carry=false
pub fn rotate_right_a(through_carry: bool, reg: &mut Registers) {
    let (mut reg_a, mut reg_f) = Registers::get_hi_lo(reg.af);
    let old_c = reg.get_c();
    let new_c = (reg_a & 0x01) == 0x01;
    reg_a = reg_a >> 1;

    if through_carry {
        reg_a = reg_a | ((old_c as u8) << 7);
    } else {
        reg_a = reg_a | ((new_c as u8) << 7);
    }

    reg_f = set_flags(
        Flag::Unset,
        Flag::Unset,
        Flag::Unset,
        set_c_flag(new_c),
        reg_f,
    );
    (*reg).af = combine_bytes(reg_a, reg_f);
}

pub fn daa(reg: &Registers) -> u16 {
    let (mut reg_a, mut reg_f) = Registers::get_hi_lo(reg.af);
    let c_flag = reg.get_c();
    let h_flag = reg.get_h();

    //https://ehaskins.com/2018-01-30%20Z80%20DAA/
    let mut carry: bool = false;
    if !reg.get_n() {
        if c_flag || reg_a > 0x99 {
            reg_a = reg_a.wrapping_add(0x60);
            carry = c_flag;
        }
        if h_flag || (reg_a & 0x0f) > 0x09 {
            reg_a = reg_a.wrapping_add(0x06);
        }
    } else {
        if c_flag {
            reg_a = reg_a.wrapping_sub(0x60);
            carry = c_flag;
        }
        if h_flag {
            reg_a = reg_a.wrapping_sub(0x06);
        }
    }
    reg_f = set_flags(
        set_z_flag(reg_a),
        Flag::Nop,
        Flag::Unset,
        set_c_flag(carry),
        reg_f,
    );
    return combine_bytes(reg_a, reg_f);
}

pub fn set_flags(z: Flag, n: Flag, h: Flag, c: Flag, reg_f: u8) -> u8 {
    // Make sure only the specific flag is set to 0 or 1, and preserve other bits in each operation
    let mut flags = reg_f;
    match z {
        Flag::Set => flags = flags | 0b10000000,
        Flag::Unset => flags = flags & 0b01111111,
        Flag::Nop => {}
    }
    match n {
        Flag::Set => flags = flags | 0b01000000,
        Flag::Unset => flags = flags & 0b10111111,
        Flag::Nop => {}
    }
    match h {
        Flag::Set => flags = flags | 0b00100000,
        Flag::Unset => flags = flags & 0b11011111,
        Flag::Nop => {}
    }
    match c {
        Flag::Set => flags = flags | 0b00010000,
        Flag::Unset => flags = flags & 0b11101111,
        Flag::Nop => {}
    }
    return flags;
}

// Determines if z flag needs to be set.
pub fn set_z_flag(result: u8) -> Flag {
    if result == 0x00 {
        return Flag::Set;
    } else {
        return Flag::Unset;
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
pub fn set_h_flag(arg1: u8, arg2: u8, op: Operation) -> Flag {
    let lo1 = arg1 & 0x0F;
    let lo2 = arg2 & 0x0F;

    match op {
        Operation::Add(c) => {
            if ((lo1 + lo2 + (c & 0x0F)) & (0x10)) == 0x10 {
                return Flag::Set;
            } else {
                return Flag::Unset;
            }
        }
        Operation::Sub(c) => {
            if (lo1.wrapping_sub(lo2).wrapping_sub(c & 0x0F) & (0x10)) == 0x10 {
                return Flag::Set;
            } else {
                return Flag::Unset;
            }
        }
    }
}

pub fn set_h_flag_16bit(arg1: u16, arg2: u16) -> Flag {
    let lower12_1 = arg1 & 0x0FFF;
    let lower12_2 = arg2 & 0x0FFF;

    if (lower12_1 + lower12_2) >= 0x1000 {
        return Flag::Set;
    } else {
        return Flag::Unset;
    }
}

// Determines if c flag needs to be set.
pub fn set_c_flag(is_carry: bool) -> Flag {
    if is_carry == true {
        return Flag::Set;
    } else {
        return Flag::Unset;
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

use super::*;

#[test]
fn test_instruction_creation() {
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
fn test_combine_bytes() {
    let x1 = 0x12;
    let x2 = 0xAB;
    let x3 = 0x12AB;
    assert_eq!(x3, combine_bytes(x1, x2));
}

#[test]
fn test_set_flags() {
    let reg_1 = 0b1010_1010;
    let flags1 = set_flags(
        FlagMod::Nop,
        FlagMod::Unset,
        FlagMod::Unset,
        FlagMod::Unset,
        reg_1,
    );

    let reg_2 = 0b0011_1110;
    let flags2 = set_flags(
        FlagMod::Set,
        FlagMod::Unset,
        FlagMod::Set,
        FlagMod::Nop,
        reg_2,
    );

    let reg_3 = 0b1010_1010;
    let flags3 = set_flags(
        FlagMod::Nop,
        FlagMod::Nop,
        FlagMod::Nop,
        FlagMod::Nop,
        reg_3,
    );

    let reg_4 = 0b1010_0000;
    let flags4 = set_flags(
        FlagMod::Unset,
        FlagMod::Set,
        FlagMod::Unset,
        FlagMod::Set,
        reg_4,
    );

    assert_eq!(flags1, 0b10001010);
    assert_eq!(flags2, 0b10111110);
    assert_eq!(flags3, 0b10101010);
    assert_eq!(flags4, 0b01010000);
}

#[test]
fn test_half_carry_add() {
    // Any numbers where the bottom four bits added together is over
    // 15 should result in set, and everything else should result in unset
    let reg_1 = 0b1010_1010;
    let reg_2 = 0b0011_1110;
    let h_flag_1 = set_h_flag(reg_1, reg_2, Operation::Add(0));

    let reg_1 = 0b1010_0000;
    let reg_2 = 0b0011_1111;
    let h_flag_2 = set_h_flag(reg_1, reg_2, Operation::Add(0));

    let reg_1 = 0b1111_0001;
    let reg_2 = 0b0111_1110;
    let h_flag_3 = set_h_flag(reg_1, reg_2, Operation::Add(0));

    let reg_1 = 0b1010_1111;
    let reg_2 = 0b0011_0001;
    let h_flag_4 = set_h_flag(reg_1, reg_2, Operation::Add(0));

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
    let flag1 = set_h_flag(0xA9, 0x5C, Operation::Sub(0));
    let flag2 = set_h_flag(0x5C, 0xA9, Operation::Sub(0));

    assert_eq!(flag1, FlagMod::Set);
    assert_eq!(flag2, FlagMod::Unset);
}

#[test]
fn test_post_incr() {
    let mut value: u16 = 423;
    assert_eq!(post_incr(&mut value), 423);
    assert_eq!(value, 424);

    let mut value: u16 = u16::MAX;
    assert_eq!(post_incr(&mut value), u16::MAX);
    assert_eq!(value, u16::MIN);
}

#[test]
fn test_post_decr() {
    let mut value: u16 = 423;
    assert_eq!(post_decr(&mut value), 423);
    assert_eq!(value, 422);

    let mut value: u16 = u16::MIN;
    assert_eq!(post_decr(&mut value), u16::MIN);
    assert_eq!(value, u16::MAX);
}
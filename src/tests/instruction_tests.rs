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

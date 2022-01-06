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
pub fn combine_bytes(hi: u8, lo: u8) -> u16{
    let mut res = hi as u16;
    res = (res << 8) + (lo as u16);
    return res;
}

// Load 16 bit immediate into register
pub fn load_d16(register: &mut u16, cycles: &mut usize, hi: u8, lo: u8){
    let imm_val = combine_bytes(hi, lo);
    *register = imm_val;
    *cycles = 12;
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
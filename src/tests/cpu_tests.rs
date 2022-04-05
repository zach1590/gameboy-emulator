use super::*;

#[test]
fn test_register_destructuring() {
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
    assert_eq!(low, 0x0A); // A: accumulator, F: flags
}

#[test]
fn test_load_d16() {
    let mut cpu = Cpu::new();
    cpu.mem
        .write_bytes(cpu.pc, vec![0xA7, 0xFF, 0xF0, 0xFF, 0x01, 0xFF, 0xFF, 0x00]);
    cpu.match_instruction(Instruction::get_instruction(0x01));
    cpu.match_instruction(Instruction::get_instruction(0x11));
    cpu.match_instruction(Instruction::get_instruction(0x21));
    cpu.match_instruction(Instruction::get_instruction(0x31));
    assert_eq!(cpu.reg.bc, 0xFFA7);
    assert_eq!(cpu.reg.de, 0xFFF0);
    assert_eq!(cpu.reg.hl, 0xFF01);
    assert_eq!(cpu.sp, 0x00FF);
}

#[test]
fn test_load_bcr() {
    let mut cpu = Cpu::new();

    cpu.reg.bc = 0x2345;
    cpu.match_instruction(Instruction::get_instruction(0x41));
    assert_eq!(cpu.reg.bc, 0x4545);
    assert_eq!(cpu.curr_cycles, 4);

    cpu.reg.de = 0xA03F;
    cpu.match_instruction(Instruction::get_instruction(0x42));
    assert_eq!(cpu.reg.bc, 0xA045);
    assert_eq!(cpu.curr_cycles, 4);

    cpu.reg.de = 0xA03F;
    cpu.match_instruction(Instruction::get_instruction(0x43));
    assert_eq!(cpu.reg.bc, 0x3F45);
    assert_eq!(cpu.curr_cycles, 4);

    cpu.reg.de = 0xA03F;
    cpu.match_instruction(Instruction::get_instruction(0x4B));
    assert_eq!(cpu.reg.bc, 0x3F3F);
    assert_eq!(cpu.curr_cycles, 4);

    cpu.reg.bc = 0x2345;
    cpu.match_instruction(Instruction::get_instruction(0x48));
    assert_eq!(cpu.reg.bc, 0x2323);
    assert_eq!(cpu.curr_cycles, 4);

    cpu.reg.hl = 0x1111;
    cpu.mem.write_bytes(cpu.reg.hl, vec![0x2C]);
    cpu.match_instruction(Instruction::get_instruction(0x4E));
    assert_eq!(cpu.reg.bc, 0x232C);
    assert_eq!(cpu.curr_cycles, 8);
}

#[test]
fn test_load_der() {
    let mut cpu = Cpu::new();

    cpu.reg.de = 0x2345;
    cpu.reg.bc = 0x0000;
    cpu.match_instruction(Instruction::get_instruction(0x51));
    assert_eq!(cpu.reg.de, 0x0045);
    assert_eq!(cpu.curr_cycles, 4);

    cpu.reg.af = 0xA03F;
    cpu.match_instruction(Instruction::get_instruction(0x57));
    assert_eq!(cpu.reg.de, 0xA045);
    assert_eq!(cpu.curr_cycles, 4);

    cpu.reg.hl = 0xA0A0;
    cpu.match_instruction(Instruction::get_instruction(0x55));
    assert_eq!(cpu.reg.de, 0xA045);
    assert_eq!(cpu.curr_cycles, 4);

    cpu.reg.de = 0xA03F;
    cpu.match_instruction(Instruction::get_instruction(0x5B));
    assert_eq!(cpu.reg.de, 0xA03F);
    assert_eq!(cpu.curr_cycles, 4);

    cpu.reg.bc = 0x2345;
    cpu.match_instruction(Instruction::get_instruction(0x58));
    assert_eq!(cpu.reg.de, 0xA023);
    assert_eq!(cpu.curr_cycles, 4);

    cpu.reg.hl = 0x1111;
    cpu.mem.write_bytes(cpu.reg.hl, vec![0x2C]);
    cpu.match_instruction(Instruction::get_instruction(0x5E));
    assert_eq!(cpu.reg.de, 0xA02C);
    assert_eq!(cpu.curr_cycles, 8);
}

#[test]
fn test_load_hlr() {
    let mut cpu = Cpu::new();

    cpu.reg.bc = 0x2345;
    cpu.match_instruction(Instruction::get_instruction(0x60));
    assert_eq!(cpu.reg.hl, 0x2300);
    assert_eq!(cpu.curr_cycles, 4);

    cpu.reg.af = 0xA03F;
    cpu.match_instruction(Instruction::get_instruction(0x6F));
    assert_eq!(cpu.reg.hl, 0x23A0);
    assert_eq!(cpu.curr_cycles, 4);
}

#[test]
fn test_halt() {
    let mut cpu = Cpu::new();
    cpu.match_instruction(Instruction::get_instruction(0x76));
    assert_eq!(cpu.curr_cycles, 4);
}

#[test]
fn test_load_hlr_mem() {
    let mut cpu = Cpu::new();

    cpu.reg.bc = 0x2345;
    cpu.reg.hl = 0x1111;
    cpu.match_instruction(Instruction::get_instruction(0x70));
    assert_eq!(cpu.mem.read_byte(cpu.reg.hl), 0x23);
    assert_eq!(cpu.curr_cycles, 8);

    cpu.reg.af = 0xA03F;
    cpu.reg.hl = 0x1114;
    cpu.match_instruction(Instruction::get_instruction(0x77));
    assert_eq!(cpu.mem.read_byte(cpu.reg.hl), 0xA0);
    assert_eq!(cpu.curr_cycles, 8);
}

#[test]
fn test_load_ar() {
    let mut cpu = Cpu::new();

    cpu.reg.af = 0x2345;
    cpu.reg.hl = 0x1111;
    cpu.mem.write_bytes(cpu.reg.hl, vec![0xBB]);
    cpu.match_instruction(Instruction::get_instruction(0x7E));
    assert_eq!(cpu.reg.af, 0xBB45);
    assert_eq!(cpu.curr_cycles, 8);

    cpu.reg.af = 0xA03F;
    cpu.reg.de = 0x1114;
    cpu.match_instruction(Instruction::get_instruction(0x7A));
    assert_eq!(cpu.reg.af, 0x113F);
    assert_eq!(cpu.curr_cycles, 4);
}

#[test]
fn test_xor_a() {
    let mut cpu = Cpu::new();

    cpu.reg.af = 0xA800;
    cpu.reg.bc = 0xA800;
    cpu.match_instruction(Instruction::get_instruction(0xA8));
    assert_eq!(cpu.reg.af, 0b0000000010000000);
    assert_eq!(cpu.curr_cycles, 4);

    cpu.reg.af = 0xA800;
    cpu.reg.bc = 0x00A8;
    cpu.match_instruction(Instruction::get_instruction(0xA9));
    assert_eq!(cpu.reg.af, 0b0000000010000000);
    assert_eq!(cpu.curr_cycles, 4);

    cpu.reg.af = 0xA800;
    cpu.reg.de = 0xFE01;
    cpu.match_instruction(Instruction::get_instruction(0xAA));
    assert_eq!(cpu.reg.af, 0x5600);
    assert_eq!(cpu.curr_cycles, 4);

    cpu.reg.af = 0xA800;
    cpu.reg.de = 0x01FE;
    cpu.match_instruction(Instruction::get_instruction(0xAB));
    assert_eq!(cpu.reg.af, 0x5600);
    assert_eq!(cpu.curr_cycles, 4);

    cpu.reg.af = 0xA800;
    cpu.reg.hl = 0xF0FE;
    cpu.match_instruction(Instruction::get_instruction(0xAC));
    assert_eq!(cpu.reg.af, 0x5800);
    assert_eq!(cpu.curr_cycles, 4);

    cpu.reg.af = 0xA800;
    cpu.reg.hl = 0xFEF0;
    cpu.match_instruction(Instruction::get_instruction(0xAD));
    assert_eq!(cpu.reg.af, 0x5800);
    assert_eq!(cpu.curr_cycles, 4);

    cpu.reg.af = 0xA800;
    cpu.reg.hl = 0xFFF0;
    cpu.match_instruction(Instruction::get_instruction(0xAE));
    assert_eq!(cpu.reg.af, 0xA800);
    assert_eq!(cpu.curr_cycles, 8);

    cpu.reg.af = 0xA800;
    cpu.match_instruction(Instruction::get_instruction(0xAF));
    assert_eq!(cpu.reg.af, 0x0080);
    assert_eq!(cpu.curr_cycles, 4);
}

#[test]
fn test_and_a() {
    let mut cpu = Cpu::new();

    cpu.reg.af = 0xA800;
    cpu.reg.bc = 0xA800;
    cpu.match_instruction(Instruction::get_instruction(0xA0));
    assert_eq!(cpu.reg.af, 0b1010_1000_0010_0000);
    assert_eq!(cpu.curr_cycles, 4);

    cpu.reg.af = 0xA800;
    cpu.reg.bc = 0x00A8;
    cpu.match_instruction(Instruction::get_instruction(0xA1));
    assert_eq!(cpu.reg.af, 0b1010_1000_0010_0000);
    assert_eq!(cpu.curr_cycles, 4);

    cpu.reg.af = 0xA801;
    cpu.reg.de = 0xFE01;
    cpu.match_instruction(Instruction::get_instruction(0xA2));
    assert_eq!(cpu.reg.af, 0xA821);
    assert_eq!(cpu.curr_cycles, 4);

    cpu.reg.af = 0xA806;
    cpu.reg.de = 0x01FE;
    cpu.match_instruction(Instruction::get_instruction(0xA3));
    assert_eq!(cpu.reg.af, 0xA826);
    assert_eq!(cpu.curr_cycles, 4);

    cpu.reg.af = 0xA8F8;
    cpu.reg.hl = 0x00FE;
    cpu.match_instruction(Instruction::get_instruction(0xA4));
    assert_eq!(cpu.reg.af, 0x00A8);
    assert_eq!(cpu.curr_cycles, 4);

    cpu.reg.af = 0xA8FF;
    cpu.reg.hl = 0xFE00;
    cpu.match_instruction(Instruction::get_instruction(0xA5));
    assert_eq!(cpu.reg.af, 0x00AF);
    assert_eq!(cpu.curr_cycles, 4);

    cpu.reg.af = 0xA8CD;
    cpu.reg.hl = 0xFFF0;
    cpu.mem.write_bytes(cpu.reg.hl, vec![0x7A]); // 0111 and 1010 = 0010
    cpu.match_instruction(Instruction::get_instruction(0xA6));
    assert_eq!(cpu.reg.af, 0x282D);
    assert_eq!(cpu.curr_cycles, 8);

    cpu.reg.af = 0xA823;
    cpu.match_instruction(Instruction::get_instruction(0xA7));
    assert_eq!(cpu.reg.af, 0xA823);
    assert_eq!(cpu.curr_cycles, 4);
}
#[test]
fn test_or_a() {
    let mut cpu = Cpu::new();

    cpu.reg.af = 0xA800;
    cpu.reg.bc = 0xA800;
    cpu.match_instruction(Instruction::get_instruction(0xB0));
    assert_eq!(cpu.reg.af, 0xA800);
    assert_eq!(cpu.curr_cycles, 4);

    cpu.reg.af = 0xA800;
    cpu.reg.bc = 0x00A8;
    cpu.match_instruction(Instruction::get_instruction(0xB1));
    assert_eq!(cpu.reg.af, 0xA800);
    assert_eq!(cpu.curr_cycles, 4);

    cpu.reg.af = 0xA801;
    cpu.reg.de = 0xFE01;
    cpu.match_instruction(Instruction::get_instruction(0xB2));
    assert_eq!(cpu.reg.af, 0xFE01);
    assert_eq!(cpu.curr_cycles, 4);

    cpu.reg.af = 0xA806;
    cpu.reg.de = 0x01FE;
    cpu.match_instruction(Instruction::get_instruction(0xB3));
    assert_eq!(cpu.reg.af, 0xFE06);
    assert_eq!(cpu.curr_cycles, 4);

    cpu.reg.af = 0x00F8;
    cpu.reg.hl = 0x00FE;
    cpu.match_instruction(Instruction::get_instruction(0xB4));
    assert_eq!(cpu.reg.af, 0x0088);
    assert_eq!(cpu.curr_cycles, 4);

    cpu.reg.af = 0xA8FF;
    cpu.reg.hl = 0xFE00;
    cpu.match_instruction(Instruction::get_instruction(0xB5));
    assert_eq!(cpu.reg.af, 0xA80F);
    assert_eq!(cpu.curr_cycles, 4);

    cpu.reg.af = 0xA8CD;
    cpu.reg.hl = 0xFFF0;
    cpu.mem.write_bytes(cpu.reg.hl, vec![0x7A]); // 0111 and 1010 = 0010
    cpu.match_instruction(Instruction::get_instruction(0xB6));
    assert_eq!(cpu.reg.af, 0xFA0D);
    assert_eq!(cpu.curr_cycles, 8);

    cpu.reg.af = 0xA823;
    cpu.match_instruction(Instruction::get_instruction(0xB7));
    assert_eq!(cpu.reg.af, 0xA803);
    assert_eq!(cpu.curr_cycles, 4);
}
#[test]
fn test_add_a() {
    let mut cpu = Cpu::new();

    cpu.reg.af = 0xA800;
    cpu.reg.bc = 0xA800;
    cpu.match_instruction(Instruction::get_instruction(0x80));
    assert_eq!(cpu.reg.af, 0x5030);
    assert_eq!(cpu.curr_cycles, 4);

    cpu.reg.af = 0xA800;
    cpu.reg.bc = 0x00A8;
    cpu.match_instruction(Instruction::get_instruction(0x81));
    assert_eq!(cpu.reg.af, 0x5030);
    assert_eq!(cpu.curr_cycles, 4);

    cpu.reg.af = 0xA801;
    cpu.reg.de = 0xFE01;
    cpu.match_instruction(Instruction::get_instruction(0x82));
    assert_eq!(cpu.reg.af, 0xA631);
    assert_eq!(cpu.curr_cycles, 4);

    cpu.reg.af = 0xA806;
    cpu.reg.de = 0x01FE;
    cpu.match_instruction(Instruction::get_instruction(0x83));
    assert_eq!(cpu.reg.af, 0xA636);
    assert_eq!(cpu.curr_cycles, 4);

    cpu.reg.af = 0x00F8;
    cpu.reg.hl = 0x00FE;
    cpu.match_instruction(Instruction::get_instruction(0x84));
    assert_eq!(cpu.reg.af, 0x0088);
    assert_eq!(cpu.curr_cycles, 4);

    cpu.reg.af = 0xA8FF;
    cpu.reg.hl = 0xFE00;
    cpu.match_instruction(Instruction::get_instruction(0x85));
    assert_eq!(cpu.reg.af, 0xA80F);
    assert_eq!(cpu.curr_cycles, 4);

    cpu.reg.af = 0xA8CD;
    cpu.reg.hl = 0xFFF0;
    cpu.mem.write_bytes(cpu.reg.hl, vec![0x74]);
    cpu.match_instruction(Instruction::get_instruction(0x86));
    assert_eq!(cpu.reg.af, 0x1C1D);
    assert_eq!(cpu.curr_cycles, 8);

    cpu.reg.af = 0xA8CD;
    cpu.reg.hl = 0xFFF0;
    cpu.mem.write_bytes(cpu.reg.hl, vec![0x49]);
    cpu.match_instruction(Instruction::get_instruction(0x86));
    assert_eq!(cpu.reg.af, 0xF12D);
    assert_eq!(cpu.curr_cycles, 8);

    cpu.reg.af = 0xA8CD;
    cpu.reg.hl = 0xFFF0;
    cpu.mem.write_bytes(cpu.reg.hl, vec![0x44]);
    cpu.match_instruction(Instruction::get_instruction(0x86));
    assert_eq!(cpu.reg.af, 0xEC0D);
    assert_eq!(cpu.curr_cycles, 8);

    cpu.reg.af = 0xA823;
    cpu.match_instruction(Instruction::get_instruction(0x87));
    assert_eq!(cpu.reg.af, 0x5033);
    assert_eq!(cpu.curr_cycles, 4);
}

#[test]
fn test_sub_a() {
    let mut cpu = Cpu::new();

    cpu.reg.af = 0xA800;
    cpu.reg.bc = 0xA800;
    cpu.match_instruction(Instruction::get_instruction(0x90));
    assert_eq!(cpu.reg.af, 0x00C0);
    assert_eq!(cpu.curr_cycles, 4);

    cpu.reg.af = 0xA800;
    cpu.reg.bc = 0x00A8;
    cpu.match_instruction(Instruction::get_instruction(0x91));
    assert_eq!(cpu.reg.af, 0x00C0);
    assert_eq!(cpu.curr_cycles, 4);

    cpu.reg.af = 0xA801;
    cpu.reg.de = 0xFE01;
    cpu.match_instruction(Instruction::get_instruction(0x92));
    assert_eq!(cpu.reg.af, 0xAA71);
    assert_eq!(cpu.curr_cycles, 4);

    cpu.reg.af = 0xA806;
    cpu.reg.de = 0x01FE;
    cpu.match_instruction(Instruction::get_instruction(0x93));
    assert_eq!(cpu.reg.af, 0xAA76);
    assert_eq!(cpu.curr_cycles, 4);

    cpu.reg.af = 0x00F8;
    cpu.reg.hl = 0x00FE;
    cpu.match_instruction(Instruction::get_instruction(0x94));
    assert_eq!(cpu.reg.af, 0x00C8);
    assert_eq!(cpu.curr_cycles, 4);

    cpu.reg.af = 0xA8FF;
    cpu.reg.hl = 0xFE00;
    cpu.match_instruction(Instruction::get_instruction(0x95));
    assert_eq!(cpu.reg.af, 0xA84F);
    assert_eq!(cpu.curr_cycles, 4);

    cpu.reg.af = 0xA8CD;
    cpu.reg.hl = 0xFFF0;
    cpu.mem.write_bytes(cpu.reg.hl, vec![0x74]);
    cpu.match_instruction(Instruction::get_instruction(0x96));
    assert_eq!(cpu.reg.af, 0x344D);
    assert_eq!(cpu.curr_cycles, 8);

    cpu.reg.af = 0xA8CD;
    cpu.reg.hl = 0xFFF0;
    cpu.mem.write_bytes(cpu.reg.hl, vec![0x49]);
    cpu.match_instruction(Instruction::get_instruction(0x96));
    assert_eq!(cpu.reg.af, 0x5F6D);
    assert_eq!(cpu.curr_cycles, 8);

    cpu.reg.af = 0xA8CD;
    cpu.reg.hl = 0xFFF0;
    cpu.mem.write_bytes(cpu.reg.hl, vec![0xB4]);
    cpu.match_instruction(Instruction::get_instruction(0x96));
    assert_eq!(cpu.reg.af, 0xF45D);
    assert_eq!(cpu.curr_cycles, 8);

    cpu.reg.af = 0xA823;
    cpu.match_instruction(Instruction::get_instruction(0x97));
    assert_eq!(cpu.reg.af, 0x00C3);
    assert_eq!(cpu.curr_cycles, 4);
}

#[test]
fn test_adc_a() {
    let mut cpu = Cpu::new();

    cpu.reg.af = 0xA810;
    cpu.reg.bc = 0x5600;
    cpu.match_instruction(Instruction::get_instruction(0x88));
    assert_eq!(cpu.reg.af, 0xFF00);
    assert_eq!(cpu.curr_cycles, 4);

    cpu.reg.af = 0xA810;
    cpu.reg.bc = 0x0057;
    cpu.match_instruction(Instruction::get_instruction(0x89));
    assert_eq!(cpu.reg.af, 0x00B0);
    assert_eq!(cpu.curr_cycles, 4);

    cpu.reg.af = 0xA801;
    cpu.reg.de = 0xFE01;
    cpu.match_instruction(Instruction::get_instruction(0x8A));
    assert_eq!(cpu.reg.af, 0xA631);
    assert_eq!(cpu.curr_cycles, 4);

    cpu.reg.af = 0xA811;
    cpu.reg.de = 0x3301;
    cpu.match_instruction(Instruction::get_instruction(0x8A));
    assert_eq!(cpu.reg.af, 0xDC01);
    assert_eq!(cpu.curr_cycles, 4);
}

#[test]
fn test_sbc_a() {
    let mut cpu = Cpu::new();

    cpu.reg.af = 0xA81D;
    cpu.reg.hl = 0xFFF0;
    cpu.mem.write_bytes(cpu.reg.hl, vec![0x49]);
    cpu.match_instruction(Instruction::get_instruction(0x9E));
    assert_eq!(cpu.reg.af, 0x5E6D);
    assert_eq!(cpu.curr_cycles, 8);

    cpu.reg.af = 0xA83D;
    cpu.reg.hl = 0xFFF0;
    cpu.mem.write_bytes(cpu.reg.hl, vec![0xB4]);
    cpu.match_instruction(Instruction::get_instruction(0x9E));
    assert_eq!(cpu.reg.af, 0xF35D);
    assert_eq!(cpu.curr_cycles, 8);

    cpu.reg.af = 0xA823;
    cpu.reg.de = 0xA823;
    cpu.match_instruction(Instruction::get_instruction(0x9A));
    assert_eq!(cpu.reg.af, 0x00C3);
    assert_eq!(cpu.curr_cycles, 4);
}

#[test]
fn test_cp_a() {
    let mut cpu = Cpu::new();

    cpu.reg.af = 0x001D;
    cpu.reg.hl = 0xFFF0;
    cpu.mem.write_bytes(cpu.reg.hl, vec![0x00]);
    cpu.match_instruction(Instruction::get_instruction(0xBE));
    assert_eq!(cpu.reg.af, 0x00CD);
    assert_eq!(cpu.curr_cycles, 8);

    cpu.reg.af = 0xA83D;
    cpu.reg.hl = 0xFFF0;
    cpu.mem.write_bytes(cpu.reg.hl, vec![0xB4]);
    cpu.match_instruction(Instruction::get_instruction(0xBE));
    assert_eq!(cpu.reg.af, 0xA85D);
    assert_eq!(cpu.curr_cycles, 8);

    cpu.reg.af = 0xA823;
    cpu.reg.de = 0xA923;
    cpu.match_instruction(Instruction::get_instruction(0xBA));
    assert_eq!(cpu.reg.af, 0xA873);
    assert_eq!(cpu.curr_cycles, 4);
}

#[test]
fn test_ld_a_into_memory() {
    // 0x02, 0x12, 0x22, 0x32
    let mut cpu = Cpu::new();

    cpu.reg.af = 0x1D2E;
    cpu.reg.bc = 0xFFF0;
    cpu.match_instruction(Instruction::get_instruction(0x02));
    assert_eq!(cpu.mem.read_byte(0xFFF0), 0x1D);
    assert_eq!(cpu.reg.bc, 0xFFF0);
    assert_eq!(cpu.curr_cycles, 8);

    cpu.reg.af = 0x472E;
    cpu.reg.de = 0xFFFF;
    cpu.match_instruction(Instruction::get_instruction(0x12));
    assert_eq!(cpu.mem.read_byte(0xFFFF), 0x47);
    assert_eq!(cpu.reg.de, 0xFFFF);
    assert_eq!(cpu.curr_cycles, 8);

    cpu.reg.af = 0x5B2E;
    cpu.reg.hl = 0xFFFF;
    cpu.match_instruction(Instruction::get_instruction(0x22));
    assert_eq!(cpu.mem.read_byte(0xFFFF), 0x5B);
    assert_eq!(cpu.reg.hl, 0x0000);
    assert_eq!(cpu.curr_cycles, 8);

    cpu.reg.af = 0x6A2E;
    cpu.reg.hl = 0xFFF0;
    cpu.match_instruction(Instruction::get_instruction(0x32));
    assert_eq!(cpu.mem.read_byte(0xFFF0), 0x6A);
    assert_eq!(cpu.reg.hl, 0xFFEF);
    assert_eq!(cpu.curr_cycles, 8);
}

#[test]
fn test_ld_imm_8bit_bdh() {
    // 0x06, 0x16, 0x26
    let mut cpu = Cpu::new();

    cpu.pc = 0x2300;
    cpu.mem.write_bytes(cpu.pc, vec![0xFF, 0x10, 0x3A]);

    cpu.match_instruction(Instruction::get_instruction(0x06));
    assert_eq!(cpu.reg.bc, 0xFF00);
    assert_eq!(cpu.curr_cycles, 8);

    cpu.match_instruction(Instruction::get_instruction(0x16));
    assert_eq!(cpu.reg.de, 0x1000);
    assert_eq!(cpu.curr_cycles, 8);

    cpu.match_instruction(Instruction::get_instruction(0x26));
    assert_eq!(cpu.reg.hl, 0x3A00);
    assert_eq!(cpu.curr_cycles, 8);

    assert_eq!(cpu.pc, 0x2303);
}

#[test]
fn test_ld_imm_8bit_hl() {
    // 0x36
    let mut cpu = Cpu::new();

    cpu.pc = 0x2300;
    cpu.mem.write_byte(cpu.pc, 0xB7);
    cpu.match_instruction(Instruction::get_instruction(0x36));
    assert_eq!(cpu.mem.read_byte(0x2300), 0xB7);
    assert_eq!(cpu.curr_cycles, 12);
    assert_eq!(cpu.pc, 0x2301);
}

#[test]
fn test_ld_imm_8bit_cela() {
    // 0x0E, 0x1E, 0x2E, 0x3E
    let mut cpu = Cpu::new();

    cpu.pc = 0x2300;
    cpu.mem.write_bytes(cpu.pc, vec![0xFF, 0x10, 0x3A, 0xB7]);

    cpu.match_instruction(Instruction::get_instruction(0x0E));
    assert_eq!(cpu.reg.bc, 0x00FF);
    assert_eq!(cpu.curr_cycles, 8);

    cpu.match_instruction(Instruction::get_instruction(0x1E));
    assert_eq!(cpu.reg.de, 0x0010);
    assert_eq!(cpu.curr_cycles, 8);

    cpu.match_instruction(Instruction::get_instruction(0x2E));
    assert_eq!(cpu.reg.hl, 0x003A);
    assert_eq!(cpu.curr_cycles, 8);

    cpu.match_instruction(Instruction::get_instruction(0x3E));
    assert_eq!(cpu.reg.af, 0xB700);
    assert_eq!(cpu.curr_cycles, 8);

    assert_eq!(cpu.pc, 0x2304);
}

#[test]
fn test_ld_memory_into_a() {
    // 0x0A, 0x1A, 0x2A, 0x3A
    let mut cpu = Cpu::new();

    cpu.reg.bc = 0x3456;
    cpu.reg.de = 0x3457;
    cpu.reg.hl = 0x3458;
    cpu.reg.af = 0x1D2E;
    cpu.mem.write_bytes(cpu.reg.bc, vec![0xEF, 0xAB, 0xC3]);

    cpu.match_instruction(Instruction::get_instruction(0x0A));
    assert_eq!(cpu.reg.af, 0xEF2E);
    assert_eq!(cpu.reg.bc, 0x3456);
    assert_eq!(cpu.curr_cycles, 8);

    cpu.match_instruction(Instruction::get_instruction(0x1A));
    assert_eq!(cpu.reg.af, 0xAB2E);
    assert_eq!(cpu.reg.de, 0x3457);
    assert_eq!(cpu.curr_cycles, 8);

    cpu.match_instruction(Instruction::get_instruction(0x2A));
    assert_eq!(cpu.reg.af, 0xC32E);
    assert_eq!(cpu.reg.hl, 0x3459);
    assert_eq!(cpu.curr_cycles, 8);

    cpu.mem.write_byte(cpu.reg.hl, 0x5D);
    cpu.match_instruction(Instruction::get_instruction(0x3A));
    assert_eq!(cpu.reg.af, 0x5D2E);
    assert_eq!(cpu.reg.hl, 0x3458);
    assert_eq!(cpu.curr_cycles, 8);
}

#[test]
fn test_0xe0() {
    let mut cpu = Cpu::new();
    let a8 = 0xB6;
    cpu.pc = 0xff;
    cpu.reg.af = 0xC321;
    cpu.mem.write_byte(cpu.pc, a8);

    cpu.match_instruction(Instruction::get_instruction(0xE0));
    assert_eq!(cpu.mem.read_byte(0xFF00 + a8 as u16), 0xC3);
    assert_eq!(cpu.curr_cycles, 12);
    assert_eq!(cpu.pc, 0x100);
}

#[test]
fn test_0xf0() {
    let mut cpu = Cpu::new();
    let a8 = 0xB6;
    let data_at_a8 = 0x32;
    cpu.pc = 0xff;
    cpu.reg.af = 0xC321;
    cpu.mem.write_byte(cpu.pc, a8);
    cpu.mem.write_byte(0xFF00 + a8 as u16, data_at_a8);

    cpu.match_instruction(Instruction::get_instruction(0xF0));
    assert_eq!(cpu.reg.af, 0x3221);
    assert_eq!(cpu.curr_cycles, 12);
    assert_eq!(cpu.pc, 0x100);
}

#[test]
fn test_0xe2() {
    let mut cpu = Cpu::new();
    cpu.reg.af = 0xC321;
    cpu.reg.bc = 0x00B6;
    let location = 0xFF00 + 0x00B6;

    cpu.match_instruction(Instruction::get_instruction(0xE2));
    assert_eq!(cpu.mem.read_byte(location), 0xC3);
    assert_eq!(cpu.curr_cycles, 8);
    assert_eq!(cpu.pc, 0);
}

#[test]
fn test_0xf2() {
    let mut cpu = Cpu::new();
    cpu.reg.af = 0xC321;
    cpu.reg.bc = 0x00B6;
    let data_at_c = 0x32;
    let location = 0xFF00 + 0x00B6;
    cpu.mem.write_byte(location, data_at_c);

    cpu.match_instruction(Instruction::get_instruction(0xF2));
    assert_eq!(cpu.reg.af, 0x3221);
    assert_eq!(cpu.curr_cycles, 8);
    assert_eq!(cpu.pc, 0);
}

#[test]
fn test_0x08() {
    let mut cpu = Cpu::new();
    cpu.sp = 0xC321;
    cpu.pc = 0x1234;
    cpu.mem.write_bytes(cpu.pc, vec![0x60, 0xF0]);

    cpu.match_instruction(Instruction::get_instruction(0x08));
    assert_eq!(cpu.mem.read_byte(0xF060), 0x21);
    assert_eq!(cpu.mem.read_byte(0xF060 + 0x0001), 0xC3);
    assert_eq!(cpu.curr_cycles, 20);
    assert_eq!(cpu.pc, 0x1236);
}

#[test]
fn test_0xf9() {
    let mut cpu = Cpu::new();
    cpu.sp = 0xC321;
    cpu.reg.hl = 0x1234;

    cpu.match_instruction(Instruction::get_instruction(0xF9));
    assert_eq!(cpu.sp, 0x1234);
    assert_eq!(cpu.reg.hl, 0x1234);
    assert_eq!(cpu.curr_cycles, 8);
}

#[test]
fn test_0xea() {
    let mut cpu = Cpu::new();
    cpu.reg.af = 0xC321;
    cpu.pc = 0x1234;
    cpu.mem.write_bytes(cpu.pc, vec![0x60, 0xF0]);

    cpu.match_instruction(Instruction::get_instruction(0xEA));
    assert_eq!(cpu.mem.read_byte(0xF060), 0xC3);
    assert_eq!(cpu.curr_cycles, 16);
    assert_eq!(cpu.pc, 0x1236);
}

#[test]
fn test_0xfa() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x1234;
    cpu.mem.write_bytes(cpu.pc, vec![0x60, 0xF0]);
    cpu.mem.write_byte(0xF060, 0xDB);

    cpu.match_instruction(Instruction::get_instruction(0xFA));
    assert_eq!(cpu.reg.af, 0xDB00);
    assert_eq!(cpu.curr_cycles, 16);
    assert_eq!(cpu.pc, 0x1236);
}

#[test]
fn test_0xe8() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x1234;
    cpu.sp = 1013; // 0x03F5
    let r8: i8 = -97; // 0x9F (97 is 0x61)
    cpu.mem.write_byte(cpu.pc, r8 as u8);

    cpu.match_instruction(Instruction::get_instruction(0xe8));
    assert_eq!(cpu.sp, 916); // 0x0394
    assert_eq!(cpu.reg.af, 0x0030);
    assert_eq!(cpu.curr_cycles, 16);
    assert_eq!(cpu.pc, 0x1235);

    cpu.sp = 500; // 0x01F4
    let r8: i8 = 97; // 0x61 as unsigned bits
    cpu.mem.write_byte(cpu.pc, r8 as u8);

    cpu.match_instruction(Instruction::get_instruction(0xe8));
    assert_eq!(cpu.sp, 597); // 0x0255
    assert_eq!(cpu.reg.af, 0x0010);
    assert_eq!(cpu.curr_cycles, 16);
    assert_eq!(cpu.pc, 0x1236);

    cpu.sp = 0xFFFF;
    let r8: i8 = -1;
    cpu.mem.write_byte(cpu.pc, r8 as u8);

    cpu.match_instruction(Instruction::get_instruction(0xe8));
    assert_eq!(cpu.sp, 0xFFFE);
    assert_eq!(cpu.reg.af, 0x0030);
    assert_eq!(cpu.curr_cycles, 16);
    assert_eq!(cpu.pc, 0x1237);
}

#[test]
fn test_0xf8() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x1234;
    cpu.sp = 1013; // 0x03F5
    let r8: i8 = -97; // 0x9F (97 is 0x61)
    cpu.mem.write_byte(cpu.pc, r8 as u8);

    cpu.match_instruction(Instruction::get_instruction(0xf8));
    assert_eq!(cpu.reg.hl, 916); // 0x0394
    assert_eq!(cpu.reg.af, 0x0030);
    assert_eq!(cpu.curr_cycles, 12);
    assert_eq!(cpu.pc, 0x1235);

    cpu.sp = 500; // 0x01F4
    let r8: i8 = 97; // 0x61 as unsigned bits
    cpu.mem.write_byte(cpu.pc, r8 as u8);

    cpu.match_instruction(Instruction::get_instruction(0xf8));
    assert_eq!(cpu.reg.hl, 597); // 0x0255
    assert_eq!(cpu.reg.af, 0x0010);
    assert_eq!(cpu.curr_cycles, 12);
    assert_eq!(cpu.pc, 0x1236);

    cpu.sp = 0xFFFF;
    let r8: i8 = -1;
    cpu.mem.write_byte(cpu.pc, r8 as u8);

    cpu.match_instruction(Instruction::get_instruction(0xf8));
    assert_eq!(cpu.reg.hl, 0xFFFE);
    assert_eq!(cpu.reg.af, 0x0030);
    assert_eq!(cpu.curr_cycles, 12);
    assert_eq!(cpu.pc, 0x1237);

    cpu.sp = 0xFFFF;
    let r8: u8 = 0xFF;
    cpu.mem.write_byte(cpu.pc, r8);

    cpu.match_instruction(Instruction::get_instruction(0xf8));
    assert_eq!(cpu.reg.hl, 0xFFFE);
    assert_eq!(cpu.reg.af, 0x0030);
    assert_eq!(cpu.curr_cycles, 12);
    assert_eq!(cpu.pc, 0x1238);
}

#[test]
fn test_16bit_increment() {
    let mut cpu = Cpu::new();

    cpu.reg.bc = 0xABCD;
    cpu.reg.de = 0xBAFF;
    cpu.reg.hl = 0xFFFF;
    cpu.sp = 0x3456;

    cpu.match_instruction(Instruction::get_instruction(0x03));
    assert_eq!(cpu.reg.bc, 0xABCE);
    assert_eq!(cpu.curr_cycles, 8);

    cpu.match_instruction(Instruction::get_instruction(0x13));
    assert_eq!(cpu.reg.de, 0xBB00);
    assert_eq!(cpu.curr_cycles, 8);

    cpu.match_instruction(Instruction::get_instruction(0x23));
    assert_eq!(cpu.reg.hl, 0x0000);
    assert_eq!(cpu.curr_cycles, 8);

    cpu.match_instruction(Instruction::get_instruction(0x33));
    assert_eq!(cpu.sp, 0x3457);
    assert_eq!(cpu.curr_cycles, 8);

    assert_eq!(cpu.reg.af, 0x0000);
}

#[test]
fn test_16bit_decrement() {
    let mut cpu = Cpu::new();

    cpu.reg.bc = 0xABCD;
    cpu.reg.de = 0xBA00;
    cpu.reg.hl = 0x0000;
    cpu.sp = 0x3456;

    cpu.match_instruction(Instruction::get_instruction(0x0B));
    assert_eq!(cpu.reg.bc, 0xABCC);
    assert_eq!(cpu.curr_cycles, 8);

    cpu.match_instruction(Instruction::get_instruction(0x1B));
    assert_eq!(cpu.reg.de, 0xB9FF);
    assert_eq!(cpu.curr_cycles, 8);

    cpu.match_instruction(Instruction::get_instruction(0x2B));
    assert_eq!(cpu.reg.hl, 0xFFFF);
    assert_eq!(cpu.curr_cycles, 8);

    cpu.match_instruction(Instruction::get_instruction(0x3B));
    assert_eq!(cpu.sp, 0x3455);
    assert_eq!(cpu.curr_cycles, 8);

    assert_eq!(cpu.reg.af, 0x0000);
}

#[test]
fn test_hl_add_rr() {
    let mut cpu = Cpu::new();

    cpu.reg.bc = 0xABCD;
    cpu.reg.de = 0xBA00;
    cpu.reg.hl = 0x7A5D;
    cpu.sp = 0x14C6;

    cpu.match_instruction(Instruction::get_instruction(0x09));
    assert_eq!(cpu.reg.hl, 0x262A);
    assert_eq!(cpu.curr_cycles, 8);
    assert_eq!(cpu.reg.af, 0x0030);

    cpu.match_instruction(Instruction::get_instruction(0x19));
    assert_eq!(cpu.reg.hl, 0xE02A);
    assert_eq!(cpu.curr_cycles, 8);
    assert_eq!(cpu.reg.af, 0x0020);

    cpu.match_instruction(Instruction::get_instruction(0x29));
    assert_eq!(cpu.reg.hl, 0xC054);
    assert_eq!(cpu.curr_cycles, 8);
    assert_eq!(cpu.reg.af, 0x0010);

    cpu.match_instruction(Instruction::get_instruction(0x39));
    assert_eq!(cpu.reg.hl, 0xD51A);
    assert_eq!(cpu.curr_cycles, 8);
    assert_eq!(cpu.reg.af, 0x0000);
}

#[test]
fn test_z_flag_not_set_hl_add_rr() {
    let mut cpu = Cpu::new();

    cpu.reg.hl = 0xFFFF;
    cpu.sp = 0x0001;

    cpu.match_instruction(Instruction::get_instruction(0x39));
    assert_eq!(cpu.reg.hl, 0x0000);
    assert_eq!(cpu.curr_cycles, 8);
    assert_eq!(cpu.reg.af, 0x0030);
}

#[test]
fn test_8bit_increment() {
    let mut cpu = Cpu::new();

    cpu.reg.af = 0xFF00;
    cpu.reg.bc = 0xABCD;
    cpu.reg.de = 0x6F22;
    cpu.reg.hl = 0x7AFF;

    cpu.match_instruction(Instruction::get_instruction(0x04));
    assert_eq!(cpu.reg.bc, 0xACCD);
    assert_eq!(cpu.curr_cycles, 4);
    assert_eq!(cpu.reg.af, 0xFF00);

    cpu.match_instruction(Instruction::get_instruction(0x0C));
    assert_eq!(cpu.reg.bc, 0xACCE);
    assert_eq!(cpu.curr_cycles, 4);
    assert_eq!(cpu.reg.af, 0xFF00);

    cpu.match_instruction(Instruction::get_instruction(0x14));
    assert_eq!(cpu.reg.de, 0x7022);
    assert_eq!(cpu.curr_cycles, 4);
    assert_eq!(cpu.reg.af, 0xFF20);

    cpu.match_instruction(Instruction::get_instruction(0x1C));
    assert_eq!(cpu.reg.de, 0x7023);
    assert_eq!(cpu.curr_cycles, 4);
    assert_eq!(cpu.reg.af, 0xFF00);

    cpu.match_instruction(Instruction::get_instruction(0x24));
    assert_eq!(cpu.reg.hl, 0x7BFF);
    assert_eq!(cpu.curr_cycles, 4);
    assert_eq!(cpu.reg.af, 0xFF00);

    cpu.match_instruction(Instruction::get_instruction(0x2C));
    assert_eq!(cpu.reg.hl, 0x7B00);
    assert_eq!(cpu.curr_cycles, 4);
    assert_eq!(cpu.reg.af, 0xFFA0);

    cpu.match_instruction(Instruction::get_instruction(0x3C));
    assert_eq!(cpu.reg.af, 0x00A0);
    assert_eq!(cpu.curr_cycles, 4);
}

#[test]
fn test_8bit_decrement() {
    let mut cpu = Cpu::new();

    cpu.reg.af = 0xFF00;
    cpu.reg.bc = 0xACCE;
    cpu.reg.de = 0x7001;
    cpu.reg.hl = 0x7B00;

    cpu.match_instruction(Instruction::get_instruction(0x05));
    assert_eq!(cpu.reg.bc, 0xABCE);
    assert_eq!(cpu.curr_cycles, 4);
    assert_eq!(cpu.reg.af, 0xFF40);

    cpu.match_instruction(Instruction::get_instruction(0x0D));
    assert_eq!(cpu.reg.bc, 0xABCD);
    assert_eq!(cpu.curr_cycles, 4);
    assert_eq!(cpu.reg.af, 0xFF40);

    cpu.match_instruction(Instruction::get_instruction(0x15));
    assert_eq!(cpu.reg.de, 0x6F01);
    assert_eq!(cpu.curr_cycles, 4);
    assert_eq!(cpu.reg.af, 0xFF60);

    cpu.match_instruction(Instruction::get_instruction(0x1D));
    assert_eq!(cpu.reg.de, 0x6F00);
    assert_eq!(cpu.curr_cycles, 4);
    assert_eq!(cpu.reg.af, 0xFFC0);

    cpu.match_instruction(Instruction::get_instruction(0x25));
    assert_eq!(cpu.reg.hl, 0x7A00);
    assert_eq!(cpu.curr_cycles, 4);
    assert_eq!(cpu.reg.af, 0xFF40);

    cpu.match_instruction(Instruction::get_instruction(0x2D));
    assert_eq!(cpu.reg.hl, 0x7AFF); // 0 - 1 wraps around
    assert_eq!(cpu.curr_cycles, 4);
    assert_eq!(cpu.reg.af, 0xFF60);

    cpu.match_instruction(Instruction::get_instruction(0x3D));
    assert_eq!(cpu.reg.af, 0xFE40);
    assert_eq!(cpu.curr_cycles, 4);
}

#[test]
fn test_a_arithemetic_imm8() {
    let mut cpu = Cpu::new();

    cpu.reg.af = 0xFF00;
    cpu.pc = 0x3456;
    cpu.mem
        .write_bytes(cpu.pc, vec![0x01, 0x01, 0x8E, 0x05, 0xA4, 0x7A, 0x34, 0xDB]);

    cpu.match_instruction(Instruction::get_instruction(0xC6));
    assert_eq!(cpu.curr_cycles, 8);
    assert_eq!(cpu.reg.af, 0x00B0);

    cpu.match_instruction(Instruction::get_instruction(0xD6));
    assert_eq!(cpu.curr_cycles, 8);
    assert_eq!(cpu.reg.af, 0xFF70);

    cpu.match_instruction(Instruction::get_instruction(0xE6));
    assert_eq!(cpu.curr_cycles, 8);
    assert_eq!(cpu.reg.af, 0x8E20);

    cpu.match_instruction(Instruction::get_instruction(0xF6));
    assert_eq!(cpu.curr_cycles, 8);
    assert_eq!(cpu.reg.af, 0x8F00);

    cpu.match_instruction(Instruction::get_instruction(0xCE));
    assert_eq!(cpu.curr_cycles, 8);
    assert_eq!(cpu.reg.af, 0x3330);

    cpu.match_instruction(Instruction::get_instruction(0xDE));
    assert_eq!(cpu.curr_cycles, 8);
    assert_eq!(cpu.reg.af, 0xB870);

    cpu.match_instruction(Instruction::get_instruction(0xEE));
    assert_eq!(cpu.reg.af, 0x8C00);
    assert_eq!(cpu.curr_cycles, 8);

    cpu.match_instruction(Instruction::get_instruction(0xFE));
    assert_eq!(cpu.reg.af, 0x8C50);
    assert_eq!(cpu.curr_cycles, 8);
}

#[test]
fn test_push_rr() {
    let mut cpu = Cpu::new();

    cpu.reg.bc = 0xACCE;
    cpu.reg.de = 0x7001;
    cpu.reg.hl = 0x7B00;
    cpu.reg.af = 0xFF00;
    cpu.sp = 0xFA00;

    cpu.match_instruction(Instruction::get_instruction(0xC5));
    assert_eq!(cpu.curr_cycles, 16);
    assert_eq!(cpu.mem.read_byte(cpu.sp), 0xCE);
    assert_eq!(cpu.mem.read_byte(cpu.sp + 1), 0xAC);
    assert_eq!(cpu.sp, 0xF9FE);

    cpu.match_instruction(Instruction::get_instruction(0xD5));
    assert_eq!(cpu.curr_cycles, 16);
    assert_eq!(cpu.mem.read_byte(cpu.sp), 0x01);
    assert_eq!(cpu.mem.read_byte(cpu.sp + 1), 0x70);
    assert_eq!(cpu.sp, 0xF9FC);

    cpu.match_instruction(Instruction::get_instruction(0xE5));
    assert_eq!(cpu.curr_cycles, 16);
    assert_eq!(cpu.mem.read_byte(cpu.sp), 0x00);
    assert_eq!(cpu.mem.read_byte(cpu.sp + 1), 0x7B);
    assert_eq!(cpu.sp, 0xF9FA);

    cpu.match_instruction(Instruction::get_instruction(0xF5));
    assert_eq!(cpu.curr_cycles, 16);
    assert_eq!(cpu.mem.read_byte(cpu.sp), 0x00);
    assert_eq!(cpu.mem.read_byte(cpu.sp + 1), 0xFF);
    assert_eq!(cpu.sp, 0xF9F8);
}

#[test]
fn test_pop_rr() {
    let mut cpu = Cpu::new();

    cpu.sp = 0xF9F8;
    cpu.mem
        .write_bytes(cpu.sp, vec![0x01, 0x0A, 0x8E, 0x05, 0xA4, 0x7A, 0x34, 0xDB]);

    cpu.match_instruction(Instruction::get_instruction(0xC1));
    assert_eq!(cpu.curr_cycles, 12);
    assert_eq!(cpu.reg.bc, 0x0A01);
    assert_eq!(cpu.sp, 0xF9FA);

    cpu.match_instruction(Instruction::get_instruction(0xD1));
    assert_eq!(cpu.curr_cycles, 12);
    assert_eq!(cpu.reg.de, 0x058E);
    assert_eq!(cpu.sp, 0xF9FC);

    cpu.match_instruction(Instruction::get_instruction(0xE1));
    assert_eq!(cpu.curr_cycles, 12);
    assert_eq!(cpu.reg.hl, 0x7AA4);
    assert_eq!(cpu.sp, 0xF9FE);

    cpu.match_instruction(Instruction::get_instruction(0xF1));
    assert_eq!(cpu.curr_cycles, 12);
    assert_eq!(cpu.reg.af, 0xDB30); // bottom half of f is zeroes out
    assert_eq!(cpu.sp, 0xFA00);
}

#[test]
fn test_jr_cond_false() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.reg.af = 0x00F0;
    cpu.match_instruction(Instruction::get_instruction(0x20));
    assert_eq!(cpu.pc, 0x0101);
    assert_eq!(cpu.curr_cycles, 8);

    cpu.match_instruction(Instruction::get_instruction(0x30));
    assert_eq!(cpu.pc, 0x0102);
    assert_eq!(cpu.curr_cycles, 8);

    cpu.reg.af = 0x0000;
    cpu.match_instruction(Instruction::get_instruction(0x28));
    assert_eq!(cpu.pc, 0x0103);
    assert_eq!(cpu.curr_cycles, 8);

    cpu.match_instruction(Instruction::get_instruction(0x38));
    assert_eq!(cpu.pc, 0x0104);
    assert_eq!(cpu.curr_cycles, 8);
}

#[test]
fn test_jr_cond_true() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;
    cpu.mem.write_byte(cpu.pc, i8::from(-0x37) as u8);
    cpu.mem.write_byte(0x00CA, i8::from(-0x7A) as u8);
    cpu.mem.write_byte(0x0051, 0xFE);
    cpu.mem.write_byte(0x0050, i8::from(0x7F) as u8);
    cpu.mem.write_byte(0x00D0, 0x7F);

    cpu.reg.af = 0x0000;
    cpu.match_instruction(Instruction::get_instruction(0x20));
    assert_eq!(cpu.pc, 0x00CA);
    assert_eq!(cpu.curr_cycles, 12);

    cpu.match_instruction(Instruction::get_instruction(0x30));
    assert_eq!(cpu.pc, 0x0051);
    assert_eq!(cpu.curr_cycles, 12);

    cpu.match_instruction(Instruction::get_instruction(0x18));
    assert_eq!(cpu.pc, 0x0050);
    assert_eq!(cpu.curr_cycles, 12);

    cpu.reg.af = 0x00F0;
    cpu.match_instruction(Instruction::get_instruction(0x28));
    assert_eq!(cpu.pc, 0x00D0);
    assert_eq!(cpu.curr_cycles, 12);

    cpu.match_instruction(Instruction::get_instruction(0x38));
    assert_eq!(cpu.pc, 0x0150);
    assert_eq!(cpu.curr_cycles, 12);
}

#[test]
fn test_ret_cond_false() {
    // Never take the return, so PC never moves
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;
    cpu.sp = 0;

    cpu.reg.af = 0x00F0;
    cpu.match_instruction(Instruction::get_instruction(0xC0));
    assert_eq!(cpu.pc, 0x0100);
    assert_eq!(cpu.sp, 0x0000);
    assert_eq!(cpu.curr_cycles, 8);

    cpu.match_instruction(Instruction::get_instruction(0xD0));
    assert_eq!(cpu.pc, 0x0100);
    assert_eq!(cpu.sp, 0x0000);
    assert_eq!(cpu.curr_cycles, 8);

    cpu.reg.af = 0x0000;
    cpu.match_instruction(Instruction::get_instruction(0xC8));
    assert_eq!(cpu.pc, 0x0100);
    assert_eq!(cpu.sp, 0x0000);
    assert_eq!(cpu.curr_cycles, 8);

    cpu.match_instruction(Instruction::get_instruction(0xD8));
    assert_eq!(cpu.pc, 0x0100);
    assert_eq!(cpu.sp, 0x0000);
    assert_eq!(cpu.curr_cycles, 8);
}

#[test]
fn test_ret_cond_true() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;
    cpu.sp = 0xF000 - 12;

    cpu.mem.write_bytes(
        cpu.sp,
        vec![
            0x25, 0xA3, 0x6B, 0x7F, 0x88, 0x94, 0xDE, 0x5F, 0x4C, 0x67, 0xEE, 0x52,
        ],
    );

    cpu.reg.af = 0x0000;
    cpu.match_instruction(Instruction::get_instruction(0xC0));
    assert_eq!(cpu.pc, 0xA325);
    assert_eq!(cpu.sp, 0xEFF6);
    assert_eq!(cpu.curr_cycles, 20);

    cpu.match_instruction(Instruction::get_instruction(0xD0));
    assert_eq!(cpu.pc, 0x7F6B);
    assert_eq!(cpu.sp, 0xEFF8);
    assert_eq!(cpu.curr_cycles, 20);

    cpu.reg.af = 0x00F0;
    cpu.match_instruction(Instruction::get_instruction(0xC8));
    assert_eq!(cpu.pc, 0x9488);
    assert_eq!(cpu.sp, 0xEFFA);
    assert_eq!(cpu.curr_cycles, 20);

    cpu.match_instruction(Instruction::get_instruction(0xD8));
    assert_eq!(cpu.pc, 0x5FDE);
    assert_eq!(cpu.sp, 0xEFFC);
    assert_eq!(cpu.curr_cycles, 20);

    cpu.match_instruction(Instruction::get_instruction(0xC9));
    assert_eq!(cpu.pc, 0x674C);
    assert_eq!(cpu.sp, 0xEFFE);
    assert_eq!(cpu.curr_cycles, 16);

    cpu.match_instruction(Instruction::get_instruction(0xD9));
    assert_eq!(cpu.pc, 0x52EE);
    assert_eq!(cpu.sp, 0xF000);
    assert_eq!(cpu.curr_cycles, 16);
    assert_eq!(cpu.ime, false);
    assert_eq!(cpu.ime_scheduled, true);
}

#[test]
fn test_call_cond_false() {
    // Never take the return, so PC never moves
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;
    cpu.sp = 0xF000;

    cpu.mem
        .write_bytes(cpu.pc, vec![0x88, 0x89, 0x9A, 0xC7, 0xB5, 0x65, 0x43, 0x4A]);

    cpu.reg.af = 0x00F0;
    cpu.match_instruction(Instruction::get_instruction(0xC4));
    assert_eq!(cpu.pc, 0x0102);
    assert_eq!(cpu.sp, 0xF000);
    assert_eq!(cpu.curr_cycles, 12);

    cpu.match_instruction(Instruction::get_instruction(0xD4));
    assert_eq!(cpu.pc, 0x0104);
    assert_eq!(cpu.sp, 0xF000);
    assert_eq!(cpu.curr_cycles, 12);

    cpu.reg.af = 0x0000;
    cpu.match_instruction(Instruction::get_instruction(0xCC));
    assert_eq!(cpu.pc, 0x0106);
    assert_eq!(cpu.sp, 0xF000);
    assert_eq!(cpu.curr_cycles, 12);

    cpu.match_instruction(Instruction::get_instruction(0xDC));
    assert_eq!(cpu.pc, 0x0108);
    assert_eq!(cpu.sp, 0xF000);
    assert_eq!(cpu.curr_cycles, 12);
}

#[test]
fn test_call_cond_true() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;
    cpu.sp = 0xF000;

    cpu.mem.write_bytes(
        cpu.pc,
        vec![0x25, 0xA3, 0x6B, 0x7F, 0x88, 0x94, 0xDE, 0x5F, 0x4C, 0x67],
    );

    cpu.reg.af = 0x0000;
    cpu.match_instruction(Instruction::get_instruction(0xC4));
    assert_eq!(cpu.pc, 0xA325);
    assert_eq!(cpu.mem.read_byte(0xF000), 0x00); // Subtract first, so starting location of sp should be empty
    assert_eq!(cpu.mem.read_byte(0xEFFF), 0x01);
    assert_eq!(cpu.mem.read_byte(0xEFFE), 0x02);
    assert_eq!(cpu.sp, 0xEFFE);
    assert_eq!(cpu.curr_cycles, 24);

    cpu.pc = 0x0102;
    cpu.match_instruction(Instruction::get_instruction(0xD4));
    assert_eq!(cpu.pc, 0x7F6B);
    assert_eq!(cpu.mem.read_byte(0xEFFD), 0x01);
    assert_eq!(cpu.mem.read_byte(0xEFFC), 0x04);
    assert_eq!(cpu.sp, 0xEFFC);
    assert_eq!(cpu.curr_cycles, 24);

    cpu.pc = 0x0104;
    cpu.reg.af = 0x00F0;
    cpu.match_instruction(Instruction::get_instruction(0xCC));
    assert_eq!(cpu.pc, 0x9488);
    assert_eq!(cpu.mem.read_byte(0xEFFB), 0x01);
    assert_eq!(cpu.mem.read_byte(0xEFFA), 0x06);
    assert_eq!(cpu.sp, 0xEFFA);
    assert_eq!(cpu.curr_cycles, 24);

    cpu.pc = 0x0106;
    cpu.match_instruction(Instruction::get_instruction(0xDC));
    assert_eq!(cpu.pc, 0x5FDE);
    assert_eq!(cpu.mem.read_byte(0xEFF9), 0x01);
    assert_eq!(cpu.mem.read_byte(0xEFF8), 0x08);
    assert_eq!(cpu.sp, 0xEFF8);
    assert_eq!(cpu.curr_cycles, 24);

    cpu.pc = 0x0108;
    cpu.match_instruction(Instruction::get_instruction(0xCD));
    assert_eq!(cpu.pc, 0x674C);
    assert_eq!(cpu.mem.read_byte(0xEFF7), 0x01);
    assert_eq!(cpu.mem.read_byte(0xEFF6), 0x0A);
    assert_eq!(cpu.sp, 0xEFF6);
    assert_eq!(cpu.curr_cycles, 24);
}

#[test]
fn test_jp_cond_false() {
    // Never take the return, so PC never moves
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.mem
        .write_bytes(cpu.pc, vec![0x88, 0x89, 0x9A, 0xC7, 0xB5, 0x65, 0x43, 0x4A]);

    cpu.reg.af = 0x00F0;
    cpu.match_instruction(Instruction::get_instruction(0xC2));
    assert_eq!(cpu.pc, 0x0102);
    assert_eq!(cpu.curr_cycles, 12);

    cpu.match_instruction(Instruction::get_instruction(0xD2));
    assert_eq!(cpu.pc, 0x0104);
    assert_eq!(cpu.curr_cycles, 12);

    cpu.reg.af = 0x0000;
    cpu.match_instruction(Instruction::get_instruction(0xCA));
    assert_eq!(cpu.pc, 0x0106);
    assert_eq!(cpu.curr_cycles, 12);

    cpu.match_instruction(Instruction::get_instruction(0xDA));
    assert_eq!(cpu.pc, 0x0108);
    assert_eq!(cpu.curr_cycles, 12);
}

#[test]
fn test_jp_cond_true() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0100;

    cpu.mem.write_bytes(
        cpu.pc,
        vec![0x25, 0xA3, 0x6B, 0x7F, 0x88, 0x94, 0xDE, 0x5F, 0x4C, 0x67],
    );

    cpu.reg.af = 0x0000;
    cpu.match_instruction(Instruction::get_instruction(0xC2));
    assert_eq!(cpu.pc, 0xA325);
    assert_eq!(cpu.curr_cycles, 16);

    cpu.pc = 0x0102;
    cpu.match_instruction(Instruction::get_instruction(0xD2));
    assert_eq!(cpu.pc, 0x7F6B);
    assert_eq!(cpu.curr_cycles, 16);

    cpu.pc = 0x0104;
    cpu.reg.af = 0x00F0;
    cpu.match_instruction(Instruction::get_instruction(0xCA));
    assert_eq!(cpu.pc, 0x9488);
    assert_eq!(cpu.curr_cycles, 16);

    cpu.pc = 0x0106;
    cpu.match_instruction(Instruction::get_instruction(0xDA));
    assert_eq!(cpu.pc, 0x5FDE);
    assert_eq!(cpu.curr_cycles, 16);

    cpu.pc = 0x0108;
    cpu.match_instruction(Instruction::get_instruction(0xC3));
    assert_eq!(cpu.pc, 0x674C);
    assert_eq!(cpu.curr_cycles, 16);
}

#[test]
fn test_rst() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x3245;
    cpu.sp = 0xF000;

    cpu.match_instruction(Instruction::get_instruction(0xC7));
    assert_eq!(cpu.mem.read_byte(0xEFFF), 0x32);
    assert_eq!(cpu.mem.read_byte(0xEFFE), 0x45);
    assert_eq!(cpu.sp, 0xEFFE);
    assert_eq!(cpu.pc, 0x0000);

    cpu.match_instruction(Instruction::get_instruction(0xD7));
    assert_eq!(cpu.mem.read_byte(0xEFFD), 0x00);
    assert_eq!(cpu.mem.read_byte(0xEFFC), 0x00);
    assert_eq!(cpu.sp, 0xEFFC);
    assert_eq!(cpu.pc, 0x0010);

    cpu.match_instruction(Instruction::get_instruction(0xE7));
    assert_eq!(cpu.mem.read_byte(0xEFFB), 0x00);
    assert_eq!(cpu.mem.read_byte(0xEFFA), 0x10);
    assert_eq!(cpu.sp, 0xEFFA);
    assert_eq!(cpu.pc, 0x0020);

    cpu.match_instruction(Instruction::get_instruction(0xF7));
    assert_eq!(cpu.mem.read_byte(0xEFF9), 0x00);
    assert_eq!(cpu.mem.read_byte(0xEFF8), 0x20);
    assert_eq!(cpu.sp, 0xEFF8);
    assert_eq!(cpu.pc, 0x0030);

    cpu.match_instruction(Instruction::get_instruction(0xCF));
    assert_eq!(cpu.mem.read_byte(0xEFF7), 0x00);
    assert_eq!(cpu.mem.read_byte(0xEFF6), 0x30);
    assert_eq!(cpu.sp, 0xEFF6);
    assert_eq!(cpu.pc, 0x0008);

    cpu.match_instruction(Instruction::get_instruction(0xDF));
    assert_eq!(cpu.mem.read_byte(0xEFF5), 0x00);
    assert_eq!(cpu.mem.read_byte(0xEFF4), 0x08);
    assert_eq!(cpu.sp, 0xEFF4);
    assert_eq!(cpu.pc, 0x0018);

    cpu.match_instruction(Instruction::get_instruction(0xEF));
    assert_eq!(cpu.mem.read_byte(0xEFF3), 0x00);
    assert_eq!(cpu.mem.read_byte(0xEFF2), 0x18);
    assert_eq!(cpu.sp, 0xEFF2);
    assert_eq!(cpu.pc, 0x0028);

    cpu.match_instruction(Instruction::get_instruction(0xFF));
    assert_eq!(cpu.mem.read_byte(0xEFF1), 0x00);
    assert_eq!(cpu.mem.read_byte(0xEFF0), 0x28);
    assert_eq!(cpu.sp, 0xEFF0);
    assert_eq!(cpu.pc, 0x0038);
}

#[test]
fn test_rla_rlca() {
    let mut cpu = Cpu::new();
    cpu.reg.af = 0x3215;

    cpu.match_instruction(Instruction::get_instruction(0x07));
    assert_eq!(cpu.reg.af, 0x6405);

    cpu.match_instruction(Instruction::get_instruction(0x07));
    assert_eq!(cpu.reg.af, 0xC805);

    cpu.match_instruction(Instruction::get_instruction(0x07));
    assert_eq!(cpu.reg.af, 0x9115);

    cpu.match_instruction(Instruction::get_instruction(0x17));
    assert_eq!(cpu.reg.af, 0x2315);

    cpu.match_instruction(Instruction::get_instruction(0x17));
    assert_eq!(cpu.reg.af, 0x4705);
}

#[test]
fn test_rra_rrca() {
    let mut cpu = Cpu::new();
    cpu.reg.af = 0x3215;

    cpu.match_instruction(Instruction::get_instruction(0x0F));
    assert_eq!(cpu.reg.af, 0x1905);

    cpu.match_instruction(Instruction::get_instruction(0x0F));
    assert_eq!(cpu.reg.af, 0x8C15);

    cpu.match_instruction(Instruction::get_instruction(0x0F));
    assert_eq!(cpu.reg.af, 0x4605);

    cpu.match_instruction(Instruction::get_instruction(0x1F));
    assert_eq!(cpu.reg.af, 0x2305);

    cpu.match_instruction(Instruction::get_instruction(0x1F));
    assert_eq!(cpu.reg.af, 0x1115);

    cpu.match_instruction(Instruction::get_instruction(0x1F));
    assert_eq!(cpu.reg.af, 0x8815);

    cpu.match_instruction(Instruction::get_instruction(0x1F));
    assert_eq!(cpu.reg.af, 0xC405);

    cpu.match_instruction(Instruction::get_instruction(0x1F));
    assert_eq!(cpu.reg.af, 0x6205);
}

#[test]
fn test_cpl() {
    let mut cpu = Cpu::new();

    cpu.reg.af = 0x3215;
    cpu.match_instruction(Instruction::get_instruction(0x2F));
    assert_eq!(cpu.reg.af, 0xCD75);

    cpu.reg.af = 0x8890;
    cpu.match_instruction(Instruction::get_instruction(0x2F));
    assert_eq!(cpu.reg.af, 0x77F0);

    cpu.reg.af = 0x8800;
    cpu.match_instruction(Instruction::get_instruction(0x2F));
    assert_eq!(cpu.reg.af, 0x7760);
}

#[test]
fn test_scf() {
    let mut cpu = Cpu::new();

    cpu.reg.af = 0x3215;
    cpu.match_instruction(Instruction::get_instruction(0x37));
    assert_eq!(cpu.reg.af, 0x3215);

    cpu.reg.af = 0x8890;
    cpu.match_instruction(Instruction::get_instruction(0x37));
    assert_eq!(cpu.reg.af, 0x8890);

    cpu.reg.af = 0x8800;
    cpu.match_instruction(Instruction::get_instruction(0x37));
    assert_eq!(cpu.reg.af, 0x8810);
}

#[test]
fn test_ccf() {
    let mut cpu = Cpu::new();

    cpu.reg.af = 0x3215;
    cpu.match_instruction(Instruction::get_instruction(0x3F));
    assert_eq!(cpu.reg.af, 0x3205);

    cpu.reg.af = 0x8890;
    cpu.match_instruction(Instruction::get_instruction(0x3F));
    assert_eq!(cpu.reg.af, 0x8880);

    cpu.reg.af = 0x8800;
    cpu.match_instruction(Instruction::get_instruction(0x3F));
    assert_eq!(cpu.reg.af, 0x8810);
}

#[test]
fn test_daa() {
    let mut cpu = Cpu::new();

    cpu.reg.af = 0x0A15;
    cpu.match_instruction(Instruction::get_instruction(0x27));
    assert_eq!(cpu.reg.af, 0x7015);

    cpu.reg.af = 0x8890;
    cpu.match_instruction(Instruction::get_instruction(0x27));
    assert_eq!(cpu.reg.af, 0xE810);

    cpu.reg.af = 0x8840;
    cpu.match_instruction(Instruction::get_instruction(0x27));
    assert_eq!(cpu.reg.af, 0x8840);

    cpu.reg.af = 0x9970;
    cpu.match_instruction(Instruction::get_instruction(0x27));
    assert_eq!(cpu.reg.af, 0x3350);

    cpu.reg.af = 0x9930;
    cpu.match_instruction(Instruction::get_instruction(0x27));
    assert_eq!(cpu.reg.af, 0xFF10);

    cpu.reg.af = 0xBB30;
    cpu.match_instruction(Instruction::get_instruction(0x27));
    assert_eq!(cpu.reg.af, 0x2110);
}

#[test]
fn test_set_top_byte() {
    let value = Registers::set_top_byte(0xFFFF, 0x32);
    assert_eq!(value, 0x32FF);
}

#[test]
fn test_set_bottom_byte() {
    let value = Registers::set_bottom_byte(0xFFFF, 0x32);
    assert_eq!(value, 0xFF32);
}

#[test]
fn test_get_hi() {
    let imm16: u16 = 0x3AF8;
    let top = Registers::get_hi(imm16);
    assert_eq!(top, 0x3A);
}

#[test]
fn test_get_lo() {
    let imm16: u16 = 0x3AF8;
    let bottom = Registers::get_lo(imm16);
    assert_eq!(bottom, 0xF8);
}

#[test]
fn test_is_z_set() {
    let mut reg = Registers::new();
    reg.af = 0b0000_0000_1000_0000;
    assert_eq!(true, reg.get_z());
}

#[test]
fn test_is_z_not_set() {
    let mut reg = Registers::new();
    reg.af = 0b1111_1111_0111_1111;
    assert_eq!(false, reg.get_z());
}

#[test]
fn test_is_n_set() {
    let mut reg = Registers::new();
    reg.af = 0b0000_0000_0100_0000;
    assert_eq!(true, reg.get_n());
}

#[test]
fn test_is_n_not_set() {
    let mut reg = Registers::new();
    reg.af = 0b1111_1111_1011_1111;
    assert_eq!(false, reg.get_n());
}

#[test]
fn test_is_h_set() {
    let mut reg = Registers::new();
    reg.af = 0b0000_0000_0010_0000;
    assert_eq!(true, reg.get_h());
}

#[test]
fn test_is_h_not_set() {
    let mut reg = Registers::new();
    reg.af = 0b1111_1111_1101_1111;
    assert_eq!(false, reg.get_h());
}

#[test]
fn test_is_c_set() {
    let mut reg = Registers::new();
    reg.af = 0b0000_0000_0001_0000;
    assert_eq!(true, reg.get_c());
}

#[test]
fn test_is_c_not_set() {
    let mut reg = Registers::new();
    reg.af = 0b1111_1111_1110_1111;
    assert_eq!(false, reg.get_c());
}

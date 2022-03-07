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
    assert_eq!(cpu.reg.bc, 0xA7FF);
    assert_eq!(cpu.reg.de, 0xF0FF);
    assert_eq!(cpu.reg.hl, 0x01FF);
    assert_eq!(cpu.sp, 0xFF00);
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
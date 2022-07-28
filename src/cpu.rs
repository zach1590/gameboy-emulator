mod alu;
mod instruction;
mod registers;

use super::bus::Bus;
use super::mbc::Mbc;
use super::memory::Memory;
use instruction::Instruction;

#[cfg(feature = "debug")]
use sdl2::render::Texture;

use registers::Registers as Reg;

pub struct Cpu {
    bus: Bus,
    reg: Reg,
    pc: u16,                // Program Counter
    sp: u16,                // Stack Pointer
    pub curr_cycles: usize, // The number of cycles the current instruction should take to execute
    ime: bool,
    ime_scheduled: bool,
    ime_flipped: bool, // Just tells us that the previous instruction was an EI (For haltbug)(Set up to not apply for reti)
    pub is_running: bool,
}

impl Cpu {
    pub fn new() -> Cpu {
        return Cpu {
            bus: Bus::new(),
            reg: Reg::new(),
            pc: 0x0100,
            sp: 0,
            curr_cycles: 0,
            ime: false,
            ime_scheduled: false,
            is_running: true, // Controlled by halt
            ime_flipped: false,
        };
    }

    pub fn dmg_init(self: &mut Self, checksum: u8) {
        self.reg.dmg_init(checksum);
        self.bus.dmg_init();
        self.sp = 0xFFFE;
    }

    pub fn set_mbc(self: &mut Self, cart_mbc: Box<dyn Mbc>) {
        self.bus.set_mbc(cart_mbc);
    }

    pub fn execute(self: &mut Self) {
        if self.ime_scheduled == true {
            self.ime_scheduled = false;
            self.ime = true; // Now interrupts should occur delayed one instruction
            self.ime_flipped = true;
        } else {
            self.ime_flipped = false;
        }

        let opcode = self.read_pc();
        let i = Instruction::get_instruction(opcode);

        if i.values == (0x0C, 0x0B) {
            let opcode = self.read_pc();
            let cb_i = Instruction::get_instruction(opcode);
            self.match_cb_instruction(cb_i);
        } else {
            self.match_instruction(i);
        }
    }

    // Ripped off from somewhere online, need to go back and find the original
    #[cfg(feature = "debug")]
    pub fn is_blargg_done(self: &Self) -> bool {
        if self.bus.read_byte(self.pc + 0) == 0x18 && self.bus.read_byte(self.pc + 1) == 0xFE {
            return true;
        } else if self.bus.read_byte(self.pc + 0) == 0xc3
            && self.bus.read_byte(self.pc + 1) == ((self.pc & 0xFF) as u8)
            && self.bus.read_byte(self.pc + 2) == ((self.pc >> 8) as u8)
        {
            return true;
        }
        return false;
    }

    fn handle_interrupt(&mut self) {
        let i_enable = self.read_addr(0xFFFF);
        let mut i_fired = self.read_addr(0xFF0F);
        self.ime = false;

        for i in 0..=4 {
            if i_enable & i_fired & (0x01 << i) == (0x01 << i) {
                // https://www.reddit.com/r/EmuDev/comments/u9itc2/problem_with_halt_gameboy_and_dr_mario/
                i_fired = i_fired & !(0x01 << i);
                self.write_byte(0xFF0F, i_fired);

                self.stack_push(self.pc);

                // Set PC based on corresponding interrupt vector
                self.pc = 0x0040 + (0x0008 * i);
                break; // Only handle the highest priority interrupt
            }
        }
    }

    pub fn adv_cycles(self: &mut Self, cycles: usize) {
        self.bus.adv_cycles(cycles);
    }

    fn emulate_haltbug(self: &mut Self) {
        self.pc = self.pc.wrapping_sub(1);
    }

    pub fn check_interrupts(self: &mut Self) {
        if self.bus.interrupt_pending() {
            self.is_running = true;
        }
        if self.ime && self.bus.interrupt_pending() {
            self.handle_interrupt();
        }
    }

    pub fn update_input(self: &mut Self) {
        // ???
    }

    fn match_instruction(self: &mut Self, i: Instruction) {
        // Create a method for every instruction
        match i.opcode {
            0x00 => { /* NOP */ }
            0x10 => { /* STOP (Never used outside CGB Speed Switching) */ }
            0x20 | 0x30 | 0x18 | 0x28 | 0x38 => {
                // JR NZ/NC/C/Z, r8
                let r8 = self.read_byte();
                let eval_cond = match i.values {
                    (0x02, 0x00) => !self.reg.get_z(),
                    (0x03, 0x00) => !self.reg.get_c(),
                    (0x01, 0x08) => true,
                    (0x02, 0x08) => self.reg.get_z(),
                    (0x03, 0x08) => self.reg.get_c(),
                    _ => panic!(
                        "Valid: 0x20, 0x30, 0x28, 0x38, Current: {:#04X}, {:#04X}",
                        i.values.0, i.values.1
                    ),
                };
                if eval_cond {
                    self.internal_cycle();
                    let (result, _) = alu::reg_add_8bit_signed(self.pc, r8);
                    self.pc = result;
                }
            }
            0x01 | 0x11 | 0x21 | 0x31 => {
                // Load 16 bit immediate into BC/DE/HL/SP
                let (hi, lo) = self.read_long();
                let register = self.get_mut_reg16_by_opcode(i.values.0);
                alu::load_d16(register, hi, lo);
            }
            0x02 | 0x12 | 0x22 | 0x32 => {
                // LD (BC)/(DE)/(HL+)/(HL-), A
                let (str_val_a, _) = Reg::get_hi_lo(self.reg.af);
                let location = match i.values.0 {
                    0x00 => self.reg.bc,
                    0x01 => self.reg.de,
                    0x02 => alu::post_incr(&mut self.reg.hl),
                    0x03 => alu::post_decr(&mut self.reg.hl),
                    _ => panic!(
                        "Valid: 0x02, 0x12, 0x22, 0x32, Current: {:#04X}, {:#04X}",
                        i.values.0, i.values.1
                    ),
                };
                self.write_byte(location, str_val_a);
            }
            0x03 | 0x13 | 0x23 | 0x33 => {
                // INC BC/DE/HL/SP
                let register = self.get_mut_reg16_by_opcode(i.values.0);
                alu::post_incr(register); // Writing a 16 bit register is +4 cycles?
                self.internal_cycle();
            }
            0x04 | 0x14 | 0x24 | 0x05 | 0x15 | 0x25 | 0x0C | 0x1C | 0x2C | 0x0D | 0x1D | 0x2D => {
                // 8 Bit increment and decrement for bc, de, hl
                let register = self.get_reg16_by_opcode(i.values.0);
                let inc_dec = if (i.values.1 == 0x04) || (i.values.1 == 0x05) {
                    Reg::get_hi(register)
                } else {
                    Reg::get_lo(register)
                };
                let result = if (i.values.1 == 0x04) || i.values.1 == 0x0C {
                    alu::incr_8bit(inc_dec, &mut self.reg.af)
                } else {
                    alu::decr_8bit(inc_dec, &mut self.reg.af)
                };
                let mut_reg = self.get_mut_reg16_by_opcode(i.values.0);
                if (i.values.1 == 0x04) || (i.values.1 == 0x05) {
                    *mut_reg = Reg::set_hi(*mut_reg, result);
                } else {
                    *mut_reg = Reg::set_lo(*mut_reg, result);
                }
            }
            0x34 | 0x35 => {
                // 8 Bit increment and decrement for (hl)
                let val_at_hl = self.read_hl();
                let result = if i.values.1 == 0x04 {
                    alu::incr_8bit(val_at_hl, &mut self.reg.af)
                } else {
                    alu::decr_8bit(val_at_hl, &mut self.reg.af)
                };
                self.write_byte(self.reg.hl, result);
            }
            0x3C | 0x3D => {
                // 8 Bit increment and decrement for A
                let inc_dec = Reg::get_hi(self.reg.af);
                let result = if i.values.1 == 0x0C {
                    alu::incr_8bit(inc_dec, &mut self.reg.af)
                } else {
                    alu::decr_8bit(inc_dec, &mut self.reg.af)
                };
                self.reg.af = Reg::set_hi(self.reg.af, result);
            }
            0x06 | 0x16 | 0x26 => {
                // LD B/D/H, d8
                let ld_value = self.read_byte();
                let register = self.get_mut_reg16_by_opcode(i.values.0);
                alu::load_imm_d8(register, ld_value, true);
            }
            0x36 => {
                // LD (HL), d8
                let ld_value = self.read_byte();
                self.write_byte(self.reg.hl, ld_value);
            }
            0x07 | 0x17 => {
                // RLCA and RLA
                alu::rotate_left_a(i.values.0 == 1, &mut self.reg);
            }
            0x0F | 0x1F => {
                // RRCA and RRA
                alu::rotate_right_a(i.values.0 == 1, &mut self.reg);
            }
            0x27 => {
                // DAA
                self.reg.af = alu::daa(&self.reg);
            }
            0x2F => {
                // CPL
                self.reg.af = alu::cpl(self.reg.af);
            }
            0x37 => {
                // SCF
                self.reg.af = alu::scf(self.reg.af);
            }
            0x3F => {
                // CCF
                self.reg.af = alu::ccf(self.reg.af);
            }
            0x08 => {
                // LD (a16), SP
                let (hi, lo) = self.read_long();
                let imm16 = alu::combine_bytes(hi, lo);
                self.write_long(imm16, self.sp);
            }
            0x09 | 0x19 | 0x29 | 0x39 => {
                // EX: ADD HL RR
                let add_value = self.get_reg16_by_opcode(i.values.0);
                alu::hl_add_rr(&mut self.reg.hl, add_value, &mut self.reg.af);
                self.internal_cycle();
            }
            0x0A | 0x1A | 0x2A | 0x3A => {
                // LD A, (BC)/(DE)/(HL+)/(HL-)
                let location = match i.values.0 {
                    0x00 => self.reg.bc,
                    0x01 => self.reg.de,
                    0x02 => alu::post_incr(&mut self.reg.hl),
                    0x03 => alu::post_decr(&mut self.reg.hl),
                    _ => panic!(
                        "Valid: 0x0A, 0x1A, 0x2A, 0x3A, Current: {:#04X}, {:#04X}",
                        i.values.0, i.values.1
                    ),
                };
                let new_a_val = self.read_addr(location);
                self.reg.af = Reg::set_hi(self.reg.af, new_a_val);
            }
            0x0B | 0x1B | 0x2B | 0x3B => {
                // DEC BC/DE/HL/SP
                let register = self.get_mut_reg16_by_opcode(i.values.0);
                alu::post_decr(register);
                self.internal_cycle();
            }
            0x0E | 0x1E | 0x2E => {
                // LD C/E/L, d8
                let ld_value = self.read_byte();
                let register = self.get_mut_reg16_by_opcode(i.values.0);
                alu::load_imm_d8(register, ld_value, false);
            }
            0x3E => {
                // LD A, d8
                let ld_value = self.read_byte();
                alu::load_imm_d8(&mut self.reg.af, ld_value, true);
            }
            0x40..=0x4F => {
                // LD B/C, R
                // B for 0x40 - 0x47    C for 0x48 - 0x4F
                let ld_hi = i.values.1 <= 0x07;
                let ld_value = self.get_reg_by_opcode(i.values.1);
                alu::load_8_bit_into_reg(&mut self.reg.bc, ld_hi, ld_value);
            }
            0x50..=0x5F => {
                // LD D/E, R
                // D for 0x50 - 0x57    E for 0x58 - 0x5F
                let ld_hi = i.values.1 <= 0x07;
                let ld_value = self.get_reg_by_opcode(i.values.1);
                alu::load_8_bit_into_reg(&mut self.reg.de, ld_hi, ld_value);
            }
            0x60..=0x6F => {
                // LD H/L, R
                // H for 0x60 - 0x67    L for 0x68 - 0x6F
                let ld_hi = i.values.1 <= 0x07;
                let ld_value = self.get_reg_by_opcode(i.values.1);
                alu::load_8_bit_into_reg(&mut self.reg.hl, ld_hi, ld_value);
                self.curr_cycles = match i.values.1 {
                    0x06 | 0x0E => 8,
                    _ => 4,
                };
            }
            0x76 => {
                // HALT
                if self.ime {
                    // Since ime is enabled interrupts will be serviced once we exit
                    // Also possible to get haltbug if we just executed EI
                    self.is_running = false;
                    if self.ime_flipped {
                        self.emulate_haltbug(); // EI followed by HALT
                    }
                } else {
                    if !self.bus.interrupt_pending() {
                        // When the interrupts becomes pending we wont service them
                        self.is_running = false;
                    } else {
                        // Dont enter halt and haltbug occurs
                        self.is_running = true;
                        self.emulate_haltbug();
                    }
                }
            }
            0x70..=0x75 | 0x77 => {
                // LD (HL), R
                let ld_value = self.get_reg_by_opcode(i.values.1);
                self.write_byte(self.reg.hl, ld_value);
            }
            0x78..=0x7F => {
                // LD A, R
                let ld_value = self.get_reg_by_opcode(i.values.1);
                alu::load_8_bit_into_reg(&mut self.reg.af, true, ld_value);
            }
            0x80..=0x87 => {
                // A = A ADD R
                let add_value = self.get_reg_by_opcode(i.values.1);
                alu::a_add_r(&mut self.reg.af, add_value);
            }
            0x88..=0x8F => {
                // A = A ADC R
                let adc_value = self.get_reg_by_opcode(i.values.1);
                alu::a_adc_r(&mut self.reg.af, adc_value);
            }
            0x90..=0x97 => {
                // A = A SUB R
                let sub_value = self.get_reg_by_opcode(i.values.1);
                alu::a_sub_r(&mut self.reg.af, sub_value);
            }
            0x98..=0x9F => {
                // A = A SBC R
                let sbc_value = self.get_reg_by_opcode(i.values.1);
                alu::a_sbc_r(&mut self.reg.af, sbc_value);
            }
            0xA0..=0xA7 => {
                // A = A AND R
                let and_value = self.get_reg_by_opcode(i.values.1);
                alu::a_and_r(&mut self.reg.af, and_value);
            }
            0xA8..=0xAF => {
                // A = A XOR R
                let xor_value = self.get_reg_by_opcode(i.values.1);
                alu::a_xor_r(&mut self.reg.af, xor_value);
            }
            0xB0..=0xB7 => {
                // A = A OR R
                let or_value = self.get_reg_by_opcode(i.values.1);
                alu::a_or_r(&mut self.reg.af, or_value);
            }
            0xB8..=0xBF => {
                // A CP R (just update flags, dont store result)
                let cp_value = self.get_reg_by_opcode(i.values.1);
                alu::a_cp_r(&mut self.reg.af, cp_value);
            }
            0xC0 | 0xD0 | 0xC8 | 0xD8 => {
                // RET NZ/NC/C/Z
                let eval_cond = match i.values {
                    (0x0C, 0x00) => !self.reg.get_z(),
                    (0x0D, 0x00) => !self.reg.get_c(),
                    (0x0C, 0x08) => self.reg.get_z(),
                    (0x0D, 0x08) => self.reg.get_c(),
                    _ => panic!(
                        "Valid: 0xC0, 0xD0, 0xC8, 0xD8, Current: {:#04X}, {:#04X}",
                        i.values.0, i.values.1
                    ),
                };

                self.internal_cycle();
                if eval_cond {
                    let value = self.stack_pop();
                    self.internal_cycle();
                    self.pc = value;
                }
            }
            0xC9 | 0xD9 => {
                // RET(I)
                self.pc = self.stack_pop();
                self.internal_cycle();

                if i.values.0 == 0x0D {
                    // https://gekkio.fi/files/gb-docs/gbctr.pdf does ime_scheduled=1 for IE but ime=1
                    // for RETI implying there is a difference where RETI immedietely handles interrupts
                    self.ime = true
                }
            }
            0xC2 | 0xD2 | 0xCA | 0xDA | 0xC3 => {
                // JP X, a16
                let (hi, lo) = self.read_long();
                let eval_cond = match i.values {
                    (0x0C, 0x02) => !self.reg.get_z(),
                    (0x0D, 0x02) => !self.reg.get_c(),
                    (0x0C, 0x03) => true,
                    (0x0C, 0x0A) => self.reg.get_z(),
                    (0x0D, 0x0A) => self.reg.get_c(),
                    _ => panic!(
                        "Valid: 0xC2, 0xD2, 0xCA, 0xDA, 0xC3 Current: {:#04X}, {:#04X}",
                        i.values.0, i.values.1
                    ),
                };
                if eval_cond {
                    self.internal_cycle();
                    self.pc = alu::combine_bytes(hi, lo);
                }
            }
            0xE9 => {
                // JP (HL) but really JP HL
                self.pc = self.reg.hl;
            }
            0xC4 | 0xD4 | 0xCC | 0xDC | 0xCD => {
                // CALL X, a16
                let (hi, lo) = self.read_long();
                let eval_cond = match i.values {
                    (0x0C, 0x04) => !self.reg.get_z(),
                    (0x0D, 0x04) => !self.reg.get_c(),
                    (0x0C, 0x0D) => true,
                    (0x0C, 0x0C) => self.reg.get_z(),
                    (0x0D, 0x0C) => self.reg.get_c(),
                    _ => panic!(
                        "Valid: 0xC4, 0xD4, 0xCC, 0xDC, 0xCD Current: {:#04X}, {:#04X}",
                        i.values.0, i.values.1
                    ),
                };
                if eval_cond {
                    self.internal_cycle();
                    self.stack_push(self.pc);
                    self.pc = alu::combine_bytes(hi, lo);
                }
            }
            0xC7 | 0xD7 | 0xE7 | 0xF7 | 0xCF | 0xDF | 0xEF | 0xFF => {
                // RST XXH
                self.internal_cycle();
                self.stack_push(self.pc);
                self.pc =
                    0x0000 | u16::from((i.values.0 - 0x0C) << 4) | u16::from(i.values.1 - 0x07);
            }
            0xC1 | 0xD1 | 0xE1 | 0xF1 => {
                // POP
                match i.values.0 {
                    0x0C => self.reg.bc = self.stack_pop(),
                    0x0D => self.reg.de = self.stack_pop(),
                    0x0E => self.reg.hl = self.stack_pop(),
                    0x0F => self.reg.af = self.stack_pop(),
                    _ => panic!(
                        "Valid: 0xC1, D1, E1, F1, Current: {:#04X}, {:#04X}",
                        i.values.0, i.values.1
                    ),
                }
                self.reg.af = self.reg.af & 0xFFF0;
            }
            0xC5 | 0xD5 | 0xE5 | 0xF5 => {
                // PUSH
                self.internal_cycle();
                match i.values.0 {
                    0x0C => self.stack_push(self.reg.bc),
                    0x0D => self.stack_push(self.reg.de),
                    0x0E => self.stack_push(self.reg.hl),
                    0x0F => self.stack_push(self.reg.af),
                    _ => panic!(
                        "Valid: 0xC5, D5, E5, F5 Current: {:#04X}, {:#04X}",
                        i.values.0, i.values.1
                    ),
                };
            }
            0xC6 | 0xD6 | 0xE6 | 0xF6 => {
                let d8 = self.read_byte();
                match i.values.0 {
                    0x0C => alu::a_add_r(&mut self.reg.af, d8),
                    0x0D => alu::a_sub_r(&mut self.reg.af, d8),
                    0x0E => alu::a_and_r(&mut self.reg.af, d8),
                    0x0F => alu::a_or_r(&mut self.reg.af, d8),
                    _ => panic!(
                        "Valid: 0xC6, D6, E6, F6 Current: {:#04X}, {:#04X}",
                        i.values.0, i.values.1
                    ),
                }
            }
            0xCE | 0xDE | 0xEE | 0xFE => {
                let d8 = self.read_byte();
                match i.values.0 {
                    0x0C => alu::a_adc_r(&mut self.reg.af, d8),
                    0x0D => alu::a_sbc_r(&mut self.reg.af, d8),
                    0x0E => alu::a_xor_r(&mut self.reg.af, d8),
                    0x0F => alu::a_cp_r(&mut self.reg.af, d8),
                    _ => panic!(
                        "Valid: 0xCE, DE, EE, FE Current: {:#04X}, {:#04X}",
                        i.values.0, i.values.1
                    ),
                }
            }
            0xE0 | 0xF0 => {
                // Read and Write to IO Ports
                let offset = self.read_byte();
                let location = u16::from(offset) + 0xFF00;
                if i.values.0 == 0x0E {
                    self.write_byte(location, Reg::get_hi(self.reg.af));
                } else {
                    let val = self.read_addr(location);
                    self.reg.af = Reg::set_hi(self.reg.af, val);
                }
            }
            0xE2 | 0xF2 => {
                // Read and Write to IO Ports
                let offset = Reg::get_lo(self.reg.bc);
                let location = offset as u16 + 0xFF00;
                if i.values.0 == 0x0E {
                    self.write_byte(location, Reg::get_hi(self.reg.af));
                } else {
                    let val = self.read_addr(location);
                    self.reg.af = Reg::set_hi(self.reg.af, val);
                }
            }
            0xEA | 0xFA => {
                //ld (nn), A     and     ld A, (nn)
                let (hi, lo) = self.read_long();
                let location = alu::combine_bytes(hi, lo);
                if i.values.0 == 0x0E {
                    self.write_byte(location, Reg::get_hi(self.reg.af));
                }
                if i.values.0 == 0x0F {
                    let val = self.read_addr(location);
                    self.reg.af = Reg::set_hi(self.reg.af, val);
                }
            }
            0xE8 => {
                let byte = self.read_byte();
                let (result, new_af) = alu::sp_add_dd(self.sp, byte, self.reg.af);
                self.reg.af = new_af;

                self.internal_cycle();
                self.internal_cycle();
                self.sp = result;
            }
            0xF3 => {
                // DI
                self.ime = false;
            }
            0xFB => {
                // EI
                self.ime_scheduled = true;
            }
            0xF8 => {
                let byte = self.read_byte();
                let (result, new_af) = alu::sp_add_dd(self.sp, byte, self.reg.af);
                self.reg.af = new_af;
                self.reg.hl = result;
                self.internal_cycle();
            }
            0xF9 => {
                self.sp = self.reg.hl;
                self.internal_cycle();
            }
            _ => panic!("Opcode not supported"),
        } // End of match statement
    } // match instruction function

    fn match_cb_instruction(self: &mut Self, i: Instruction) {
        // https://meganesulli.com/generate-gb-opcodes/
        match i.opcode {
            0x00..=0x3F => {
                let reg = self.get_reg_by_opcode(i.values.1);

                let result: u8 = match i.opcode {
                    0x00..=0x07 => alu::rlc(reg, &mut self.reg.af), /* RLC */
                    0x08..=0x0F => alu::rrc(reg, &mut self.reg.af), /* RRC */
                    0x10..=0x17 => alu::rl(reg, self.reg.get_c(), &mut self.reg.af), /* RL  */
                    0x18..=0x1F => alu::rr(reg, self.reg.get_c(), &mut self.reg.af), /* RR  */
                    0x20..=0x27 => alu::sla(reg, &mut self.reg.af), /* SLA */
                    0x28..=0x2F => alu::sra(reg, &mut self.reg.af), /* SRA */
                    0x30..=0x37 => alu::swap(reg, &mut self.reg.af), /* SWAP */
                    0x38..=0x3F => alu::srl(reg, &mut self.reg.af), /* SRL */
                    _ => panic!("Should not be possible #1"),
                };

                self.write_reg_by_opcode(i.values.1, result);
            }

            0x40..=0x7F => {
                let reg = self.get_reg_by_opcode(i.values.1);

                match i.opcode {
                    0x40..=0x47 => alu::bit(reg, 0, &mut self.reg.af), /* BIT 0 */
                    0x48..=0x4F => alu::bit(reg, 1, &mut self.reg.af), /* BIT 1 */
                    0x50..=0x57 => alu::bit(reg, 2, &mut self.reg.af), /* BIT 2 */
                    0x58..=0x5F => alu::bit(reg, 3, &mut self.reg.af), /* BIT 3 */
                    0x60..=0x67 => alu::bit(reg, 4, &mut self.reg.af), /* BIT 4 */
                    0x68..=0x6F => alu::bit(reg, 5, &mut self.reg.af), /* BIT 5 */
                    0x70..=0x77 => alu::bit(reg, 6, &mut self.reg.af), /* BIT 6 */
                    0x78..=0x7F => alu::bit(reg, 7, &mut self.reg.af), /* BIT 7 */
                    _ => panic!("Should not be possible #2"),
                }
            }

            0x80..=0xBF => {
                let reg = self.get_reg_by_opcode(i.values.1);

                let reset = match i.opcode {
                    0x80..=0x87 => alu::res(reg, 0), /* RES 0 */
                    0x88..=0x8F => alu::res(reg, 1), /* RES 1 */
                    0x90..=0x97 => alu::res(reg, 2), /* RES 2 */
                    0x98..=0x9F => alu::res(reg, 3), /* RES 3 */
                    0xA0..=0xA7 => alu::res(reg, 4), /* RES 4 */
                    0xA8..=0xAF => alu::res(reg, 5), /* RES 5 */
                    0xB0..=0xB7 => alu::res(reg, 6), /* RES 6 */
                    0xB8..=0xBF => alu::res(reg, 7), /* RES 7 */
                    _ => panic!("Should not be possible #3"),
                };

                self.write_reg_by_opcode(i.values.1, reset);
            }

            0xC0..=0xFF => {
                let reg = self.get_reg_by_opcode(i.values.1);

                let set = match i.opcode {
                    0xC0..=0xC7 => alu::set(reg, 0), /* SET 0 */
                    0xC8..=0xCF => alu::set(reg, 1), /* SET 1 */
                    0xD0..=0xD7 => alu::set(reg, 2), /* SET 2 */
                    0xD8..=0xDF => alu::set(reg, 3), /* SET 3 */
                    0xE0..=0xE7 => alu::set(reg, 4), /* SET 4 */
                    0xE8..=0xEF => alu::set(reg, 5), /* SET 5 */
                    0xF0..=0xF7 => alu::set(reg, 6), /* SET 6 */
                    0xF8..=0xFF => alu::set(reg, 7), /* SET 7 */
                    _ => panic!("Should not be possible #4"),
                };

                self.write_reg_by_opcode(i.values.1, set);
            }
        }
    }

    fn read_hl(self: &mut Self) -> u8 {
        return self.read_addr(self.reg.hl);
    }

    fn read_sp(self: &mut Self) -> u8 {
        return self.read_addr(self.sp);
    }

    // Instructions that are 3 bytes long will call this method to get the next two bytes required
    // gameboy is little endian so the second byte is actually supposed to the higher order bits
    fn read_long(self: &mut Self) -> (u8, u8) {
        let lo = self.read_pc();
        let hi = self.read_pc();
        return (hi, lo);
    }

    // Instructions that are 2 bytes long will call this method to get the next byte required
    fn read_byte(self: &mut Self) -> u8 {
        return self.read_pc();
    }

    fn read_pc(self: &mut Self) -> u8 {
        let byte = self.read_addr(self.pc);
        self.pc = self.pc + 1;
        return byte;
    }

    fn read_addr(self: &mut Self, addr: u16) -> u8 {
        let byte = self.bus.read_byte(addr);
        self.adv_cycles(4);
        self.curr_cycles += 4;
        return byte;
    }

    fn write_long(self: &mut Self, addr: u16, reg: u16) {
        self.write_byte(addr, Reg::get_lo(reg));
        self.write_byte(addr + 1, Reg::get_hi(reg));
    }

    fn write_byte(self: &mut Self, addr: u16, data: u8) {
        self.bus.write_byte(addr, data);
        self.adv_cycles(4);
        self.curr_cycles += 4;
    }

    fn stack_push(self: &mut Self, value: u16) {
        self.sp = self.sp.wrapping_sub(1);
        self.write_byte(self.sp, Reg::get_hi(value));

        self.sp = self.sp.wrapping_sub(1);
        self.write_byte(self.sp, Reg::get_lo(value));
    }

    fn stack_pop(self: &mut Self) -> u16 {
        let lo = self.read_sp();
        self.sp = self.sp.wrapping_add(1);
        let hi = self.read_sp();
        self.sp = self.sp.wrapping_add(1);

        return alu::combine_bytes(hi, lo);
    }

    // ??????????????????????????????????????
    // ??????????????????????????????????????
    // ??????????????????????????????????????
    pub fn internal_cycle(self: &mut Self) {
        self.adv_cycles(4);
        self.curr_cycles += 4;
    }
    // ??????????????????????????????????????
    // ??????????????????????????????????????
    // ??????????????????????????????????????

    // Takes the lower 8 bits of the opcode and returns the value of the register
    fn get_reg_by_opcode(self: &mut Self, opcode_lo: u8) -> u8 {
        return match opcode_lo {
            0x00 | 0x08 => Reg::get_hi(self.reg.bc),
            0x01 | 0x09 => Reg::get_lo(self.reg.bc),
            0x02 | 0x0A => Reg::get_hi(self.reg.de),
            0x03 | 0x0B => Reg::get_lo(self.reg.de),
            0x04 | 0x0C => Reg::get_hi(self.reg.hl),
            0x05 | 0x0D => Reg::get_lo(self.reg.hl),
            0x06 | 0x0E => self.read_hl(),
            0x07 | 0x0F => Reg::get_hi(self.reg.af),
            _ => panic!("Expected Value between 0x00 and 0x0F"),
        };
    }

    // Takes the lower 8 bits of the opcode and writes a value to the corresponding register
    fn write_reg_by_opcode(self: &mut Self, opcode_lo: u8, val: u8) {
        match opcode_lo {
            0x00 | 0x08 => self.reg.bc = Reg::set_hi(self.reg.bc, val),
            0x01 | 0x09 => self.reg.bc = Reg::set_lo(self.reg.bc, val),
            0x02 | 0x0A => self.reg.de = Reg::set_hi(self.reg.de, val),
            0x03 | 0x0B => self.reg.de = Reg::set_lo(self.reg.de, val),
            0x04 | 0x0C => self.reg.hl = Reg::set_hi(self.reg.hl, val),
            0x05 | 0x0D => self.reg.hl = Reg::set_lo(self.reg.hl, val),
            0x06 | 0x0E => self.write_byte(self.reg.hl, val),
            0x07 | 0x0F => self.reg.af = Reg::set_hi(self.reg.af, val),
            _ => panic!("Expected Value between 0x00 and 0x0F"),
        }
    }

    fn get_mut_reg16_by_opcode(self: &mut Self, opcode_hi: u8) -> &mut u16 {
        return match opcode_hi {
            0x00 => &mut self.reg.bc,
            0x01 => &mut self.reg.de,
            0x02 => &mut self.reg.hl,
            0x03 => &mut self.sp,
            _ => panic!(
                "Expected Value between 0x00 and 0x03, got {:#04X}",
                opcode_hi
            ),
        };
    }

    fn get_reg16_by_opcode(self: &Self, opcode_hi: u8) -> u16 {
        return match opcode_hi {
            0x00 => self.reg.bc,
            0x01 => self.reg.de,
            0x02 => self.reg.hl,
            0x03 => self.sp,
            _ => panic!(
                "Expected Value between 0x00 and 0x03, got {:#04X}",
                opcode_hi
            ),
        };
    }

    pub fn get_memory(self: &Self) -> &Memory {
        return self.bus.get_mem();
    }

    #[cfg(feature = "debug")]
    pub fn get_bus_mut(self: &mut Self) -> &mut Bus {
        return &mut self.bus;
    }

    #[cfg(feature = "debug")]
    pub fn display_tiles(self: &mut Self, texture: &mut Texture) {
        self.bus.display_tiles(texture);
    }
} // Impl CPU

#[cfg(test)]
#[path = "./tests/cpu_tests.rs"]
mod cpu_tests;

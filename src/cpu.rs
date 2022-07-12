use super::instruction::Instruction;
use super::mbc::Mbc;
use super::memory::Memory;
use super::timer::Timer;
use super::alu;

use Registers as Reg;

pub struct Cpu {
    mem: Memory,
    timer: Timer,  
    reg: Registers,
    pc: u16,                // Program Counter
    sp: u16,                // Stack Pointer
    pub curr_cycles: usize, // The number of cycles the current instruction should take to execute
    ime: bool,
    ime_scheduled: bool,
    ime_flipped: bool, // Just tells us that the previous instruction was an EI (For haltbug)(Set up to not apply for reti)
    pub is_running: bool,
    pub haltbug: bool,
    first_halt_cycle: bool,
}

impl Cpu {
    pub fn new() -> Cpu {
        return Cpu {
            mem: Memory::new(),
            timer: Timer::new(),
            reg: Registers::new(),
            pc: 0x0100,
            sp: 0,
            curr_cycles: 0,
            ime: false,
            ime_scheduled: false,
            is_running: true,   // Controlled by halt
            ime_flipped: false,
            haltbug: false,
            first_halt_cycle: false,
        };
    }

    pub fn dmg_init(self: &mut Self, checksum: u8) {
        self.reg.dmg_init(checksum);
        self.mem.dmg_init();
        self.sp = 0xFFFE;
    }
    
    pub fn set_mbc(self: &mut Self, cart_mbc: Box<dyn Mbc>) {
        self.mem.set_mbc(cart_mbc);
    }

    // pub fn load_game(self: &mut Self, game_bytes: Vec<u8>) {
    //     self.mem.load_game(game_bytes);
    // }

    pub fn execute(self: &mut Self) {
        if self.ime_scheduled == true {
            self.ime_scheduled = false;
            self.ime = true; // Now interrupts should occur delayed one instruction
            self.ime_flipped = true;
        } else {
            self.ime_flipped = false;
        }

        let opcode = self.read_pc(); // Instruction Fetch
        let i = Instruction::get_instruction(opcode);

        self.emulate_haltbug();

        if i.values == (0x0C, 0x0B) {   
            let opcode = self.read_pc();
            let cb_i = Instruction::get_instruction(opcode);
            self.match_cb_instruction(cb_i);
        } else {
            self.match_instruction(i);
        }
        
        println!("Opcode: {:#04X} | cycles: {}", opcode, self.curr_cycles);
        println!("sp: {:#06X} | pc: {:#06X} | AF: {:#06X} | BC: {:#06X} | DE: {:#06X} | HL: {:#06X}", 
            self.sp, self.pc, self.reg.af, self.reg.bc, self.reg.de, self.reg.hl);
        println!("sb_reg: {}", {self.mem.read_byte(0xFF01)});
        println!("------------------------------");

        if self.reg.bc == 0x0101 && self.reg.bc == 0xFFFF && self.reg.hl == 0xCE46 {
            println!("nice");
        }

    }

    // The user writes to IE and the CPU is supposed to set/unset IF
    // In memory, do I check if FFFF is being written to and then also write
    // to FF0F or os that handled by the ROM?
    // NEEDS A TEST
    fn handle_interrupt(&mut self) {
        let i_enable = self.mem.read_byte(0xFFFF);
        let mut i_fired = self.mem.read_byte(0xFF0F);

        for i in 0..=4 {
            if i_enable & i_fired & (0x01 << i) == (0x01 << i) {
                self.timer.reset_clock();

                self.ime = false;
                i_fired = i_fired & !(0x01 << i);   // https://www.reddit.com/r/EmuDev/comments/u9itc2/problem_with_halt_gameboy_and_dr_mario/
                self.mem.write_byte(0xFF0F, i_fired);

                /* If we have the haltbug decrement the pc so that we return
                to the HALT instruction after the interrupt is serviced */
                self.emulate_haltbug();
                self.stack_push(self.pc);

                // Set PC based on corresponding interrupt vector
                self.pc = 0x0040 + (0x0008 * i);

                self.curr_cycles = 20;
                self.handle_clocks();
                break; // Only handle the highest priority interrupt
            }
        }
    }

    pub fn handle_clocks(self: &mut Self) {
        let io = self.mem.get_io_mut();
        self.timer.handle_clocks(io, self.curr_cycles);
    }
    pub fn reset_clock(self: &mut Self) {
        self.timer.reset_clock();
    }

    fn emulate_haltbug(self: &mut Self) {
        if self.haltbug {
            // Ensure that the byte is read twice
            self.haltbug = false;
            self.pc = self.pc.wrapping_sub(1);
        }
    }

    // Most likely place for errors
    pub fn check_interrupts(self: &mut Self) {
        if self.ime {
            if !self.is_running && self.ime_flipped && self.mem.interrupt_pending() {
                // EI was followed by a HALT (service interrupt and then return to halt state)
                // When the interrupt returns to the HALT instruction it will execute the halt and thus set self.is_running = false
                self.haltbug = true;
            }
            if self.mem.interrupt_pending() {
                self.is_running = true;
                self.handle_interrupt();
            }
        } else {
            if !self.is_running && self.first_halt_cycle && self.mem.interrupt_pending() {
                // Interrupt pending with IME not set and halt instruction is FIRST executed
                // https://gbdev.io/pandocs/halt.html
                self.is_running = true;
                self.haltbug = true;
            }
            if !self.is_running && self.mem.interrupt_pending() {
                // Interrupt pending so we can resume. Not handled though since ime=0
                self.is_running = true;
            }
            // else no interrupt pending and ime not enabled so just continue in halted state
            
        }
        self.first_halt_cycle = false;  // so that we dont get the haltbug simply for going into halt with no ime
    }

    pub fn update_input(self: &mut Self) {
        // ???
        // let input = 0;
        // while input != 1 {}
    }

    fn match_instruction(self: &mut Self, i: Instruction) {
        // Create a method for every instruction
        match i.opcode {
            0x00 => {
                // NOP
                self.curr_cycles = 4;
            }
            0x10 => {
                // STOP
                // Stop instruction is followed by addition byte (usually 0) that is ignored by the cpu
                // No licensed rom makes use of STOP outside of CGB speed switching.
                /*  
                    self.timer.start_stop(&mut self.mem);
                    self.pc = self.pc.wrapping_add(1);
                    self.curr_cycles = 4; 
                */
                panic!("No licensed rom makes use of STOP outside of CGB speed switching");
            }
            0x20 | 0x30 | 0x18 | 0x28 | 0x38 => {
                // JR NZ/NC/C/Z, r8 (r8 is added the pc and the pc
                // should have been incremented during its reads) NEEDS TESTS
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
                    self.curr_cycles = 12;
                    let (result, _) = alu::reg_add_8bit_signed(self.pc, r8);
                    self.pc = result;
                } else {
                    self.curr_cycles = 8;
                }
            }
            0x01 | 0x11 | 0x21 | 0x31 => {
                // Load 16 bit immediate into BC/DE/HL/SP
                let (hi, lo) = self.read_long();
                let register = self.get_mut_reg16_by_opcode(i.values.0);
                alu::load_d16(register, hi, lo);
                self.curr_cycles = 12;
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
                self.mem.write_byte(location, str_val_a);
                self.curr_cycles = 8;
            }
            0x03 | 0x13 | 0x23 | 0x33 => {
                // INC BC/DE/HL/SP
                let register = self.get_mut_reg16_by_opcode(i.values.0);
                alu::post_incr(register);
                self.curr_cycles = 8;
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
                self.curr_cycles = 4;
            }
            0x34 | 0x35 => {
                // 8 Bit increment and decrement for (hl)
                let val_at_hl = self.mem.read_byte(self.reg.hl);
                let result = if i.values.1 == 0x04 {
                    alu::incr_8bit(val_at_hl, &mut self.reg.af)
                } else {
                    alu::decr_8bit(val_at_hl, &mut self.reg.af)
                };
                self.mem.write_byte(self.reg.hl, result);
                self.curr_cycles = 12;
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
                self.curr_cycles = 4;
            }
            0x06 | 0x16 | 0x26 => {
                // LD B/D/H, d8
                let ld_value = self.read_byte();
                let register = self.get_mut_reg16_by_opcode(i.values.0);
                alu::load_imm_d8(register, ld_value, true);
                self.curr_cycles = 8;
            }
            0x36 => {
                // LD (HL), d8
                let ld_value = self.read_byte();
                self.mem.write_byte(self.reg.hl, ld_value);
                self.curr_cycles = 12;
            }
            0x07 | 0x17 => {
                // RLCA and RLA
                alu::rotate_left_a(i.values.0 == 1, &mut self.reg);
                self.curr_cycles = 4;
            }
            0x0F | 0x1F => {
                // RRCA and RRA
                alu::rotate_right_a(i.values.0 == 1, &mut self.reg);
                self.curr_cycles = 4;
            }
            0x27 => {
                // DAA
                self.reg.af = alu::daa(&self.reg);
                self.curr_cycles = 4;
            }
            0x2F => {
                // CPL
                self.reg.af = alu::cpl(self.reg.af);
                self.curr_cycles = 4;
            }
            0x37 => {
                // SCF
                self.reg.af = alu::scf(self.reg.af);
                self.curr_cycles = 4;
            }
            0x3F => {
                // CCF
                self.reg.af = alu::ccf(self.reg.af);
                self.curr_cycles = 4;
            }
            0x08 => {
                // LD (a16), SP
                let (hi, lo) = self.read_long();
                let imm16 = alu::combine_bytes(hi, lo);
                self.write_reg(imm16, self.sp);
                self.curr_cycles = 20;
            }
            0x09 | 0x19 | 0x29 | 0x39 => {
                // EX: ADD HL RR
                let add_value = self.get_reg16_by_opcode(i.values.0);
                alu::hl_add_rr(&mut self.reg.hl, add_value, &mut self.reg.af);
                self.curr_cycles = 8;
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
                let new_a_val = self.mem.read_byte(location);
                self.reg.af = Reg::set_hi(self.reg.af, new_a_val);
                self.curr_cycles = 8;
            }
            0x0B | 0x1B | 0x2B | 0x3B => {
                // DEC BC/DE/HL/SP
                let register = self.get_mut_reg16_by_opcode(i.values.0);
                alu::post_decr(register);
                self.curr_cycles = 8;
            }
            0x0E | 0x1E | 0x2E => {
                // LD C/E/L, d8
                let ld_value = self.read_byte();
                let register = self.get_mut_reg16_by_opcode(i.values.0);
                alu::load_imm_d8(register, ld_value, false);
                self.curr_cycles = 8;
            }
            0x3E => {
                // LD A, d8
                let ld_value = self.read_byte();
                alu::load_imm_d8(&mut self.reg.af, ld_value, true);
                self.curr_cycles = 8;
            }
            0x40..=0x4F => {
                // LD B/C, R
                // B for 0x40 - 0x47    C for 0x48 - 0x4F
                let ld_hi = i.values.1 <= 0x07;
                let ld_value = self.get_reg_by_opcode(i.values.1);
                alu::load_8_bit_into_reg(&mut self.reg.bc, ld_hi, ld_value);
                self.curr_cycles = match i.values.1 { 0x06 | 0x0E => 8, _ => 4 };
                // self.curr_cycles = num_cycles_8bit_arithmetic_loads(i.values.1);
            }
            0x50..=0x5F => {
                // LD D/E, R
                // D for 0x50 - 0x57    E for 0x58 - 0x5F
                let ld_hi = i.values.1 <= 0x07;
                let ld_value = self.get_reg_by_opcode(i.values.1);
                alu::load_8_bit_into_reg(&mut self.reg.de, ld_hi, ld_value);
                self.curr_cycles = match i.values.1 { 0x06 | 0x0E => 8, _ => 4 };
                // self.curr_cycles = num_cycles_8bit_arithmetic_loads(i.values.1);
            }
            0x60..=0x6F => {
                // LD H/L, R
                // H for 0x60 - 0x67    L for 0x68 - 0x6F
                let ld_hi = i.values.1 <= 0x07;
                let ld_value = self.get_reg_by_opcode(i.values.1);
                alu::load_8_bit_into_reg(&mut self.reg.hl, ld_hi, ld_value);
                self.curr_cycles = match i.values.1 { 0x06 | 0x0E => 8, _ => 4 };
                // self.curr_cycles = num_cycles_8bit_arithmetic_loads(i.values.1);
            }
            0x76 => {
                // HALT
                // Gameboy stops executing instructions until, an interrupt occurs
                // ISR is serviced and we continue execution from the next address
                // If IME=0, the ISR is not serviced and execution continues after
                // http://www.devrs.com/gb/files/gbspec.txt
                self.is_running = false;
                self.first_halt_cycle = true;
                self.curr_cycles = 4;
            }
            0x70..=0x75 | 0x77 => {
                // LD (HL), R
                let ld_value = self.get_reg_by_opcode(i.values.1);
                self.mem.write_byte(self.reg.hl, ld_value);
                self.curr_cycles = 8;
            }
            0x78..=0x7F => {
                // LD A, R
                let ld_value = self.get_reg_by_opcode(i.values.1);
                alu::load_8_bit_into_reg(&mut self.reg.af, true, ld_value);
                self.curr_cycles = match i.values.1 { 0x06 | 0x0E => 8, _ => 4 };
            }
            0x80..=0x87 => {
                // A = A ADD R
                let add_value = self.get_reg_by_opcode(i.values.1);
                alu::a_add_r(&mut self.reg.af, add_value);
                self.curr_cycles = match i.values.1 { 0x06 | 0x0E => 8, _ => 4 };
            }
            0x88..=0x8F => {
                // A = A ADC R
                let adc_value = self.get_reg_by_opcode(i.values.1);
                alu::a_adc_r(&mut self.reg.af, adc_value);
                self.curr_cycles = match i.values.1 { 0x06 | 0x0E => 8, _ => 4 };
            }
            0x90..=0x97 => {
                // A = A SUB R
                let sub_value = self.get_reg_by_opcode(i.values.1);
                alu::a_sub_r(&mut self.reg.af, sub_value);
                self.curr_cycles = match i.values.1 { 0x06 | 0x0E => 8, _ => 4 };
            }
            0x98..=0x9F => {
                // A = A SBC R
                let sbc_value = self.get_reg_by_opcode(i.values.1);
                alu::a_sbc_r(&mut self.reg.af, sbc_value);
                self.curr_cycles = match i.values.1 { 0x06 | 0x0E => 8, _ => 4 };
            }
            0xA0..=0xA7 => {
                // A = A AND R
                let and_value = self.get_reg_by_opcode(i.values.1);
                alu::a_and_r(&mut self.reg.af, and_value);
                self.curr_cycles = match i.values.1 { 0x06 | 0x0E => 8, _ => 4 };
            }
            0xA8..=0xAF => {
                // A = A XOR R
                let xor_value = self.get_reg_by_opcode(i.values.1);
                alu::a_xor_r(&mut self.reg.af, xor_value);
                self.curr_cycles = match i.values.1 { 0x06 | 0x0E => 8, _ => 4 };
            }
            0xB0..=0xB7 => {
                // A = A OR R
                let or_value = self.get_reg_by_opcode(i.values.1);
                alu::a_or_r(&mut self.reg.af, or_value);
                self.curr_cycles = match i.values.1 { 0x06 | 0x0E => 8, _ => 4 };
            }
            0xB8..=0xBF => {
                // A CP R (just update flags, dont store result)
                let cp_value = self.get_reg_by_opcode(i.values.1);
                alu::a_cp_r(&mut self.reg.af, cp_value);
                self.curr_cycles = match i.values.1 { 0x06 | 0x0E => 8, _ => 4 };
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
                if eval_cond {
                    self.pc = self.stack_pop();
                    self.curr_cycles = 20;
                } else {
                    self.curr_cycles = 8;
                }
            }
            0xC9 | 0xD9 => {
                // RET(I)
                self.pc = self.stack_pop();
                self.curr_cycles = 16;
                if i.values.0 == 0x0D {
                    // https://gekkio.fi/files/gb-docs/gbctr.pdf does ime_scheduled=1 for IE but ime=1
                    // for RETI implying there is a difference where RETI immedietely handles interrupts
                    self.ime = true // enable interrupts (IME = 1)
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
                    self.pc = alu::combine_bytes(hi, lo);
                    self.curr_cycles = 16;
                } else {
                    self.curr_cycles = 12;
                }
            }
            0xE9 => {
                // JP (HL)
                /*
                    Sometimes written as JP [HL]. Misleading, since brackets are usually
                    to indicate memory reads. This instruction only copies the value.
                */
                self.pc = self.reg.hl;
                self.curr_cycles = 4;
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
                    self.stack_push(self.pc);
                    self.pc = alu::combine_bytes(hi, lo);
                    self.curr_cycles = 24;
                } else {
                    self.curr_cycles = 12;
                }
            }
            0xC7 | 0xD7 | 0xE7 | 0xF7 | 0xCF | 0xDF | 0xEF | 0xFF => {
                // RST XXH
                self.stack_push(self.pc);
                self.pc =
                    0x0000 | u16::from((i.values.0 - 0x0C) << 4) | u16::from(i.values.1 - 0x07);
                self.curr_cycles = 16;
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
                self.reg.af = self.reg.af & 0xFFF0; // Lower 4 bits of f should always be 0
                self.curr_cycles = 12;
            }
            0xC5 | 0xD5 | 0xE5 | 0xF5 => {
                // PUSH
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
                self.curr_cycles = 16;
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
                self.curr_cycles = 8;
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
                self.curr_cycles = 8;
            }
            0xE0 | 0xF0 => {
                // Read and Write to IO Ports
                let offset = self.read_byte();
                let location = u16::from(offset) + 0xFF00;
                if i.values.0 == 0x0E {
                    self.mem.write_byte(location, Reg::get_hi(self.reg.af));
                } else {
                    self.reg.af = Reg::set_hi(self.reg.af, self.mem.read_byte(location));
                }
                self.curr_cycles = 12;
            }
            0xE2 | 0xF2 => {
                // Read and Write to IO Ports
                let offset = Reg::get_lo(self.reg.bc);
                let location = offset as u16 + 0xFF00;
                if i.values.0 == 0x0E {
                    self.mem
                        .write_byte(location, Reg::get_hi(self.reg.af));
                } else {
                    self.reg.af =
                    Reg::set_hi(self.reg.af, self.mem.read_byte(location));
                }
                self.curr_cycles = 8;
            }
            0xEA | 0xFA => {
                //ld (nn), A     and     ld A, (nn)
                let (hi, lo) = self.read_long();
                let location = alu::combine_bytes(hi, lo);
                if i.values.0 == 0x0E {
                    self.mem
                        .write_byte(location, Reg::get_hi(self.reg.af));
                }
                if i.values.0 == 0x0F {
                    self.reg.af =
                    Reg::set_hi(self.reg.af, self.mem.read_byte(location));
                }
                self.curr_cycles = 16;
            }
            0xE8 => {
                let byte = self.read_byte();
                let (result, new_af) = alu::sp_add_dd(self.sp, byte, self.reg.af);
                self.curr_cycles = 16;
                self.reg.af = new_af;
                self.sp = result;
            }
            0xF3 => {
                // DI
                self.curr_cycles = 4;
                self.ime = false;
            }
            0xFB => {
                // EI
                // However, one more instruction should execute before interrupt
                self.curr_cycles = 4;
                self.ime_scheduled = true;
            }
            0xF8 => {
                let byte = self.read_byte();
                let (result, new_af) = alu::sp_add_dd(self.sp, byte, self.reg.af);
                self.curr_cycles = 12;
                self.reg.af = new_af;
                self.reg.hl = result;
            }
            0xF9 => {
                self.sp = self.reg.hl;
                self.curr_cycles = 8;
            }
            _ => panic!("Opcode not supported"),
        } // End of match statement
    } // match instruction function

    fn match_cb_instruction(self: &mut Self, i: Instruction) {
        // https://meganesulli.com/generate-gb-opcodes/
        match i.opcode {
            0x00..=0x07 => {
                /* RLC */
                let reg = self.get_reg_by_opcode(i.values.1);
                let rotated = alu::rlc(reg, &mut self.reg.af);
                self.write_reg_by_opcode(i.values.1, rotated);
            },
            0x08..=0x0F => {
                /* RRC */
                let reg = self.get_reg_by_opcode(i.values.1);
                let rotated = alu::rrc(reg, &mut self.reg.af);
                self.write_reg_by_opcode(i.values.1, rotated);
            },
            0x10..=0x17 => {/* RL */},
            0x18..=0x1F => {/* RR */},
            0x20..=0x27 => {/* SLA */},
            0x28..=0x2F => {/* SRA */},
            0x30..=0x37 => {/* SWAP */},
            0x38..=0x3F => {/* SRL */},
            0x40..=0x47 => {/* BIT 0 */},
            0x48..=0x4F => {/* BIT 1 */},
            0x50..=0x57 => {/* BIT 2 */},
            0x58..=0x5F => {/* BIT 3 */},
            0x60..=0x67 => {/* BIT 4 */},
            0x68..=0x6F => {/* BIT 5 */},
            0x70..=0x77 => {/* BIT 6 */},
            0x78..=0x7F => {/* BIT 7 */},
            0x80..=0x87 => {/* RES 0 */},
            0x88..=0x8F => {/* RES 1 */},
            0x90..=0x97 => {/* RES 2 */},
            0x98..=0x9F => {/* RES 3 */},
            0xA0..=0xA7 => {/* RES 4 */},
            0xA8..=0xAF => {/* RES 5 */},
            0xB0..=0xB7 => {/* RES 6 */},
            0xB8..=0xBF => {/* RES 7 */},
            0xC0..=0xC7 => {/* SET 0 */},
            0xC8..=0xCF => {/* SET 1 */},
            0xD0..=0xD7 => {/* SET 2 */},
            0xD8..=0xDF => {/* SET 3 */},
            0xE0..=0xE7 => {/* SET 4 */},
            0xE8..=0xEF => {/* SET 5 */},
            0xF0..=0xF7 => {/* SET 6 */},
            0xF8..=0xFF => {/* SET 7 */},
        }
        panic!("Not Implemented");
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
        let byte = self.mem.read_byte(self.pc);
        self.pc = self.pc + 1;
        return byte;
    }

    fn write_reg(self: &mut Self, addr: u16, register: u16) {
        self.mem.write_byte(addr, Reg::get_lo(register));
        self.mem.write_byte(addr + 1, Reg::get_hi(register));
    }

    fn stack_push(self: &mut Self, value: u16) {
        self.sp = self.sp.wrapping_sub(2);
        self.write_reg(self.sp, value);
    }
    fn stack_pop(self: &mut Self) -> u16 {
        let lo = self.mem.read_byte(self.sp);
        let hi = self.mem.read_byte(self.sp + 1);
        self.sp = self.sp.wrapping_add(2);
        return alu::combine_bytes(hi, lo);
    }

    // Takes the lower 8 bits of the opcode and returns the value of the register
    fn get_reg_by_opcode(self: &Self, opcode_lo: u8) -> u8 {
        return match opcode_lo {
            0x00 | 0x08 => Reg::get_hi_lo(self.reg.bc).0,
            0x01 | 0x09 => Reg::get_hi_lo(self.reg.bc).1,
            0x02 | 0x0A => Reg::get_hi_lo(self.reg.de).0,
            0x03 | 0x0B => Reg::get_hi_lo(self.reg.de).1,
            0x04 | 0x0C => Reg::get_hi_lo(self.reg.hl).0,
            0x05 | 0x0D => Reg::get_hi_lo(self.reg.hl).1,
            0x06 | 0x0E => self.mem.read_byte(self.reg.hl),
            0x07 | 0x0F => Reg::get_hi_lo(self.reg.af).0,
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
            0x06 | 0x0E => self.mem.write_byte(self.reg.hl, val),
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
        return &self.mem;
    }

    #[cfg(feature = "debug")]
    pub fn get_memory_mut(self: &mut Self) -> &mut Memory {
        return &mut self.mem;
    }

} // Impl CPU

// Each one may also be addressed as just the upper or lower 8 bits
pub struct Registers {
    pub af: u16, // A: accumulator, F: flags as 0bZNHC0000
    pub bc: u16,
    pub de: u16,
    pub hl: u16,
}

impl Registers {
    fn new() -> Registers {
        return Registers {
            af: 0x0000,
            bc: 0x0000,
            de: 0x0000,
            hl: 0x0000,
        };
    }

    pub fn dmg_init(self: &mut Self, checksum: u8) {
        if checksum == 0x00 {
            self.af = 0x0180;
        } else {
            self.af = 0x01B0;
        }
        self.bc = 0x0013;
        self.de = 0x00D8;
        self.hl = 0x014D;
    } 

    // returns true if z is set
    pub fn get_z(self: &Self) -> bool {
        ((self.af & 0x0080) >> 7) == 1
    }
    // returns true if n is set
    pub fn get_n(self: &Self) -> bool {
        ((self.af & 0x0040) >> 6) == 1
    }
    // returns true if h is set
    pub fn get_h(self: &Self) -> bool {
        ((self.af & 0x0020) >> 5) == 1
    }
    // returns true if c is set
    pub fn get_c(self: &Self) -> bool {
        ((self.af & 0x0010) >> 4) == 1
    }
    // Registers are stored as big endian so its easier in my head
    // returns the given register as 2 u8s in a tuple as (High, Low)
    pub fn get_hi_lo(xy: u16) -> (u8, u8) {
        return ((xy >> 8) as u8, xy as u8);
    }
    pub fn get_hi(xy: u16) -> u8 {
        return (xy >> 8) as u8;
    }
    pub fn get_lo(xy: u16) -> u8 {
        return xy as u8;
    }

    pub fn set_hi(reg: u16, byte: u8) -> u16 {
        let mut new_reg = reg & 0x00FF;
        new_reg = new_reg | ((byte as u16) << 8);
        return new_reg;
    }
    pub fn set_lo(reg: u16, byte: u8) -> u16 {
        let mut new_reg = reg & 0xFF00;
        new_reg = new_reg | (byte as u16);
        return new_reg;
    }
}

#[cfg(test)]
#[path = "./tests/cpu_tests.rs"]
mod cpu_tests;

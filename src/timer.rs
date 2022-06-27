use crate::memory::{IF_REG, TIMA_REG};

use super::memory::Memory;
use std::time::Instant;

const DIV_PERIOD_NANOS: f64 = 61035.15;
const CPU_PERIOD_NANOS: f64 = 238.418579; 

pub struct Timer {
    prev_time: Instant,
    wait_time: f64,
    acc_div_cycles: u8,
    acc_tima_cycles: usize,
}

impl Timer {
    pub fn new() -> Timer {
        return Timer {
            prev_time: Instant::now(),
            wait_time: 0.0,
            acc_div_cycles: 0,
            acc_tima_cycles: 0,
        }
    }

    pub fn reset_clock(self: &mut Self) {
        self.prev_time = Instant::now();
    }
    
    pub fn handle_clocks(self: &mut Self, mem: &mut Memory, curr_cycles: usize) {
        self.handle_timer_registers(mem, curr_cycles);

        self.wait_time = (curr_cycles as f64) * CPU_PERIOD_NANOS;
        while (self.prev_time.elapsed().as_nanos() as f64) < self.wait_time {}
    }

    fn handle_timer_registers(self: &mut Self, mem: &mut Memory, curr_cycles: usize) {
        self.handle_div(mem, curr_cycles);
        self.handle_tima(mem, curr_cycles);
    }

    fn handle_div(self: &mut Self, mem: &mut Memory, curr_cycles: usize) {

        let prev_cycles = self.acc_div_cycles;

        // curr_cycles shouldnt ever be great than 24 so cast to u8 is okay
        self.acc_div_cycles = self.acc_div_cycles.wrapping_add(curr_cycles as u8);

        if self.acc_div_cycles < prev_cycles {  // if we overflow, then 256 cycles have passed (div_period/cpu_period)
            mem.incr_div_reg();             // Which means its time to increment the div register
        } 

        /*  
            We arent cycle accurate so this could be bad but should be really rare since it requires
            acc_cycle + curr_cycles to overflow on an instruction where we write to the divider register
            and also needs the write to the register to come in the cycles after the acc_cycles overflows

            Okay Example: executing instruction that takes 20 cycles and the acc_cycles is at 246,
            Imagine a write occurs to the register during the second machine cycle (acc_cycles 250-254)
            Real gameboy would set div_reg is reset to 0 first, then instructions completes and acc_cycles overflows second,
            causing div_reg to increment to 1.
            Our code is okay here and do the same thing.

            Bad Example: executing instruction that takes 20 cycles and the acc_cycles is at 246,
            Imagine a write occurs to the div_register during the last machine cycle of the instruction
            (acc_cycles 262-266 which would overflow and be 7-11). Real gameboy would increment the div_reg
            first since enough time has passed midway through the instruction, and then once the write occurs 
            (at the end of the instruction) reset the div_reg to 0
            Our code will have the div_reg at 1 since the write will be done first (reset to 0) and timer calcs
            done second. Thus our code will cause div_reg to be 1 when it should be 0
        */
    }

    fn handle_tima(self: &mut Self, mem: &mut Memory, curr_cycles: usize) {
        let mut tima;
        let (timer_enable, tac_cycles) = mem.decode_tac();
        let tima_prev = mem.read_byte(TIMA_REG);

        if timer_enable {
            self.acc_tima_cycles = self.acc_tima_cycles.wrapping_add(curr_cycles);

            while self.acc_tima_cycles > tac_cycles {
                tima = tima_prev.wrapping_add(1);

                if tima < tima_prev {   // Overflow

                /*
                    Another problem due to not being cycle accurate. This requires a write
                    to a not normally written to register and for that write to occur on the 
                    exact same machine cycle as an overflow so it should be really uncommon

                    If TIMA overflows on the exact same machine cycle that a write occurs to
                    TMA then we are supposed to reset TIMA to the old value of TMA

                    Current setup means that the new value of TMA is always chosen.
                */

                    let tma = mem.read_tma();
                    mem.write_byte(TIMA_REG, tma);
                    self.request_interrupt(mem);
                }

                self.acc_tima_cycles -= tac_cycles;
            }
        }
    }

    fn request_interrupt(self: &mut Self, mem: &mut Memory) {
        let if_reg = mem.read_byte(IF_REG);
        mem.write_byte(IF_REG, if_reg | 0x04);
    }
}
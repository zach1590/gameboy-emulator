use crate::io::{Io, IF_REG, TIMA_REG};

use std::time::Instant;

const DIV_PERIOD_NANOS: f64 = 61035.15;
const CPU_PERIOD_NANOS: f64 = 238.418579; 

pub struct Timer {
    prev_time: Instant,
    wait_time: f64,
    acc_div_cycles: usize,
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
    
    pub fn handle_clocks(self: &mut Self, io: &mut Io, curr_cycles: usize) {
        self.handle_timer_registers(io, curr_cycles);

        self.wait_time = (curr_cycles as f64) * CPU_PERIOD_NANOS;
        while (self.prev_time.elapsed().as_nanos() as f64) < self.wait_time {}
        self.prev_time = Instant::now();
    }

    fn handle_timer_registers(self: &mut Self, io: &mut Io, curr_cycles: usize) {
        self.handle_div(io, curr_cycles);
        self.handle_tima(io, curr_cycles);
    }

    fn handle_div(self: &mut Self, io: &mut Io, curr_cycles: usize) {

        self.acc_div_cycles = self.acc_div_cycles.wrapping_add(curr_cycles);

        while self.acc_div_cycles >= 256 {
            io.incr_div();
            self.acc_div_cycles  = self.acc_div_cycles.wrapping_sub(256);
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

    fn handle_tima(self: &mut Self, io: &mut Io, curr_cycles: usize) {
        
        let mut tima = io.read_byte(TIMA_REG);
        let (timer_enable, tac_cycles) = io.decode_tac();

        if timer_enable {
            
            self.acc_tima_cycles = self.acc_tima_cycles.wrapping_add(curr_cycles);

            while self.acc_tima_cycles >= tac_cycles {
                tima = tima.wrapping_add(1);

                if tima == 0 {   // Overflow

                /*
                    Another problem due to not being cycle accurate. This requires a write
                    to a not normally written to register and for that write to occur on the 
                    exact same machine cycle as an overflow so it should be really uncommon

                    If TIMA overflows on the exact same machine cycle that a write occurs to
                    TMA then we are supposed to reset TIMA to the old value of TMA

                    Current setup means that the new value of TMA is always chosen.
                */

                    let tma = io.read_tma();
                    io.write_byte(TIMA_REG, tma);
                    self.request_interrupt(io);
                } else {
                    io.write_byte(TIMA_REG, tima);
                }

                self.acc_tima_cycles = self.acc_tima_cycles.wrapping_sub(tac_cycles);
            }
        }
    }

    fn request_interrupt(self: &mut Self, io: &mut Io) {
        let if_reg = io.read_byte(IF_REG);
        io.write_byte(IF_REG, if_reg | 0x04);
    }
}
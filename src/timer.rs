use super::memory::Memory;
use std::time::Instant;

const div_period_nanos: f64 = 61035.15;
const cpu_period_nanos: f64 = 238.418579; 

pub struct Timer {
    prev_time: Instant,
    div_timer: Instant,
    wait_time: f64,
    acc_cycles: u8,
    increments: u8,
    is_stop: bool,  // not used since no licensed rom uses STOP except CGB speed switching.
}

impl Timer {
    pub fn new() -> Timer {
        return Timer {
            prev_time: Instant::now(),
            div_timer: Instant::now(),
            wait_time: 0.0,
            acc_cycles: 0,
            increments: 0,
            is_stop: false, // not used since no licensed rom uses STOP except CGB speed switching.
        }
    }

    
    pub fn wait_and_sync(self: &mut Self, curr_cycles: usize) {
        self.calc_div_incr(curr_cycles);
        self.wait_time = (curr_cycles as f64) * cpu_period_nanos;
        while (self.prev_time.elapsed().as_nanos() as f64) < self.wait_time {}
    }

    pub fn reset_clock(self: &mut Self) {
        self.prev_time = Instant::now();
    }

    pub fn handle_timer_registers(self: &mut Self, mem: &mut Memory) {
        self.handle_div(mem);
    }

    fn handle_div(self: &mut Self, mem: &mut Memory) {
        if self.is_stop { return; } // Never true

        // We arent cycle accurate so this could be bad but should be really rare since it requires
        // acc_cycle + curr_cycles to overflow on an instruction where we write to the divider register
        // and also needs the write to the register to come in the cycles after the acc_cycles overflows

        // Okay Example: executing instruction that takes 20 cycles and the acc_cycles is at 246,
        // Imagine a write occurs to the register during the second machine cycle (acc_cycles 250-254)
        // Real gameboy would set div_reg is reset to 0 first, then instructions completes and acc_cycles overflows second,
        // causing div_reg to increment to 1.
        // Our code is okay here and do the same thing.

        // Bad Example: executing instruction that takes 20 cycles and the acc_cycles is at 246,
        // Imagine a write occurs to the div_register during the last machine cycle of the instruction
        // (acc_cycles 262-266 which would overflow and be 7-11). Real gameboy would increment the div_reg
        // first since enough time has passed midway through the instruction, and then once the write occurs 
        // (at the end of the instruction) reset the div_reg to 0
        // Our code will have the div_reg at 1 since the write will be done first (reset to 0) and timer calcs
        // done second. Thus our code will cause div_reg to be 1 when it should be 0

        mem.incr_div_reg(self.increments);
        self.increments = 0;    
    }

    fn calc_div_incr(self: &mut Self, curr_cycles: usize) {
        let prev_cycles = self.acc_cycles;

        // curr_cycles shouldnt ever be great than 24
        self.acc_cycles = self.acc_cycles.wrapping_add(curr_cycles as u8);

        if self.acc_cycles < prev_cycles {  // if we overflow, then 256 cycles have passed
            self.increments += 1;           // the div register will be need to be incremented by one more
        }
    }

    // Never called
    // No licensed rom uses STOP except CGB speed switching.
    pub fn start_stop(self: &mut Self, mem: &mut Memory) {
        mem.set_div_reg(0);
        self.is_stop = true;
    }

    // Never called
    // No licensed rom uses STOP except CGB speed switching.
    pub fn end_stop(self: &mut Self) {
        self.is_stop = false;
    }
}
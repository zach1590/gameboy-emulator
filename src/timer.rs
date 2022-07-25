use crate::io::{Io, TIMA_REG};

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
        };
    }

    pub fn adv_cycles(self: &mut Self, io: &mut Io, curr_cycles: usize) {
        self.handle_timer_registers(io, curr_cycles);

        // self.wait_time = (curr_cycles as f64) * CPU_PERIOD_NANOS;
        // while (self.prev_time.elapsed().as_nanos() as f64) < self.wait_time {}
        // self.prev_time = Instant::now();
    }

    fn handle_timer_registers(self: &mut Self, io: &mut Io, curr_cycles: usize) {
        self.handle_div(io, curr_cycles);
        self.handle_tima(io, curr_cycles);
    }

    fn handle_div(self: &mut Self, io: &mut Io, curr_cycles: usize) {
        self.acc_div_cycles = self.acc_div_cycles.wrapping_add(curr_cycles);

        while self.acc_div_cycles >= 256 {
            io.incr_div();
            self.acc_div_cycles = self.acc_div_cycles.wrapping_sub(256);
        }
    }

    fn handle_tima(self: &mut Self, io: &mut Io, curr_cycles: usize) {
        let mut tima = io.read_byte(TIMA_REG);
        let (timer_enable, tac_cycles) = io.decode_tac();

        if timer_enable {
            self.acc_tima_cycles = self.acc_tima_cycles.wrapping_add(curr_cycles);

            while self.acc_tima_cycles >= tac_cycles {
                tima = tima.wrapping_add(1);

                if tima == 0 {
                    // Overflow
                    /*
                        If TIMA overflows on the exact same machine cycle that a write occurs to
                        TMA then we are supposed to reset TIMA to the old value of TMA.
                    */
                    let tma = io.read_tma();
                    io.write_byte(TIMA_REG, tma);
                    io.request_timer_interrupt();
                } else {
                    io.write_byte(TIMA_REG, tima);
                }

                self.acc_tima_cycles = self.acc_tima_cycles.wrapping_sub(tac_cycles);
            }
        }
        // Done handling timers so if tma was written to on this clock cycle
        // we dont care anymorefor the future cycles/until next write to tma.
        io.clean_tma();
    }
}

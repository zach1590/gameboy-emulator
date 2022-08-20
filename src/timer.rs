use crate::io::Io;
pub const TIMER_START: u16 = 0xFF04;
pub const TIMER_END: u16 = 0xFF07;
pub const DIV_REG: u16 = 0xFF04;
pub const TIMA_REG: u16 = 0xFF05;
pub const TMA_REG: u16 = 0xFF06;
pub const TAC_REG: u16 = 0xFF07;

pub struct Timer {
    div: u16,
    tima: u8,
    tma: u8,
    tac: u8,
    tma_prev: u8,
    tma_dirty: bool,
    acc_div_cycles: usize,
    acc_tima_cycles: usize,
}

impl Timer {
    pub fn new() -> Timer {
        return Timer {
            div: 0,
            tima: 0,
            tma: 0,
            tac: 0,
            tma_prev: 0x00,
            tma_dirty: false,
            acc_div_cycles: 0,
            acc_tima_cycles: 0,
        };
    }

    pub fn read_byte(self: &Self, addr: u16) -> u8 {
        return match addr {
            DIV_REG => self.div as u8,
            TIMA_REG => self.tima,
            TMA_REG => self.tma,
            TAC_REG => self.tac,
            _ => panic!("Timer does not handle reads from addr: {}", addr),
        };
    }

    pub fn write_byte(self: &mut Self, addr: u16, data: u8) {
        match addr {
            DIV_REG => self.reset_div(),
            TIMA_REG => self.tima = data,
            TMA_REG => {
                self.tma_dirty = true;
                self.tma_prev = self.tma;
                self.tma = data;
            }
            TAC_REG => self.tac = (data & 0x07) | 0xF8,
            _ => panic!("Timer does not handle writes to addr: {}", addr),
        }
    }

    pub fn adv_cycles(self: &mut Self, io: &mut Io, curr_cycles: usize) {
        self.handle_div(io, curr_cycles);
        self.handle_tima(io, curr_cycles);
    }

    fn handle_div(self: &mut Self, io: &mut Io, curr_cycles: usize) {
        self.acc_div_cycles = self.acc_div_cycles.wrapping_add(curr_cycles);

        while self.acc_div_cycles >= 256 {
            self.incr_div();
            self.acc_div_cycles = self.acc_div_cycles.wrapping_sub(256);
        }
    }

    fn handle_tima(self: &mut Self, io: &mut Io, curr_cycles: usize) {
        let (timer_enable, tac_cycles) = self.decode_tac();

        if timer_enable {
            self.acc_tima_cycles = self.acc_tima_cycles.wrapping_add(curr_cycles);

            while self.acc_tima_cycles >= tac_cycles {
                self.tima = self.tima.wrapping_add(1);

                if self.tima == 0 {
                    // Overflow
                    /*
                        If TIMA overflows on the exact same machine cycle that a write occurs to
                        TMA then we are supposed to reset TIMA to the old value of TMA.
                    */
                    self.tima = self.read_tma();
                    io.request_timer_interrupt();
                }
                self.acc_tima_cycles = self.acc_tima_cycles.wrapping_sub(tac_cycles);
            }
        }
        // Done handling timers so if tma was written to on this clock cycle
        // we dont care anymore for the future cycles/until next write to tma.
        self.clean_tma();
    }

    pub fn reset_div(self: &mut Self) {
        self.div = 0;
    }

    pub fn incr_div(self: &mut Self) {
        self.div = self.div.wrapping_add(1);
    }

    pub fn read_tma(self: &mut Self) -> u8 {
        if self.tma_dirty {
            return self.tma_prev;
        }
        return self.tma;
    }

    pub fn clean_tma(self: &mut Self) {
        self.tma_dirty = false;
        self.tma_prev = 0x00;
    }

    pub fn decode_tac(self: &mut Self) -> (bool, usize) {
        let tac = self.tac;

        return match ((tac & 0x04) == 0x04, tac & 0x03) {
            (enable, 0) => (enable, 1024),
            (enable, 1) => (enable, 16),
            (enable, 2) => (enable, 64),
            (enable, 3) => (enable, 256),
            _ => panic!("Should be impossible"),
        };
    }

    pub fn dmg_init(self: &mut Self) {
        self.div = 0xAB;
        self.tima = 0x00;
        self.tma = 0x00;
        self.tac = 0xF8;
    }

    #[cfg(feature = "debug")]
    pub fn get_debug_info(self: &Self) -> String {
        format!(
            "div: {:04X}, tima: {:02X}, tma: {:02X}, tac: {:02X}\n",
            self.div, self.tima, self.tma, self.tac,
        )
    }
}

#[test]
fn test_decode_tac() {
    let mut timer = Timer::new();

    timer.write_byte(TAC_REG, 0x07);
    let (enabled, cycles) = timer.decode_tac();
    assert_eq!(enabled, true);
    assert_eq!(cycles, 256);

    timer.write_byte(TAC_REG, 0x06);
    let (enabled, cycles) = timer.decode_tac();
    assert_eq!(enabled, true);
    assert_eq!(cycles, 64);

    timer.write_byte(TAC_REG, 0x012);
    let (enabled, cycles) = timer.decode_tac();
    assert_eq!(enabled, false);
    assert_eq!(cycles, 64);

    timer.write_byte(TAC_REG, 0x08);
    let (enabled, cycles) = timer.decode_tac();
    assert_eq!(enabled, false);
    assert_eq!(cycles, 1024);

    timer.write_byte(TAC_REG, 0x09);
    let (enabled, cycles) = timer.decode_tac();
    assert_eq!(enabled, false);
    assert_eq!(cycles, 16);
}

// Most information regarding the timer will be based on this (Section 5)
// https://github.com/AntonioND/giibiiadvance/blob/master/docs/TCAGBD.pdf

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
    overflow_source: TimaOverflowState,
}

enum TimaOverflowState {
    Done,
    Advancing,
    None,
}

impl TimaOverflowState {
    pub fn is_none(self: &Self) -> bool {
        match self {
            TimaOverflowState::None => true,
            _ => false,
        }
    }

    pub fn is_done(self: &Self) -> bool {
        match self {
            TimaOverflowState::Done => true,
            _ => false,
        }
    }

    pub fn is_advcing(self: &Self) -> bool {
        match self {
            TimaOverflowState::Advancing => true,
            _ => false,
        }
    }
}

impl Timer {
    pub fn new() -> Timer {
        return Timer {
            div: 0,
            tima: 0,
            tma: 0,
            tac: 0,
            overflow_source: TimaOverflowState::None,
        };
    }

    pub fn read_byte(self: &Self, addr: u16) -> u8 {
        return match addr {
            DIV_REG => (self.div >> 8) as u8,
            TIMA_REG => self.tima,
            TMA_REG => self.tma,
            TAC_REG => self.tac,
            _ => panic!("Timer does not handle reads from addr: {}", addr),
        };
    }

    pub fn write_byte(self: &mut Self, addr: u16, data: u8) {
        match addr {
            DIV_REG => self.write_div(),
            TIMA_REG => {
                self.tima = match self.overflow_source {
                    TimaOverflowState::Advancing => {
                        // We wont reload the tima if we wrote on the
                        // same cycle it is to be reloaded
                        self.overflow_source = TimaOverflowState::None;
                        data
                    }
                    TimaOverflowState::Done => {
                        // If we write to the tima on the cycle it was reloaded
                        // it should stay as the reloaded value
                        self.tima // self.tma should also be fine
                    }
                    TimaOverflowState::None => data,
                }
            }
            TMA_REG => {
                self.tma = data;
                if self.overflow_source.is_done() {
                    self.tima = self.tma;
                }
            }
            TAC_REG => {
                self.write_tac(data);
            }
            _ => panic!("Timer does not handle writes to addr: {}", addr),
        }
    }

    pub fn adv_cycles(self: &mut Self, io: &mut Io, cycles: usize) {
        match self.overflow_source {
            TimaOverflowState::Advancing => {
                self.tima = self.tma;
                self.overflow_source = TimaOverflowState::Done;
                io.request_timer_interrupt();
            }
            TimaOverflowState::Done => self.overflow_source = TimaOverflowState::None,
            TimaOverflowState::None => {}
        }

        let (timer_enable, _) = self.decode_tac();
        let old_div_bit = self.div_tac_multiplexer();

        self.div = self.div.wrapping_add(cycles as u16);

        let new_div_bit = self.div_tac_multiplexer();

        // Falling edge detector
        let should_incr =
            self.detected_falling_edge(old_div_bit, new_div_bit, timer_enable, timer_enable);

        if should_incr {
            self.incr_timer();
        }
        io.clean_ifired();
    }

    fn write_div(self: &mut Self) {
        let (timer_enable, _) = self.decode_tac();
        let old_div_bit = self.div_tac_multiplexer();

        self.div = 0;

        let new_div_bit = self.div_tac_multiplexer();

        let should_incr =
            self.detected_falling_edge(old_div_bit, new_div_bit, timer_enable, timer_enable);
        if should_incr {
            self.incr_timer();
        }
    }

    fn write_tac(self: &mut Self, data: u8) {
        let old_div_bit = self.div_tac_multiplexer();
        let (old_enbl, _) = self.decode_tac();

        self.tac = (data & 0x07) | 0xF8;

        let new_div_bit = self.div_tac_multiplexer();
        let (new_enbl, _) = self.decode_tac();

        let should_incr = self.detected_falling_edge(old_div_bit, new_div_bit, old_enbl, new_enbl);

        if should_incr {
            self.incr_timer();
        }
    }

    // Increments the timer and returns if it overflowed
    fn incr_timer(self: &mut Self) -> bool {
        let (new_tima, overflow) = self.tima.overflowing_add(1);

        self.tima = new_tima;

        if overflow {
            // Should be 0 anyways due to overflow so this is kinda useless
            self.tima = 0x00; // Delay reload for 1 M-cycle
            self.overflow_source = TimaOverflowState::Advancing;
        }

        return overflow;
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

    fn div_tac_multiplexer(self: &Self) -> bool {
        return match self.tac & 0x03 {
            0 => (self.div >> 9) & 0x01 == 0x01,
            1 => (self.div >> 3) & 0x01 == 0x01,
            2 => (self.div >> 5) & 0x01 == 0x01,
            3 => (self.div >> 7) & 0x01 == 0x01,
            _ => panic!("double check the `AND` operation"),
        };
    }

    // Return true on a 1 to 0 transition
    fn detected_falling_edge(
        self: &Self,
        old_div: bool,
        new_div: bool,
        old_enbl: bool,
        new_enbl: bool,
    ) -> bool {
        return (old_div && old_enbl) && !(new_div && new_enbl);
    }

    pub fn dmg_init(self: &mut Self) {
        self.div = 0xABCC;
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

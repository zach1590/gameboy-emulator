pub struct MbcTimer {
    pub seconds: u8,
    pub minutes: u8,
    pub hours: u8,
    pub days_lo: u8, // Lower 8 bits of the day counter
    pub days_hi: u8, // Upper 1 bit of the day counter, the carry flag, and the halt flag
}

impl MbcTimer {
    // On new populate these registers with updated values
    pub fn new() -> MbcTimer {
        return MbcTimer {
            seconds: 0x00,
            minutes: 0x00,
            hours: 0x00,
            days_lo: 0x00,
            days_hi: 0x00,
        };
    }

    pub fn is_halted(self: &Self) -> bool {
        return (self.days_hi >> 6) == 0x01;
    }

    pub fn is_counter_overflow(self: &Self) -> bool {
        return (self.days_hi >> 7) == 0x01;
    }

    pub fn get_day_msb(self: &Self) -> u8 {
        return self.days_hi & 0x01;
    }

    // Call this when the latch has changed from 0 to 1
    pub fn on_latch_register(self: &mut Self, new_rtc: &MbcTimer) {
        if self.is_halted() {
            // Dont think its actually possible to stop the internal clock
            // of a CPU (or Quartz Oscillator) so hopefully the documentation
            // just meant dont update the register with new values
            return;
        }

        self.seconds = new_rtc.seconds;
        self.minutes = new_rtc.minutes;
        self.hours = new_rtc.hours;
        self.days_lo = new_rtc.days_lo;
        self.days_hi = new_rtc.days_hi;
    }

    // This is where the math will happen
    pub fn update_timer(self: &mut Self) {}
}

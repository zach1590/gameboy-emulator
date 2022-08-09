pub const RTC_FREQ: usize = 32_768;
pub const RTC_PERIOD_MICROS: f64 = 30.51757;

pub struct MbcTimer {
    pub seconds: u8,
    pub minutes: u8,
    pub hours: u8,
    pub days_lo: u8, // Lower 8 bits of the day counter
    pub days_hi: u8, // Upper 1 bit of the day counter, the carry flag, and the halt flag
    pub cycles: usize,
    pub int_cycles: usize,
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
            cycles: 0,
            int_cycles: 0,
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
    // self should be the latched timer, and new_rtc should be the constantly updating one
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

        // Keep the former halt flags and take the new bit 0
        // If the days overflowed, set the bottom bit will be set to new no matter what
        // If the days did not overflow, then let it keep its old value so that in case
        // it is set, then the program can decide when it should be unset.
        // If the days did overflow, then it will set the top bit of the latched rtc
        self.days_hi = (self.days_hi & 0xC0) | (new_rtc.days_hi & 0x81);
    }

    pub fn update_timer(self: &mut Self, diff_seconds: i32, carry: bool) {
        let mut rtc_as_seconds = self.to_secs();

        if diff_seconds >= 0 {
            // Move forward in time
            rtc_as_seconds = rtc_as_seconds.wrapping_add(diff_seconds as u64);
        } else {
            // Move backwards
            let positive_diff = diff_seconds * -1;
            rtc_as_seconds = rtc_as_seconds.wrapping_sub(positive_diff as u64);
        }

        self.from_secs(rtc_as_seconds);

        if carry {
            self.days_hi |= 0x80;
        }
    }

    pub fn from_secs(self: &mut Self, rtc_as_seconds: u64) {
        self.seconds = (rtc_as_seconds % 60) as u8;
        self.minutes = ((rtc_as_seconds / 60) % 60) as u8;
        self.hours = ((rtc_as_seconds / (3600)) % (24)) as u8;

        let days = rtc_as_seconds / 86400;
        self.days_lo = (days % 256) as u8;

        self.days_hi = self.days_hi & 0xFE;
        if (days >= 256) && days <= 511 {
            self.days_hi |= 0x01;
        }
        if days >= 512 {
            self.days_hi |= 0x80;
        }
    }

    pub fn to_secs(self: &Self) -> u64 {
        return u64::from(self.seconds)
            + (u64::from(self.minutes) * 60)
            + (u64::from(self.hours) * 3600)
            + (u64::from(self.days_lo) * 86400)
            + (u64::from((self.days_hi & 0x01) << 8) * 86400);
    }
}

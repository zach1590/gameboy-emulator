/*
    Things that might be wrong

    1. When the registers are written to through normal operation or through
       the latch register changing from 0 to 1. Does the halt flag need to be
       set for the latch operation or normal writes to the register or both?

    2. When the rtc is written a value to it (write to ram with bank 0x08 - 0x0C
       selected). Is does the value only get written to the latched data, or does
       it get written to both the latched data and the constantly updating timer.

    What makes sense is that for the program to actually set the halt flag, we need
    to be able to write to the rtc registers that are currently latched. Thus writing
    to latched registers in general should be okay otherwise why would halt be treated
    differently (unless the pins on the actual hardware wired to bit6 were special).

    Since we need to be able to set the halt flag, writing to latched data is always okay
    and thus the halt flag must be specifically for attempting to latch the data from the
    timer into the latched registers that can be easily accessed.

    Thus when writes to ram are performed, it probably should not affect the actual rtc
    at all, and it should always be synched to real time. However, if it is synched to
    real time, then that means everytime the registers are latched, the real time is
    restored which would defeat the purpose of writing to the registers in the first place
    apart from the carry and halt flag.

    3. When in halted state, should the rtc continue ticking on adv_cycles. I doubt its
       possible to stop the quartz oscillator from ticking forwards in time so I'm guessing
       that halt only means to no latch the data
*/
use std::time::SystemTime;

pub const RTC_FREQ: usize = 32_768;
pub const RTC_PERIOD_MICROS: f64 = 30.51757;
pub const COUNTER_MAX_SECONDS: u64 = 44_236_799;

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

    pub fn update_timer_pos(self: &mut Self, diff_seconds: u64, carry: bool) {
        let mut rtc_as_seconds = self.to_secs();
        rtc_as_seconds = rtc_as_seconds.wrapping_add(diff_seconds);
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
            + (((u64::from(self.days_hi & 0x01)) << 8) * 86400);
    }

    // Im gonna return 0 rather than panic if it fails since this is gonna be called on
    // program exit while we are dropping stuff, and I dont know if panic is good idea
    // during that
    pub fn get_current_time() -> u64 {
        return match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(duration) => duration.as_secs(),
            Err(_err) => {
                println!("Error getting the system time");
                0
            }
        };
    }
}

#[test]
fn test_convert_from_secs() {
    let mut timer = MbcTimer::new();
    let time_seconds = COUNTER_MAX_SECONDS; // 510 days, 23 hours, 59 mins 59 secs

    timer.from_secs(time_seconds);

    assert_eq!(timer.days_hi & 0x01, 0x01);
    assert_eq!(timer.days_lo, 255);
    assert_eq!(timer.hours, 23);
    assert_eq!(timer.minutes, 59);
    assert_eq!(timer.seconds, 59);
}

#[test]
fn test_convert_to_secs() {
    let mut timer = MbcTimer::new();

    timer.days_hi = 0x01;
    timer.days_lo = 255;
    timer.hours = 23;
    timer.minutes = 59;
    timer.seconds = 59;

    let time_seconds = timer.to_secs();
    assert_eq!(time_seconds, COUNTER_MAX_SECONDS);
}

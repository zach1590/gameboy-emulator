// The unused bits in all io registers should always return 1
// This is for both registers that have and dont have dedicated purposes
// https://github.com/Gekkio/mooneye-test-suite/blob/main/acceptance/bits/unused_hwio-GS.s#L21

pub const IO_START: u16 = 0xFF00;
pub const IF_REG: u16 = 0xFF0F;
pub const DIV_REG: u16 = 0xFF04; // Writing any value to this register resets it to 0
pub const TIMA_REG: u16 = 0xFF05;
pub const TMA_REG: u16 = 0xFF06;
pub const TAC_REG: u16 = 0xFF07;

pub struct Io {
    io: [u8; 128],
    ifired_dirty: bool,
}

impl Io {
    pub fn new() -> Io {
        Io {
            io: [0xFF; 128],
            ifired_dirty: false,
        }
    }

    pub fn read_byte(self: &Self, addr: u16) -> u8 {
        return self.io[usize::from(addr - IO_START)];
    }

    // https://gbdev.io/pandocs/CGB_Registers.html#ff74---bits-0-7-readwrite---cgb-mode-only
    pub fn write_byte(self: &mut Self, addr: u16, data: u8) {
        match addr {
            0xFF72 => self.io[usize::from(addr - IO_START)] = data | 0xFF, // Contradicts Pandocs but Ill trust Mooneye
            0xFF73 => self.io[usize::from(addr - IO_START)] = data | 0xFF, // Contradicts Pandocs but Ill trust Mooneye
            0xFF75 => self.io[usize::from(addr - IO_START)] = data | 0xFF, // Contradicts Pandocs but Ill trust Mooneye
            IF_REG => {
                self.io[usize::from(IF_REG - IO_START)] = data | 0xE0;
                self.ifired_dirty = true;
            }
            _ => return,
        }
    }

    pub fn clean_ifired(self: &mut Self) {
        self.ifired_dirty = false;
    }

    pub fn request_joypad_interrupt(self: &mut Self) {
        let ifired = usize::from(IF_REG - IO_START);
        self.io[ifired] = self.io[ifired] | 0xF0;
    }

    pub fn request_serial_interrupt(self: &mut Self) {
        let ifired = usize::from(IF_REG - IO_START);
        self.io[ifired] = self.io[ifired] | 0xE8;
    }

    pub fn request_timer_interrupt(self: &mut Self) {
        // If we wrote to the IF register on the same cycle
        // requesting the timer_interrupt. We should keep
        // the written value
        if !self.ifired_dirty {
            let ifired = usize::from(IF_REG - IO_START);
            self.io[ifired] = self.io[ifired] | 0xE4;
        }
    }

    pub fn request_stat_interrupt(self: &mut Self) {
        let ifired = usize::from(IF_REG - IO_START);
        self.io[ifired] = self.io[ifired] | 0xE2;
    }

    pub fn request_vblank_interrupt(self: &mut Self) {
        let ifired = usize::from(IF_REG - IO_START);
        self.io[ifired] = self.io[ifired] | 0xE1;
    }

    pub fn dmg_init(self: &mut Self) {
        self.io[usize::from(IF_REG - IO_START)] = 0xE1;

        // Not sure
        self.io[usize::from(0xFF03 - IO_START)] = 0xFF;

        // These are cgb registers
        self.io[usize::from(0xFF4D - IO_START)] = 0xFF;
        self.io[usize::from(0xFF4F - IO_START)] = 0xFF;
        self.io[usize::from(0xFF51 - IO_START)] = 0xFF;
        self.io[usize::from(0xFF52 - IO_START)] = 0xFF;
        self.io[usize::from(0xFF53 - IO_START)] = 0xFF;
        self.io[usize::from(0xFF54 - IO_START)] = 0xFF;
        self.io[usize::from(0xFF55 - IO_START)] = 0xFF;
        self.io[usize::from(0xFF56 - IO_START)] = 0xFF;
        self.io[usize::from(0xFF68 - IO_START)] = 0xFF;
        self.io[usize::from(0xFF69 - IO_START)] = 0xFF;
        self.io[usize::from(0xFF6A - IO_START)] = 0xFF;
        self.io[usize::from(0xFF6B - IO_START)] = 0xFF;
        self.io[usize::from(0xFF70 - IO_START)] = 0xFF;

        // https://gbdev.io/pandocs/CGB_Registers.html#undocumented-registers
        self.io[usize::from(0xFF72 - IO_START)] = 0x00;
        self.io[usize::from(0xFF73 - IO_START)] = 0x00;
        self.io[usize::from(0xFF74 - IO_START)] = 0xFF; // R/W in cgb, otherwise read only as 0xFF
        self.io[usize::from(0xFF75 - IO_START)] = 0x8F;
    }
}

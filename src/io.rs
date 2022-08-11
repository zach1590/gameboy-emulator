// The unused bits in all io registers should always return 1
// This is for both registers that have and dont have dedicated purposes
// https://github.com/Gekkio/mooneye-test-suite/blob/main/acceptance/bits/unused_hwio-GS.s#L21

pub const IO_START: u16 = 0xFF00;
pub const IF_REG: u16 = 0xFF0F;
pub const DIV_REG: u16 = 0xFF04; // Writing any value to this register resets it to 0
pub const TIMA_REG: u16 = 0xFF05;
pub const TMA_REG: u16 = 0xFF06;
pub const TAC_REG: u16 = 0xFF07;

// Sound
pub const NR10: u16 = 0xFF10;
pub const NR11: u16 = 0xFF11;
pub const NR12: u16 = 0xFF12;
pub const NR13: u16 = 0xFF13;
pub const NR14: u16 = 0xFF14;
pub const NR21: u16 = 0xFF16;
pub const NR22: u16 = 0xFF17;
pub const NR23: u16 = 0xFF18;
pub const NR24: u16 = 0xFF19;
pub const NR30: u16 = 0xFF1A;
pub const NR31: u16 = 0xFF1B;
pub const NR32: u16 = 0xFF1C;
pub const NR33: u16 = 0xFF1D;
pub const NR34: u16 = 0xFF1E;
pub const NR41: u16 = 0xFF20;
pub const NR42: u16 = 0xFF21;
pub const NR43: u16 = 0xFF22;
pub const NR44: u16 = 0xFF23;
pub const NR50: u16 = 0xFF24;
pub const NR51: u16 = 0xFF25;
pub const NR52: u16 = 0xFF26;

pub struct Io {
    io: [u8; 128],
    tma_prev: u8,
    tma_dirty: bool,
    dma_transfer: bool,
}

impl Io {
    pub fn new() -> Io {
        Io {
            io: [0xFF; 128],
            tma_prev: 0x00,
            tma_dirty: false,
            dma_transfer: false,
        }
    }

    pub fn read_byte(self: &Self, addr: u16) -> u8 {
        return self.io[usize::from(addr - IO_START)];
    }

    // https://gbdev.io/pandocs/CGB_Registers.html#ff74---bits-0-7-readwrite---cgb-mode-only
    pub fn write_byte(self: &mut Self, addr: u16, data: u8) {
        match addr {
            DIV_REG => self.reset_div(),
            TMA_REG => {
                self.tma_dirty = true;
                self.tma_prev = self.io[usize::from(TMA_REG - IO_START)];
                self.io[usize::from(TMA_REG - IO_START)] = data;
            }
            TAC_REG => self.io[usize::from(TAC_REG - IO_START)] = data & 0x07, // bottom 3 bits
            0xFF74 => return,
            0xFF75 => self.io[usize::from(addr - IO_START)] = data & 0b0111_0000,
            IF_REG => self.io[usize::from(IF_REG - IO_START)] = data | 0xE0,
            _ => self.io[usize::from(addr - IO_START)] = data,
        }
    }

    pub fn reset_div(self: &mut Self) {
        self.io[usize::from(DIV_REG - IO_START)] = 0;
    }

    pub fn incr_div(self: &mut Self) {
        self.io[usize::from(DIV_REG - IO_START)] =
            self.io[usize::from(DIV_REG - IO_START)].wrapping_add(1);
    }

    pub fn read_tma(self: &mut Self) -> u8 {
        if self.tma_dirty {
            return self.tma_prev;
        }
        return self.io[usize::from(TMA_REG - IO_START)];
    }

    pub fn clean_tma(self: &mut Self) {
        self.tma_dirty = false;
        self.tma_prev = 0x00;
    }

    pub fn decode_tac(self: &mut Self) -> (bool, usize) {
        let tac = self.io[usize::from(TAC_REG - IO_START)];

        return match ((tac & 0x04) == 0x04, tac & 0x03) {
            (enable, 0) => (enable, 1024),
            (enable, 1) => (enable, 16),
            (enable, 2) => (enable, 64),
            (enable, 3) => (enable, 256),
            _ => panic!("Should be impossible"),
        };
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
        let ifired = usize::from(IF_REG - IO_START);
        self.io[ifired] = self.io[ifired] | 0xE4;
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
        self.io[usize::from(DIV_REG - IO_START)] = 0xAB;
        self.io[usize::from(TIMA_REG - IO_START)] = 0x00;
        self.io[usize::from(TMA_REG - IO_START)] = 0x00;
        self.io[usize::from(TAC_REG - IO_START)] = 0xF8;
        self.io[usize::from(IF_REG - IO_START)] = 0xE1;

        // Sound
        self.io[usize::from(NR10 - IO_START)] = 0x80;
        self.io[usize::from(NR11 - IO_START)] = 0xBF;
        self.io[usize::from(NR12 - IO_START)] = 0xF3;
        self.io[usize::from(NR13 - IO_START)] = 0xFF;
        self.io[usize::from(NR14 - IO_START)] = 0xBF;
        self.io[usize::from(NR21 - IO_START)] = 0x3F;
        self.io[usize::from(NR22 - IO_START)] = 0x00;
        self.io[usize::from(NR23 - IO_START)] = 0xFF;
        self.io[usize::from(NR24 - IO_START)] = 0xBF;
        self.io[usize::from(NR30 - IO_START)] = 0x7F;
        self.io[usize::from(NR31 - IO_START)] = 0xFF;
        self.io[usize::from(NR32 - IO_START)] = 0x9F;
        self.io[usize::from(NR33 - IO_START)] = 0xFF;
        self.io[usize::from(NR34 - IO_START)] = 0xBF;
        self.io[usize::from(NR41 - IO_START)] = 0xFF;
        self.io[usize::from(NR42 - IO_START)] = 0x00;
        self.io[usize::from(NR43 - IO_START)] = 0x00;
        self.io[usize::from(NR44 - IO_START)] = 0xBF;
        self.io[usize::from(NR50 - IO_START)] = 0x77;
        self.io[usize::from(NR51 - IO_START)] = 0xF3;
        self.io[usize::from(NR52 - IO_START)] = 0xF1;

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
    }
}

#[test]
fn test_decode_tac() {
    let mut io = Io::new();

    io.write_byte(TAC_REG, 0x07);
    let (enabled, cycles) = io.decode_tac();
    assert_eq!(enabled, true);
    assert_eq!(cycles, 256);

    io.write_byte(TAC_REG, 0x06);
    let (enabled, cycles) = io.decode_tac();
    assert_eq!(enabled, true);
    assert_eq!(cycles, 64);

    io.write_byte(TAC_REG, 0x012);
    let (enabled, cycles) = io.decode_tac();
    assert_eq!(enabled, false);
    assert_eq!(cycles, 64);

    io.write_byte(TAC_REG, 0x08);
    let (enabled, cycles) = io.decode_tac();
    assert_eq!(enabled, false);
    assert_eq!(cycles, 1024);

    io.write_byte(TAC_REG, 0x09);
    let (enabled, cycles) = io.decode_tac();
    assert_eq!(enabled, false);
    assert_eq!(cycles, 16);
}

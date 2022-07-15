pub const IO_START: u16 = 0xFF00;
pub const SB_REG: u16 = 0xFF01;
pub const SC_REG: u16 = 0xFF02;
pub const IF_REG: u16 = 0xFF0F;
pub const DIV_REG: u16 = 0xFF04;    // Writing any value to this register resets it to 0
pub const TIMA_REG: u16 = 0xFF05;
pub const TMA_REG: u16 = 0xFF06;
pub const TAC_REG: u16 = 0xFF07;

pub struct Io {
    io: [u8; 128],
}

impl Io {
    pub fn new() -> Io {
        Io { io: [0; 128], }
    }

    pub fn read_byte(self: &Self, addr: u16) -> u8 {
        return self.io[usize::from(addr - IO_START)];
    }

    pub fn write_byte(self: &mut Self, addr: u16, data: u8) {
        match addr {
            DIV_REG => self.reset_div(),
            TAC_REG => self.io[usize::from(TAC_REG - IO_START)] = data & 0x07,   // bottom 3 bits
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
        return self.io[usize::from(TMA_REG - IO_START)];
    }

    pub fn decode_tac(self: &mut Self) -> (bool, usize) {
        let tac = self.io[usize::from(TAC_REG - IO_START)];

        return match ((tac & 0x04) == 0x04, tac & 0x03) {
            (enable, 0) => (enable, 1024),
            (enable, 1) => (enable, 16),
            (enable, 2) => (enable, 64),
            (enable, 3) => (enable, 256),
            _ => panic!("Should be impossible")
        };
    }

    pub fn dmg_init(self: &mut Self) {
        self.io[usize::from(SB_REG - IO_START)] = 0x00;
        self.io[usize::from(SC_REG - IO_START)] = 0x7E;
        self.io[usize::from(DIV_REG - IO_START)] = 0xAB;
        self.io[usize::from(TIMA_REG - IO_START)] = 0x00;
        self.io[usize::from(TMA_REG - IO_START)] = 0x00;
        self.io[usize::from(TAC_REG - IO_START)] = 0xF8;
        self.io[usize::from(IF_REG - IO_START)] = 0xE1;
    }
}

#[test]
fn test_decode_tac(){
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
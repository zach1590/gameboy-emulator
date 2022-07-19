pub const IO_START: u16 = 0xFF00;
pub const SB_REG: u16 = 0xFF01;
pub const SC_REG: u16 = 0xFF02;
pub const IF_REG: u16 = 0xFF0F;
pub const DIV_REG: u16 = 0xFF04;    // Writing any value to this register resets it to 0
pub const TIMA_REG: u16 = 0xFF05;
pub const TMA_REG: u16 = 0xFF06;
pub const TAC_REG: u16 = 0xFF07;
pub const LCDC_REG: u16 = 0xFF40;
pub const STAT_REG: u16 = 0xFF41;   // LCD Status
pub const SCY_REG: u16 = 0xFF42; // Used to scroll the background
pub const SCX_REG: u16 = 0xFF43;
pub const LY_REG: u16 = 0xFF44;
pub const LYC_REG: u16 = 0xFF45;
pub const DMA_REG: u16 = 0xFF46;
pub const WY_REG: u16 = 0xFF4A; // Top left coordinates of the window
pub const WX_REG: u16 = 0xFF4B; // Think this is only important when drawing

pub struct Io {
    io: [u8; 128],
    tma_prev: u8,
    tma_dirty: bool,
    dma_transfer: bool,
}

impl Io {
    pub fn new() -> Io {
        Io { 
            io: [0; 128],
            tma_prev: 0x00, 
            tma_dirty: false,
            dma_transfer: false,
        }
    }

    pub fn read_byte(self: &Self, addr: u16) -> u8 {
        return self.io[usize::from(addr - IO_START)];
    }

    pub fn write_byte(self: &mut Self, addr: u16, data: u8) {
        match addr {
            DIV_REG => self.reset_div(),
            TMA_REG => {
                self.tma_dirty = true;
                self.tma_prev = self.io[usize::from(TMA_REG - IO_START)];
                self.io[usize::from(TMA_REG - IO_START)] = data;
            },
            TAC_REG => self.io[usize::from(TAC_REG - IO_START)] = data & 0x07,   // bottom 3 bits
            STAT_REG  => { 
                let stat = usize::from(STAT_REG - IO_START);
                self.io[stat] = (data & 0b0111_1000) | (self.io[stat] & 0b0000_0111);
                if self.ly_stat_enable() && self.ly_compare() { 
                    self.request_stat();        // Should I be doing this?
                }
            },
            LY_REG => {
                self.io[usize::from(LY_REG - IO_START)] = data;
                let equal = self.ly_compare();
                self.update_stat(equal);
            },
            LYC_REG => {
                self.io[usize::from(LYC_REG - IO_START)] = data;
                let equal = self.ly_compare();
                self.update_stat(equal);
            },
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
            _ => panic!("Should be impossible")
        };
    }

    // lcdc can be modified mid scanline (I dont know how??)
    // Maybe better to call this function from each of other helper methods?
    pub fn get_lcdc(self: &Self) -> u8 {
        return self.read_byte(LCDC_REG);
    }

    pub fn get_ly(self: &Self) -> u8 {
        return self.read_byte(LY_REG);
    }

    pub fn get_dma_dest(self: &Self) -> u16 {
        return (self.read_byte(DMA_REG) as u16) * 0x0100;
    }

    pub fn is_dma_transfer(self: &Self) -> bool {
        return self.dma_transfer;
    }

    pub fn ly_compare(self: &mut Self) -> bool {
        return self.io[usize::from(LYC_REG - IO_START)] 
                    == self.io[usize::from(LY_REG - IO_START)];
    }

    pub fn update_stat(self: &mut Self, equal: bool) {
        let stat = usize::from(STAT_REG - IO_START);
        if equal {
            self.io[stat] = self.io[stat] | 0b0000_0100;
            if self.ly_stat_enable() { self.request_stat(); }       // Should I be doing this?
                
        } else {
            self.io[stat] = self.io[stat] & 0b1111_1011;
        }
    }

    pub fn ly_stat_enable(self: &mut Self) -> bool {
        let stat = usize::from(STAT_REG - IO_START);
        return (self.io[stat] & 0x40) == 0x40;
    }

    pub fn request_stat(self: &mut Self) {
        let ifired = usize::from(IF_REG - IO_START);
        self.io[ifired] = self.io[ifired] | 0b0000_0010;
    }

    pub fn get_lcd_mode(self: &Self) -> u8 {
        return self.io[usize::from(STAT_REG - IO_START)] & 0x03;
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
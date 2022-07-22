use super::sprite::Sprite;
use std::collections::VecDeque;

pub const LCDC_REG: u16 = 0xFF40;
pub const STAT_REG: u16 = 0xFF41; // LCD Status
pub const SCY_REG: u16 = 0xFF42; // Used to scroll the background
pub const SCX_REG: u16 = 0xFF43;
pub const LY_REG: u16 = 0xFF44;
pub const LYC_REG: u16 = 0xFF45;
pub const DMA_REG: u16 = 0xFF46;
pub const PALLETE_REG: u16 = 0xFF47;
pub const OPB0_REG: u16 = 0xFF48;
pub const OPB1_REG: u16 = 0xFF49;
pub const WY_REG: u16 = 0xFF4A; // Top left coordinates of the window
pub const WX_REG: u16 = 0xFF4B; // Think this is only important when drawing

pub struct GpuMemory {
    pub vram: [u8; 8_192], // 0x8000 - 0x9FFF
    pub oam: [u8; 160],    // OAM 0xFE00 - 0xFE9F  40 sprites, each takes 4 bytes
    pub lcdc: u8,          // 0xFF40
    pub stat: u8,          // 0xFF41
    pub scy: u8,           // 0xFF42
    pub scx: u8,           // 0xFF43
    pub ly: u8,            // 0xFF44
    pub lyc: u8,           // 0xFF45
    pub dma: u8,           // 0xFF46
    pub pallete: u8,       // 0xFF47
    pub opb0: u8,          // 0xFF48
    pub opb1: u8,          // 0xFF49
    pub wy: u8,            // 0xFF4A
    pub wx: u8,            // 0xFF4B
    pub dma_transfer: bool,
    pub stat_int: bool, // For the cgb specific io we will continue to write them to Io rather than here
    pub sprite_list: Vec<Sprite>,
    pub oam_pixel_fifo: VecDeque<u8>,
    pub bg_pixel_fifo: VecDeque<u8>,
}

impl GpuMemory {
    pub fn new() -> GpuMemory {
        return GpuMemory {
            vram: [0; 8_192],
            oam: [0; 160],
            lcdc: 0,
            stat: 0,
            scy: 0,
            scx: 0,
            ly: 0,
            lyc: 0,
            dma: 0,
            pallete: 0,
            opb0: 0,
            opb1: 0,
            wy: 0,
            wx: 0,
            dma_transfer: false,
            stat_int: false,
            sprite_list: Vec::<Sprite>::new(),
            oam_pixel_fifo: VecDeque::new(),
            bg_pixel_fifo: VecDeque::new(),
        };
    }

    // This will only handle io related to ppu
    pub fn read_ppu_io(self: &Self, addr: u16) -> u8 {
        return match addr {
            LCDC_REG => self.lcdc,
            STAT_REG => self.stat,
            SCY_REG => self.scy,
            SCX_REG => self.scx,
            LY_REG => self.ly,
            LYC_REG => self.lyc,
            DMA_REG => self.dma,
            PALLETE_REG => self.pallete,
            OPB0_REG => self.opb0,
            OPB1_REG => self.opb1,
            WY_REG => self.wx,
            WX_REG => self.wy,
            _ => panic!("PPU IO does not handle reads from: {:04X}", addr),
        };
    }

    // Double check that writing to these registers is okay
    pub fn write_ppu_io(self: &mut Self, addr: u16, data: u8) {
        match addr {
            LCDC_REG => self.lcdc = data,
            STAT_REG => {
                self.stat = (data & 0b0111_1000) | (self.stat & 0b0000_0111);
                if self.ly_stat_enable() && self.ly_compare() {
                    self.request_stat();
                }
            }
            SCY_REG => self.scy = data,
            SCX_REG => self.scx = data,
            LY_REG => return, // read only
            LYC_REG => {
                self.lyc = data;
                self.update_stat(self.ly_compare());
            }
            DMA_REG => self.dma = data,
            PALLETE_REG => self.pallete = data,
            OPB0_REG => self.opb0 = data,
            OPB1_REG => self.opb1 = data,
            WY_REG => self.wx = data,
            WX_REG => self.wy = data,
            _ => panic!("PPU IO does not handle writes to: {:04X}", addr),
        }
    }

    pub fn set_ly(self: &mut Self, val: u8) {
        if val >= 154 {
            panic!("ly register cannot be greater than 154 - ly: {}", val);
        }
        self.ly = val;
        self.update_stat(self.ly_compare());
    }

    pub fn get_dma_dest(self: &Self) -> u16 {
        return (self.dma as u16) * 0x0100;
    }

    pub fn is_dma_transfer(self: &Self) -> bool {
        return self.dma_transfer;
    }

    pub fn ly_compare(self: &Self) -> bool {
        return self.lyc == self.ly;
    }

    pub fn update_stat(self: &mut Self, equal: bool) {
        if equal {
            self.stat = self.stat | 0b0000_0100;
            if self.ly_stat_enable() {
                self.request_stat();
            }
        } else {
            self.stat = self.stat & 0b1111_1011;
        }
    }

    pub fn ly_stat_enable(self: &Self) -> bool {
        return (self.stat & 0x40) == 0x40;
    }

    // when i_fired is unset, we need to somehow update stat_int to false
    // or on adv_cycles do the comparison to see if stat needs to be set
    // or unset rather than here.
    pub fn request_stat(self: &mut Self) {
        self.stat_int = true;
    }

    pub fn set_stat_mode(self: &mut Self, mode: u8) {
        self.stat = (self.stat & 0b0111_1100) | mode;
    }

    pub fn get_lcd_mode(self: &Self) -> u8 {
        return self.stat & 0x03;
    }

    // When bit 0 is cleared, the background and window become white (disabled) and
    // and the window display bit is ignored.
    pub fn is_bgw_enabled(self: &Self) -> bool {
        return (self.lcdc & 0x01) == 0x01;
    }

    // Are sprites enabled or not (bit 1 of lcdc)
    pub fn is_obj_enabled(self: &Self) -> bool {
        return (self.lcdc & 0x02) == 0x02;
    }

    // Are sprites a single tile or 2 stacked vertically (bit 2 of lcdc)
    pub fn is_big_sprite(self: &Self) -> bool {
        return (self.lcdc & 0x04) == 0x04;
    }

    // Bit 3 controls what area to look for the bg tile map area
    pub fn get_bg_tile_map_area(self: &Self) -> u8 {
        return self.lcdc & 0x08;
    }

    // Bit4 of lcdc gives Background and Window Tile data area
    // 1 will mean indexing from 0x8000, and 0 will mean indexing from 0x8800
    pub fn get_addr_mode(self: &Self) -> bool {
        return (self.lcdc & 0x10) == 0x10;
    }

    // Bit 5 controls whether the window is displayed or not.
    // Can be overriden by bit 0 hence the call to is_bgw_enabled
    pub fn is_window_enabled(self: &Self) -> bool {
        return ((self.lcdc & 0x20) == 0x20) && self.is_bgw_enabled();
    }

    // Bit 6 controls what area to look for the window tile map area
    pub fn get_window_tile_map_area(self: &Self) -> u8 {
        return self.lcdc & 0x40;
    }

    // LCD and PPU enabled when bit 7 of lcdc register is 1
    pub fn is_ppu_enabled(self: &Self) -> bool {
        return (self.lcdc & 0x80) == 0x80;
    }

    // Specify the top left coordinate of the visible 160x144 pixel area
    // within the 256x256 pixel background map. Returned as (x, y)
    pub fn get_scx_scy(self: &Self) -> (u8, u8) {
        return (self.scx, self.scx);
    }

    pub fn get_window_pos(self: &Self) -> (u8, u8) {
        return (self.wx, self.wy);
    }
}

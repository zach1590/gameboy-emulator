// For the cgb specific io we will continue to write them to Io rather than here

use super::oam_search::Sprite;
use std::collections::VecDeque;

pub const LCDC_REG: u16 = 0xFF40;
pub const STAT_REG: u16 = 0xFF41; // LCD Status
pub const SCY_REG: u16 = 0xFF42; // Used to scroll the background
pub const SCX_REG: u16 = 0xFF43;
pub const LY_REG: u16 = 0xFF44;
pub const LYC_REG: u16 = 0xFF45;
pub const DMA_REG: u16 = 0xFF46;
pub const BGP_REG: u16 = 0xFF47; // Background Palette
pub const OBP0_REG: u16 = 0xFF48; // Sprite Palette
pub const OBP1_REG: u16 = 0xFF49; // Sprite Palette
pub const WY_REG: u16 = 0xFF4A; // Top left coordinates of the window
pub const WX_REG: u16 = 0xFF4B; // Think this is only important when drawing

pub const OAM_START: u16 = 0xFE00;
pub const OAM_END: u16 = 0xFE9F;
pub const VRAM_START: u16 = 0x8000;
pub const VRAM_END: u16 = 0x9FFF;
pub const PPUIO_START: u16 = 0xFF40;
pub const PPUIO_END: u16 = 0xFF4B;

pub const LY_MAX: u8 = 153;
pub const DMA_MAX_CYCLES: u16 = 159;

// Should be ARGB888 but is being read as BGRA8888
// Maybe its expecting it in little endian format?
pub const COLORS: [[u8; 4]; 4] = [
    [0xF0, 0xF8, 0xF8, 0xFF], // #F8F8F0    // FF FF FF FF
    [0xD0, 0xDA, 0xE7, 0xFF], // #E7DAD0    // AA AA AA FF
    [0x9E, 0x91, 0xE0, 0xFF], // #E0919E    // 55 55 55 FF
    [0x98, 0x8A, 0xC9, 0xFF], // #C98A98    // 00 00 00 FF
];
pub const BYTES_PER_PIXEL: usize = 4;

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
    pub bgp: u8,           // 0xFF47
    pub obp0: u8,          // 0xFF48
    pub obp1: u8,          // 0xFF49
    pub wy: u8,            // 0xFF4A
    pub wx: u8,            // 0xFF4B
    pub dma_transfer: bool,
    pub dma_cycles: usize,
    pub dma_delay_cycles: usize,
    pub stat_int: bool,
    pub sprite_list: Vec<Sprite>,
    pub oam_pixel_fifo: VecDeque<u8>,
    pub bg_pixel_fifo: VecDeque<u8>,
    pub bg_colors: [[u8; 4]; 4],
    pub obp0_colors: [[u8; 4]; 4],
    pub obp1_colors: [[u8; 4]; 4],
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
            bgp: 0,
            obp0: 0,
            obp1: 0,
            wy: 0,
            wx: 0,
            dma_transfer: false,
            dma_cycles: 0,
            dma_delay_cycles: 0,
            stat_int: false,
            sprite_list: Vec::<Sprite>::new(),
            oam_pixel_fifo: VecDeque::new(),
            bg_pixel_fifo: VecDeque::new(),
            bg_colors: COLORS,
            obp0_colors: COLORS,
            obp1_colors: COLORS,
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
            BGP_REG => self.bgp,
            OBP0_REG => self.obp0,
            OBP1_REG => self.obp1,
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
            DMA_REG => {
                self.dma = data;
                self.dma_cycles = 0;
                self.dma_delay_cycles = 2;
            }
            BGP_REG => self.set_bg_palette(data),
            OBP0_REG => self.set_obp0_palette(data),
            OBP1_REG => self.set_obp1_palette(data),
            WY_REG => self.wx = data,
            WX_REG => self.wy = data,
            _ => panic!("PPU IO does not handle writes to: {:04X}", addr),
        }
    }

    pub fn dmg_init(self: &mut Self) {
        self.lcdc = 0x91;
        self.stat = 0x85;
        self.scy = 0x00;
        self.scx = 0x00;
        self.ly = 0x00;
        self.lyc = 0x00;
        self.dma = 0xFF;
        self.bgp = 0xFC;
        self.obp0 = 0x00; // Unitialized (0x00 or 0xFF)
        self.obp1 = 0x00; // Unitialized (0x00 or 0xFF)
        self.wy = 0x00;
        self.wx = 0x00;
    }

    pub fn set_ly(self: &mut Self, val: u8) {
        if val >= 154 {
            panic!("ly register cannot be greater than 154 - ly: {}", val);
        }
        self.ly = val;
        self.update_stat(self.ly_compare());
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

    // Im guessing the reason to assign a color to each index
    // and not have them be static is to allow for stuff like
    // inverting colors or making everything the same color
    // to make something like a silohoette appear.
    fn set_bg_palette(self: &mut Self, data: u8) {
        self.bgp = data;
        self.bg_colors[0] = COLORS[usize::from(data & 0x03)];
        self.bg_colors[1] = COLORS[usize::from((data >> 2) & 0x03)];
        self.bg_colors[2] = COLORS[usize::from((data >> 4) & 0x03)];
        self.bg_colors[3] = COLORS[usize::from((data >> 6) & 0x03)];
    }

    fn set_obp0_palette(self: &mut Self, mut data: u8) {
        self.obp0 = data;
        data = data & 0x0FC; // For sprites color index 0 should be transparent
        self.obp0_colors[0] = COLORS[usize::from(data & 0x03)];
        self.obp0_colors[1] = COLORS[usize::from((data >> 2) & 0x03)];
        self.obp0_colors[2] = COLORS[usize::from((data >> 4) & 0x03)];
        self.obp0_colors[3] = COLORS[usize::from((data >> 6) & 0x03)];
    }

    fn set_obp1_palette(self: &mut Self, mut data: u8) {
        self.obp1 = data;
        data = data & 0x0FC; // For sprites color index 0 should be transparent
        self.obp1_colors[0] = COLORS[usize::from(data & 0x03)];
        self.obp1_colors[1] = COLORS[usize::from((data >> 2) & 0x03)];
        self.obp1_colors[2] = COLORS[usize::from((data >> 4) & 0x03)];
        self.obp1_colors[3] = COLORS[usize::from((data >> 6) & 0x03)];
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
    // Returns the start and end address of vram containing the 32x32 tile map
    pub fn get_bg_tile_map(self: &Self) -> (u16, u16) {
        return match (self.lcdc & 0x08) == 0x08 {
            false => (0x9800, 0x9BFF),
            true => (0x9C00, 0x9FFF),
        };
    }

    // Bit4 of lcdc gives Background and Window Tile data area
    // 1 will mean indexing from 0x8000, and 0 will mean addressing from 0x8800
    // However 8800 addressing actually means indexing from 0x9000
    pub fn get_addr_mode_start(self: &Self) -> u16 {
        return match (self.lcdc & 0x10) == 0x10 {
            true => 0x8000,
            false => 0x9000,
        };
    }

    // Bit 5 controls whether the window is displayed or not.
    // Can be overriden by bit 0 hence the call to is_bgw_enabled
    pub fn is_window_enabled(self: &Self) -> bool {
        return ((self.lcdc & 0x20) == 0x20) && self.is_bgw_enabled();
    }

    // Bit 6 controls what area to look for the window tile map area
    // Returns the start and end address of vram containing the 32x32 tile map
    pub fn get_window_tile_map(self: &Self) -> (u16, u16) {
        return match (self.lcdc & 0x40) == 0x40 {
            false => (0x9800, 0x9BFF),
            true => (0x9C00, 0x9FFF),
        };
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

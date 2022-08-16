// For the cgb specific io we will continue to write them to Io rather than here
use super::oam_search::Sprite;
use super::NUM_PIXEL_BYTES;
use std::collections::VecDeque;

pub const LCDC_REG: u16 = 0xFF40;
pub const STAT_REG: u16 = 0xFF41; // LCD Status
pub const SCY_REG: u16 = 0xFF42; // Used to scroll the background
pub const SCX_REG: u16 = 0xFF43;
pub const LY_REG: u16 = 0xFF44;
pub const LYC_REG: u16 = 0xFF45;
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
pub const UNUSED_START: u16 = 0xFEA0;
pub const UNUSED_END: u16 = 0xFEFF;

pub const LY_MAX: u8 = 153;
pub const DMA_MAX_CYCLES: u16 = 159;

// Should be ARGB888 but is being read as BGRA8888
// Maybe its expecting it in little endian format?
pub const COLORS: [[u8; 4]; 4] = [
    [0xF0, 0xF8, 0xF8, 0xFF], // #F8F8F0    // FF FF FF FF
    [0xD0, 0xDA, 0xE7, 0xFF], // #E7DAD0    // AA AA AA FF
    [0x9E, 0x91, 0xE0, 0xFF], // #E0919E    // 55 55 55 FF
    [0x75, 0x6C, 0x91, 0xFF], // #916C75    // 00 00 00 FF
];
pub const BYTES_PER_PIXEL: usize = 4;

// Theres a lot of stuff. Clean it up? Split up?
pub struct GpuMemory {
    pub pixels: [u8; NUM_PIXEL_BYTES],
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
    pub window_line_counter: u8,
    pub dma_transfer: bool,
    pub stat_int: bool,
    pub stat_low_to_high: bool,
    pub vblank_int: bool,
    pub dmg_stat_quirk: Option<u8>,
    pub dmg_stat_quirk_delay: bool,
    pub sprite_list: Vec<Sprite>,
    pub bg_pixel_fifo: VecDeque<[u8; 4]>,
    pub bg_colors: [[u8; 4]; 4],
    pub obp0_colors: [[u8; 4]; 4],
    pub obp1_colors: [[u8; 4]; 4],
}

impl GpuMemory {
    pub fn new() -> GpuMemory {
        return GpuMemory {
            pixels: [0; NUM_PIXEL_BYTES],
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
            window_line_counter: 0,
            dma_transfer: false,
            stat_int: false,
            stat_low_to_high: false,
            vblank_int: false,
            dmg_stat_quirk: None,
            dmg_stat_quirk_delay: false,
            sprite_list: Vec::<Sprite>::new(),
            bg_pixel_fifo: VecDeque::new(),
            bg_colors: COLORS.clone(),
            obp0_colors: COLORS.clone(),
            obp1_colors: COLORS.clone(),
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
                self.stat = 0x80 | (data & 0xE8) | (self.stat & 0x07);
                self.check_interrupt_sources();
            }
            SCY_REG => self.scy = data,
            SCX_REG => self.scx = data,
            LY_REG => return, // read only
            LYC_REG => {
                self.lyc = data;
                if self.is_ppu_enabled() {
                    // https://github.com/Gekkio/mooneye-test-suite/blob/main/acceptance/ppu/stat_lyc_onoff.s#L56
                    self.update_stat_ly(self.ly_compare());
                }
            }
            BGP_REG => self.set_bg_palette(data),
            OBP0_REG => self.set_obp0_palette(data),
            OBP1_REG => self.set_obp1_palette(data),
            WY_REG => self.wy = data,
            WX_REG => {
                // https://gbdev.io/pandocs/pixel_fifo.html#the-window  implement this eventually
                self.wx = data
            }
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
        self.ly = val;
        self.update_stat_ly(self.ly_compare());
    }

    // Check should also occur when LCD is shut down and enabled again
    // When the above occurs should also call update_stat_ly
    pub fn ly_compare(self: &Self) -> bool {
        return self.lyc == self.ly;
    }

    pub fn update_stat_ly(self: &mut Self, equal: bool) {
        if equal {
            self.stat = self.stat | 0b0000_0100;
        } else {
            self.stat = self.stat & 0b1111_1011;
        }
        self.check_interrupt_sources();
    }

    // Dont call this except on state transitions
    pub fn set_stat_mode(self: &mut Self, mode: u8) {
        if mode == 0x01 && self.ly == 144 {
            self.vblank_int = true;
        } else {
            self.vblank_int = false;
        }
        self.stat = (self.stat & 0b1111_1100) | mode; // Set the mode flag
        self.check_interrupt_sources();
    }

    // Only request interrupts on low to high
    pub fn check_interrupt_sources(self: &mut Self) {
        let mut new_stat_int = false;

        if self.lyc_int_match()
            || self.oam_int_match()
            || self.hblank_int_match()
            || self.vblank_int_match()
        {
            new_stat_int = true;
        }
        if !self.stat_int && new_stat_int {
            // The actual interrupt will be requested in adv_cyles
            self.stat_low_to_high = true;
        }
        self.stat_int = new_stat_int;
    }

    pub fn lyc_int_match(self: &mut Self) -> bool {
        let source = (self.stat & 0b0100_0000) == 0b0100_0000;
        let flag = (self.stat & 0b0000_0100) == 0b0000_0100;
        return source && flag;
    }

    pub fn oam_int_match(self: &mut Self) -> bool {
        let source = (self.stat & 0b0010_0000) == 0b0010_0000;
        let flag = (self.stat & 0b0000_0011) == 0b0000_0010;
        return source && flag;
    }

    pub fn hblank_int_match(self: &mut Self) -> bool {
        let source = (self.stat & 0b0000_1000) == 0b0000_1000;
        let flag = (self.stat & 0b0000_0011) == 0b0000_0000;
        return source && flag;
    }

    pub fn vblank_int_match(self: &mut Self) -> bool {
        let source = (self.stat & 0b0001_0000) == 0b0001_0000;
        let flag = (self.stat & 0b0000_0011) == 0b0000_0001;
        return source && flag;
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
        self.bg_colors[0] = COLORS[usize::from(data & 0x03)]; // Double check these bit manip
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
    pub fn is_spr_enabled(self: &Self) -> bool {
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
        return (self.lcdc & 0x20) == 0x20;
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

    pub fn is_window_visible(self: &Self) -> bool {
        return (self.ly >= self.wy)
            && ((self.ly as u16) < (self.wy as u16) + 144)
            && (self.wx <= 166)
            && (self.wy <= 143);
    }

    /* Just to make some things cleaner elsewhere */
    pub fn ly(self: &Self) -> usize {
        return self.ly as usize;
    }

    pub fn scx(self: &Self) -> usize {
        return self.scx as usize;
    }

    pub fn scy(self: &Self) -> usize {
        return self.scy as usize;
    }

    pub fn wx(self: &Self) -> usize {
        return self.wx as usize;
    }

    pub fn wy(self: &Self) -> usize {
        return self.wy as usize;
    }
}

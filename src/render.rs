/*
    A tile takes 16 bytes (384 tiles total)
    A tile has 8x8 pixels
    Tile data is held from 0x8000 - 0x97FF

    16 bytes is 128 bits
    8x8 pixels would be 64 bits if we had 1bit per pixel but
    We have each pixel as 2-bits (0-3 to represent 4 color)

    Rather than indexing vram specifically, we index by the tiles (0-383)
    Two addressing modes
    Sprites use 0x8000 with a u8 to address
    Background and Window can use the above or 0x9000 with an i8 (based on LCDC bit 4)
        If lcdc bit 4 is 0 => index from 0x9000 as signed
        IF lcdc bit 4 is 1 => index from 0x8000 as unsigned
    This allows us to use an 8bit value to index more than 255 tiles

    How to know if drawing a sprite or background/window tile?? (I guess based on LCDC)

    Tile Maps
    There are 2 tile maps and they are both 32x32 tiles
    0x9800 - 0x9BFF and 0x9C00 - 0x9FFF
    Each tile map contains the 1-byte indexes of the tiles to be displayed
    A tile is 8x8 pixels, so a map holds 256x256 pixels. (Only 160x144 are displayed)
    How to know which tile map we want at the moment?

    There is also a window internal line counter that is incremented when the
    window is visible

    Window is not scrollable
    Background is scrollable

    Sprite Attribute Table (OAM) is stored in 0xFE00 - 0xFE9F

*/

use super::memory::Memory;
use super::sprite;
use super::sprite::Sprite;
use super::io::Io;
use super::io::{ LCDC_REG, LY_REG, SCY_REG, SCX_REG, WY_REG, WX_REG };



pub struct Render {
    pixels: Vec<u8>, // Dont know if I'm using this yet
    vram: [u8; 8_192],      // 0x8000 - 0x9FFF
    spr_table: [u8; 160],    // OAM 0xFE00 - 0xFE9F  40 sprites, each takes 4 bytes
    pub oam_blocked: bool,
}

impl Render {
    pub fn new() -> Render {
        Render { 
            pixels: Vec::new(),
            vram: [0; 8_192],
            spr_table: [0; 160],    
            oam_blocked: false,
        }
    }

    pub fn read_byte(self: &Self, addr: u16) -> u8 {
        let byte = match addr {
            0x8000..=0x9FFF => self.vram[usize::from(addr - 0x8000)],
            0xFE00..=0xFE9F => self.spr_table[usize::from(addr - 0xFE00)],
            _ => panic!("OAM doesnt read from address: {:04X}", addr),
        };
        return byte;
    }

    pub fn write_byte(self: &mut Self, addr: u16, data: u8) {
        match addr {
            0x8000..=0x9FFF => self.vram[usize::from(addr - 0x8000)] = data,
            0xFE00..=0xFE9F => self.spr_table[usize::from(addr - 0xFE00)] = data,
            _ => panic!("OAM doesnt write to address: {:04X}", addr),
        }
    }

    pub fn handle_clocks(self: &mut Self, io: &mut Io, curr_cycles: usize) {}

    // Probably call from emulator.rs?
    pub fn update_screen(self: &mut Self, _io: &Io) {}

    // **(This probably wont be what we need)**
    // Returns a vector of the actual tiles we want to see on screen
    fn weave_tiles_from_map(self: &mut Self, map_no: u8, io: &Io) -> Vec<u8> {
        let mut tile_no;
        let mut tile;
        let mut all_tiles = Vec::new();
        let tile_indices = get_tile_map(map_no);

        for tile_index in (tile_indices.0)..=tile_indices.1 {
            tile_no = self.read_byte(tile_index);
            tile = self.weave_tile_from_index(tile_no, io);
            all_tiles.append(&mut tile);
        }

        return all_tiles;   // all the tiles that were specified by the tile map (256x256 pixels)
    }

    // Takes the index of a tile and returns the 
    // result is a vector of 64 bytes (8x8 pixels). Each byte is a pixel represented by a color (0-3)
    fn weave_tile_from_index(self: &mut Self, tile_no: u8, io: &Io) -> Vec<u8> {
        let addr = calculate_addr(tile_no, io);
        let mut tile: Vec<u8> = Vec::new();

        for i in (0..=15).step_by(2) {
            let byte0 = self.read_byte(addr + i);
            let byte1 = self.read_byte(addr + i + 1);
            let mut tile_row = weave_bytes(byte0, byte1);
            tile.append(&mut tile_row);
        }

        return tile;
    }

    // Each scanline does an OAM scan during which time we need to determine
    // which sprites should be displayed. (Max of 10)
    // Should be called on every scanline
    fn determine_sprites(self: &mut Self, io: &Io) -> Vec<Sprite> {
        let ly = io.get_ly();  // This does not change for a scanline
        return sprite::find_sprites(&self.spr_table, io, ly);
    }

    // Write multiple bytes into memory starting from location
    // This should only be used for tests
    pub fn write_bytes(self: &mut Self, location: u16, data: &Vec<u8>) {
        for (i, byte) in data.into_iter().enumerate() {
            self.write_byte(location + (i as u16), *byte);
        }
    }

}

// Returns the start and end address of vram containing the 32x32 tile map
// LCDC Bit 3 gives the background tile map area
// LCDC Bit 6 gives the window tile map area
fn get_tile_map(map_no: u8) -> (u16, u16) {
    match map_no {
        0 => (0x9800, 0x9BFF),
        1 => (0x9C00, 0x9FFF),
        _ => panic!("Can only select tile map 0 or 1"),
    }
}

// Takes the index of a tile (should be in the tile map) and returns the address
// that the data for this tile is stored in
fn calculate_addr(tile_no: u8, io: &Io) -> u16 {
    let lcdc = io.get_lcdc();
    let is_sprite = is_obj_enabled(lcdc);

    let addr: u16 = match is_sprite {
        true => 0x8000 + (u16::from(tile_no as u8) * 16),
        false => {
            let lcdc_b4 = get_addr_mode(lcdc);
            match lcdc_b4 {
                true => 0x8000 + (u16::from(tile_no as u8) * 16),
                false => {
                    let index = isize::from(tile_no as i8) * 16;
                    (0x9000 + index) as u16
                }
            }
        }
    };
    return addr;
}

// Will take 2 bytes and return an array of 8 values that are between 0-3
// weaves the bits together to form the correct output for graphics
// The two bit0s are concatenated to form the last value of the returned array
// The two bit7s are concatenated to form the first value of the returned array
// arg-byte0 should be the byte that is a lower address in memory
fn weave_bytes(byte0: u8, byte1: u8) -> Vec<u8> {
    let mut tile_row = Vec::new();
    for shift in 0..=7 {
        let p1 = (byte1 >> (7 - shift)) & 0x01;
        let p0 = (byte0 >> (7 - shift)) & 0x01;

        match (p1, p0) {
            (0, 0) => tile_row.push(0),
            (0, 1) => tile_row.push(1),
            (1, 0) => tile_row.push(2),
            (1, 1) => tile_row.push(3),
            _ => panic!(
                "Impossible pixel value: {}, {}",
                byte1 & shift,
                byte0 & shift
            ),
        }
    }
    return tile_row;
}

// When bit 0 is cleared, the background and window become white (disabled) and
// and the window display bit is ignored.
pub fn is_bgw_enabled(lcdc: u8) -> bool {
    return (lcdc & 0x01) == 0x01;
}

// Are sprites enabled or not (bit 1 of lcdc)
pub fn is_obj_enabled(lcdc: u8) -> bool {
    return (lcdc & 0x02) == 0x02;
}

// Are sprites a single tile or 2 stacked vertically (bit 2 of lcdc)
pub fn is_big_sprite(lcdc: u8) -> bool {
    return (lcdc & 0x04) == 0x04;
}

// Bit 3 controls what area to look for the bg tile map area
pub fn get_bg_tile_map_area(lcdc: u8) -> u8 {
    return lcdc & 0x08;
}

// Bit4 of lcdc gives Background and Window Tile data area
// 1 will mean indexing from 0x8000, and 0 will mean indexing from 0x8800
pub fn get_addr_mode(lcdc: u8) -> bool {
    return (lcdc & 0x10) == 0x10;
}

// Bit 5 controls whether the window is displayed or not. 
// Can be overriden by bit 0 hence the call to is_bgw_enabled
pub fn is_window_enabled(lcdc: u8) -> bool {
    return ((lcdc & 0x20) == 0x20) && is_bgw_enabled(lcdc);
}    

// Bit 6 controls what area to look for the window tile map area
pub fn get_window_tile_map_area(lcdc: u8) -> u8 {
    return lcdc & 0x40;
}

// LCD and PPU enabled when bit 7 of lcdc register is 1
pub fn is_ppu_enabled(lcdc: u8) -> bool {
    return (lcdc & 0x80) == 0x80;
}

// Specify the top left coordinate of the visible 160x144 pixel area
// within the 256x256 pixel background map. Returned as (x, y)
pub fn get_scx_scy(io: &Io) -> (u8, u8) {
    return (io.read_byte(SCX_REG), io.read_byte(SCY_REG));
}

pub fn get_window_pos(io: &Io) -> (u8, u8) {
    return (io.read_byte(WX_REG), io.read_byte(WY_REG))
}

#[cfg(test)]
#[path = "./tests/render_tests.rs"]
mod render_tests;
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

const LCDC_REG: u16 = 0xFF40;

const SCY_REG: u16 = 0xFF42; // Used to scroll the background
const SCX_REG: u16 = 0xFF43;

const WY_REG: u16 = 0xFF4A; // Top left coordinates of the window
const WX_REG: u16 = 0xFF4B; // Think this is only important when drawing

pub struct Render {
    pixels: Vec<u8>, // Dont know if I'm using this yet
}

impl Render {
    pub fn new() -> Render {
        Render { pixels: Vec::new() }
    }

    pub fn update_screen(self: &mut Self, _mem: &Memory) {}

    // Returns a reference to a slice of vram containing the 32x32 tile map
    // LCDC Bit 3 gives the background tile map area
    // LCDC Bit 6 gives the window tile map area
    fn get_tile_map(map_no: u8, mem: &Memory) -> &[u8] {
        match map_no {
            0 => &(mem.get_vram())[(0x9800 - 0x8000)..=(0x9BFF - 0x8000)],
            1 => &(mem.get_vram())[(0x9C00 - 0x8000)..=(0x9FFF - 0x8000)],
            _ => panic!("Can only select tile map 0 or 1"),
        }
    }

    // **(This probably wont be what we need)**
    // Returns a vector of the actual tiles we want to see on screen
    fn weave_tiles_from_map(map_no: u8, mem: &Memory) -> Vec<u8> {
        let mut all_tiles = Vec::new();
        let tile_indices = Render::get_tile_map(map_no, mem);

        for tile_no in tile_indices {
            let mut tile = Render::weave_tile_from_index(*tile_no, mem);
            all_tiles.append(&mut tile);
        }

        return all_tiles;
    }

    // Takes the index of a tile and returns the 
    // result is a vector of 64 bytes (8x8 pixels). Each byte is a pixel represented by a color (0-3)
    fn weave_tile_from_index(tile_no: u8, mem: &Memory) -> Vec<u8> {
        let addr = Render::calculate_addr(tile_no, mem);
        let mut tile: Vec<u8> = Vec::new();

        for i in (0..=15).step_by(2) {
            let byte0 = mem.read_byte(addr + i);
            let byte1 = mem.read_byte(addr + i + 1);
            let mut tile_row = Render::weave_bytes(byte0, byte1);
            tile.append(&mut tile_row);
        }

        return tile;
    }

    // Takes the index of a tile (should be in the tile map) and returns the address
    // that the data for this tile is stored in
    fn calculate_addr(tile_no: u8, mem: &Memory) -> u16 {
        let lcdc = Render::get_lcdc(mem);
        let is_sprite = Render::is_obj_enabled(lcdc);

        let addr: u16 = match is_sprite {
            true => 0x8000 + (u16::from(tile_no as u8) * 16),
            false => {
                let lcdc_b4 = Render::get_addr_mode(lcdc);
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

    // lcdc can be modified mid scanline (I dont know how??)
    // Maybe better to call this function from each of other helper methods?
    fn get_lcdc(mem: &Memory) -> u8 {
        return mem.read_byte(LCDC_REG);
    }

    // When bit 0 is cleared, the background and window become white (disabled) and
    // and the window display bit is ignored.
    fn is_bgw_enabled(lcdc: u8) -> bool {
        return (lcdc & 0x01) == 0x01;
    }

    // Are sprites enabled or not (bit 1 of lcdc)
    fn is_obj_enabled(lcdc: u8) -> bool {
        return (lcdc & 0x02) == 0x02;
    }

    // Are sprites a single tile or 2 stacked vertically (bit 2 of lcdc)
    fn is_big_sprite(lcdc: u8) -> bool {
        return (lcdc & 0x04) == 0x04;
    }

    // Bit 3 controls what area to look for the bg tile map area
    fn get_bg_tile_map_area(lcdc: u8) -> u8 {
        return lcdc & 0x08;
    }

    // Bit4 of lcdc gives Background and Window Tile data area
    // 1 will mean indexing from 0x8000, and 0 will mean indexing from 0x8800
    fn get_addr_mode(lcdc: u8) -> bool {
        return (lcdc & 0x10) == 0x10;
    }

    // Bit 5 controls whether the window is displayed or not. 
    // Can be overriden by bit 0 hence the call to is_bgw_enabled
    fn is_window_enabled(lcdc: u8) -> bool {
        return ((lcdc & 0x20) == 0x20) && Render::is_bgw_enabled(lcdc);
    }    

    // Bit 6 controls what area to look for the window tile map area
    fn get_window_tile_map_area(lcdc: u8) -> u8 {
        return lcdc & 0x40;
    }

    // LCD and PPU enabled when bit 7 of lcdc register is 1
    fn is_ppu_enabled(lcdc: u8) -> bool {
        return (lcdc & 0x80) == 0x80;
    }

    // Specify the top left coordinate of the visible 160x144 pixel area
    // within the 256x256 pixel background map. Returned as (x, y)
    fn get_scx_scy(mem: &Memory) -> (u8, u8) {
        return (mem.read_byte(SCX_REG), mem.read_byte(SCY_REG));
    }
}

#[cfg(test)]
#[test]
fn test_weave_bytes() {
    // Using the pandocs example
    // https://gbdev.io/pandocs/Tile_Data.html
    assert_eq!(
        Render::weave_bytes(0x3C, 0x7E),
        Vec::from([0, 2, 3, 3, 3, 3, 2, 0])
    );
    assert_eq!(
        Render::weave_bytes(0x42, 0x42),
        Vec::from([0, 3, 0, 0, 0, 0, 3, 0])
    );
    assert_eq!(
        Render::weave_bytes(0x42, 0x42),
        Vec::from([0, 3, 0, 0, 0, 0, 3, 0])
    );
    assert_eq!(
        Render::weave_bytes(0x42, 0x42),
        Vec::from([0, 3, 0, 0, 0, 0, 3, 0])
    );
    assert_eq!(
        Render::weave_bytes(0x7E, 0x5E),
        Vec::from([0, 3, 1, 3, 3, 3, 3, 0])
    );
    assert_eq!(
        Render::weave_bytes(0x7E, 0x0A),
        Vec::from([0, 1, 1, 1, 3, 1, 3, 0])
    );
    assert_eq!(
        Render::weave_bytes(0x7C, 0x56),
        Vec::from([0, 3, 1, 3, 1, 3, 2, 0])
    );
    assert_eq!(
        Render::weave_bytes(0x38, 0x7C),
        Vec::from([0, 2, 3, 3, 3, 2, 0, 0])
    );
}

#[test]
fn test_get_lcdc_b4() {
    let mut mem = Memory::new();

    mem.write_byte(LCDC_REG, 0x07);
    assert_eq!(Render::get_addr_mode(Render::get_lcdc(&mem)), false);

    mem.write_byte(LCDC_REG, 0xFF);
    assert_eq!(Render::get_addr_mode(Render::get_lcdc(&mem)), true);

    mem.write_byte(LCDC_REG, 0xEF);
    assert_eq!(Render::get_addr_mode(Render::get_lcdc(&mem)), false);

    mem.write_byte(LCDC_REG, 0x0F);
    assert_eq!(Render::get_addr_mode(Render::get_lcdc(&mem)), false);
}

#[test]
fn test_weave_tile_from_index_b4_as_1() {
    let mut mem = Memory::new();
    mem.write_byte(LCDC_REG, 0x17);

    let tile_no: u8 = 134;
    let addr = (134 * 16) + 0x8000;
    mem.write_bytes(
        addr,
        &Vec::from([
            0x3C, 0x7E, 0x42, 0x42, 0x42, 0x42, 0x42, 0x42, 0x7E, 0x5E, 0x7E, 0x0A, 0x7C, 0x56,
            0x38, 0x7C,
        ]),
    );
    let tile = Render::weave_tile_from_index(tile_no, &mem);
    assert_eq!(
        tile,
        Vec::from([
            0, 2, 3, 3, 3, 3, 2, 0, 0, 3, 0, 0, 0, 0, 3, 0, 0, 3, 0, 0, 0, 0, 3, 0, 0, 3, 0, 0, 0,
            0, 3, 0, 0, 3, 1, 3, 3, 3, 3, 0, 0, 1, 1, 1, 3, 1, 3, 0, 0, 3, 1, 3, 1, 3, 2, 0, 0, 2,
            3, 3, 3, 2, 0, 0
        ])
    );
}

#[test]
fn test_weave_tile_from_index_b4_as_0() {
    let mut mem = Memory::new();
    mem.write_byte(LCDC_REG, 0x07);

    let tile_no: u8 = i8::from(-0x74) as u8;
    let addr = 0x9000 - (0x74 * 16);
    mem.write_bytes(
        addr,
        &Vec::from([
            0x3C, 0x7E, 0x42, 0x42, 0x42, 0x42, 0x42, 0x42, 0x7E, 0x5E, 0x7E, 0x0A, 0x7C, 0x56,
            0x38, 0x7C,
        ]),
    );
    let tile = Render::weave_tile_from_index(tile_no, &mem);
    assert_eq!(
        tile,
        Vec::from([
            0, 2, 3, 3, 3, 3, 2, 0, 0, 3, 0, 0, 0, 0, 3, 0, 0, 3, 0, 0, 0, 0, 3, 0, 0, 3, 0, 0, 0,
            0, 3, 0, 0, 3, 1, 3, 3, 3, 3, 0, 0, 1, 1, 1, 3, 1, 3, 0, 0, 3, 1, 3, 1, 3, 2, 0, 0, 2,
            3, 3, 3, 2, 0, 0
        ])
    );
}
// Get another test with if its a sprite vs background/window

#[test]
fn test_get_tile_map1() {
    let mut mem = Memory::new();
    let mut vram_data = Vec::new();
    let mut mod255;

    for i in 0..(32 * 32) {
        mod255 = (i64::from(i) % 255) as u8;
        vram_data.push(mod255);
    }
    mem.write_bytes(0x9800, &vram_data);
    let tile_map = Render::get_tile_map(0, &mem);
    assert_eq!(Vec::from(tile_map), vram_data);
    assert_eq!(tile_map.len(), 0x0400);
}

#[test]
fn test_get_tile_map2() {
    let mut mem = Memory::new();
    let mut vram_data = Vec::new();
    let mut mod255;

    for i in 0..(32 * 32) {
        mod255 = (i64::from(i) % 255) as u8;
        vram_data.push(mod255);
    }
    mem.write_bytes(0x9C00, &vram_data);
    let tile_map = Render::get_tile_map(1, &mem);
    assert_eq!(Vec::from(tile_map), vram_data);
    assert_eq!(tile_map.len(), 0x0400);
}
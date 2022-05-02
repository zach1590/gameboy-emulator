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
*/

use super::memory::Memory;

const LDLC_REG: u16 = 0xFF40;
const SCY_REG: u16 = 0xFF42;
const SCX_REG: u16 = 0xFF43;

pub struct Render {
    pixels: Vec<u8>, // Dont know if I'm using this yet
}

impl Render {
    pub fn new() -> Render {
        Render { pixels: Vec::new() }
    }

    pub fn update_screen(self: &mut Self, _mem: &Memory) {}

    // Returns a reference to a slice of vram containing the 32x32 tile map
    fn get_tile_map(map_no: u8, mem: &Memory) -> &[u8] {
        match map_no {
            0 => &(mem.get_vram())[(0x9800 - 0x8000)..=(0x9BFF - 0x8000)],
            1 => &(mem.get_vram())[(0x9C00 - 0x8000)..=(0x9FFF - 0x8000)],
            _ => panic!("Can only select tile map 0 or 1"),
        }
    }

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

    fn calculate_addr(tile_no: u8, mem: &Memory) -> u16 {
        let is_sprite = false; // Just for now untill I figure out the correct way
        let ldlc = Render::get_ldlc(mem);

        let addr: u16 = match is_sprite {
            true => 0x8000 + (u16::from(tile_no as u8) * 16),
            false => {
                let ldlc_b4 = Render::get_addr_mode(ldlc);
                match ldlc_b4 {
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

    // ldlc can be modified mid scanline (I dont know how??)
    // Maybe better to call this function from each of other helper methods?
    fn get_ldlc(mem: &Memory) -> u8 {
        return mem.read_byte(LDLC_REG);
    }

    fn get_addr_mode(ldlc: u8) -> bool {
        return ((ldlc >> 4) & 0x01) == 0x01;
    }

    // When bit 0 is cleared, the background and window become white (disabled) and
    // and the window display bit is ignored.
    fn is_bg_enabled(ldlc: u8) -> bool {
        return (ldlc & 0x01) == 0x01;
    }

    // Controls whether the window is displayed or not. Can be overriden by bit 0
    // hence the call to is_bg_enabled
    fn is_window_enabled(ldlc: u8) -> bool {
        return (((ldlc >> 5) & 0x01) == 0x01) & Render::is_bg_enabled(ldlc);
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
fn test_get_ldlc_b4() {
    let mut mem = Memory::new();

    mem.write_byte(LDLC_REG, 0x07);
    assert_eq!(Render::get_addr_mode(Render::get_ldlc(&mem)), false);

    mem.write_byte(LDLC_REG, 0xFF);
    assert_eq!(Render::get_addr_mode(Render::get_ldlc(&mem)), true);

    mem.write_byte(LDLC_REG, 0xEF);
    assert_eq!(Render::get_addr_mode(Render::get_ldlc(&mem)), false);

    mem.write_byte(LDLC_REG, 0x0F);
    assert_eq!(Render::get_addr_mode(Render::get_ldlc(&mem)), false);
}

#[test]
fn test_weave_tile_from_index_b4_as_1() {
    let mut mem = Memory::new();
    mem.write_byte(LDLC_REG, 0x17);

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
    mem.write_byte(LDLC_REG, 0x07);

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

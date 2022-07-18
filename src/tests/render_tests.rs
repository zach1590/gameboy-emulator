use super::*;

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

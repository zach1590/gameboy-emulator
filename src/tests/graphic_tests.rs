use super::*;
use super::gpu_memory::LCDC_REG;
#[test]
fn test_weave_bytes() {
    // Using the pandocs example
    // https://gbdev.io/pandocs/Tile_Data.html
    assert_eq!(
        weave_bytes(0x3C, 0x7E),
        Vec::from([0, 2, 3, 3, 3, 3, 2, 0])
    );
    assert_eq!(
        weave_bytes(0x42, 0x42),
        Vec::from([0, 3, 0, 0, 0, 0, 3, 0])
    );
    assert_eq!(
        weave_bytes(0x42, 0x42),
        Vec::from([0, 3, 0, 0, 0, 0, 3, 0])
    );
    assert_eq!(
        weave_bytes(0x42, 0x42),
        Vec::from([0, 3, 0, 0, 0, 0, 3, 0])
    );
    assert_eq!(
        weave_bytes(0x7E, 0x5E),
        Vec::from([0, 3, 1, 3, 3, 3, 3, 0])
    );
    assert_eq!(
        weave_bytes(0x7E, 0x0A),
        Vec::from([0, 1, 1, 1, 3, 1, 3, 0])
    );
    assert_eq!(
        weave_bytes(0x7C, 0x56),
        Vec::from([0, 3, 1, 3, 1, 3, 2, 0])
    );
    assert_eq!(
        weave_bytes(0x38, 0x7C),
        Vec::from([0, 2, 3, 3, 3, 2, 0, 0])
    );
}

#[test]
fn test_get_lcdc_b4() {
    let mut gpu_mem = GpuMemory::new();

    gpu_mem.write_ppu_io(LCDC_REG, 0x07);
    assert_eq!(gpu_mem.get_addr_mode(), false);

    gpu_mem.write_ppu_io(LCDC_REG, 0xFF);
    assert_eq!(gpu_mem.get_addr_mode(), true);

    gpu_mem.write_ppu_io(LCDC_REG, 0xEF);
    assert_eq!(gpu_mem.get_addr_mode(), false);

    gpu_mem.write_ppu_io(LCDC_REG, 0x0F);
    assert_eq!(gpu_mem.get_addr_mode(), false);
}

#[test]
fn test_weave_tile_from_index_b4_as_1() {

    let mut graphics = Graphics::new();
    graphics.write_io_byte(LCDC_REG, 0x17);

    let tile_no: u8 = 134;
    let addr = (134 * 16) + 0x8000;
    graphics.write_bytes(
        addr,
        &Vec::from([
            0x3C, 0x7E, 0x42, 0x42, 0x42, 0x42, 0x42, 0x42, 0x7E, 0x5E, 0x7E, 0x0A, 0x7C, 0x56,
            0x38, 0x7C,
        ]),
    );
    let tile = graphics.weave_tile_from_index(tile_no);
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

    let mut graphics = Graphics::new();
    graphics.write_io_byte(LCDC_REG, 0x07);

    let tile_no: u8 = i8::from(-0x74) as u8;
    let addr = 0x9000 - (0x74 * 16);
    graphics.write_bytes(
        addr,
        &Vec::from([
            0x3C, 0x7E, 0x42, 0x42, 0x42, 0x42, 0x42, 0x42, 0x7E, 0x5E, 0x7E, 0x0A, 0x7C, 0x56,
            0x38, 0x7C,
        ]),
    );
    let tile = graphics.weave_tile_from_index(tile_no);
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
    let mut graphics = Graphics::new();
    let mut vram_data = Vec::new();
    let mut mod255;

    for i in 0..(32 * 32) {
        mod255 = (i64::from(i) % 255) as u8;
        vram_data.push(mod255);
    }
    graphics.write_bytes(0x9800, &vram_data);
    let tile_map = get_tile_map(0);

    let mut tile_index;
    for i in tile_map.0..=tile_map.1 {
        tile_index = graphics.read_byte(i);
        assert_eq!(tile_index, vram_data[(i-tile_map.0) as usize]);
    }
}

#[test]
fn test_get_tile_map2() {
    let mut graphics = Graphics::new();
    let mut vram_data = Vec::new();
    let mut mod255;

    for i in 0..(32 * 32) {
        mod255 = (i64::from(i) % 255) as u8;
        vram_data.push(mod255);
    }
    graphics.write_bytes(0x9C00, &vram_data);

    let tile_map = get_tile_map(1);
    let mut tile_index;
    for i in tile_map.0..=tile_map.1 {
        tile_index = graphics.read_byte(i);
        assert_eq!(tile_index, vram_data[(i-tile_map.0) as usize]);
    }

}

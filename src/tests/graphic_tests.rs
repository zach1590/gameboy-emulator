use super::gpu_memory::LCDC_REG;
use super::*;
#[test]
fn test_weave_bytes() {
    // Using the pandocs example
    // https://gbdev.io/pandocs/Tile_Data.html
    assert_eq!(weave_bytes(0x3C, 0x7E), Vec::from([0, 2, 3, 3, 3, 3, 2, 0]));
    assert_eq!(weave_bytes(0x42, 0x42), Vec::from([0, 3, 0, 0, 0, 0, 3, 0]));
    assert_eq!(weave_bytes(0x42, 0x42), Vec::from([0, 3, 0, 0, 0, 0, 3, 0]));
    assert_eq!(weave_bytes(0x42, 0x42), Vec::from([0, 3, 0, 0, 0, 0, 3, 0]));
    assert_eq!(weave_bytes(0x7E, 0x5E), Vec::from([0, 3, 1, 3, 3, 3, 3, 0]));
    assert_eq!(weave_bytes(0x7E, 0x0A), Vec::from([0, 1, 1, 1, 3, 1, 3, 0]));
    assert_eq!(weave_bytes(0x7C, 0x56), Vec::from([0, 3, 1, 3, 1, 3, 2, 0]));
    assert_eq!(weave_bytes(0x38, 0x7C), Vec::from([0, 2, 3, 3, 3, 2, 0, 0]));
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

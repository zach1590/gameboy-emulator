use super::gpu_memory::GpuMemory;

pub struct Sprite {
    ypos: u8,
    xpos: u8,
    tile_index: u8,
    bgw_ontop: bool,
    flip_y: bool,
    flip_x: bool,
    palette_no: bool,
    big: bool,
}

impl Sprite {
    pub fn new(sprite_bytes: &[u8], big: bool) -> Sprite {
        return Sprite {
            ypos: sprite_bytes[0],
            xpos: sprite_bytes[1],
            tile_index: sprite_bytes[2],
            bgw_ontop: (sprite_bytes[3] >> 7) & 0x01 == 0x01,
            flip_y: (sprite_bytes[3] >> 6) & 0x01 == 0x01,
            flip_x: (sprite_bytes[3] >> 5) & 0x01 == 0x01,
            palette_no: (sprite_bytes[3] >> 4) & 0x01 == 0x01,
            big: big,
        }
    }
}

// Should be called on every scanline
// How does lcdc get modified mid scanline??????
pub fn find_sprites(gpu_mem: &GpuMemory, sprites: &mut Vec<Sprite>, num_entries: usize) {
    // let mut sprites: Vec<Sprite> = Vec::new();
    let mut lcdc;
    let mut ypos;
    let mut big_sprite;

    for i in (0..gpu_mem.oam.len()).step_by(4) {

        //This value (lcdc) changes mid scanline
        lcdc = gpu_mem.lcdc;
        big_sprite = gpu_mem.is_big_sprite();
        ypos = gpu_mem.oam[i];

        if ypos == 0 || ypos >= 160 || (!big_sprite && ypos <= 8) {
            continue;
        } 

        if gpu_mem.ly == ypos { // Should this be a range of numbers for ypos?
            sprites.push(Sprite::new(&gpu_mem.oam[i..i+4], big_sprite));
        }

        if sprites.len() == 10 { break; }
    }
}
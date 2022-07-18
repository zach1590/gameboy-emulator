use super::memory::Memory;
pub struct Sprite {
    ypos: u8,
    xpos: u8,
    tile_index: u8,
    bgw_ontop: bool,
    flip_y: bool,
    flip_x: bool,
    palette_no: bool,
}

impl Sprite {
    pub fn new(sprite_bytes: &[u8]) -> Sprite {
        return Sprite {
            ypos: sprite_bytes[0],
            xpos: sprite_bytes[1],
            tile_index: sprite_bytes[2],
            bgw_ontop: (sprite_bytes[3] >> 7) & 0x01 == 0x01,
            flip_y: (sprite_bytes[3] >> 6) & 0x01 == 0x01,
            flip_x: (sprite_bytes[3] >> 5) & 0x01 == 0x01,
            palette_no: (sprite_bytes[3] >> 4) & 0x01 == 0x01,
        }
    }
}

// Should be called on every scanline
// How does lcdc get modified mid scanline??????
pub fn find_sprites(spr_table: &[u8], mem: &Memory, ly: u8) -> Vec<Sprite> {
    let mut sprites: Vec<Sprite> = Vec::new();
    let mut lcdc;

    for i in (0..spr_table.len()).step_by(4) {

        //This value (lcdc) changes mid scanline
        lcdc = mem.get_lcdc();
        
        // Check if the sprite will actually be on the screen based on the y posisiton
        // if so add it to the list
        sprites.push(Sprite::new(&spr_table[i..i+4]));
    }

    return sprites;
}
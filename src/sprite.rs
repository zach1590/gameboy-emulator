use super::io::Io;
use super::graphics;
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
pub fn find_sprites(spr_table: &[u8], io: &Io, ly: u8) -> Vec<Sprite> {
    let mut sprites: Vec<Sprite> = Vec::new();
    let mut lcdc;
    let mut ypos;
    let mut big_sprite;

    for i in (0..spr_table.len()).step_by(4) {

        //This value (lcdc) changes mid scanline
        lcdc = io.get_lcdc();
        big_sprite = graphics::is_big_sprite(lcdc);
        ypos = spr_table[i];

        if ypos == 0 || ypos >= 160 || (!big_sprite && ypos <= 8) {
            continue;
        } 

        if ly == ypos { // Should this be a range of numbers for ypos?
            sprites.push(Sprite::new(&spr_table[i..i+4], big_sprite));
        }

        if sprites.len() == 10 { break; }
    }

    return sprites;
}
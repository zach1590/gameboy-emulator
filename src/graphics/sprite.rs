use super::gpu_memory::GpuMemory;
use super::ppu::OamSearch;

pub struct Sprite {
    ypos: u8,
    xpos: u8,
    tile_index: u8, // 0x00 - 0xFF indexing from 0x8000 - 0x8FFF
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
            tile_index: if big {
                // https://gbdev.io/pandocs/OAM.html#byte-2---tile-index
                sprite_bytes[2] & 0xFE
            } else {
                sprite_bytes[2]
            },
            bgw_ontop: (sprite_bytes[3] >> 7) & 0x01 == 0x01,
            flip_y: (sprite_bytes[3] >> 6) & 0x01 == 0x01,
            flip_x: (sprite_bytes[3] >> 5) & 0x01 == 0x01,
            palette_no: (sprite_bytes[3] >> 4) & 0x01 == 0x01,
            big: big,
        };
    }
}

// Number of sprites added for the scanline
// How many entries we will process
// The entry we last left off at
pub fn find_sprites(
    gpu_mem: &mut GpuMemory,
    sl_sprites_added: &mut usize,
    proc_howmany: usize,
    num_entries: usize,
) {
    let mut ypos;
    let mut big_sprite;

    for i in 0..proc_howmany {
        let curr_entry = (num_entries + i) * 4;

        // Added 10 sprites on the current scanline so done searching
        // Reached 40 entries added in list so done searching for more
        // 40 entries (0 - 39) so at the end of of OAM memory
        if *sl_sprites_added == OamSearch::MAX_SCANLINE_SPRITES
            || gpu_mem.sprite_list.len() == OamSearch::MAX_SPRITES
            || curr_entry >= OamSearch::OAM_LENGTH
        {
            break;
        }

        ypos = if gpu_mem.dma_transfer {
            // if dma_transfer is in progress then values read from oam
            // will be 0xFF. 0xFF would result in the sprite being off
            // the screen. Thus all sprites are off the screen on this run
            // so we can just break and exit the function. Similar logic
            // is needed in the PictureGeneration state
            break;
        } else {
            gpu_mem.oam[curr_entry]
        };

        big_sprite = gpu_mem.is_big_sprite();

        if ypos == 0 || ypos >= 160 || (!big_sprite && ypos <= 8) {
            continue;
        }

        // Should this be a range of numbers for ypos? (probably not or we would count the same
        // on multiple scanlines)
        if gpu_mem.ly == ypos {
            gpu_mem
                .sprite_list
                .push(Sprite::new(&gpu_mem.oam[i..i + 4], big_sprite));
            *sl_sprites_added += 1;
        }
    }
}

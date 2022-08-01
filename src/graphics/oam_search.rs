use super::gpu_memory::{GpuMemory, OAM_END, OAM_START, VRAM_END, VRAM_START};
use super::picture_generation::PictureGeneration;
use super::ppu::{PpuState, MODE_PICTGEN};
use std::collections::VecDeque;

// mode 2
pub struct OamSearch {
    // DMA transfer overrides mode-2 access to OAM.
    cycles_counter: usize,
    sl_sprites_added: usize, // Number of sprites added on the current scanline
}

impl OamSearch {
    pub const MAX_SPRITES: usize = 40;
    pub const MAX_SCANLINE_SPRITES: usize = 10;
    pub const OAM_LENGTH: usize = 160;
    pub const MAX_CYCLES: usize = 80;

    // Each scanline does an OAM scan during which time we need to determine
    // which sprites should be displayed. (Max of 10 per scan line).
    pub fn new() -> PpuState {
        return PpuState::OamSearch(OamSearch {
            cycles_counter: 0,
            sl_sprites_added: 0,
        });
    }

    fn next(self: Self, gpu_mem: &mut GpuMemory) -> PpuState {
        if self.cycles_counter < OamSearch::MAX_CYCLES {
            return PpuState::OamSearch(self);
        } else {
            gpu_mem.set_stat_mode(MODE_PICTGEN);

            //  https://gbdev.io/pandocs/pixel_fifo.html#mode-3-operation
            gpu_mem.bg_pixel_fifo = VecDeque::new();
            gpu_mem.oam_pixel_fifo = VecDeque::new();

            return PpuState::PictureGeneration(PictureGeneration::new(self.sl_sprites_added));
        }
    }

    pub fn render(mut self, gpu_mem: &mut GpuMemory, cycles: usize) -> PpuState {
        let entries_todo = cycles / 2;
        let entries_done = self.cycles_counter / 2;

        self.find_sprites(gpu_mem, entries_todo, entries_done);

        self.cycles_counter += cycles;
        return self.next(gpu_mem); // For Now
    }

    pub fn read_byte(self: &Self, gpu_mem: &GpuMemory, addr: u16) -> u8 {
        return match addr {
            VRAM_START..=VRAM_END => gpu_mem.vram[usize::from(addr - VRAM_START)],
            OAM_START..=OAM_END => 0xFF,
            0xFEA0..=0xFEFF => 0xFF,
            _ => panic!("PPU (O Search) doesnt read from address: {:04X}", addr),
        };
    }

    pub fn write_byte(self: &mut Self, gpu_mem: &mut GpuMemory, addr: u16, data: u8) {
        match addr {
            VRAM_START..=VRAM_END => gpu_mem.vram[usize::from(addr - VRAM_START)] = data,
            OAM_START..=OAM_END => return,
            0xFEA0..=0xFEFF => return,
            _ => panic!("PPU (O Search) doesnt write to address: {:04X}", addr),
        }
    }

    /*
        Double Check with this: (https://hacktix.github.io/GBEDG/ppu/)
        A sprite is only added to the buffer if all of the following conditions apply:

         - Sprite X-Position must be greater than 0
         - LY + 16 must be greater than or equal to Sprite Y-Position
         - LY + 16 must be less than Sprite Y-Position + Sprite Height (8 in Normal Mode, 16 in Tall-Sprite-Mode)
         - The amount of sprites already stored in the OAM Buffer must be less than 10
    */
    pub fn find_sprites(
        self: &mut Self,
        gpu_mem: &mut GpuMemory,
        entries_todo: usize,
        entries_done: usize,
    ) {
        let mut ypos;
        let mut xpos;
        let mut big_sprite;

        for i in 0..entries_todo {
            let curr_entry = (entries_done + i) * 4;

            // Added 10 sprites on the current scanline so done searching
            // Reached 40 entries added in list so done searching for more
            // 40 entries (0 - 39) so at the end of of OAM memory
            if self.sl_sprites_added == OamSearch::MAX_SCANLINE_SPRITES
                || gpu_mem.sprite_list.len() == OamSearch::MAX_SPRITES
                || curr_entry >= OamSearch::OAM_LENGTH
            {
                break;
            }

            // Would like to just use self.read_byte but that always returns 0xFF
            // since thats mostly for the cpu. So gotta do this instead.
            if gpu_mem.dma_transfer {
                ypos = 0xFF;
                xpos = 0xFF
            } else {
                ypos = gpu_mem.oam[curr_entry];
                xpos = gpu_mem.oam[curr_entry + 1];
            };

            big_sprite = gpu_mem.is_big_sprite();
            if ypos == 0 || ypos >= 160 || (!big_sprite && ypos <= 8) {
                continue;
            }

            // This should be a range
            if gpu_mem.ly == ypos {
                gpu_mem
                    .sprite_list
                    .push(Sprite::new(&gpu_mem.oam[i..i + 4], big_sprite));
                self.sl_sprites_added += 1;
            }
        }
    }
}

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

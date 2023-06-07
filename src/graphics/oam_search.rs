use super::picture_generation::PictureGeneration;
use super::ppu::{PpuState, MODE_PICTGEN};
use super::*;

// mode 2
pub struct OamSearch {
    cycles_counter: usize,
}

impl OamSearch {
    pub const MAX_CYCLES: usize = 80;
    const MAX_SPRITES: usize = 40;
    const MAX_SCANLINE_SPRITES: usize = 10;
    const OAM_LENGTH: usize = 160;

    // Each scanline does an OAM scan during which time we need to determine
    // which sprites should be displayed. (Max of 10 per scan line).
    pub fn new() -> PpuState {
        return PpuState::OamSearch(OamSearch { cycles_counter: 0 });
    }

    // oamsearch may return itself or picturegeneration
    fn next(self: Self, gpu_mem: &mut GpuMemory) -> PpuState {
        if self.cycles_counter < OamSearch::MAX_CYCLES {
            return PpuState::OamSearch(self);
        } else {
            gpu_mem.set_stat_mode(MODE_PICTGEN);

            // https://gbdev.io/pandocs/pixel_fifo.html#mode-3-operation
            gpu_mem.bg_pixel_fifo.clear();

            return PpuState::PictureGeneration(PictureGeneration::new());
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
            UNUSED_START..=UNUSED_END => 0xFF,
            _ => panic!("PPU (O Search) doesnt read from address: {:04X}", addr),
        };
    }

    pub fn write_byte(self: &mut Self, gpu_mem: &mut GpuMemory, addr: u16, data: u8) {
        match addr {
            VRAM_START..=VRAM_END => gpu_mem.vram[usize::from(addr - VRAM_START)] = data,
            OAM_START..=OAM_END => return,
            UNUSED_START..=UNUSED_END => return,
            _ => panic!("PPU (O Search) doesnt write to address: {:04X}", addr),
        }
    }

    /*
        Double Check with this: (https://hacktix.github.io/GBEDG/ppu/)
        A sprite is only added to the buffer if all of the following conditions apply: (I think +16 should be 0)

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
        let mut sprite_height;

        for i in 0..entries_todo {
            let curr_entry = (entries_done + i) * 4;

            if gpu_mem.sprite_list.len() == OamSearch::MAX_SCANLINE_SPRITES
                || curr_entry >= OamSearch::OAM_LENGTH
            {
                break;
            }

            // DMA transfer overrides mode-2 access to OAM. (Reads to OAM return 0xFF)
            // During dma transfer the sprites wont appear on the screen since gpu_mem.ly + 16
            // can never be greater than or equal to 255.
            if gpu_mem.dma_transfer {
                ypos = 0xFF;
                xpos = 0xFF
            } else {
                ypos = gpu_mem.oam[curr_entry];
                xpos = gpu_mem.oam[curr_entry + 1];
            }

            sprite_height = 8; // I think this is the only part that can change mid scanline
            if gpu_mem.is_big_sprite() {
                sprite_height = 16;
            }

            if ((gpu_mem.ly + 16) >= ypos) && ((gpu_mem.ly + 16) < ypos + sprite_height) {
                let mut idx = 0;
                for sprite in gpu_mem.sprite_list.iter() {
                    // https://gbdev.io/pandocs/OAM.html#drawing-priority
                    idx += 1;
                    if sprite.xpos > xpos {
                        idx -= 1;
                        break;
                    }
                }
                gpu_mem.sprite_list.insert(
                    idx,
                    Sprite::new(&gpu_mem.oam[curr_entry..(curr_entry + 4)], sprite_height),
                );
            }
        }
    }
}

pub struct Sprite {
    pub ypos: u8,
    pub xpos: u8,
    pub tile_index: u8, // 0x00 - 0xFF indexing from 0x8000 - 0x8FFF
    pub bgw_ontop: bool,
    pub flip_y: bool,
    pub flip_x: bool,
    pub palette_no: bool,
    pub height: u8,
}

impl Sprite {
    pub fn new(sprite_bytes: &[u8], sprite_height: u8) -> Sprite {
        return Sprite {
            ypos: sprite_bytes[0],
            xpos: sprite_bytes[1],
            tile_index: sprite_bytes[2],
            bgw_ontop: (sprite_bytes[3] >> 7) & 0x01 == 0x01,
            flip_y: (sprite_bytes[3] >> 6) & 0x01 == 0x01,
            flip_x: (sprite_bytes[3] >> 5) & 0x01 == 0x01,
            palette_no: (sprite_bytes[3] >> 4) & 0x01 == 0x01,
            height: sprite_height, // Dont actually care about this
        };
    }
}

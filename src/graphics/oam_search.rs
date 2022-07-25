use super::fifo_states::FifoState;
use super::gpu_memory::GpuMemory;
use super::ppu::PpuState;
use super::ppu::{HBlank, PictureGeneration, VBlank};
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

    // Each scanline does an OAM scan during which time we need to determine
    // which sprites should be displayed. (Max of 10 per scan line). We will update
    // the running of list of sprites within gpu_mem
    pub fn new() -> OamSearch {
        return OamSearch {
            cycles_counter: 0,
            sl_sprites_added: 0,
        };
    }

    fn next(self: Self, gpu_mem: &mut GpuMemory) -> PpuState {
        // If we have a new mode, remember to update the lcd register
        if self.cycles_counter < 80 {
            return PpuState::OamSearch(self);
        } else {
            gpu_mem.set_stat_mode(3);

            //  https://gbdev.io/pandocs/pixel_fifo.html#mode-3-operation
            gpu_mem.bg_pixel_fifo = VecDeque::new();
            gpu_mem.oam_pixel_fifo = VecDeque::new();

            return PpuState::PictureGeneration(PictureGeneration::new(self.sl_sprites_added));
        }
    }

    pub fn render(mut self, gpu_mem: &mut GpuMemory, cycles: usize) -> PpuState {
        let proc_howmany = cycles / 2;
        let num_entries = self.cycles_counter / 2;

        self.find_sprites(gpu_mem, proc_howmany, num_entries);

        self.cycles_counter += cycles;
        return self.next(gpu_mem); // For Now
    }

    pub fn read_byte(self: &Self, gpu_mem: &GpuMemory, addr: usize) -> u8 {
        return match addr {
            0x8000..=0x9FFF => gpu_mem.vram[(addr - 0x8000)],
            0xFE00..=0xFE9F => 0xFF,
            0xFEA0..=0xFEFF => 0xFF, // oam corruption bug to be implemented
            _ => panic!("PPU (O Search) doesnt read from address: {:04X}", addr),
        };
    }

    pub fn write_byte(self: &mut Self, gpu_mem: &mut GpuMemory, addr: usize, data: u8) {
        match addr {
            0x8000..=0x9FFF => gpu_mem.vram[(addr - 0x8000)] = data,
            0xFE00..=0xFE9F => return,
            0xFEA0..=0xFEFF => return,
            _ => panic!("PPU (O Search) doesnt write to address: {:04X}", addr),
        }
    }

    pub fn find_sprites(
        self: &mut Self,
        gpu_mem: &mut GpuMemory,
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
            if self.sl_sprites_added == OamSearch::MAX_SCANLINE_SPRITES
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

// Number of sprites added for the scanline
// How many entries we will process
// The entry we last left off at
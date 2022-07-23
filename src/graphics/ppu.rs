use std::collections::VecDeque;

use super::fifo_states;
use super::fifo_states::FifoState;
use super::gpu_memory::GpuMemory;
use super::sprite;
use super::sprite::Sprite;

pub enum PpuState {
    OamSearch(OamSearch),
    PictureGeneration(PictureGeneration),
    HBlank(HBlank),
    VBlank(VBlank),
    None,
}

pub fn init() -> OamSearch {
    return OamSearch {
        cycles_counter: 0,
        sl_sprites_added: 0,
    };
}

// mode 2
pub struct OamSearch {
    cycles_counter: usize,
    sl_sprites_added: usize, // Number of sprites added on the current scanline
}

// mode 3
pub struct PictureGeneration {
    cycles_counter: usize,
    carry_cycles: usize,
    sl_sprites_added: usize,
    fifo_state: FifoState,
}

// mode 0
pub struct HBlank {
    cycles_counter: usize,
    sl_sprites_added: usize,
    cycles_to_run: usize,
}

// mode 1
pub struct VBlank {
    cycles_counter: usize,
    sl_sprites_added: usize, // Dont know if we care still at this state
}

impl OamSearch {
    pub const MAX_SPRITES: usize = 40;
    pub const MAX_SCANLINE_SPRITES: usize = 10;
    pub const OAM_LENGTH: usize = 160;

    // Each scanline does an OAM scan during which time we need to determine
    // which sprites should be displayed. (Max of 10 per scan line). We will update
    // the running of list of sprites within gpu_mem

    fn next(mut self, gpu_mem: &mut GpuMemory) -> PpuState {
        // If we have a new mode, remember to update the lcd register
        if self.cycles_counter < 80 {
            return PpuState::OamSearch(self);
        } else {
            gpu_mem.set_stat_mode(3);

            //  https://gbdev.io/pandocs/pixel_fifo.html#mode-3-operation
            gpu_mem.bg_pixel_fifo = VecDeque::new();
            gpu_mem.oam_pixel_fifo = VecDeque::new();

            return PpuState::PictureGeneration(PictureGeneration {
                cycles_counter: 0,
                carry_cycles: 0,
                sl_sprites_added: self.sl_sprites_added,
                fifo_state: FifoState::GetTile,
            });
        }
    }

    pub fn render(mut self, gpu_mem: &mut GpuMemory, cycles: usize) -> PpuState {
        let proc_howmany = cycles / 2;
        let num_entries = self.cycles_counter / 2;

        sprite::find_sprites(
            gpu_mem,
            &mut self.sl_sprites_added,
            proc_howmany,
            num_entries,
        );

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
}

impl PictureGeneration {
    pub const FIFO_MAX_PIXELS: usize = 16;
    pub const FIFO_MIN_PIXELS: usize = 8;

    fn next(mut self, gpu_mem: &mut GpuMemory) -> PpuState {
        // LCD Status section of pandocs explains how to actually calculate the cycles to run for
        if self.cycles_counter < 291 {
            return PpuState::PictureGeneration(self);
        } else {
            gpu_mem.set_stat_mode(0);
            return PpuState::HBlank(HBlank {
                cycles_counter: 0,
                sl_sprites_added: self.sl_sprites_added,
                cycles_to_run: 456 - 80 - self.cycles_counter,
            });
        }
    }

    pub fn render(mut self, gpu_mem: &mut GpuMemory, cycles: usize) -> PpuState {
        /*
            We got two fifos (background and sprites)
            The two fifos are only mixed when popping items
            Sprites take priority unless transparent (color 0)
            fifos are only manipulated during mode 3
            the pixel fetcher makes sure each fifo has at least 8 pixels

            pixels have three properties for dmg (cgb has a fourth)
                color between 0 and 3
                palette between 0 and 7 only for sprites
                background priority: value of the OBJ-to-BG Priority bit

            https://gbdev.io/pandocs/pixel_fifo.html#get-tile <--- Continue from here
        */
        let mut cycles_to_run = cycles + self.carry_cycles;
        self.fifo_state = fifo_states::do_work(self.fifo_state, gpu_mem, &mut cycles_to_run);

        self.cycles_counter += (cycles + self.carry_cycles) - cycles_to_run;
        self.carry_cycles = cycles_to_run;
        return self.next(gpu_mem); // For Now
    }

    pub fn read_byte(self: &Self, gpu_mem: &GpuMemory, addr: usize) -> u8 {
        return match addr {
            0x8000..=0x9FFF => 0xFF,
            0xFE00..=0xFE9F => 0xFF,
            0xFEA0..=0xFEFF => 0x00,
            _ => panic!("PPU (Pict Gen) doesnt read from address: {:04X}", addr),
        };
    }

    pub fn write_byte(self: &mut Self, gpu_mem: &mut GpuMemory, addr: usize, _data: u8) {
        match addr {
            0x8000..=0x9FFF => return,
            0xFE00..=0xFE9F => return,
            0xFEA0..=0xFEFF => return,
            _ => panic!("PPU (Pict Gen) doesnt write to address: {:04X}", addr),
        }
    }
}

impl HBlank {
    fn next(mut self, gpu_mem: &mut GpuMemory) -> PpuState {
        if self.cycles_counter < self.cycles_to_run {
            return PpuState::HBlank(self);
        } else if gpu_mem.ly < 143 {
            // If this was < 144 then we would do 143+1 = 144, and repeat oam_search
            // for scanline 144 however at 144, we should be at VBlank
            gpu_mem.set_ly(gpu_mem.ly + 1);
            gpu_mem.set_stat_mode(2);
            return PpuState::OamSearch(OamSearch {
                cycles_counter: 0,
                sl_sprites_added: 0, // reset the number of sprites added as we move to new scanline
            });
        } else {
            gpu_mem.set_ly(gpu_mem.ly + 1); // Should be 144
            gpu_mem.set_stat_mode(1);
            return PpuState::VBlank(VBlank {
                cycles_counter: 0,
                sl_sprites_added: 0, // We probabaly wont need this field
            });
        }
    }

    // HBlank may go to either Itself, OamSearch, or VBlank
    pub fn render(mut self, gpu_mem: &mut GpuMemory, cycles: usize) -> PpuState {
        return self.next(gpu_mem); // For Now
    }

    pub fn read_byte(self: &Self, gpu_mem: &GpuMemory, addr: usize) -> u8 {
        return match addr {
            0x8000..=0x9FFF => gpu_mem.vram[(addr - 0x8000)],
            0xFE00..=0xFE9F => gpu_mem.oam[(addr - 0xFE00)],
            0xFEA0..=0xFEFF => 0x00,
            _ => panic!("PPU (HB) doesnt read from address: {:04X}", addr),
        };
    }

    pub fn write_byte(self: &mut Self, gpu_mem: &mut GpuMemory, addr: usize, data: u8) {
        match addr {
            0x8000..=0x9FFF => gpu_mem.vram[(addr - 0x8000)] = data,
            0xFE00..=0xFE9F => gpu_mem.oam[(addr - 0xFE00)] = data,
            0xFEA0..=0xFEFF => return,
            _ => panic!("PPU (HB) doesnt write to address: {:04X}", addr),
        }
    }
}

impl VBlank {
    fn next(mut self, gpu_mem: &mut GpuMemory) -> PpuState {
        if self.cycles_counter < 456 {
            return PpuState::VBlank(self);
        } else if gpu_mem.ly < 153 {
            // If this was < 154 then we would do 153+1 = 154, but scanlines only
            // exist from 0 - 153 so we would be on a scanline that doesnt exist.
            self.cycles_counter = 0; // reset the counter
            gpu_mem.set_ly(gpu_mem.ly + 1);
            return PpuState::VBlank(self);
        } else {
            gpu_mem.set_stat_mode(2);
            gpu_mem.set_ly(0); // I think this is supposed to be set earlier
            gpu_mem.sprite_list = Vec::<Sprite>::new(); // reset the sprite list since we are done a full cycles of the ppu states.
            return PpuState::OamSearch(OamSearch {
                cycles_counter: 0,
                sl_sprites_added: 0,
            });
        }
    }

    pub fn render(mut self, gpu_mem: &mut GpuMemory, cycles: usize) -> PpuState {
        return self.next(gpu_mem); // For Now
    }

    pub fn read_byte(self: &Self, gpu_mem: &GpuMemory, addr: usize) -> u8 {
        return match addr {
            0x8000..=0x9FFF => gpu_mem.vram[(addr - 0x8000)],
            0xFE00..=0xFE9F => gpu_mem.oam[(addr - 0xFE00)],
            0xFEA0..=0xFEFF => 0x00,
            _ => panic!("PPU (VB) doesnt read from address: {:04X}", addr),
        };
    }

    pub fn write_byte(self: &mut Self, gpu_mem: &mut GpuMemory, addr: usize, data: u8) {
        match addr {
            0x8000..=0x9FFF => gpu_mem.vram[(addr - 0x8000)] = data,
            0xFE00..=0xFE9F => gpu_mem.oam[(addr - 0xFE00)] = data,
            0xFEA0..=0xFEFF => return,
            _ => panic!("PPU (VB) doesnt write to address: {:04X}", addr),
        }
    }
}

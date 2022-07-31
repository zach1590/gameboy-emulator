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

/*
    Any time I access gpu_mem.oam need to do a check to make sure that a
    dma_transfer is not in progress as it would have control of the bus.

    Vram can be accessed normally within this state. Though cpu cant access it.
    (Cause we are working with it)

    Once again dont use picture_generation.read_byte or picture_generation.write_byte
    since those are for the cpu and are rejecting everything due to being in this state
*/
/*
    Pixel Fetcher
    fetches a row of 8 background or window pixels and queues them
    to be mixed with sprite pixels. There are 5 steps
        1. Get Tile (2 Cycles)
        2. Get Tile Data Low (2 Cycles)
        3. Get Tile Data High (2 Cycles)
        4. Sleep (2 Cycles)
        5. Push (1 Cycle each time until complete)
*/

use super::gpu_memory::{GpuMemory, OAM_END, OAM_START, VRAM_END, VRAM_START};
use super::oam_search::OamSearch;
use super::ppu::HBlank;
use super::ppu::{PpuState, MODE_HBLANK};
use std::collections::VecDeque;

// mode 3
pub struct PictureGeneration {
    cycles_counter: usize,
    cycles_to_run: usize,
    sl_sprites_added: usize,
    fifo_state: FifoState,
    xpos: usize,
}

impl PictureGeneration {
    pub const MODE230_CYCLES: usize = 456;
    pub const FIFO_MAX_PIXELS: usize = 16;
    pub const FIFO_MIN_PIXELS: usize = 8;

    pub fn new(sl_sprites_added: usize) -> PictureGeneration {
        return PictureGeneration {
            cycles_counter: 0,
            cycles_to_run: 0,
            sl_sprites_added: sl_sprites_added,
            fifo_state: FifoState::GetTile,
            xpos: 0,
        };
    }

    fn next(self: Self, gpu_mem: &mut GpuMemory) -> PpuState {
        // TODO: Get rid of magic number
        if self.cycles_counter < 291 {
            // Probably are gonna need to return done or not from fifo_states
            return PpuState::PictureGeneration(self);
        } else {
            gpu_mem.set_stat_mode(MODE_HBLANK);
            return HBlank::new(
                self.sl_sprites_added,
                PictureGeneration::MODE230_CYCLES - OamSearch::MAX_CYCLES - self.cycles_counter,
            );
        }
    }

    pub fn render(mut self, gpu_mem: &mut GpuMemory, cycles: usize) -> PpuState {
        self.cycles_to_run += cycles;
        self.do_work(gpu_mem);

        return self.next(gpu_mem); // For Now
    }

    pub fn do_work(self: &mut Self, gpu_mem: &mut GpuMemory) {
        // After push completes should it loop around to GetTile or should it return completely?
        while self.cycles_to_run >= 2 {
            self.fifo_state = match self.fifo_state {
                FifoState::GetTile => self.get_tile(gpu_mem),
                FifoState::GetTileDataLow => self.get_tile_data_low(gpu_mem),
                FifoState::GetTileDataHigh => self.get_tile_data_high(gpu_mem),
                FifoState::Sleep => self.sleep(gpu_mem),
                FifoState::Push => self.push(gpu_mem),
                FifoState::None => panic!("Fifo should not be in None State"),
            };
        }
        // Push can still do some work with only 1 cycle
        if let FifoState::Push = self.fifo_state {
            if self.cycles_to_run == 1 {
                self.fifo_state = self.push(gpu_mem);
            }
        }
    }

    pub fn get_tile(self: &mut Self, _gpu_mem: &mut GpuMemory) -> FifoState {
        self.cycles_to_run -= 2;
        return FifoState::GetTileDataLow;
    }

    pub fn get_tile_data_low(self: &mut Self, _gpu_mem: &mut GpuMemory) -> FifoState {
        self.cycles_to_run -= 2;
        return FifoState::GetTileDataHigh;
    }

    pub fn get_tile_data_high(self: &mut Self, _gpu_mem: &mut GpuMemory) -> FifoState {
        self.cycles_to_run -= 2;
        return FifoState::Sleep;
    }

    pub fn sleep(self: &mut Self, _gpu_mem: &mut GpuMemory) -> FifoState {
        self.cycles_to_run -= 2;
        return FifoState::Push;
    }

    pub fn push(self: &mut Self, _gpu_mem: &mut GpuMemory) -> FifoState {
        self.cycles_to_run -= 1;

        if self.cycles_to_run == 0 {
            return FifoState::GetTile;
        } else {
            return FifoState::Push;
        }
    }

    pub fn read_byte(self: &Self, _gpu_mem: &GpuMemory, addr: u16) -> u8 {
        return match addr {
            VRAM_START..=VRAM_END => 0xFF,
            OAM_START..=OAM_END => 0xFF, // Dont need special handling for dma since it returns 0xFF anyways
            0xFEA0..=0xFEFF => 0x00,
            _ => panic!("PPU (Pict Gen) doesnt read from address: {:04X}", addr),
        };
    }

    pub fn write_byte(self: &mut Self, _gpu_mem: &mut GpuMemory, addr: u16, _data: u8) {
        match addr {
            VRAM_START..=VRAM_END => return,
            OAM_START..=OAM_END => return, // Dont need special handling for dma since it ignores writes anyways
            0xFEA0..=0xFEFF => return,
            _ => panic!("PPU (Pict Gen) doesnt write to address: {:04X}", addr),
        }
    }
}

pub enum FifoState {
    GetTile,
    GetTileDataLow,
    GetTileDataHigh,
    Sleep,
    Push,
    None,
}

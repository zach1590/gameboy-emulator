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
// On each dot during mode 3, either the PPU outputs a pixel or the fetcher is stalling the FIFOs
use super::oam_search::OamSearch;
use super::ppu::{HBlank, PpuState, MODE_HBLANK};
use super::*;
use std::collections::VecDeque;

// mode 3
pub struct PictureGeneration {
    cycles_counter: usize,
    sl_sprites_added: usize,
    fifo_state: FifoState,
    fetch_x: usize,
    byte_index: u8,
    bgw_lo: u8,
    bgw_hi: u8,
    tile_type: TileRepr,
    push_x: u8,
    scanline_x: u8,
}

pub enum FifoState {
    GetTile,
    GetTileDataLow,
    GetTileDataHigh,
    Sleep,
    Push,
    None,
}

pub enum TileRepr {
    Background,
    Window,
    Sprite,
    None,
}

impl PictureGeneration {
    pub const MODE230_CYCLES: usize = 456;
    pub const FIFO_MAX_PIXELS: usize = 16;
    pub const FIFO_MIN_PIXELS: usize = 8;

    pub fn new(sl_sprites_added: usize) -> PictureGeneration {
        return PictureGeneration {
            cycles_counter: 0,
            sl_sprites_added: sl_sprites_added,
            fifo_state: FifoState::GetTile,
            fetch_x: 0,    // scanline x position
            byte_index: 0, // Used for calculating the address
            bgw_lo: 0,     // Tile data low
            bgw_hi: 0,     // Tile data high
            tile_type: TileRepr::None,
            push_x: 0,
            scanline_x: 0,
        };
    }

    fn next(self: Self, gpu_mem: &mut GpuMemory) -> PpuState {
        if (self.push_x as u32) < NUM_PIXELS_X {
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
        for _ in 0..cycles {
            self.cycles_counter += 1;
            self.do_work(gpu_mem);

            if (self.push_x as u32) >= NUM_PIXELS_X {
                break;
            }
        }
        return self.next(gpu_mem);
    }

    pub fn do_work(self: &mut Self, gpu_mem: &mut GpuMemory) {
        // Attempt every other dot
        if (self.cycles_counter & 2) == 0 {
            self.fifo_state = match self.fifo_state {
                FifoState::GetTile => self.get_tile_num(gpu_mem),
                FifoState::GetTileDataLow => self.get_tile_data_low(gpu_mem),
                FifoState::GetTileDataHigh => self.get_tile_data_high(gpu_mem),
                FifoState::Sleep => self.sleep(gpu_mem),
                FifoState::Push => self.push(gpu_mem),
                FifoState::None => panic!("Fifo should not be in None State"),
            };
        } else {
            // Attempt every dot if in the state
            if let FifoState::Push = self.fifo_state {
                // Might do nothing so then we just stay in push which is fine
                self.fifo_state = self.push(gpu_mem);
            }
        }

        // Always attempted
        self.pop_fifo(gpu_mem);
    }

    // What do I do for sprites
    pub fn get_tile_num(self: &mut Self, gpu_mem: &mut GpuMemory) -> FifoState {
        let mut curr_tile = self.fetch_x;
        let mut map_start;

        // Is it necessary to check if bg is enabled? Should it happen earlier?
        // curr_tile will be between 0 and 1023(0x3FF) inclusive
        if gpu_mem.is_bgw_enabled() {
            //&& !gpu_mem.is_window_enabled() {
            curr_tile = (curr_tile + (gpu_mem.scx() / 8)) & 0x1F;
            curr_tile += 32 * (((gpu_mem.ly() + gpu_mem.scy()) & 0xFF) / 8);
            map_start = (gpu_mem.get_bg_tile_map().0 - VRAM_START) as usize;
            self.byte_index = gpu_mem.vram[map_start + curr_tile];
            self.tile_type = TileRepr::Background;
        }

        // if gpu_mem.is_window_enabled() {
        //     curr_tile += 32 * (gpu_mem.window_line_counter as usize / 8);
        //     map_start = (gpu_mem.get_window_tile_map().0 - VRAM_START) as usize;
        //     self.byte_index = gpu_mem.vram[map_start + curr_tile];
        //     self.tile_type = TileRepr::Window;
        // }

        self.fetch_x += 1;
        return FifoState::GetTileDataLow;
    }

    pub fn get_tile_data_low(self: &mut Self, gpu_mem: &mut GpuMemory) -> FifoState {
        let mut offset = 0;
        let addr = calculate_addr(self.byte_index, gpu_mem);

        if let TileRepr::Background = self.tile_type {
            offset = 2 * ((gpu_mem.ly() + gpu_mem.scy()) % 8) as u16;
        }

        // if let TileRepr::Window = self.tile_type {
        //     offset = 2 * (gpu_mem.window_line_counter % 8) as u16;
        // }

        self.bgw_lo = gpu_mem.vram[usize::from(addr + offset - VRAM_START)];
        return FifoState::GetTileDataHigh;
    }

    pub fn get_tile_data_high(self: &mut Self, gpu_mem: &mut GpuMemory) -> FifoState {
        let mut offset = 0;
        let addr = calculate_addr(self.byte_index, gpu_mem);

        if let TileRepr::Background = self.tile_type {
            offset = (2 * ((gpu_mem.ly() + gpu_mem.scy()) % 8) as u16) + 1;
        }

        // if let TileRepr::Window = self.tile_type {
        //     offset = (2 * (gpu_mem.window_line_counter % 8) as u16) + 1;
        // }

        self.bgw_hi = gpu_mem.vram[usize::from(addr + offset - VRAM_START)];
        return FifoState::Sleep;
    }

    pub fn sleep(self: &mut Self, _gpu_mem: &mut GpuMemory) -> FifoState {
        return FifoState::Push;
    }

    pub fn push(self: &mut Self, gpu_mem: &mut GpuMemory) -> FifoState {
        if gpu_mem.bg_pixel_fifo.len() > 8 {
            // FIFO full
            return FifoState::Push;
        }

        let x = (self.fetch_x * 8) as i32 - (8 - (gpu_mem.scx() % 8)) as i32;
        self.weave_bytes_bgw(gpu_mem, x);
        return FifoState::GetTile;
    }

    // weaves the bits together to form the correct output for graphics
    fn weave_bytes_bgw(self: &mut Self, gpu_mem: &mut GpuMemory, x: i32) {
        for shift in 0..=7 {
            let p1 = (self.bgw_hi >> (7 - shift)) & 0x01;
            let p0 = (self.bgw_lo >> (7 - shift)) & 0x01;
            let bit_col = (p1 << 1 | p0) as usize;

            // Need to implement flipping the order in which they are pushed in
            // when the tile is flipped horizontally
            if x >= 0 {
                gpu_mem.bg_pixel_fifo.push_back(COLORS[bit_col]);
            }
        }
    }

    // Not one of the states with mode 3 but a necessary step in mode 3 I think
    // Probably do the merging with sprite fifo here
    pub fn pop_fifo(self: &mut Self, gpu_mem: &mut GpuMemory) {
        if gpu_mem.bg_pixel_fifo.len() > PictureGeneration::FIFO_MIN_PIXELS {
            let pixel = gpu_mem.bg_pixel_fifo.pop_front();

            if let Some(val) = pixel {
                if (gpu_mem.scx % 8) <= self.scanline_x {
                    for i in 0..=3 {
                        gpu_mem.pixels[(usize::from(gpu_mem.ly) * BYTES_PER_ROW)
                            + (usize::from(self.push_x) * BYTES_PER_PIXEL)
                            + i] = val[i];
                    }
                    self.push_x += 1;
                }
                self.scanline_x += 1; // This should extend the duration of mode 3
            }
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

    fn calculate_addr(tile_index: u8, gpu_mem: &GpuMemory) -> u16 {
        let addr: u16 = match gpu_mem.get_addr_mode_start() {
            0x8000 => 0x8000 + (u16::from(tile_index) * 16),
            0x9000 => (0x9000 + (isize::from(tile_index as i8) * BYTES_PER_TILE_SIGNED)) as u16,
            _ => panic!("get_addr_mode only returns 0x9000 or 0x8000"),
        };
        return addr;
    }
}

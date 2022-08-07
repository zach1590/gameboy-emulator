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

// mode 3
pub struct PictureGeneration {
    cycles_counter: usize,
    fifo_state: FifoState,
    fetch_x: usize, // x tile position in map
    byte_index: u8, // Index with the tile we want
    bgw_lo: u8,
    bgw_hi: u8,
    scanline_pos: u8,   // Where in the scanline we are
    push_x: u8,         // What pixel is to be pushed to the screen
    discard_pixels: u8, // Number of pixels discarded at the beginning
    spr_indicies: Vec<usize>,
    spr_data_lo: Vec<u8>,
    spr_data_hi: Vec<u8>,
}

pub enum FifoState {
    GetTile,
    GetTileDataLow,
    GetTileDataHigh,
    Sleep,
    Push,
    None,
}

impl PictureGeneration {
    const SCANLINE_CYCLES: usize = 456;
    const FIFO_MAX_PIXELS: usize = 16;
    const FIFO_MIN_PIXELS: usize = 8;

    pub fn new() -> PictureGeneration {
        return PictureGeneration {
            cycles_counter: 0,
            fifo_state: FifoState::GetTile,
            fetch_x: 0,
            byte_index: 0,
            bgw_lo: 0,
            bgw_hi: 0,
            scanline_pos: 0,
            push_x: 0,
            discard_pixels: 0,
            spr_indicies: Vec::new(),
            spr_data_lo: Vec::new(),
            spr_data_hi: Vec::new(),
        };
    }

    fn next(self: Self, gpu_mem: &mut GpuMemory) -> PpuState {
        if (self.push_x as u32) < NUM_PIXELS_X {
            return PpuState::PictureGeneration(self);
        } else {
            gpu_mem.set_stat_mode(MODE_HBLANK);
            return HBlank::new(
                PictureGeneration::SCANLINE_CYCLES - OamSearch::MAX_CYCLES - self.cycles_counter,
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
        if (self.cycles_counter % 2) == 0 {
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
        let curr_tile;
        let map_start;
        self.spr_indicies.clear();

        // Is it necessary to check if bg is enabled? Should it happen earlier?
        // curr_tile will be between 0 and 1023(0x3FF) inclusive
        if gpu_mem.is_bgw_enabled() {
            curr_tile = ((self.fetch_x + (gpu_mem.scx() / 8)) & 0x1F)
                + (32 * (((gpu_mem.ly() + gpu_mem.scy()) & 0xFF) / 8));
            map_start = (gpu_mem.get_bg_tile_map().0 - VRAM_START) as usize;

            self.byte_index = gpu_mem.vram[map_start + curr_tile];
        }

        if gpu_mem.is_window_enabled() && gpu_mem.is_bgw_enabled() {
            self.find_window_tile_num(gpu_mem);
        }

        if gpu_mem.is_spr_enabled() && gpu_mem.sprite_list.len() > 0 {
            self.search_spr_list(gpu_mem);
        }

        self.fetch_x += 1;
        return FifoState::GetTileDataLow;
    }

    // refer to https://gbdev.io/pandocs/Scrolling.html#window
    fn find_window_tile_num(self: &mut Self, gpu_mem: &mut GpuMemory) {
        let fetcher_pos = (self.fetch_x * 8) + 7;
        if (fetcher_pos >= gpu_mem.wx()) && (fetcher_pos < gpu_mem.wx() + 144 + 14) {
            if gpu_mem.ly() >= gpu_mem.wy() {
                let index = (32 * (gpu_mem.window_line_counter as usize / 8))
                    + ((self.fetch_x) - (gpu_mem.wx() / 8))
                    + usize::from(gpu_mem.get_window_tile_map().0);

                self.byte_index = gpu_mem.vram[index - usize::from(VRAM_START)];
            }
        }
    }
    /*
        Sprite X = position on screen + 8. I can either
        subtract 8 from sprx or add 8 to the comparisons

        The || with xpos + 8 is then because its possible that there are two sprites
        almost on top of each other but with maybe the last pixel of the second sprite
        not covered by anything. Its x position would not match up with the xpos of the
        fetcher since we would have passed it so this makes sure we can still catch it
    */
    fn search_spr_list(self: &mut Self, gpu_mem: &mut GpuMemory) {
        for (i, sprite) in gpu_mem.sprite_list.iter().enumerate() {
            let sprx = usize::from(sprite.xpos + (gpu_mem.scx % 8));
            if ((sprx >= (self.fetch_x * 8) + 8) && (sprx < ((self.fetch_x * 8) + 16)))
                || ((sprx + 8 >= (self.fetch_x * 8) + 8) && (sprx + 8 < ((self.fetch_x * 8) + 16)))
            {
                self.spr_indicies.push(i);
            }
        }
    }

    pub fn get_tile_data_low(self: &mut Self, gpu_mem: &mut GpuMemory) -> FifoState {
        let mut offset = 0;
        let addr = PictureGeneration::calculate_addr(self.byte_index, gpu_mem);
        self.spr_data_lo.clear();

        if gpu_mem.is_bgw_enabled() {
            offset = 2 * ((gpu_mem.ly() + gpu_mem.scy()) % 8) as u16;
        }

        if gpu_mem.is_window_enabled() {
            offset = 2 * (gpu_mem.window_line_counter % 8) as u16;
        }

        if gpu_mem.is_spr_enabled() && self.spr_indicies.len() > 0 {
            self.get_spr_tile_data(gpu_mem, 0);
        }

        self.bgw_lo = gpu_mem.vram[usize::from(addr + offset - VRAM_START)];
        return FifoState::GetTileDataHigh;
    }

    pub fn get_tile_data_high(self: &mut Self, gpu_mem: &mut GpuMemory) -> FifoState {
        let mut offset = 0;
        let addr = PictureGeneration::calculate_addr(self.byte_index, gpu_mem);
        self.spr_data_hi.clear();

        if gpu_mem.is_bgw_enabled() {
            offset = (2 * ((gpu_mem.ly() + gpu_mem.scy()) % 8) as u16) + 1;
        }

        if gpu_mem.is_window_enabled() {
            offset = (2 * (gpu_mem.window_line_counter % 8) as u16) + 1;
        }

        if gpu_mem.is_spr_enabled() && self.spr_indicies.len() > 0 {
            self.get_spr_tile_data(gpu_mem, 1);
        }

        self.bgw_hi = gpu_mem.vram[usize::from(addr + offset - VRAM_START)];
        return FifoState::Sleep;
    }

    fn get_spr_tile_data(self: &mut Self, gpu_mem: &mut GpuMemory, offset: usize) {
        let ly = gpu_mem.ly as i32;
        let spr_height = if gpu_mem.is_big_sprite() { 16 } else { 8 };

        for i in &self.spr_indicies {
            let spr = &gpu_mem.sprite_list[*i];
            // the +16 to ly is because ypos = sprite position on screen + 16
            // And a sprite line takes 2 bytes so this gets us what line of the
            // sprite is to be rendered relative from the start of its position on screen
            // During oamsearch we already confirmed the following:
            // (ly + 16) >= ypos) && ((ly + 16) < ypos + height)
            // Thus y-offset being a usize is okay
            /*
                ex. ly = 10 and ypos = 20 and height = 8
                actual screen position will be 4 (20-16) and thus the sprite will be
                visible from scanlines 4 - 12. ly being 10 means that we want the 6th
                line of the sprite to be rendered, however each line takes two bytes
                so that is 12 bytes from the sprite start (multiply bt two later).
                We determine if we need the high or low of the 2 bytes for sprite after
                knowing if its flipped since that also changes the order we should be
                calculating the y-offset
            */
            let mut y_offset = (ly + 16) - (spr.ypos as i32);

            /*
                Continue from above example, spr_height is either 8 or 16 but tiles
                are 0 indexed hence -1. By subtracting the y-offset which was already
                0-indexed as well (ly and spr.ypos begins at 0) the order in which we
                take the bytes for this sprite are reversed.
                spr_height - 1 is always greater than the y_offset otherwise it would
                not have been added during oam_search
            */
            if spr.flip_y {
                y_offset = (spr_height - 1) - y_offset;
            }

            let index = ((gpu_mem.sprite_list[*i].tile_index as i32) * 16) + (y_offset * 2);

            // The index is already relative from 0x8000 so need to subtract 0x8000
            if offset == 0 {
                self.spr_data_lo.push(gpu_mem.vram[index as usize]);
            } else {
                self.spr_data_hi.push(gpu_mem.vram[index as usize + offset]);
            }
        }
    }

    pub fn sleep(self: &mut Self, _gpu_mem: &mut GpuMemory) -> FifoState {
        return FifoState::Push;
    }

    pub fn push(self: &mut Self, gpu_mem: &mut GpuMemory) -> FifoState {
        if gpu_mem.bg_pixel_fifo.len() > 8 {
            // FIFO full
            return FifoState::Push;
        }

        self.get_color_and_push(gpu_mem);

        return FifoState::GetTile;
    }

    // weaves the bits together to form the correct output for graphics
    fn get_color_and_push(self: &mut Self, gpu_mem: &mut GpuMemory) {
        for shift in 0..=7 {
            let p1 = (self.bgw_hi >> (7 - shift)) & 0x01;
            let p0 = (self.bgw_lo >> (7 - shift)) & 0x01;
            let bit_col = (p1 << 1 | p0) as usize;

            let bg_color = if gpu_mem.is_bgw_enabled() { bit_col } else { 0 };
            let mut color = gpu_mem.bg_colors[bg_color];
            if gpu_mem.is_spr_enabled() {
                color = self.fetch_and_merge(gpu_mem, bg_color)
            }

            // If I want to do this properly with 2 seperate fifos, the sprite fifo also
            // needs to store the bg priority bit and which pallete
            if ((self.fetch_x * 8) as i32 - (8 - (gpu_mem.scx() % 8)) as i32) >= 0 {
                gpu_mem.bg_pixel_fifo.push_back(color);
                self.scanline_pos += 1;
            }
        }
    }

    fn fetch_and_merge(self: &mut Self, gpu_mem: &mut GpuMemory, bg_col: usize) -> [u8; 4] {
        let mut scr_xpos;
        let mut spr;
        for (list_idx, orig_idx) in self.spr_indicies.iter().enumerate() {
            spr = &gpu_mem.sprite_list[*orig_idx];
            scr_xpos = (spr.xpos as i32) - 8 + (gpu_mem.scx % 8) as i32;

            if scr_xpos + 8 < self.scanline_pos as i32 {
                continue;
            }

            let mut offset = self.scanline_pos as i32 - scr_xpos;
            if offset < 0 || offset > 7 {
                // Sprite is not within bounds of current x position
                continue;
            }

            if spr.flip_x {
                offset = 7 - offset;
            }

            let p1 = (self.spr_data_hi[list_idx] >> (7 - offset)) & 0x01;
            let p0 = (self.spr_data_lo[list_idx] >> (7 - offset)) & 0x01;
            let bit_col = (p1 << 1 | p0) as usize;
            // If we wanted to push to a sprite fifo, could probably do it here
            // and then merge later. The sprite fifo would also hold the priority
            // and pallete information.

            if bit_col == 0 {
                continue; // transparent sprite pixel
            }

            if !spr.bgw_ontop || bg_col == 0 {
                return if spr.palette_no {
                    gpu_mem.obp1_colors[bit_col]
                } else {
                    gpu_mem.obp0_colors[bit_col]
                };
            }
        }

        return gpu_mem.bg_colors[bg_col]; // All candidate sprite pixels were transparent or out of bounds
    }

    // Not one of the states with mode 3 but a necessary step in mode 3 I think
    // Probably do the merging with sprite fifo here
    fn pop_fifo(self: &mut Self, gpu_mem: &mut GpuMemory) {
        if gpu_mem.bg_pixel_fifo.len() > PictureGeneration::FIFO_MIN_PIXELS {
            let pixel = gpu_mem.bg_pixel_fifo.pop_front();

            if let Some(val) = pixel {
                // Discard scx % 8 pixels at beginning of scanline
                // Doing the calculation here means that the number may change while discarding
                // Am I supposed to calculate upon entering picture generation instead and compare
                // to the static number?
                if (gpu_mem.scx % 8) <= self.discard_pixels {
                    for i in 0..=3 {
                        gpu_mem.pixels[(usize::from(gpu_mem.ly) * BYTES_PER_ROW)
                            + (usize::from(self.push_x) * BYTES_PER_PIXEL)
                            + i] = val[i];
                    }
                    self.push_x += 1;
                } else {
                    self.discard_pixels += 1;
                }
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

mod fifo_states;
pub mod gpu_memory;
mod oam_search;
mod ppu;

#[cfg(feature = "debug")]
use sdl2::render::Texture;

use super::io::Io;
use gpu_memory::GpuMemory;
use gpu_memory::COLORS;
use ppu::PpuState;
use ppu::PpuState::{HBlank, OamSearch, PictureGeneration, VBlank};

pub const SCALE: u32 = 2;

pub struct Graphics {
    state: PpuState,
    gpu_data: GpuMemory,
    pixels: [u8; 98304],
    dirty: bool,
}

impl Graphics {
    pub fn new() -> Graphics {
        Graphics {
            state: ppu::PpuState::OamSearch(ppu::init()),
            gpu_data: GpuMemory::new(),
            pixels: [0; 98304],
            dirty: false,
        }
    }

    pub fn read_byte(self: &Self, addr: u16) -> u8 {
        return match &self.state {
            OamSearch(os) => os.read_byte(&self.gpu_data, usize::from(addr)),
            PictureGeneration(pg) => pg.read_byte(&self.gpu_data, usize::from(addr)),
            HBlank(hb) => hb.read_byte(&self.gpu_data, usize::from(addr)),
            VBlank(vb) => vb.read_byte(&self.gpu_data, usize::from(addr)),
            PpuState::None => panic!("Ppu state should never be None"),
        };
    }

    pub fn write_byte(self: &mut Self, addr: u16, data: u8) {
        if self.dma_transfer_active() && (0xFE00..=0xFE9F).contains(&addr) {
            // During a dma transfer, cpu cannot access OAM
            // Technically more complicated but I'm okay with just this
            // https://github.com/Gekkio/mooneye-gb/issues/39#issuecomment-265953981
            return;
        }

        if addr >= 0x8000 && addr < 0x9FFF {
            self.dirty = true;
        }
        match &mut self.state {
            OamSearch(os) => os.write_byte(&mut self.gpu_data, usize::from(addr), data),
            PictureGeneration(pg) => pg.write_byte(&mut self.gpu_data, usize::from(addr), data),
            HBlank(hb) => hb.write_byte(&mut self.gpu_data, usize::from(addr), data),
            VBlank(vb) => vb.write_byte(&mut self.gpu_data, usize::from(addr), data),
            PpuState::None => panic!("Ppu state should never be None"),
        }
    }

    // Its possible for dma to want to read from vram
    pub fn read_byte_for_dma(self: &Self, addr: u16) -> u8 {
        let addr = usize::from(addr);
        return match addr {
            0x8000..=0x9FFF => self.gpu_data.vram[(addr - 0x8000)],
            _ => panic!("DMA shouldnt not read from address: {:04X}", addr),
        };
    }

    // addr should be from 0 - 159 inclusive
    pub fn write_byte_for_dma(self: &mut Self, addr: u16, data: u8) {
        if addr > 159 {
            println!("dma function not completing correctly addr: {}", addr);
            return;
        }
        self.gpu_data.oam[usize::from(addr)] = data;
    }

    pub fn read_io_byte(self: &Self, addr: u16) -> u8 {
        self.gpu_data.read_ppu_io(addr)
    }

    pub fn write_io_byte(self: &mut Self, addr: u16, data: u8) {
        self.gpu_data.write_ppu_io(addr, data);
    }

    pub fn adv_cycles(self: &mut Self, io: &mut Io, cycles: usize) {
        let state = std::mem::replace(&mut self.state, PpuState::None);

        self.state = match state {
            OamSearch(os) => os.render(&mut self.gpu_data, cycles),
            PictureGeneration(pg) => pg.render(&mut self.gpu_data, cycles),
            HBlank(hb) => hb.render(&mut self.gpu_data, cycles),
            VBlank(vb) => vb.render(&mut self.gpu_data, cycles),
            PpuState::None => panic!("Ppu state should never be None"),
        };

        if self.gpu_data.stat_int {
            io.request_stat_interrupt();
        }
    }

    pub fn dmg_init(self: &mut Self) {
        self.gpu_data.dmg_init();
    }

    pub fn dma_transfer_active(self: &Self) -> bool {
        return self.gpu_data.dma_transfer;
    }

    pub fn stop_dma_transfer(self: &mut Self) {
        self.gpu_data.dma_transfer = false;
        self.gpu_data.dma_cycles = 0;
    }

    pub fn get_dma_src(self: &Self) -> u16 {
        return (self.gpu_data.dma as u16) * 0x0100;
    }

    pub fn dma_cycles(self: &Self) -> usize {
        return self.gpu_data.dma_cycles;
    }

    pub fn dma_delay(self: &Self) -> usize {
        return self.gpu_data.dma_delay_cycles;
    }

    pub fn incr_dma_cycles(self: &mut Self) {
        self.gpu_data.dma_cycles += 1;
    }

    pub fn decr_dma_delay(self: &mut Self) {
        self.gpu_data.dma_delay_cycles -= 1;
        if self.gpu_data.dma_delay_cycles == 0 {
            self.gpu_data.dma_transfer = true;
        }
    }

    // **(This probably wont be what we need)**
    // Returns a vector of the actual tiles we want to see on screen
    fn weave_tiles_from_map(self: &mut Self) -> Vec<u8> {
        let mut tile_no;
        let mut tile;
        let mut all_tiles = Vec::new();
        let tile_indices = self.gpu_data.get_bg_tile_map();

        for tile_index in (tile_indices.0)..=tile_indices.1 {
            tile_no = self.read_byte(tile_index);
            tile = self.weave_tile_from_index(tile_no);
            all_tiles.append(&mut tile);
        }

        return all_tiles; // all the tiles that were specified by the tile map (256x256 pixels)
    }

    // Takes the index of a tile and returns the
    // result is a vector of 64 bytes (8x8 pixels). Each byte is a pixel represented by a color (0-3)
    fn weave_tile_from_index(self: &mut Self, tile_no: u8) -> Vec<u8> {
        let addr = calculate_addr(tile_no, &self.gpu_data);
        let mut tile: Vec<u8> = Vec::new();

        for i in (0..=15).step_by(2) {
            let byte0 = self.read_byte(addr + i);
            let byte1 = self.read_byte(addr + i + 1);
            let mut tile_row = weave_bytes(byte0, byte1);
            tile.append(&mut tile_row);
        }

        return tile;
    }

    // Write multiple bytes into memory starting from location
    // This should only be used for tests (How to configure to only compile for tests)
    pub fn write_bytes(self: &mut Self, location: u16, data: &Vec<u8>) {
        for (i, byte) in data.into_iter().enumerate() {
            self.write_byte(location + (i as u16), *byte);
        }
    }

    // Later change this to be with a different screen than the main one
    #[cfg(feature = "debug")]
    pub fn update_pixels_with_tiles(self: &mut Self, texture: &mut Texture) {
        if self.dirty {
            let mut xdraw = 0.0; // where should the tile be drawn
            let mut ydraw = 0.0;
            let mut tile_no = 0;

            // Iterate though all 384 tiles, displaying them in a  16 x 24 grid
            for y in 0..24 {
                for x in 0..16 {
                    self.add_tile(tile_no, xdraw, ydraw);
                    xdraw = xdraw + 8.0;
                    tile_no += 1;
                }
                ydraw = ydraw + 8.0;
                xdraw = 0.0;
            }

            self.dirty = false;
            texture
                .update(None, &self.pixels, 16 * 8 * 4)
                .expect("updating texture didnt work");
        }
    }

    #[cfg(feature = "debug")]
    pub fn add_tile(self: &mut Self, tile_no: usize, xdraw: f32, ydraw: f32) {
        for i in (0..=15).step_by(2) {
            let byte0 = self.gpu_data.vram[(tile_no * 16) + i];
            let byte1 = self.gpu_data.vram[(tile_no * 16) + i + 1];
            let tile_row = weave_bytes(byte0, byte1);

            // 16 * 8 is the number of pixels in row, multiplied by 4 because each pixel will have 4 u8s for rgba
            // Basically to get the current pixel we need to multiply current y by the pitch (width)
            let y = (ydraw as usize + (i / 2)) * 16 * 8 * 4;
            for (j, pix) in tile_row.iter().enumerate() {
                let pix_location = y + ((xdraw as usize + j) * 4);

                self.pixels[pix_location] = COLORS[(*pix) as usize][0];
                self.pixels[pix_location + 1] = COLORS[(*pix) as usize][1];
                self.pixels[pix_location + 2] = COLORS[(*pix) as usize][2];
                self.pixels[pix_location + 3] = COLORS[(*pix) as usize][3];
            }
        }
    }
}

// Takes the index of a tile (should be in the tile map) and returns the address
// that the data for this tile is stored in
fn calculate_addr(tile_no: u8, gpu_mem: &GpuMemory) -> u16 {
    let is_sprite = gpu_mem.is_obj_enabled();

    let addr: u16 = match is_sprite {
        true => 0x8000 + (u16::from(tile_no as u8) * 16),
        false => {
            let lcdc_b4 = gpu_mem.get_addr_mode();
            match lcdc_b4 {
                true => 0x8000 + (u16::from(tile_no as u8) * 16),
                false => {
                    let index = isize::from(tile_no as i8) * 16;
                    (0x9000 + index) as u16
                }
            }
        }
    };
    return addr;
}

// Will take 2 bytes and return an array of 8 values that are between 0-3
// weaves the bits together to form the correct output for graphics
// The two bit0s are concatenated to form the last value of the returned array
// The two bit7s are concatenated to form the first value of the returned array
// arg-byte0 should be the byte that is a lower address in memory
fn weave_bytes(byte0: u8, byte1: u8) -> Vec<u8> {
    let mut tile_row = Vec::new();
    for shift in 0..=7 {
        let p1 = (byte1 >> (7 - shift)) & 0x01;
        let p0 = (byte0 >> (7 - shift)) & 0x01;

        match (p1, p0) {
            (0, 0) => tile_row.push(0),
            (0, 1) => tile_row.push(1),
            (1, 0) => tile_row.push(2),
            (1, 1) => tile_row.push(3),
            _ => panic!(
                "Impossible pixel value: {}, {}",
                byte1 & shift,
                byte0 & shift
            ),
        }
    }
    return tile_row;
}

#[cfg(test)]
#[path = "./tests/graphic_tests.rs"]
mod graphic_tests;

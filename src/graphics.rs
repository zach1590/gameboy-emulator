/*
    A tile takes 16 bytes (384 tiles total)
    A tile has 8x8 pixels
    Tile data is held from 0x8000 - 0x97FF

    16 bytes is 128 bits
    8x8 pixels would be 64 bits if we had 1bit per pixel but
    We have each pixel as 2-bits (0-3 to represent 4 color)

    Rather than indexing vram specifically, we index by the tiles (0-383)
    Two addressing modes
    Sprites use 0x8000 with a u8 to address
    Background and Window can use the above or 0x9000 with an i8 (based on LCDC bit 4)
        If lcdc bit 4 is 0 => index from 0x9000 as signed
        IF lcdc bit 4 is 1 => index from 0x8000 as unsigned
    This allows us to use an 8bit value to index more than 255 tiles

    How to know if drawing a sprite or background/window tile?? (I guess based on LCDC)

    Tile Maps
    There are 2 tile maps and they are both 32x32 tiles
    0x9800 - 0x9BFF and 0x9C00 - 0x9FFF
    Each tile map contains the 1-byte indexes of the tiles to be displayed
    A tile is 8x8 pixels, so a map holds 256x256 pixels. (Only 160x144 are displayed)
    How to know which tile map we want at the moment?

    There is also a window internal line counter that is incremented when the
    window is visible

    Window is not scrollable
    Background is scrollable

    Sprite Attribute Table (OAM) is stored in 0xFE00 - 0xFE9F

*/
mod fifo_states;
pub mod gpu_memory;
mod ppu;
mod sprite;

use super::io::Io;
use gpu_memory::GpuMemory;
use ppu::PpuState;
use ppu::PpuState::{HBlank, OamSearch, PictureGeneration, VBlank};

pub struct Graphics {
    state: PpuState,
    gpu_data: GpuMemory,
    pixels: Vec<u8>, // Dont know if I'm using this yet
}

impl Graphics {
    pub fn new() -> Graphics {
        Graphics {
            state: ppu::PpuState::OamSearch(ppu::init()),
            gpu_data: GpuMemory::new(),
            pixels: Vec::new(),
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
        match &mut self.state {
            OamSearch(os) => os.write_byte(&mut self.gpu_data, usize::from(addr), data),
            PictureGeneration(pg) => pg.write_byte(&mut self.gpu_data, usize::from(addr), data),
            HBlank(hb) => hb.write_byte(&mut self.gpu_data, usize::from(addr), data),
            VBlank(vb) => vb.write_byte(&mut self.gpu_data, usize::from(addr), data),
            PpuState::None => panic!("Ppu state should never be None"),
        }
    }

    pub fn read_io_byte(self: &Self, addr: u16) -> u8 {
        self.gpu_data.read_ppu_io(addr)
    }

    // will return if there was a stat interrupt due to the write
    pub fn write_io_byte(self: &mut Self, addr: u16, data: u8) {
        self.gpu_data.write_ppu_io(addr, data);
    }

    // We have &mut self, when in reality I would really like to have Self to
    // do the state machine transistion properly. and without the option<T>
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

    // Probably call from emulator.rs?
    pub fn update_screen(self: &mut Self, _io: &Io) {}

    // **(This probably wont be what we need)**
    // Returns a vector of the actual tiles we want to see on screen
    fn weave_tiles_from_map(self: &mut Self, map_no: u8) -> Vec<u8> {
        let mut tile_no;
        let mut tile;
        let mut all_tiles = Vec::new();
        let tile_indices = get_tile_map(map_no);

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
    // This should only be used for tests
    pub fn write_bytes(self: &mut Self, location: u16, data: &Vec<u8>) {
        for (i, byte) in data.into_iter().enumerate() {
            self.write_byte(location + (i as u16), *byte);
        }
    }
}

// Returns the start and end address of vram containing the 32x32 tile map
// LCDC Bit 3 gives the background tile map area
// LCDC Bit 6 gives the window tile map area
fn get_tile_map(map_no: u8) -> (u16, u16) {
    match map_no {
        0 => (0x9800, 0x9BFF),
        1 => (0x9C00, 0x9FFF),
        _ => panic!("Can only select tile map 0 or 1"),
    }
}

// Takes the index of a tile (should be in the tile map) and returns the address
// that the data for this tile is stored in
fn calculate_addr(tile_no: u8, gpu_mem: &GpuMemory) -> u16 {
    let lcdc = gpu_mem.lcdc;
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

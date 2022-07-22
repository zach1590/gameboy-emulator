// If this ends up being really slow, change to an enum

use super::gpu_memory::GpuMemory;
use super::gpu_memory::LY_REG;
use super::sprite;
use super::sprite::Sprite;

pub fn init() -> Box<dyn PPUMode> {
    return Box::new(OamSearch {
        cycles_counter: 0,
        sl_sprites_added: 0,
    });
}

pub trait PPUMode {
    // Current state calls to return next state
    // Determine if we need to stay in the same same state or return next state
    fn new(self: &mut Self, gpu_mem: &mut GpuMemory) -> Option<Box<dyn PPUMode>>;

    // Called on adv_cycles()
    // Cant figure out how to do this while taking ownership (remove the &mut)
    fn render(self: &mut Self, gpu_mem: &mut GpuMemory, cycles: usize) -> Option<Box<dyn PPUMode>>;

    fn read_byte(self: &Self, gpu_mem: &GpuMemory, addr: usize) -> u8;

    fn write_byte(self: &mut Self, gpu_mem: &mut GpuMemory, addr: usize, data: u8);
}

impl Default for Box<dyn PPUMode> {
    fn default() -> Self {
        return init();
    }
}

// mode 2
pub struct OamSearch {
    cycles_counter: usize,
    sl_sprites_added: usize, // Number of sprites added on the current scanline
}

// mode 3
pub struct PictureGeneration {
    cycles_counter: usize,
    sl_sprites_added: usize,
}

// mode 0
pub struct HBlank {
    cycles_counter: usize,
    sl_sprites_added: usize,
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
}

impl PPUMode for OamSearch {
    fn new(self: &mut Self, gpu_mem: &mut GpuMemory) -> Option<Box<dyn PPUMode>> {
        // If we have a new mode, remember to update the lcd register
        if self.cycles_counter < 80 {
            return None;
        } else {
            gpu_mem.set_stat_mode(3);

            return Some(Box::new(PictureGeneration {
                cycles_counter: 0,
                sl_sprites_added: self.sl_sprites_added,
            }));
        }
    }

    fn render(self: &mut Self, gpu_mem: &mut GpuMemory, cycles: usize) -> Option<Box<dyn PPUMode>> {
        let proc_howmany = cycles / 2;
        let num_entries = self.cycles_counter / 2;

        sprite::find_sprites(
            gpu_mem,
            &mut self.sl_sprites_added,
            proc_howmany,
            num_entries,
        );

        self.cycles_counter += cycles;
        return self.new(gpu_mem); // For Now
    }

    fn read_byte(self: &Self, gpu_mem: &GpuMemory, addr: usize) -> u8 {
        return match addr {
            0x8000..=0x9FFF => gpu_mem.vram[(addr - 0x8000)],
            0xFE00..=0xFE9F => 0xFF,
            0xFEA0..=0xFEFF => 0xFF, // oam corruption bug to be implemented
            _ => panic!("PPU (O Search) doesnt read from address: {:04X}", addr),
        };
    }

    fn write_byte(self: &mut Self, gpu_mem: &mut GpuMemory, addr: usize, data: u8) {
        match addr {
            0x8000..=0x9FFF => gpu_mem.vram[(addr - 0x8000)] = data,
            0xFE00..=0xFE9F => return,
            0xFEA0..=0xFEFF => return,
            _ => panic!("PPU (O Search) doesnt write to address: {:04X}", addr),
        }
    }
}

impl PPUMode for PictureGeneration {
    fn new(self: &mut Self, gpu_mem: &mut GpuMemory) -> Option<Box<dyn PPUMode>> {
        // Also update the lcd with the new mode

        // not always 291 need to figure out how to calculate this value
        if self.cycles_counter < 291 {
            return None;
        } else {
            gpu_mem.set_stat_mode(0);
            return Some(Box::new(HBlank {
                cycles_counter: 0,
                sl_sprites_added: self.sl_sprites_added,
            }));
        }
    }

    fn render(self: &mut Self, gpu_mem: &mut GpuMemory, cycles: usize) -> Option<Box<dyn PPUMode>> {
        return self.new(gpu_mem); // For Now
    }

    fn read_byte(self: &Self, gpu_mem: &GpuMemory, addr: usize) -> u8 {
        return match addr {
            0x8000..=0x9FFF => 0xFF,
            0xFE00..=0xFE9F => 0xFF,
            0xFEA0..=0xFEFF => 0x00,
            _ => panic!("PPU (Pict Gen) doesnt read from address: {:04X}", addr),
        };
    }

    fn write_byte(self: &mut Self, gpu_mem: &mut GpuMemory, addr: usize, _data: u8) {
        match addr {
            0x8000..=0x9FFF => return,
            0xFE00..=0xFE9F => return,
            0xFEA0..=0xFEFF => return,
            _ => panic!("PPU (Pict Gen) doesnt write to address: {:04X}", addr),
        }
    }
}

impl PPUMode for HBlank {
    fn new(self: &mut Self, gpu_mem: &mut GpuMemory) -> Option<Box<dyn PPUMode>> {
        // Also update the lcd with the new mode
        // Can also stay at current state or move to oamsearch
        if self.cycles_counter < 208 {
            // The cycles counter may only go to 68, need to determine how to calculate
            return None;
        } else if gpu_mem.ly < 143 {
            // If this was < 144 then we would do 143+1 = 144, and repeat oam_search
            // for scanline 144 however at 144, we should be at VBlank
            gpu_mem.write_ppu_io(LY_REG, gpu_mem.ly + 1);
            gpu_mem.set_stat_mode(2);
            return Some(Box::new(OamSearch {
                cycles_counter: 0,
                sl_sprites_added: 0, // reset the number of sprites added as we move to new scanline
            }));
        } else {
            gpu_mem.write_ppu_io(LY_REG, gpu_mem.ly + 1); // Should be 144
            gpu_mem.set_stat_mode(1);
            return Some(Box::new(VBlank {
                cycles_counter: 0,
                sl_sprites_added: 0, // We probabaly wont need this field
            }));
        }
    }

    // HBlank may go to either Itself, OamSearch, or VBlank
    fn render(self: &mut Self, gpu_mem: &mut GpuMemory, cycles: usize) -> Option<Box<dyn PPUMode>> {
        return self.new(gpu_mem); // For Now
    }

    fn read_byte(self: &Self, gpu_mem: &GpuMemory, addr: usize) -> u8 {
        return match addr {
            0x8000..=0x9FFF => gpu_mem.vram[(addr - 0x8000)],
            0xFE00..=0xFE9F => gpu_mem.oam[(addr - 0xFE00)],
            0xFEA0..=0xFEFF => 0x00,
            _ => panic!("PPU (HB) doesnt read from address: {:04X}", addr),
        };
    }

    fn write_byte(self: &mut Self, gpu_mem: &mut GpuMemory, addr: usize, data: u8) {
        match addr {
            0x8000..=0x9FFF => gpu_mem.vram[(addr - 0x8000)] = data,
            0xFE00..=0xFE9F => gpu_mem.oam[(addr - 0xFE00)] = data,
            0xFEA0..=0xFEFF => return,
            _ => panic!("PPU (HB) doesnt write to address: {:04X}", addr),
        }
    }
}

impl PPUMode for VBlank {
    fn new(self: &mut Self, gpu_mem: &mut GpuMemory) -> Option<Box<dyn PPUMode>> {
        if self.cycles_counter < 456 {
            return None;
        } else if gpu_mem.ly < 153 {
            // If this was < 154 then we would do 153+1 = 154, but scanlines only
            // exist from 0 - 153 so we would be on a scanline that doesnt exist.
            self.cycles_counter = 0; // reset the counter
            gpu_mem.write_ppu_io(LY_REG, gpu_mem.ly + 1);
            return None; //
        } else {
            gpu_mem.set_stat_mode(2);
            gpu_mem.write_ppu_io(LY_REG, 0); // I think this is supposed to be set earlier
            gpu_mem.sprite_list = Vec::<Sprite>::new(); // reset the sprite list since we are done a full cycles of the ppu states.
            return Some(Box::new(OamSearch {
                cycles_counter: 0,
                sl_sprites_added: 0,
            }));
        }
    }

    fn render(self: &mut Self, gpu_mem: &mut GpuMemory, cycles: usize) -> Option<Box<dyn PPUMode>> {
        return self.new(gpu_mem); // For Now
    }

    fn read_byte(self: &Self, gpu_mem: &GpuMemory, addr: usize) -> u8 {
        return match addr {
            0x8000..=0x9FFF => gpu_mem.vram[(addr - 0x8000)],
            0xFE00..=0xFE9F => gpu_mem.oam[(addr - 0xFE00)],
            0xFEA0..=0xFEFF => 0x00,
            _ => panic!("PPU (VB) doesnt read from address: {:04X}", addr),
        };
    }

    fn write_byte(self: &mut Self, gpu_mem: &mut GpuMemory, addr: usize, data: u8) {
        match addr {
            0x8000..=0x9FFF => gpu_mem.vram[(addr - 0x8000)] = data,
            0xFE00..=0xFE9F => gpu_mem.oam[(addr - 0xFE00)] = data,
            0xFEA0..=0xFEFF => return,
            _ => panic!("PPU (VB) doesnt write to address: {:04X}", addr),
        }
    }
}

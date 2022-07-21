use super::gpu_memory::GpuMemory;
use super::sprite;
use super::sprite::Sprite;

pub fn init() -> Box<dyn PPUMode> {
    return Box::new(OamSearch {
        cycles_counter: 0,
        sprite_list: Vec::<Sprite>::new(),
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
    sprite_list: Vec<Sprite>,
    // sprite list needs to be passed from state to state
    // so that when we come back to oamsearch on the next scanline
    // the items from the previous scan line were not lost

    // Alternatively place sprite_list in gpu_memory and clear it
    // when ly = 154 or just before the transition from Vblank to oamsearch
}

// mode 3
pub struct PictureGeneration {
    cycles_counter: usize,
    sprite_list: Vec<Sprite>,
}

// mode 0
pub struct HBlank {
    cycles_counter: usize,
}

// mode 1
pub struct VBlank {
    cycles_counter: usize,
}

impl OamSearch {
    const MAX_SPRITES: usize = 10;

    // Each scanline does an OAM scan during which time we need to determine
    // which sprites should be displayed. (Max of 10). We will give the current list
    // and obtain an updated one.
    fn search_sprites(
        self: &mut Self,
        gpu_mem: &mut GpuMemory,
        proc_howmany: usize,
        num_entries: usize,
    ) {
        sprite::find_sprites(gpu_mem, &mut self.sprite_list, proc_howmany, num_entries);
    }
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
                sprite_list: std::mem::take(&mut self.sprite_list),
            }));
        }
    }

    fn render(self: &mut Self, gpu_mem: &mut GpuMemory, cycles: usize) -> Option<Box<dyn PPUMode>> {
        let proc_howmany = cycles / 2;
        let num_entries = self.cycles_counter / 2;

        self.search_sprites(gpu_mem, proc_howmany, num_entries);

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
        return Some(Box::new(HBlank { cycles_counter: 0 }));
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
        return Some(Box::new(VBlank { cycles_counter: 0 }));
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
        // Also update the lcd with the new mode
        return Some(Box::new(OamSearch {
            cycles_counter: 0,
            sprite_list: Vec::<Sprite>::new(),
        }));
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

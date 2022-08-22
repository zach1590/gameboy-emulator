use super::gpu_memory::*;
use super::oam_search::OamSearch;
use super::picture_generation::PictureGeneration;

pub const MODE_HBLANK: u8 = 0;
pub const MODE_VBLANK: u8 = 1;
pub const MODE_OSEARCH: u8 = 2;
pub const MODE_PICTGEN: u8 = 3;

pub enum PpuState {
    OamSearch(OamSearch),
    PictureGeneration(PictureGeneration),
    HBlank(HBlank),
    VBlank(VBlank),
    None,
}

// Mooneye test boot_hwio-dmgABCmgb only passes if this is
// oam search. even though initialization says stat mode
// should be 0x85 which would be Vblank
pub fn init(_gpu_mem: &mut GpuMemory) -> PpuState {
    return VBlank::init();
}

pub fn enable(gpu_mem: &mut GpuMemory) -> PpuState {
    gpu_mem.set_stat_mode(MODE_PICTGEN);
    return PpuState::PictureGeneration(PictureGeneration::new());
}

pub fn disable(gpu_mem: &mut GpuMemory) -> PpuState {
    gpu_mem.set_stat_mode(MODE_HBLANK);
    return HBlank::new(0);
}

// mode 0
pub struct HBlank {
    cycles_counter: usize,
    cycles_to_run: usize,
}

// mode 1
pub struct VBlank {
    cycles_counter: usize,
    line_counter: usize,
}

impl HBlank {
    pub fn new(cycles_remaining: usize) -> PpuState {
        return PpuState::HBlank(HBlank {
            cycles_counter: 0,
            cycles_to_run: cycles_remaining,
        });
    }

    fn next(self: Self, gpu_mem: &mut GpuMemory) -> PpuState {
        if self.cycles_counter < self.cycles_to_run {
            return PpuState::HBlank(self);
        } else {
            // Do here because https://gbdev.io/pandocs/Scrolling.html#window
            if gpu_mem.is_window_enabled() && gpu_mem.is_window_visible() {
                gpu_mem.window_line_counter += 1;
            }

            gpu_mem.set_ly(gpu_mem.ly + 1);
            gpu_mem.sprite_list.clear(); // Moving to start of next scanline, so new search will be done

            if gpu_mem.ly < 144 {
                gpu_mem.set_stat_mode(MODE_OSEARCH);
                return OamSearch::new();
            } else {
                gpu_mem.set_stat_mode(MODE_VBLANK);
                return VBlank::new();
            }
        }
    }

    // HBlank may go to either Itself, OamSearch, or VBlank
    pub fn render(mut self, gpu_mem: &mut GpuMemory, cycles: usize) -> PpuState {
        self.cycles_counter += cycles;
        return self.next(gpu_mem);
    }

    pub fn read_byte(self: &Self, gpu_mem: &GpuMemory, addr: u16) -> u8 {
        return match addr {
            VRAM_START..=VRAM_END => gpu_mem.vram[usize::from(addr - VRAM_START)],
            OAM_START..=OAM_END => gpu_mem.oam[usize::from(addr - OAM_START)],
            UNUSED_START..=UNUSED_END => 0x00,
            _ => panic!("PPU (HB) doesnt read from address: {:04X}", addr),
        };
    }

    pub fn write_byte(self: &mut Self, gpu_mem: &mut GpuMemory, addr: u16, data: u8) {
        match addr {
            VRAM_START..=VRAM_END => gpu_mem.vram[usize::from(addr - VRAM_START)] = data,
            OAM_START..=OAM_END => gpu_mem.oam[usize::from(addr - OAM_START)] = data,
            UNUSED_START..=UNUSED_END => return,
            _ => panic!("PPU (HB) doesnt write to address: {:04X}", addr),
        }
    }
}

impl VBlank {
    const MAX_LINE_CYCLES: usize = 456;
    const MAX_VBLANK_CYCLES: usize = 4560;

    pub fn new() -> PpuState {
        return PpuState::VBlank(VBlank {
            cycles_counter: 0,
            line_counter: 0,
        });
    }

    // On boot, only emulate 53 cycles. Ran another emulator to determine this
    // but dont truly know if its correct. Makes more sense than starting
    // in oam_search state though to get mooneye boot_hwio-dmgABCmgb to pass
    fn init() -> PpuState {
        return PpuState::VBlank(VBlank {
            cycles_counter: VBlank::MAX_VBLANK_CYCLES - 53,
            line_counter: 0,
        });
    }

    fn next(mut self, gpu_mem: &mut GpuMemory) -> PpuState {
        if self.cycles_counter >= VBlank::MAX_VBLANK_CYCLES {
            gpu_mem.window_line_counter = 0;
            gpu_mem.set_stat_mode(MODE_OSEARCH);
            gpu_mem.set_ly(0); // I think this is supposed to be set earlier
            gpu_mem.sprite_list.clear();
            return OamSearch::new();
        }

        if self.line_counter >= VBlank::MAX_LINE_CYCLES {
            gpu_mem.set_ly(gpu_mem.ly + 1);
            self.line_counter = self.line_counter.wrapping_sub(VBlank::MAX_LINE_CYCLES);
        }

        return PpuState::VBlank(self);
    }

    pub fn render(mut self, gpu_mem: &mut GpuMemory, cycles: usize) -> PpuState {
        self.cycles_counter += cycles;
        self.line_counter += cycles;
        return self.next(gpu_mem);
    }

    pub fn read_byte(self: &Self, gpu_mem: &GpuMemory, addr: u16) -> u8 {
        return match addr {
            VRAM_START..=VRAM_END => gpu_mem.vram[usize::from(addr - VRAM_START)],
            OAM_START..=OAM_END => gpu_mem.oam[usize::from(addr - OAM_START)],
            UNUSED_START..=UNUSED_END => 0x00,
            _ => panic!("PPU (VB) doesnt read from address: {:04X}", addr),
        };
    }

    pub fn write_byte(self: &mut Self, gpu_mem: &mut GpuMemory, addr: u16, data: u8) {
        match addr {
            VRAM_START..=VRAM_END => gpu_mem.vram[usize::from(addr - VRAM_START)] = data,
            OAM_START..=OAM_END => gpu_mem.oam[usize::from(addr - OAM_START)] = data,
            UNUSED_START..=UNUSED_END => return,
            _ => panic!("PPU (VB) doesnt write to address: {:04X}", addr),
        }
    }
}

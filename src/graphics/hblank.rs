use super::oam_search::OamSearch;
use super::vblank::VBlank;
use super::ppu::{PpuState, MODE_OSEARCH, MODE_VBLANK};
use super::*;

// mode 0
pub struct HBlank {
    cycles_counter: usize,
    cycles_to_run: usize,
}

impl HBlank {
    pub fn new(cycles_remaining: usize) -> PpuState {
        return PpuState::HBlank(HBlank {
            cycles_counter: 0,
            cycles_to_run: cycles_remaining,
        });
    }

    // HBlank may go to either Itself, OamSearch, or VBlank
    fn next(self: Self, gpu_mem: &mut GpuMemory) -> PpuState {
        if self.cycles_counter < self.cycles_to_run {
            // Nothing occurs during HBlank, its just to synchronize the number of cycles
            // a scanline takes as picturegeneration takes a variable number of cycles
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
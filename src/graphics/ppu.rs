use super::fifo_states;
use super::fifo_states::FifoState;
use super::gpu_memory::{GpuMemory, LY_MAX, OAM_END, OAM_START, VRAM_END, VRAM_START};
use super::oam_search::{OamSearch, Sprite};

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

pub fn init() -> OamSearch {
    return OamSearch::new();
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

impl PictureGeneration {
    pub const MODE230_CYCLES: usize = 456;
    pub const FIFO_MAX_PIXELS: usize = 16;
    pub const FIFO_MIN_PIXELS: usize = 8;

    pub fn new(sl_sprites_added: usize) -> PictureGeneration {
        return PictureGeneration {
            cycles_counter: 0,
            carry_cycles: 0,
            sl_sprites_added: sl_sprites_added,
            fifo_state: FifoState::GetTile,
        };
    }

    fn next(self: Self, gpu_mem: &mut GpuMemory) -> PpuState {
        // TODO: Get rid of magic number
        if self.cycles_counter < 291 {
            // Probably are gonna need to return done or not from fifo_states
            return PpuState::PictureGeneration(self);
        } else {
            gpu_mem.set_stat_mode(MODE_HBLANK);
            return PpuState::HBlank(HBlank {
                cycles_counter: 0,
                sl_sprites_added: self.sl_sprites_added,
                cycles_to_run: PictureGeneration::MODE230_CYCLES
                    - OamSearch::MAX_CYCLES
                    - self.cycles_counter,
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

impl HBlank {
    fn next(self: Self, gpu_mem: &mut GpuMemory) -> PpuState {
        if self.cycles_counter < self.cycles_to_run {
            return PpuState::HBlank(self);
        } else if gpu_mem.ly < 143 {
            gpu_mem.set_ly(gpu_mem.ly + 1);
            gpu_mem.set_stat_mode(MODE_OSEARCH);
            return PpuState::OamSearch(OamSearch::new());
        } else {
            gpu_mem.set_ly(gpu_mem.ly + 1);
            gpu_mem.set_stat_mode(MODE_VBLANK);
            return PpuState::VBlank(VBlank {
                cycles_counter: 0,
                sl_sprites_added: 0, // We probabaly wont need this field
            });
        }
    }

    // HBlank may go to either Itself, OamSearch, or VBlank
    pub fn render(self: Self, gpu_mem: &mut GpuMemory, _cycles: usize) -> PpuState {
        return self.next(gpu_mem); // For Now
    }

    pub fn read_byte(self: &Self, gpu_mem: &GpuMemory, addr: u16) -> u8 {
        return match addr {
            VRAM_START..=VRAM_END => gpu_mem.vram[usize::from(addr - VRAM_START)],
            OAM_START..=OAM_END => {
                if gpu_mem.dma_transfer {
                    0xFF
                } else {
                    gpu_mem.oam[usize::from(addr - OAM_START)]
                }
            }
            0xFEA0..=0xFEFF => 0x00,
            _ => panic!("PPU (HB) doesnt read from address: {:04X}", addr),
        };
    }

    pub fn write_byte(self: &mut Self, gpu_mem: &mut GpuMemory, addr: u16, data: u8) {
        match addr {
            VRAM_START..=VRAM_END => gpu_mem.vram[usize::from(addr - VRAM_START)] = data,
            OAM_START..=OAM_END => {
                if gpu_mem.dma_transfer {
                    return; // Dont write during dma
                } else {
                    gpu_mem.oam[usize::from(addr - OAM_START)] = data
                }
            }
            0xFEA0..=0xFEFF => return,
            _ => panic!("PPU (HB) doesnt write to address: {:04X}", addr),
        }
    }
}

impl VBlank {
    pub const MAX_CYCLES: usize = 456;

    fn next(mut self, gpu_mem: &mut GpuMemory) -> PpuState {
        if self.cycles_counter < VBlank::MAX_CYCLES {
            return PpuState::VBlank(self);
        } else if gpu_mem.ly < LY_MAX {
            self.cycles_counter = 0; // reset the counter
            gpu_mem.set_ly(gpu_mem.ly + 1);
            return PpuState::VBlank(self);
        } else {
            gpu_mem.set_stat_mode(MODE_OSEARCH);
            gpu_mem.set_ly(0); // I think this is supposed to be set earlier
            gpu_mem.sprite_list = Vec::<Sprite>::new();
            return PpuState::OamSearch(OamSearch::new());
        }
    }

    pub fn render(self: Self, gpu_mem: &mut GpuMemory, _cycles: usize) -> PpuState {
        return self.next(gpu_mem); // For Now
    }

    pub fn read_byte(self: &Self, gpu_mem: &GpuMemory, addr: u16) -> u8 {
        return match addr {
            VRAM_START..=VRAM_END => gpu_mem.vram[usize::from(addr - VRAM_START)],
            OAM_START..=OAM_END => {
                // During a dma transfer, cpu cannot access OAM
                // Technically more complicated but I'm okay with just this and the other states
                // https://github.com/Gekkio/mooneye-gb/issues/39#issuecomment-265953981
                if gpu_mem.dma_transfer {
                    0xFF
                } else {
                    gpu_mem.oam[usize::from(addr - OAM_START)]
                }
            }
            0xFEA0..=0xFEFF => 0x00,
            _ => panic!("PPU (VB) doesnt read from address: {:04X}", addr),
        };
    }

    pub fn write_byte(self: &mut Self, gpu_mem: &mut GpuMemory, addr: u16, data: u8) {
        match addr {
            VRAM_START..=VRAM_END => gpu_mem.vram[usize::from(addr - VRAM_START)] = data,
            OAM_START..=OAM_END => {
                if gpu_mem.dma_transfer {
                    return; // Dont write during dma
                } else {
                    gpu_mem.oam[usize::from(addr - OAM_START)] = data
                }
            }
            0xFEA0..=0xFEFF => return,
            _ => panic!("PPU (VB) doesnt write to address: {:04X}", addr),
        }
    }
}

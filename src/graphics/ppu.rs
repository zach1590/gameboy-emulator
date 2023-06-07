/*
	PPU executes as a state machine.
	Mode: 2 -> 3 -> 0 repeated for 144 scanlines
	Mode: 1 repeated for 10 scanlines after above
*/

use super::gpu_memory::*;
use super::oam_search::OamSearch;
use super::picture_generation::PictureGeneration;
use super::vblank::VBlank;
use super::hblank::HBlank;

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
// should be 0x85 which would be Vblank. Research more?
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
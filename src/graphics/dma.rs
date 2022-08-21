use core::panic;

use crate::bus::BusType;
use crate::graphics::Graphics;

use super::gpu_memory::{VRAM_END, VRAM_START};

pub const DMA_REG: u16 = 0xFF46;
pub const DMA_SRC_MUL: u16 = 0x0100;
pub const DMA_MAX_CYCLES: u16 = 159;

pub struct OamDma {
    dma: u8,
    value: u8,
    cycles: u16,
    delay_cycles: usize,
    in_transfer: bool,
    pub bus_conflict: BusType,
}

impl OamDma {
    pub fn new() -> OamDma {
        OamDma {
            dma: 0, // 0xFF46
            value: 0,
            cycles: 0,
            delay_cycles: 0,
            in_transfer: false,
            bus_conflict: BusType::None,
        }
    }

    pub fn read_dma(self: &Self, addr: u16) -> u8 {
        if addr != DMA_REG {
            panic!("dma should not write to addr: {:04X}", addr);
        }

        return self.dma;
    }

    pub fn write_dma(self: &mut Self, addr: u16, data: u8) {
        if addr != DMA_REG {
            panic!("dma should not write to addr: {:04X}", addr);
        }

        if data >= 0xFE {
            self.dma = data & 0xDF;
        } else {
            self.dma = data;
        }

        self.start_dma_countdown();
    }

    #[cfg(feature = "debug")]
    pub fn get_debug_info(self: &Self) -> String {
        format!(
            "dma_active: {}, dma_val: {:04X}, cycles: {}, delay: {}\n",
            self.in_transfer, self.dma, self.cycles, self.delay_cycles
        )
    }

    // Call this method when there is a bus conflict during dma transfer
    // Return this value when there is a bus conflict
    pub fn get_value(self: &Self) -> u8 {
        return self.value;
    }

    pub fn set_value(self: &mut Self, value: u8) {
        self.value = value;
    }

    pub fn start_dma_countdown(self: &mut Self) {
        self.delay_cycles = 2;
        // With this as one, the mooneye tests dont end up in a infinite loop
        // But more of them fail
    }

    pub fn dma_active(self: &Self) -> bool {
        return self.in_transfer;
    }

    pub fn calc_addr(self: &mut Self) -> u16 {
        return (self.dma as u16 * DMA_SRC_MUL) + self.cycles;
    }

    // Supposed to be but not really enforced 0x0000 - 0xDF00
    pub fn get_src(self: &Self) -> u16 {
        return (self.dma as u16) * DMA_SRC_MUL;
    }

    // 0x00 - 0x9F
    pub fn cycles(self: &Self) -> u16 {
        return self.cycles;
    }

    pub fn delay_rem(self: &Self) -> usize {
        return self.delay_cycles;
    }

    pub fn incr_cycles(self: &mut Self, graphics: &mut Graphics) {
        self.cycles += 1;
        if self.cycles > DMA_MAX_CYCLES as u16 {
            self.stop_dma_transfer();
            graphics.set_dma_transfer(false);
        }
    }

    pub fn stop_dma_transfer(self: &mut Self) {
        self.in_transfer = false;
        self.cycles = 0;
        self.bus_conflict = BusType::None;
    }

    pub fn decr_delay(self: &mut Self, graphics: &mut Graphics) {
        self.delay_cycles -= 1;
        if self.delay_cycles == 0 {
            self.start_dma_transfer();
            graphics.set_dma_transfer(true);
        }
    }

    pub fn start_dma_transfer(self: &mut Self) {
        self.in_transfer = true;
        self.cycles = 0;

        self.bus_conflict = match self.calc_addr() {
            VRAM_START..=VRAM_END => BusType::Video,
            0x0000..=0x7FFF | 0xA000..=0xFDFF => BusType::External,
            _ => BusType::None,
        };
    }

    pub fn dmg_init(self: &mut Self) {
        self.dma = 0xFF;
    }

    pub fn has_conflict(self: &Self) -> bool {
        return self.bus_conflict.is_some();
    }

    // If the addr falls within a certain range and that range happens
    // to be the range of a bus in use by dma transfer, return true
    pub fn check_bus_conflicts(self: &Self, addr: u16) -> Option<u8> {
        return if self.has_conflict() {
            match (addr, &self.bus_conflict) {
                (VRAM_START..=VRAM_END, BusType::Video) => Some(self.value),
                (0x0000..=0x7FFF | 0xA000..=0xFDFF, BusType::External) => Some(self.value),
                (0xFE00..=0xFE9F, _conflict) => {
                    if self.in_transfer {
                        Some(0xFF)
                    } else {
                        None
                    }
                }
                _ => None,
            }
        } else {
            None
        };
    }
}

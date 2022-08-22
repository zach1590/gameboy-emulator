use super::graphics::Graphics;
use super::io::{Io, IF_REG};
use super::joypad::{Joypad, JOYP_REG};
use super::mbc::Mbc;
use super::memory::Memory;
use super::serial::*;
use super::sound::*;
use super::timer::*;
use crate::graphics::dma::*;
use crate::graphics::gpu_memory::{
    OAM_END, OAM_START, PPUIO_END, PPUIO_START, UNUSED_END, UNUSED_START, VRAM_END, VRAM_START,
};
use sdl2::render::Texture;
use sdl2::EventPump;

pub struct Bus {
    mem: Memory,
    graphics: Graphics, // 0x8000 - 0x9FFF, 0xFE00 - 0xFE9F, and 0xFF40 - 0xFF4B
    io: Io,             // 0xFF01 - 0xFF7F (But not 0xFF40 - 0xFF4B)
    timer: Timer,
    joypad: Joypad, // 0xFF01
    serial: Serial,
    sound: Sound,
    oam_dma: OamDma,
}

pub enum BusType {
    Video,    //0x8000-0x9FFF
    External, //0x0000-0x7FFF, 0xA000-0xFDFF
    None,
}

impl BusType {
    pub fn is_some(self: &Self) -> bool {
        return if let BusType::None = self {
            false
        } else {
            true
        };
    }
}

impl Bus {
    pub fn new() -> Bus {
        return Bus {
            mem: Memory::new(),
            graphics: Graphics::new(),
            io: Io::new(),
            timer: Timer::new(),
            joypad: Joypad::new(),
            serial: Serial::new(),
            sound: Sound::new(),
            oam_dma: OamDma::new(),
        };
    }

    pub fn set_joypad(self: &mut Self, event_pump: EventPump) {
        self.joypad.set_joypad(event_pump);
    }

    pub fn set_mbc(self: &mut Self, cart_mbc: Box<dyn Mbc>) {
        self.mem.set_mbc(cart_mbc);
    }

    #[cfg(feature = "debug")]
    pub fn get_debug_info(self: &mut Self, dbug_output: &mut String) {
        dbug_output.push_str(&self.oam_dma.get_debug_info());
        dbug_output.push_str(&self.graphics.get_debug_info());
        dbug_output.push_str(&self.timer.get_debug_info());
    }

    // TODO: Figure out how to pattern match on const ranges somehow
    pub fn read_byte(self: &Self, addr: u16) -> u8 {
        match self.oam_dma.check_bus_conflicts(addr) {
            Some(x) => return x,
            None => { /* Continue */ }
        }

        let byte = match addr {
            VRAM_START..=VRAM_END => self.graphics.read_byte(addr),
            OAM_START..=OAM_END => self.graphics.read_byte(addr),
            DMA_REG => self.oam_dma.read_dma(addr),
            UNUSED_START..=UNUSED_END => self.graphics.read_byte(addr),
            PPUIO_START..=PPUIO_END => self.graphics.read_io_byte(addr),
            JOYP_REG => self.joypad.read_byte(addr),
            SB_REG | SC_REG => self.serial.read_byte(addr),
            TIMER_START..=TIMER_END => self.timer.read_byte(addr),
            NR10..=NR14 => self.sound.read_byte(addr),
            NR21..=NR34 => self.sound.read_byte(addr),
            NR41..=NR52 => self.sound.read_byte(addr),
            0xFF03..=0xFF0F | 0xFF15 | 0xFF1F => self.io.read_byte(addr),
            0xFF27..=0xFF3F => self.io.read_byte(addr),
            0xFF4C..=0xFF7F => self.io.read_byte(addr),
            _ => self.mem.read_byte(addr),
        };
        return byte;
    }

    pub fn write_byte(self: &mut Self, addr: u16, data: u8) {
        match self.oam_dma.check_bus_conflicts(addr) {
            Some(_) => return,
            None => { /* Continue */ }
        }

        match addr {
            VRAM_START..=VRAM_END => self.graphics.write_byte(addr, data),
            OAM_START..=OAM_END => self.graphics.write_byte(addr, data),
            DMA_REG => self.oam_dma.write_dma(addr, data),
            UNUSED_START..=UNUSED_END => self.graphics.write_byte(addr, data), // Memory area not usuable
            PPUIO_START..=PPUIO_END => self.graphics.write_io_byte(addr, data),
            JOYP_REG => self.joypad.write_byte(addr, data),
            SB_REG | SC_REG => self.serial.write_byte(addr, data),
            TIMER_START..=TIMER_END => self.timer.write_byte(addr, data),
            NR10..=NR14 => self.sound.write_byte(addr, data),
            NR21..=NR34 => self.sound.write_byte(addr, data),
            NR41..=NR52 => self.sound.write_byte(addr, data),
            0xFF03..=0xFF0F | 0xFF15 | 0xFF1F => self.io.write_byte(addr, data),
            0xFF27..=0xFF3F => self.io.write_byte(addr, data),
            0xFF4C..=0xFF7F => self.io.write_byte(addr, data),
            _ => self.mem.write_byte(addr, data),
        };
    }

    // dma should have access to whatever it wants from 0x0000 - 0xDF00
    // Define extra read_byte functions that bypass any protections
    fn read_byte_for_dma(self: &Self, addr: u16) -> u8 {
        let byte = match addr {
            VRAM_START..=VRAM_END => self.graphics.read_byte_for_dma(addr),
            OAM_START..=OAM_END => self.graphics.read_byte_for_dma(addr),
            DMA_REG => self.oam_dma.read_dma(addr),
            UNUSED_START..=UNUSED_END => self.graphics.read_byte_for_dma(addr),
            JOYP_REG => self.joypad.read_byte(addr),
            SB_REG | SC_REG => self.serial.read_byte(addr),
            TIMER_START..=TIMER_END => self.timer.read_byte(addr),
            NR10..=NR14 => self.sound.read_byte(addr),
            NR21..=NR34 => self.sound.read_byte(addr),
            NR41..=NR52 => self.sound.read_byte(addr),
            0xFF03..=0xFF0F | 0xFF15 | 0xFF1F => self.io.read_byte(addr),
            0xFF27..=0xFF3F => self.io.read_byte(addr),
            0xFF4C..=0xFF7F => self.io.read_byte(addr),
            _ => self.mem.read_byte_for_dma(addr),
        };
        return byte;
    }

    // dma should be allowed to write to oam regardless of ppu state
    // use this function to bypass any protections
    fn write_byte_for_dma(self: &mut Self, addr: u16, data: u8) {
        self.graphics.write_byte_for_dma(addr, data);
    }

    pub fn dmg_init(self: &mut Self) {
        self.mem.dmg_init();
        self.timer.dmg_init();
        self.io.dmg_init();
        self.graphics.dmg_init();
        self.serial.dmg_init();
        self.joypad.dmg_init();
        self.oam_dma.dmg_init();
        self.sound.dmg_init();
    }

    pub fn adv_cycles(self: &mut Self, cycles: usize) {
        self.timer.adv_cycles(&mut self.io, cycles);
        self.serial.adv_cycles(&mut self.io, cycles);
        self.graphics.adv_cycles(&mut self.io, cycles);
        self.mem.adv_cycles(cycles);

        if self.oam_dma.dma_active() {
            self.handle_dma_transfer();
        }
        if self.oam_dma.delay_rem() > 0 {
            self.oam_dma.decr_delay(&mut self.graphics);
        }
    }

    // Full dma transfer takes 160 machine cycles (640 T Cycles)
    // 1 Cycle per sprite entry
    fn handle_dma_transfer(self: &mut Self) {
        let addr = self.oam_dma.calc_addr();
        let value = self.read_byte_for_dma(addr);

        self.oam_dma.set_value(value);
        self.write_byte_for_dma(self.oam_dma.cycles(), value);
        self.oam_dma.incr_cycles(&mut self.graphics);
    }

    pub fn interrupt_pending(self: &Self) -> bool {
        (self.mem.i_enable & self.io.read_byte(IF_REG) & 0x1F) != 0
    }

    pub fn update_input(self: &mut Self) -> bool {
        let should_exit = self.joypad.update_input();
        if self.joypad.is_joypad_interrupt() {
            self.io.request_joypad_interrupt();
        }
        return should_exit;
    }

    pub fn write_bytes(self: &mut Self, location: u16, data: &Vec<u8>) {
        self.mem.write_bytes(location, data);
    }

    pub fn update_display(self: &mut Self, texture: &mut Texture) -> bool {
        return self.graphics.update_display(texture);
    }
}

use super::io::Io;
use super::sprite::Sprite;

pub fn init() -> Box<dyn PPUMode> {
    return Box::new(
        OamSearch {
            cycles_counter: 0,
            sprite_list: Vec::<Sprite>::new(),
        }
    );
}

pub trait PPUMode {
    // Current state calls to return next state
    fn new(self: &mut Self) -> Box<dyn PPUMode>;

    // Called on adv_cycles()
    fn render(self: &mut Self, io: &mut Io, vram: &[u8], oam: &[u8], cycles: usize) -> Box<dyn PPUMode>;

    fn read_byte(self: &Self, vram: &[u8], oam: &[u8], addr: usize) -> u8;

    fn write_byte(self: &mut Self, vram: &mut [u8], oam: &mut [u8], addr: usize, data: u8);
}

// mode 2
pub struct OamSearch {
    cycles_counter: usize,
    sprite_list: Vec<Sprite>,
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
}

impl PPUMode for OamSearch {
    fn new(self: &mut Self) -> Box<dyn PPUMode> {
        // Also update the lcd with the new mode
        return Box::new(PictureGeneration {
            cycles_counter: 0,
            sprite_list: std::mem::take(&mut self.sprite_list),
        });
    }

    fn render(self: &mut Self, io: &mut Io, vram: &[u8], oam: &[u8], cycles: usize) -> Box<dyn PPUMode> {
        return self.new();  // For Now
    }

    fn read_byte(self: &Self, vram: &[u8], _oam: &[u8], addr: usize) -> u8 {
        return match addr {
            0x8000..=0x9FFF => vram[(addr - 0x8000)],
            0xFE00..=0xFE9F => 0xFF,
            _ => panic!("PPU (O Search) doesnt read from address: {:04X}", addr),
        }
    }

    fn write_byte(self: &mut Self, vram: &mut [u8], _oam: &mut [u8], addr: usize, data: u8) {
        match addr {
            0x8000..=0x9FFF => vram[(addr - 0x8000)] = data,
            0xFE00..=0xFE9F => return,
            _ => panic!("PPU (O Search) doesnt write to address: {:04X}", addr),
        }
    }
}

impl PPUMode for PictureGeneration {
    fn new(self: &mut Self) -> Box<dyn PPUMode> {
        // Also update the lcd with the new mode
        return Box::new(HBlank {
            cycles_counter: 0,
        });
    }

    fn render(self: &mut Self, io: &mut Io, vram: &[u8], oam: &[u8], cycles: usize) -> Box<dyn PPUMode> {
        return self.new();  // For Now
    }

    fn read_byte(self: &Self, _vram: &[u8], _oam: &[u8], addr: usize) -> u8 {
        return match addr {
            0x8000..=0x9FFF => 0xFF,
            0xFE00..=0xFE9F => 0xFF,
            _ => panic!("PPU (Pict Gen) doesnt read from address: {:04X}", addr),
        }
    }

    fn write_byte(self: &mut Self, _vram: &mut [u8], _oam: &mut [u8], addr: usize, _data: u8) {
        match addr {
            0x8000..=0x9FFF => return,
            0xFE00..=0xFE9F => return,
            _ => panic!("PPU (Pict Gen) doesnt write to address: {:04X}", addr),
        }
    }
}

impl PPUMode for HBlank {
    fn new(self: &mut Self) -> Box<dyn PPUMode> {
        // Also update the lcd with the new mode
        return Box::new(VBlank {
            cycles_counter: 0,
        });
    }

    // HBlank may go to either Itself, OamSearch, or VBlank
    fn render(self: &mut Self, io: &mut Io, vram: &[u8], oam: &[u8], cycles: usize) -> Box<dyn PPUMode> {
        return self.new();  // For Now
    }

    fn read_byte(self: &Self, vram: &[u8], oam: &[u8], addr: usize) -> u8 {
        return match addr {
            0x8000..=0x9FFF => vram[(addr - 0x8000)],
            0xFE00..=0xFE9F => oam[(addr - 0xFE00)],
            _ => panic!("PPU (HB) doesnt read from address: {:04X}", addr),
        }
    }

    fn write_byte(self: &mut Self, vram: &mut [u8], oam: &mut [u8], addr: usize, data: u8) {
        match addr {
            0x8000..=0x9FFF => vram[(addr - 0x8000)] = data,
            0xFE00..=0xFE9F => oam[(addr - 0xFE00)] = data,
            _ => panic!("PPU (HB) doesnt write to address: {:04X}", addr),
        }
    }
}

impl PPUMode for VBlank {
    fn new(self: &mut Self) -> Box<dyn PPUMode> {
        // Also update the lcd with the new mode
        return Box::new(OamSearch {
            cycles_counter: 0,
            sprite_list: Vec::<Sprite>::new(),
        });
    }

    fn render(self: &mut Self, io: &mut Io, vram: &[u8], oam: &[u8], cycles: usize) -> Box<dyn PPUMode> {
        return self.new();  // For Now
    }

    fn read_byte(self: &Self, vram: &[u8], oam: &[u8], addr: usize) -> u8 {
        return match addr {
            0x8000..=0x9FFF => vram[(addr - 0x8000)],
            0xFE00..=0xFE9F => oam[(addr - 0xFE00)],
            _ => panic!("PPU (VB) doesnt read from address: {:04X}", addr),
        }
    }

    fn write_byte(self: &mut Self, vram: &mut [u8], oam: &mut [u8], addr: usize, data: u8) {
        match addr {
            0x8000..=0x9FFF => vram[(addr - 0x8000)] = data,
            0xFE00..=0xFE9F => oam[(addr - 0xFE00)] = data,
            _ => panic!("PPU (VB) doesnt write to address: {:04X}", addr),
        }
    }
}
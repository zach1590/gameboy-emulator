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
    fn render(self: &mut Self, vram: &[u8], oam: &[u8], cycles: usize) -> Box<dyn PPUMode>;    
}

pub struct OamSearch {
    cycles_counter: usize,
    sprite_list: Vec<Sprite>,
}

pub struct PictureGeneration {
    cycles_counter: usize,
}

pub struct HBlank {
    cycles_counter: usize,
}

pub struct VBlank {
    cycles_counter: usize,
}

impl OamSearch {
    const MAX_SPRITES: usize = 10;
}

impl PPUMode for OamSearch {
    fn new(self: &mut Self) -> Box<dyn PPUMode> {
        return Box::new(PictureGeneration {
            cycles_counter: 0,
        });
    }

    fn render(self: &mut Self, vram: &[u8], oam: &[u8], cycles: usize) -> Box<dyn PPUMode>{
        return self.new();  // For Now
    }
}

impl PPUMode for PictureGeneration {
    fn new(self: &mut Self) -> Box<dyn PPUMode> {
        return Box::new(HBlank {
            cycles_counter: 0,
        });
    }

    fn render(self: &mut Self, vram: &[u8], oam: &[u8], cycles: usize) -> Box<dyn PPUMode>{
        return self.new();  // For Now
    }
}

impl PPUMode for HBlank {
    fn new(self: &mut Self) -> Box<dyn PPUMode> {
        return Box::new(VBlank {
            cycles_counter: 0,
        });
    }

    // HBlank may go to either Itself, OamSearch, or VBlank  
    fn render(self: &mut Self, vram: &[u8], oam: &[u8], cycles: usize) -> Box<dyn PPUMode>{
        return self.new();  // For Now
    }
}

impl PPUMode for VBlank {
    fn new(self: &mut Self) -> Box<dyn PPUMode> {
        return Box::new(OamSearch {
            cycles_counter: 0,
            sprite_list: Vec::<Sprite>::new(),
        });
    }

    fn render(self: &mut Self, vram: &[u8], oam: &[u8], cycles: usize) -> Box<dyn PPUMode>{
        return self.new();  // For Now
    }
}
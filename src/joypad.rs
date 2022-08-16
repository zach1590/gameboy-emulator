/*
    Bit 7 - Not used
    Bit 6 - Not used
    Bit 5 - P15 Select Action buttons    (0=Select)
    Bit 4 - P14 Select Direction buttons (0=Select)
    Bit 3 - P13 Input: Down  or Start    (0=Pressed) (Read Only)
    Bit 2 - P12 Input: Up    or Select   (0=Pressed) (Read Only)
    Bit 1 - P11 Input: Left  or B        (0=Pressed) (Read Only)
    Bit 0 - P10 Input: Right or A        (0=Pressed) (Read Only)
*/

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::EventPump;

pub const JOYP_REG: u16 = 0xFF00;

pub struct Joypad {
    event_pump: Option<EventPump>,
    joyp: u8,
    directs: u8,
    actions: u8,
    high_to_low: bool,
    something_selected: bool,
}

impl Joypad {
    pub fn new() -> Joypad {
        return Joypad {
            event_pump: None,
            joyp: 0xCF,
            directs: 0x0F,
            actions: 0x0F,
            high_to_low: false,
            something_selected: false,
        };
    }

    pub fn dmg_init(self: &mut Self) {
        self.joyp = 0xCF;
    }

    pub fn set_joypad(self: &mut Self, event_pump: EventPump) {
        self.event_pump = Some(event_pump);
    }

    pub fn read_byte(self: &Self, addr: u16) -> u8 {
        let byte = match addr {
            JOYP_REG => self.joyp,
            _ => panic!("Joypad cannot read from addr: {:04X}", addr),
        };
        return byte;
    }

    pub fn write_byte(self: &mut Self, addr: u16, data: u8) {
        match addr {
            JOYP_REG => {
                self.joyp = (data & 0x30) | (self.joyp & 0xCF);
                self.something_selected = (self.joyp & 0x20 == 0x20) || (self.joyp & 0x10 == 0x10);
            }
            _ => panic!("Joypad cannot write addr: {:04X}", addr),
        };
    }

    pub fn is_joypad_interrupt(self: &Self) -> bool {
        return self.something_selected && self.high_to_low;
    }

    pub fn update_input(self: &mut Self) -> bool {
        let mut should_exit = false;

        if let Some(joypad) = &mut self.event_pump {
            let event = joypad.poll_event();

            if let Some(e) = event {
                match e {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => {
                        should_exit = true;
                    }
                    Event::KeyDown {
                        keycode: Some(x), ..
                    } => {
                        self.handle_keydown_event(x);
                    }
                    Event::KeyUp {
                        keycode: Some(x), ..
                    } => {
                        self.handle_keyup_event(x);
                    }
                    _ => self.high_to_low = false,
                }
            }
        }

        if self.joyp & 0x10 == 0x00 {
            self.joyp = (self.joyp & 0xF0) | self.directs;
        }
        if self.joyp & 0x20 == 0x00 {
            self.joyp = (self.joyp & 0xF0) | self.actions;
        }

        return should_exit;
    }

    fn handle_keydown_event(self: &mut Self, key: Keycode) {
        self.high_to_low = true;
        match key {
            Keycode::D => self.directs &= !(1 << 0),
            Keycode::J => self.actions &= !(1 << 0),
            Keycode::A => self.directs &= !(1 << 1),
            Keycode::K => self.actions &= !(1 << 1),
            Keycode::W => self.directs &= !(1 << 2),
            Keycode::L => self.actions &= !(1 << 2),
            Keycode::S => self.directs &= !(1 << 3),
            Keycode::H => self.actions &= !(1 << 3),
            _ => self.high_to_low = false,
        }
    }

    fn handle_keyup_event(self: &mut Self, key: Keycode) {
        self.high_to_low = false;
        match key {
            Keycode::D => self.directs |= 1 << 0,
            Keycode::J => self.actions |= 1 << 0,
            Keycode::A => self.directs |= 1 << 1,
            Keycode::K => self.actions |= 1 << 1,
            Keycode::W => self.directs |= 1 << 2,
            Keycode::L => self.actions |= 1 << 2,
            Keycode::S => self.directs |= 1 << 3,
            Keycode::H => self.actions |= 1 << 3,
            _ => self.high_to_low = false,
        }
    }
}

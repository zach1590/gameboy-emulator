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
    high_to_low: bool,
    something_selected: bool,
}

impl Joypad {
    pub fn new() -> Joypad {
        return Joypad {
            event_pump: None,
            joyp: 0xCF,
            high_to_low: false,
            something_selected: false,
        };
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
                self.joyp = (data & 0x30) | (self.joyp & 0x0F);
                self.something_selected = self.joyp & 0x30 != 0x30;
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
            // If this doesnt work use poll_iter()
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
                        self.handle_key_event(x);
                    }
                    _ => self.high_to_low = false, // Nothing pressed
                }
            }
        }

        return should_exit;
    }

    fn handle_key_event(self: &mut Self, key: Keycode) {
        match key {
            Keycode::D | Keycode::J => {
                // Right or A (Bit 0)
                self.high_to_low = true;
                self.joyp = (self.joyp | 0x0F) & 0xFE;
                println!("Pressed Right or A");
            }
            Keycode::A | Keycode::K => {
                // Left or B (Bit 1)
                self.high_to_low = true;
                self.joyp = (self.joyp | 0x0F) & 0xFD;
                println!("Pressed Left or B");
            }
            Keycode::W | Keycode::L => {
                // Up or Select (Bit 2)
                self.high_to_low = true;
                self.joyp = (self.joyp | 0x0F) & 0xFB;
                println!("Pressed Up or Select");
            }
            Keycode::S | Keycode::H => {
                // Down or Start (Bit 3)
                self.high_to_low = true;
                self.joyp = (self.joyp | 0x0F) & 0xF7;
                println!("Pressed Down or Start");
                }
                _ => {
                    // Nothing pressed
                    self.joyp = self.joyp | 0x0F;
                    self.high_to_low = false;
            }
        }
    }
}

#[cfg(test)]
#[test]
pub fn test_handle_key_event() {
    let mut joypad = Joypad::new();

    joypad.joyp = 0xCF;
    joypad.handle_key_event(Keycode::D);
    assert_eq!(joypad.joyp, 0xCE);

    joypad.handle_key_event(Keycode::K);
    assert_eq!(joypad.joyp, 0xCD);

    joypad.joyp = 0x2F;
    joypad.handle_key_event(Keycode::L);
    assert_eq!(joypad.joyp, 0x2B);

    joypad.handle_key_event(Keycode::H);
    assert_eq!(joypad.joyp, 0x27);
}

#[test]
pub fn test_joypad_interrupt() {
    let mut joypad = Joypad::new();

    joypad.high_to_low = true;
    joypad.write_byte(JOYP_REG, 0x27);
    assert_eq!(true, joypad.is_joypad_interrupt());

    joypad.write_byte(JOYP_REG, 0x3B);
    assert_eq!(false, joypad.is_joypad_interrupt());

    joypad.write_byte(JOYP_REG, 0x37);
    assert_eq!(false, joypad.is_joypad_interrupt());

    joypad.high_to_low = false;
    joypad.write_byte(JOYP_REG, 0x1E);
    assert_eq!(false, joypad.is_joypad_interrupt());

    // Should the case where both selection inputs are selected
    // return true or it should be useless entirely?
    joypad.high_to_low = true;
    joypad.write_byte(JOYP_REG, 0x0E);
    assert_eq!(true, joypad.is_joypad_interrupt());

    joypad.write_byte(JOYP_REG, 0x7E);
    assert_eq!(false, joypad.is_joypad_interrupt());

    joypad.high_to_low = false;
    joypad.write_byte(JOYP_REG, 0x2E);
    assert_eq!(false, joypad.is_joypad_interrupt());
}

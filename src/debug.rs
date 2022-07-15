use super::io::{Io, SB_REG, SC_REG};

pub fn update_serial_buffer(io: &mut Io) {

    if io.read_byte(SC_REG) == 0x81{
        let c: char = io.read_byte(SB_REG) as char;

        io.write_byte(SC_REG, 0x00);

        print!("{}", c);
    }
}
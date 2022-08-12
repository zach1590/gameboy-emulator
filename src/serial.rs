use super::cpu::CPU_FREQ;
use super::io::Io;

pub const SB_REG: u16 = 0xFF01;
pub const SC_REG: u16 = 0xFF02;
const INTERNAL_FREQ: usize = 8_192;
const MAX_TRANSFER_CYCLES: usize = CPU_FREQ / INTERNAL_FREQ;

pub struct Serial {
    sb: u8, // 0xFF01
    sc: u8, // 0xFF02
    transferring: bool,
    transfer_cycles: usize,
}

impl Serial {
    pub fn new() -> Serial {
        return Serial {
            sb: 0,
            sc: 0,
            transferring: false,
            transfer_cycles: 0,
        };
    }

    pub fn read_byte(self: &Self, addr: u16) -> u8 {
        return match addr {
            SB_REG => self.sb,
            SC_REG => self.sc,
            _ => panic!("Serial doesnt handle reads from addr: {}", addr),
        };
    }

    pub fn write_byte(self: &mut Self, addr: u16, data: u8) {
        match addr {
            SB_REG => self.sb = data,
            SC_REG => {
                self.sc = data | 0b01111110;

                if self.sc & 0x81 == 0x81 {
                    self.start_transfer();
                    #[cfg(feature = "debug")]
                    {
                        // Only log on the write so no need to unset here
                        // print!("{}", self.sb as char);
                        print!("{:02X}", self.sb);
                    }
                }
            }
            _ => panic!("Serial doesnt handle writes to addr: {}", addr),
        }
    }

    pub fn adv_cycles(self: &mut Self, io: &mut Io, cycles: usize) {
        if !self.transferring {
            return;
        }

        self.transfer_cycles += cycles;

        // Just do the fake transfer all at once
        // No clue if anything is right
        if self.transfer_cycles >= MAX_TRANSFER_CYCLES {
            self.transferring = false;
            self.send();
            self.sb = self.receive();
            self.sc &= 0x7F;
            io.request_serial_interrupt();
        }
    }

    pub fn dmg_init(self: &mut Self) {
        self.sb = 0x00;
        self.sc = 0x7E;
    }

    fn start_transfer(self: &mut Self) {
        self.transferring = true;
        self.transfer_cycles = 0;
    }

    fn send(self: &mut Self) {
        // Empty for now since we dont actually send out anything
    }

    fn receive(self: &mut Self) -> u8 {
        0xFF // Nothing will be connected to this emulator so it will always receive 0xFF
    }
}

mod battery;
pub mod cartridge;
pub mod mbc1;
pub mod mbc3;
pub mod mbc5;
pub mod mbc_none;
mod mbc_timer;

pub trait Mbc {
    fn read_ram_byte(self: &Self, addr: u16) -> u8;
    fn write_ram_byte(self: &mut Self, addr: u16, val: u8);
    fn read_rom_byte(self: &Self, addr: u16) -> u8;
    fn write_rom_byte(self: &mut Self, addr: u16, val: u8);
    fn adv_cycles(self: &mut Self, cycles: usize);
    fn load_game(
        self: &mut Self,
        game_path: &str,
        game_bytes: Vec<u8>,
        features: Vec<&str>,
        rom_size: usize,
        rom_banks: usize,
        ram_size: usize,
        ram_banks: usize,
    );
}

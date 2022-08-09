use super::mbc_timer::MbcTimer;
use core::panic;
use std::fs::File;
use std::io::ErrorKind;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;

pub struct Battery {
    ram_path: String,
    ram_file_size: u64,
    ram_file: Option<File>,
    ram_new_file: bool,
    rtc_file: Option<File>,
    rtc_new_file: bool,
}

impl Battery {
    pub fn new() -> Battery {
        Battery {
            ram_path: String::new(),
            ram_file_size: 0x00,
            ram_file: None,
            ram_new_file: false,
            rtc_file: None,
            rtc_new_file: false,
        }
    }

    pub fn with_ram(mut self: Self, ram_path: String, file_size: u64) -> Battery {
        let file;
        self.ram_new_file = false;
        let try_open = File::options()
            .read(true)
            .write(true)
            .open(ram_path.clone());

        match try_open {
            Ok(x) => file = x,
            Err(e) => match e.kind() {
                ErrorKind::NotFound => {
                    match File::options()
                        .read(true)
                        .write(true)
                        .create_new(true)
                        .open(ram_path.clone())
                    {
                        Ok(f) => {
                            file = f;
                            self.ram_new_file = true;
                        }
                        Err(e) => panic!("Problem creating the ram file: {:?}", e),
                    }
                }
                _ => panic!("Unable to handle open file error for ram file {}", e),
            },
        }

        if self.ram_new_file {
            if let Ok(()) = file.set_len(file_size) {
                // nice
            } else {
                panic!("Error trying to set a length for the ram file");
            }
        }

        self.ram_path = ram_path;
        self.ram_file_size = file_size;
        self.ram_file = Some(file);
        return self;
    }

    pub fn with_rtc(mut self: Self, rtc_path: String) -> Battery {
        let file;
        self.rtc_new_file = false;
        let try_open = File::options()
            .read(true)
            .write(true)
            .open(rtc_path.clone());

        match try_open {
            Ok(x) => file = x,
            Err(e) => match e.kind() {
                ErrorKind::NotFound => {
                    match File::options()
                        .read(true)
                        .write(true)
                        .create_new(true)
                        .open(rtc_path.clone())
                    {
                        Ok(f) => {
                            file = f;
                            self.rtc_new_file = true; // Needed for when we load the rtc registers
                        }
                        Err(e) => panic!("Problem creating the rtc file: {:?}", e),
                    }
                }
                _ => panic!("Unable to handle open file error for rtc file {}", e),
            },
        }

        self.rtc_file = Some(file);
        return self;
    }

    pub fn save_ram(self: &mut Self, ram_buffer: &Vec<u8>) {
        if let Some(ram_file) = &mut self.ram_file {
            ram_file.seek(SeekFrom::Start(0)).unwrap();
            if let Ok(()) = ram_file.write_all(ram_buffer) {
            } else {
                println!("Saving ram did not go well");
            }
        }
    }

    // If we created/opened a new file (no previous save state) then just return an empty
    // vector with the needed capacity. Otherwise read the entire file into a vector and
    // return the vector
    pub fn load_ram(self: &mut Self) -> Vec<u8> {
        let ram_size = usize::try_from(self.ram_file_size).unwrap();

        if self.ram_new_file {
            return vec![0; ram_size];
        } else {
            match &mut self.ram_file {
                Some(ram_file) => {
                    let mut buf = Vec::new();
                    let bufsize = ram_file.read_to_end(&mut buf).unwrap();

                    if bufsize != ram_size {
                        panic!(
                            "Ram size ({}) does not equal the file size read in ({})",
                            self.ram_file_size, bufsize
                        );
                    } else {
                        return buf;
                    }
                }
                None => panic!("Not a new ram file but somehow no ram file exists"),
            }
        }
    }

    // Will save the rtc data to a new file appended with the following: `.gbrtc`
    pub fn save_rtc(self: &mut Self, latched_rtc: &MbcTimer, rtc: &MbcTimer) {}
    pub fn load_rtc(self: &mut Self, latched_rtc: &mut MbcTimer, rtc: &mut MbcTimer) {}
}

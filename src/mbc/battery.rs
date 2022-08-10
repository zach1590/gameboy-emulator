use super::mbc_timer::MbcTimer;
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
            match ram_file.seek(SeekFrom::Start(0)) {
                Ok(_x) => {
                    if let Ok(()) = ram_file.write_all(ram_buffer) {
                    } else {
                        println!("Saving ram did not go well");
                    }
                }
                Err(_err) => println!("Saving ram failed while seeking the start of the file"),
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

    // Store the time since unix_epoch, and the current registers values inside a value
    // Save in the following order: latched_rtc, , updated_rtc, current_time
    pub fn save_rtc(
        self: &mut Self,
        latched_rtc: &MbcTimer,
        updated_rtc: &MbcTimer,
    ) -> Result<usize, std::io::Error> {
        if let Some(rtc_file) = &mut self.rtc_file {
            return match rtc_file.seek(SeekFrom::Start(0)) {
                Ok(_x) => {
                    // Is there a simpler way to do this?
                    let latch_bytes = latched_rtc.to_secs().to_le_bytes();
                    let update_bytes = updated_rtc.to_secs().to_le_bytes();
                    let save_bytes = MbcTimer::get_current_time().to_le_bytes();
                    let bytes_written = rtc_file.write(&latch_bytes)?
                        + rtc_file.write(&update_bytes)?
                        + rtc_file.write(&save_bytes)?;

                    Ok(bytes_written)
                }
                Err(err) => {
                    println!("Error while trying to seek the start of the rtc file");
                    Err(err)
                }
            };
        }
        return Ok(0);
    }
    pub fn load_rtc(
        self: &mut Self,
        latched_rtc: &mut MbcTimer,
        updated_rtc: &mut MbcTimer,
    ) -> u64 {
        if self.rtc_new_file {
            return 0;
        } else {
            match &mut self.rtc_file {
                Some(rtc_file) => {
                    let expected_size = 24;
                    let mut buf: Vec<u8> = Vec::with_capacity(expected_size);
                    let bufsize = rtc_file.read_to_end(&mut buf).unwrap();

                    if expected_size != bufsize {
                        panic!("Expected 24 bytes but got: {}", bufsize);
                    }

                    let latch_time = u64::from_le_bytes(buf[0..=7].try_into().unwrap());
                    let update_time = u64::from_le_bytes(buf[8..=15].try_into().unwrap());
                    let save_time = u64::from_le_bytes(buf[16..=23].try_into().unwrap());

                    latched_rtc.from_secs(latch_time);
                    updated_rtc.from_secs(update_time);
                    return save_time;
                }
                None => panic!("Not a new rtc file but somehow no rtc information exists"),
            }
        }
    }
}

/*
    This is worthless but I needed to make sure the length
    of the bytes is always 8 (padded with 0s) for when I read
    eventually read them back in, I know the length to read
*/
#[test]
fn test_to_le_bytes() {
    let mut timer = MbcTimer::new();
    timer.from_secs(44_236_799); // 510 days, 23 hours, 59 mins 59 secs
    assert_eq!(timer.to_secs().to_le_bytes().len(), 8);

    timer.from_secs(1); // 510 days, 23 hours, 59 mins 59 secs
    assert_eq!(timer.to_secs().to_le_bytes().len(), 8);
}

#[test]
fn test_byte_conversion() {
    let buf = [
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
        0x0F,
    ];
    let latch_time = u64::from_le_bytes(buf[0..=7].try_into().unwrap());
    let update_time = u64::from_le_bytes(buf[8..=15].try_into().unwrap());

    assert_eq!(latch_time, 0x07_06_05_04_03_02_01_00);
    assert_eq!(update_time, 0x0F_0E_0D_0C_0B_0A_09_08);
}

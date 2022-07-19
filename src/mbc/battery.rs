use std::fs::File;
use std::io::ErrorKind;
use std::io::Read;
use std::io::Write;
use std::io::Seek;
use std::io::SeekFrom;

pub struct Battery {
    ram_path: String,
    file_size: u64,
    file: File,
    new_file: bool,
}

impl Battery {
    pub fn new(ram_path: String, file_size: u64) -> Battery {
        
        let file;
        let mut new_file = false;
        let try_open = File::options().read(true).write(true).open(ram_path.clone());
        
        match try_open {
            Ok (x) => file = x,
            Err(e) => {
                match e.kind() {
                    ErrorKind::NotFound => {
                        match File::options().read(true).write(true).create_new(true).open(ram_path.clone()) {
                            Ok(f) => {
                                file = f;
                                new_file = true;
                            },
                            Err(e) => panic!("Problem creating the file: {:?}", e),
                        }
                    },
                    _ => panic!("Unable to handle open file error {}", e)
                }
            },
        }

        if new_file {
            if let Ok(()) = file.set_len(file_size){
                // nice
            } else {
                panic!("Error trying to set a length for the file");
            }
        }

        Battery {
            ram_path: ram_path.clone(),
            file_size: file_size,
            file: file,
            new_file: new_file,
        }
    }

    pub fn save_ram(self: &mut Self, ram_buffer: &Vec<u8>) {
        self.file.seek(SeekFrom::Start(0)).unwrap();
        if let Ok(()) = self.file.write_all(ram_buffer) {}
        else { println!("Saving ram did not go well"); }
    }

    // If we created/opened a new file (no previous save state) then just return an empty
    // vector with the needed capacity. Otherwise read the entire file into a vector and
    // return the vector
    pub fn load_ram(self: &mut Self) -> Vec<u8> {

        let ram_size = usize::try_from(self.file_size).unwrap();
        if self.new_file {
            return vec![0; ram_size];
        } else {
            let mut buf = Vec::new();
            let bufsize = self.file.read_to_end(&mut buf).unwrap();
            if bufsize != ram_size { 
                panic!("Ram size ({}) does not equal the file size read in ({})", self.file_size, bufsize); 
            }
            else {
                return buf;
            }
        }
    }
}
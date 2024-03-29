use super::cpu;
use super::graphics::{NUM_PIXELS_X, NUM_PIXELS_Y, SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::mbc::cartridge;

use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::Sdl;
use sdl2::VideoSubsystem;

use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::str;

const CPU_PERIOD_NANOS: f64 = 238.418579;

pub struct Emulator {
    cpu: cpu::Cpu,
    cart: cartridge::Cartridge,
    sdl_context: Option<Sdl>,
    video_subsystem: Option<VideoSubsystem>,
    file_writer: Option<BufWriter<File>>,
}

impl Emulator {
    pub fn new() -> Emulator {
        return Emulator {
            cpu: cpu::Cpu::new(),
            cart: cartridge::Cartridge::new(),
            sdl_context: None,
            video_subsystem: None,
            file_writer: None,
        };
    }

    // We just want the mbc type really, we wont bother with the nintendo logo boot
    // Will fail if anything required for setting up the emulator for playing fails
    pub fn setup_emulator(self: &mut Self, game_path: &str) {
        let sdl_context = sdl2::init().expect("Couldnt create sdl context"); // SDL for graphics, sound and input

        let video_subsystem = sdl_context // Init Display
            .video()
            .expect("Couldnt initialize video subsystem");

        let event_pump = sdl_context
            .event_pump()
            .expect("Coulnt initialize event pump"); // Init Event System

        // let sound_system = SoundSystem::initialize(&sdl_context); // Init Sound System (ownership of this one will go to sound.rs)

        let cart_mbc = self.cart.read_cartridge_header(game_path).unwrap();

        self.sdl_context = Some(sdl_context); // Just need to make sure the context doesnt die
        self.video_subsystem = Some(video_subsystem); // Just need to make sure the context doesnt die
        self.cpu.set_mbc(cart_mbc); // Cartridge header had what mbc to use
        self.cpu.set_joypad(event_pump); // Joypad will own the event pump
        self.cpu.dmg_init(self.cart.checksum_val); // Setup registers

        #[cfg(feature = "debug-file")]
        {
            self.file_writer = Some(BufWriter::new(self.setup_debug_file(game_path)));
        }
    }

    pub fn run(self: &mut Self) {
        // Put these in graphics somehow
        let video_subsystem = match &self.video_subsystem {
            Some(videosys) => videosys,
            None => panic!("No video subsystem was initialized"),
        };

        let window = video_subsystem
            .window("Rust-Gameboy-Emulator", SCREEN_WIDTH, SCREEN_HEIGHT)
            .position_centered()
            .build()
            .unwrap();

        let mut canvas = window // Canvas is the renderer
            .into_canvas()
            .accelerated()
            .build()
            .unwrap();

        let creator = canvas.texture_creator();
        let mut texture = creator
            .create_texture_streaming(PixelFormatEnum::ARGB8888, NUM_PIXELS_X, NUM_PIXELS_Y)
            .map_err(|e| e.to_string())
            .unwrap();

        let rect = Some(Rect::new(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT));

        let x1 = std::time::Instant::now();
        let mut counter: u128 = 0;

        #[cfg(feature = "debug")]
        let mut dbug = String::new();

        // Game loop
        loop {
            #[cfg(feature = "debug")]
            {
                dbug.clear();
                self.cpu.get_debug_info(counter, &mut dbug);

                #[cfg(feature = "debug-file")]
                {
                    self.write_to_file(&mut dbug);
                }
                #[cfg(feature = "debug-logs")]
                {
                    println!("{}", dbug);
                }
            }

            if self.cpu.update_input() {
                // Is true when we get the exit signal
                break;
            }
            self.cpu.check_interrupts();

            if self.cpu.is_running {
                self.cpu.curr_cycles = 0;
                self.cpu.execute();
            } else {
                // Halted
                self.cpu.curr_cycles = 4;
                self.cpu.adv_cycles(4); // Should this be 1 or 4?
            }

            if self.cpu.update_display(&mut texture) {
                canvas.copy(&texture, None, rect).unwrap();
                canvas.present();
            }

            counter = counter.wrapping_add(1);
            #[cfg(feature = "blargg")]
            {
                if self.cpu.is_blargg_done() == true {
                    let y1 = x1.elapsed().as_nanos();
                    println!("{}ns to complete test", y1);
                    println!("About {}ns per loop", y1 / counter);
                    std::thread::sleep(std::time::Duration::from_secs(5));
                    break;
                }
            }
            #[cfg(feature = "mooneye")]
            {
                if self.cpu.is_mooneye_done() == true {
                    let y1 = x1.elapsed().as_nanos();
                    println!("\n{}ns to complete test", y1);
                    println!("About {}ns per loop", y1 / counter);
                    std::thread::sleep(std::time::Duration::from_secs(5));
                    break;
                }
            }
        }
    }

    #[cfg(feature = "debug-file")]
    fn setup_debug_file(self: &mut Self, game_path: &str) -> File {
        std::fs::create_dir_all("./debug-info").unwrap();
        let clean_path = game_path.replace('\\', "/");

        let pos_last_slash = match clean_path.rfind('/') {
            Some(x) => x,
            None => 0,
        };
        let pos_dotgb = match clean_path.rfind(".gb") {
            Some(x) => x,
            None => panic!("Not a gameboy file, no .gb suffix"),
        };

        let mut path = format!(
            "./debug-info/{}.txt",
            clean_path[pos_last_slash..pos_dotgb].to_string()
        );

        println!("path: {}", path);

        let mut i = 0;
        while std::path::Path::new(&path).exists() {
            path = format!(
                "./debug-info/{}{}.txt",
                clean_path[pos_last_slash..pos_dotgb].to_string(),
                i
            );
            i += 1;
        }

        let file = File::options()
            .read(true)
            .write(true)
            .create_new(true)
            .open(path)
            .expect("Could not create logging file");

        return file;
    }

    #[cfg(feature = "debug-file")]
    pub fn write_to_file(self: &mut Self, dbug: &mut String) {
        match &mut self.file_writer {
            Some(writer) => {
                writer.write_all(dbug.as_bytes()).unwrap();
            }
            _ => {}
        }
    }
}

/*
    thread::sleep sucks, need to do either audio or video sync to emulate
    proper speed rather than trying to sync between instructions.

    Currently doing video sync but should switch to audio when implemented

    If we choose audio it works differently (Need to figure this out)
    https://forums.nesdev.org/viewtopic.php?f=3&t=15405
*/

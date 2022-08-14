use super::cpu;
use super::graphics::{NUM_PIXELS_X, NUM_PIXELS_Y, SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::mbc::cartridge;

use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::Sdl;
use sdl2::VideoSubsystem;

const CPU_PERIOD_NANOS: f64 = 238.418579;

pub struct Emulator {
    cpu: cpu::Cpu,
    cart: cartridge::Cartridge,
    sdl_context: Option<Sdl>,
    video_subsystem: Option<VideoSubsystem>,
}

impl Emulator {
    pub fn new() -> Emulator {
        return Emulator {
            cpu: cpu::Cpu::new(),
            cart: cartridge::Cartridge::new(),
            sdl_context: None,
            video_subsystem: None,
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

        #[cfg(feature = "debug")]
        let x1 = std::time::Instant::now();
        #[cfg(feature = "debug")]
        let mut counter = 0;
        // self.cpu.set_debug_file();

        // Game loop
        loop {
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

            #[cfg(feature = "debug")]
            {
                counter += 1;
                if self.cpu.is_blargg_done() == true {
                    let y1 = x1.elapsed().as_nanos();
                    println!("{}ns to complete test", y1);
                    println!("About {}ns per loop", y1 / counter);
                    std::thread::sleep(std::time::Duration::from_secs(5));
                    break;
                }
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
}

/*
    thread::sleep sucks, need to do either audio or video sync to emulate
    proper speed rather than trying to sync between instructions.

    Currently doing video sync but should switch to audio when implemented

    If we choose audio it works differently (Need to figure this out)
    https://forums.nesdev.org/viewtopic.php?f=3&t=15405
*/

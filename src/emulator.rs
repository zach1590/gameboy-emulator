use super::cartridge;
use super::cpu;
use super::graphics::{NUM_PIXELS_X, NUM_PIXELS_Y, SCREEN_HEIGHT, SCREEN_WIDTH};

use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::EventPump;
use sdl2::Sdl;
use sdl2::VideoSubsystem;

pub struct Emulator {
    cpu: cpu::Cpu,
    cart: cartridge::Cartridge,
}

impl Emulator {
    pub fn new() -> Emulator {
        return Emulator {
            cpu: cpu::Cpu::new(),
            cart: cartridge::Cartridge::new(),
        };
    }

    // We just want the mbc type really, we wont bother with the nintendo logo boot
    pub fn insert_cartridge(self: &mut Self, game_path: &str) {
        let cart_mbc = self.cart.read_cartridge_header(game_path);
        self.cpu.set_mbc(cart_mbc);
        let checksum = self.cart.checksum(self.cpu.get_memory());
        self.cpu.dmg_init(checksum);
    }

    pub fn run(self: &mut Self) {
        let sdl_context = sdl2::init().unwrap(); // SDL for graphics, sound and input
        let video_subsystem = sdl_context.video().unwrap(); // Init Display
        let event_pump = sdl_context.event_pump().unwrap(); // Init Event System

        // let mut sound_system = SoundSystem::initialize(&sdl_context); // Init Sound System

        self.cpu.set_joypad(event_pump);

        // Put these in graphics
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

        let rect = Rect::new(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT);

        #[cfg(feature = "debug")]
        let x1 = std::time::Instant::now();

        // Game loop
        loop {
            if self.cpu.update_input() {
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

            #[cfg(feature = "debug")]
            {
                if self.cpu.is_blargg_done() == true {
                    let y1 = x1.elapsed().as_millis();
                    println!("{}ms to complete test", y1);

                    let x2 = std::time::Instant::now();

                    self.cpu.display_tiles(&mut texture);
                    canvas.copy(&texture, None, Some(rect)).unwrap(); // Update canvas
                    canvas.present();

                    let y2 = x2.elapsed().as_millis();
                    println!("{}ms to render screen", y2);
                    std::thread::sleep(std::time::Duration::from_secs(10));
                    break;
                }
            }
        }
    }
}

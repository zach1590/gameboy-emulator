## **An M Cycle Accurate Gameboy Emulator Programmed in Rust**

## Currently In Development
 - The 2 mains goals of this project are to make a fairly accurate emulator and also learn the fundamentals of rust. It won't be the most accurate gameboy emulator in the world but I do want it to pass as many well known test roms as possible (blargg, mooneye, dmg-acid2, etc).
 - The main feature this emulator is lacking would be audio but that is next to be added (I am struggling with audio documentation so I will probably come back to this in the future)
 - Skip to [**Testing**](#testing) to see the current test roms its passes
 - Skip to [**Current Features**](#current-features) to see what is currently emulated
 - Skip to [**Next Features**](#next-features) to see what's planned
 - Skip to [**How to Run**](#how-to-run) to try the emulator out

## **Screenshots**

|||
| ------------- | ------------- |
| ![](./screenshots/pokemon-red-screen.jpg)  | ![](./screenshots/pokemon-silver-screen.jpg)  |
| ![](./screenshots/kirby.jpg)  | ![](./screenshots/kirby1.jpg)  |
| ![](./screenshots/megamanv.jpg) | ![](./screenshots/mario.jpg) | 
| ![](./screenshots/dmg-acid2.jpg) | |

## **Button Mappings**
#### **Gameboy Button ==> Physical Keyboard**

Up ==> Up Arrow

Down ==> Down Arrow

Left ==> Left Arrow

Right ==> Right Arrow

A ==> F

B ==> D

Start ==> Right Shift

Select ==> Enter/Return

## **How to Run**

**Install/Build Requirements**
 - Rust
 - SDL2*

*Your main issue with running will probably be that the emulator uses sdl2. I've been developing this on windows and used `features = ["bundled"]` for the sdl2 dependency in the `cargo.toml`. This should mean you wont need to install sdl2 but you will need the tools to build it. I was missing the following on windows and linux:

* **Windows:** CMake

* **Linux:** libXext-dev (This is the only one I was missing, if your trying to run this from scratch you'll probably need other libraries aswell like `build-essential`.

**Run Command**
 - `cargo run <rom-name>` at the root of the repository

**Debugging Features**
 - `cargo run --features "debug-file"` (Output some register and mmio information to a file with the name `<rom-name>.txt`)
 - `cargo run --features "debug-logs"` (Output some register and mmio information to the console)
 - `cargo run --features "blargg"` (Stop a blargg test automatically)
 - `cargo run --features "mooneye"` (Stop an mts test automatically)
Cargo also allows you to combine the above: `cargo run --features "mooneye debug-file"`

Regarding the `debug-file` feature
 - Be careful not to leave it running for very long or the file will become extremely large.
 - The file will be placed in a folder called `debug-info/` which will be created in the root of this repo if it doesnt already exist
 - If the file already exists, a number will be appended on the end of the file name.

## **Tested Games**
These are just the ones I've played at least a bit:
 - Pokemon Red
 - Kirby's Dream Land
 - Super Mario Land (World)
 - Mega Man V
 - Tetris

## **Testing**
Currently Passes the Following Test Roms:
 - **Blargg Tests**
   - cpu_instrs
   - instr_timing
   - mem_timing/mem_timing2
   - halt_bug
 - **dmg-acid2** (https://github.com/mattcurrie/dmg-acid2)
 - **Emulator-Only** (Mooneye - mts-20220522-1522-55c535c)
    - MBC1
    - MBC5
 - **Acceptance** (Mooneye - mts-20220522-1522-55c535c)
    - oam_dma/
    - bits/
    - instr/
    - timer/
    - General (Dont know what to call these)
         - boot_div-dmgABCmgb
         - boot_hwio-dmgABCmgb
         - boot_reg-dmgABC
         - call_timing2
         - call_cc_timing2
         - di_timing-GS
         - div_timing
         - ei_sequence
         - ei_timing
         - halt_ime0_ei
         - halt_ime0_nointr_timing
         - halt_ime1_timing
         - halt_ime1_timing2-GS
         - if_ie_registers
         - intr_timing
         - pop_timing
         - push_timing
         - rapid_di_ei
         - reti_intr_timing
         - rst_timing
         - oam_dma_restart
         - oam_dma_timing
         - oam_dma_start
         - add_sp_e_timing
         - call_timing
         - call_cc_timing
         - jp_cc_timing
         - jp_timing
         - ld_hl_sp_e_timing
         - ret_cc_timing
         - ret_timing
         - reti_timing
 - **Manual-Only** (Mooneye - mts-20220522-1522-55c535c)
    - sprite_priority (well.. as far as my eyes can tell)
 - **rtc3test**
    - basic tests (https://github.com/aaaaaa123456789/rtc3test/blob/master/tests.md#basic-tests)
 - **Miscalleneous**
    - lycscx.gb
    - lycscy.gb

Tests I care about that are failing from Mooneye Acceptance:
 - ppu/ - All of them fail :(

## **Features**

#### **Current Features**
 - Memory Bank Controllers
   - None
   - MBC1 (Multicart Not implemented)
   - MBC3 with RTC3 (Passes basic rtc3 test)
   - MBC5
   - Battery for ram
 - CPU
 - Haltbug
 - Interrupts
 - DMA Transfer
 - Stat Blocking (Need to Test)
 - DMG Stat Quirk/Bug (Need to test)
 - PPU (Doesnt extend mode 3 properly)

#### **Next Features**
 - Sound (Aim is to pass blargg test)
 - Mooneye Acceptance PPU
 - Pass as many of Mealybug Tearoom Tests as possible
 - MBC2

#### **Maybe Features**
 - CGB Support

#### **Not Planned Features**
 - OAM Corruption Bug
 - MBC4, and the more obscure ones
 - Peripherals (Camera, Infrared Communication)
 - Multicart Roms
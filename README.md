## **An M Cycle Accurate Gameboy Emulator Programmed in Rust**

## Currently In Development
 - The 2 mains goals of this project are to make a fairly accurate emulator and also learn the fundamentals of rust. It won't be the most accurate gameboy emulator in the world but I do want it to pass as many well known test roms as possible (blargg, mooneye, dmg-acid2, etc).
 - The main feature the emulator is currently lacking is audio but I do plan to add it
 - Skip to [**Testing**](Testing) to see the current test roms its passes
 - Skip to [**Current Features**](Current-Features) to see what is currently emulated
 - Skip to [**Next Features**](Next-Features) to see what's planned
 - Currently have issues with, I think, the interrupts. Pokemon Red hangs after Prof. Oak introduces himself but in Pokemon Silver we make it through it perfectly okay so what games work may be completely up to chance currently.

## **Screenshots**
![](./screenshots/pokemon-red-screen.jpg)
![](./screenshots/pokemon-silver-screen.jpg)
![](./screenshots/dmg-acid2.jpg)

## **Button Mappings**
#### **Gameboy Button ==> Physical Keyboard**

Up ==> W

Down ==> S

Left ==> A

Right ==> D

A ==> J

B ==> K

Start ==> H

Select ==> L

## **Testing**
Currently Passes the Following Test Roms:
 - `cpu_instrs` (Blargg)
 - `instr_timing` (Blargg)
 - `mem_timing` / `mem_timing-2` (Blargg) 
 - `halt_bug` (Blargg)
 - `dmg-acid2` (https://github.com/mattcurrie/dmg-acid2)
 - `Emulator-Only` (Mooneye - mts-20220522-1522-55c535c)
    - `MBC1`
    - `MBC5`
 - `Acceptance` (Mooneye - mts-20220522-1522-55c535c)
    - `oam_dma*` (The ones in the directory `oam_dma/`)
    - `bits`
    - `instr`
 - `rtc3test`
    - `basic tests` (https://github.com/aaaaaa123456789/rtc3test/blob/master/tests.md#basic-tests)

## **Features**

#### **Current Features**
 - Memory Bank Controllers
   - None
   - MBC1 (Multicart Not implemented)
   - MBC3 with RTC3 (Passes basic rtc3 test)
   - MBC5
 - CPU
 - Haltbug
 - Interrupts
 - DMA Transfer
 - Stat Blocking (Need to Test)
 - DMG Stat Quirk/Bug (Need to test)
 - PPU (Doesnt extend mode 3 properly)

#### **Next Features**
 - Pass as many of Mooneye's Acceptance Tests as possible
 - Sound
 - Implement Extending Mode 3 of PPU
 - Pass as many of Mealybug Tearoom Tests as possible
 - MBC2

#### **Maybe Features**
 - CGB Support

#### **Not Planned Features**
 - OAM Corruption Bug
 - MBC4, and the more obscure ones
 - Peripherals (Camera, Infrared Communication)
 - Multicart Roms
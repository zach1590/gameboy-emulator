## **An M Cycle Accurate Gameboy Emulator Programmed in Rust**

## Currently In Development
 - The 2 mains goals of this project are to make a very accurate emulator and also learn the fundamentals of rust. 
 - This won't be the most accurate gameboy emulator in the world but I do want it to pass as many well known test roms as possible (blargg, mooneye, dmg-acid2, etc).
 - The main feature the emulator is currently lacking is audio but I do plan to add it

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
    - `oam_dma`
    - `bits`
    - `instr`

## **Features**

#### **Current Features**
 - Memory Bank Controllers
   - None
   - MBC1 (Multicart Not implemented)
   - MBC3 with RTC3 (Fails RTC Test currently)
   - MBC5
 - CPU
 - Interrupts
 - DMA Transfer
 - Stat Blocking
 - DMG Stat Quirk/Bug (Need to test)
 - PPU (Passes dmg-acid2 but doesnt extend mode 3 apart from scx register)

#### **Next Features**
 - Pass as many of Mooneye's Acceptance Tests as possible
 - Sound
 - Implement Extending Mode 3 of PPU
 - Pass as many of Mealybug Tearoom Tests as possible
 - MBC2
 - MBC5

#### **Maybe Features**
 - CGB Support

#### **Not Planned Features**
 - OAM Corruption Bug
 - MBC4, and the more obscure ones
 - Peripherals (Camera, Infrared Communication)
 - Multicart Roms
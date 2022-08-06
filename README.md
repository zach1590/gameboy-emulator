## **M Cycle Accurate Gameboy Emulator Programmed in Rust**

## Currently In Development


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

## **Features**

#### **Current Features**
 - Memory Bank Controllers Implemented
   - None
   - MBC1
 - CPU
 - Interrupts
 - DMA Transfer
 - Stat Blocking
 - DMG Stat Quirk/Bug
 - PPU (Currently unable to extend mode 3 properly)

#### **Next Features**
 - MBC3
 - Sound
 - Implement Extending Mode 3 of PPU
 - Mealybug Tearoom Tests
 - Mooneye Tests
 - MBC2

#### **Maybe Features**
 - CGB Support

#### **Not Planned Features**
 - OAM Corruption Bug
 - MBC4 and higher
 - Peripherals (Camera, Infrared Communication)
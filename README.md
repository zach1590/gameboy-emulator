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
#### **Blargg**
Currently Passes Following Blargg Tests using Serial Output:
 - `cpu_instrs`
 - `instr_timing`
 - `mem_timing` / `mem_timing-2`

## **Features**

#### **Current Features**
 - Memory Bank Controllers Implemented
   - None
   - MBC1
 - CPU
 - Interrupts (Except Serial)
 - DMA Transfer
 - Stat Blocking
 - DMG Stat Quirk/Bugd
 - Serial Output (For Blargg Tests)

#### **Next Features**
 - Complete PPU
 - MBC3
 - `halt_bug` (Blargg Test)
 - Sound
 - Mooneye Tests
 - MBC2

#### **Not Planned Features**
 - OAM Corruption Bug
 - MBC4 and higher
 - Peripherals (Camera, Infrared Communication)
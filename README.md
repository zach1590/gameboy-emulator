Cycle Accurate Gameboy Emulator Programmed in Rust

Currently In Development

Currently Passes Following Blargg Tests using Serial Output:
 - `cpu_instrs`
 - `instr_timing`
 - `mem_timing` / `mem_timing-2`

 Memory Bank Controllers Implemented
 - None
 - MBC1

Next:
 - PPU
 - All Interrupts
 - Input
 - MBC2
 - MBC3
 - `interrupt_time` (Blargg)
 - `halt_bug` (Blargg)
    - Have the code for haltbug but interrupts not complete
 - Sound (Maybe)

Not Planned
 - OAM Corruption Bug
 - MBC4 and higher
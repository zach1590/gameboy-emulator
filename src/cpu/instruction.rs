pub struct Instruction {
    pub values: (u8, u8),
    pub opcode: u8,
}
impl Instruction {
    pub fn get_instruction(x: u8) -> Instruction {
        return Instruction {
            values: (((x & 0x00F0) >> 4) as u8, (x & 0x000F) as u8),
            opcode: x,
        };
    }
}

// Make this print the hexadecimal values not decimal
impl ToString for Instruction {
    fn to_string(&self) -> String {
        let mut opcode = self.values.0.to_string();
        opcode.push_str(", ");
        opcode.push_str(&self.values.1.to_string());
        return opcode;
    }
}

// TODO Opcode to Mnemonics Translator for Debugger

#[cfg(test)]
#[path = "../tests/instruction_tests.rs"]
mod instruction_tests;

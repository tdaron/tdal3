#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpCode {
    ADD,
    AND,
    BR,
    JMP, // RET is also a JMP
    JSR,
    LD,
    LDI,
    LDR,
    LEA,
    NOT,
    RTI,
    ST,
    STI,
    STR,
    TRAP,
    UNKNOWN,
}

impl From<u16> for OpCode {
    fn from(value: u16) -> Self {
        match value >> 12 {
            0b0001 => OpCode::ADD,
            0b0101 => OpCode::AND,
            0b0000 => OpCode::BR,
            0b1100 => OpCode::JMP,
            0b0100 => OpCode::JSR,
            0b0010 => OpCode::LD,
            0b1010 => OpCode::LDI,
            0b0110 => OpCode::LDR,
            0b1110 => OpCode::LEA,
            0b1001 => OpCode::NOT,
            0b1000 => OpCode::RTI,
            0b0011 => OpCode::ST,
            0b1011 => OpCode::STI,
            0b0111 => OpCode::STR,
            0b1111 => OpCode::TRAP,
            _ => OpCode::UNKNOWN,
        }
    }
}

impl Into<u16> for OpCode {
    fn into(self) -> u16 {
        match self {
            OpCode::ADD => 0b0001_0000_0000_0000,
            OpCode::AND => 0b0101_0000_0000_0000,
            OpCode::BR => 0b0000_0000_0000_0000,
            OpCode::JMP => 0b1100_0000_0000_0000,
            OpCode::JSR => 0b0100_0000_0000_0000,
            OpCode::LD => 0b0010_0000_0000_0000,
            OpCode::LDI => 0b1010_0000_0000_0000,
            OpCode::LDR => 0b0110_0000_0000_0000,
            OpCode::LEA => 0b1110_0000_0000_0000,
            OpCode::NOT => 0b1001_0000_0000_0000,
            OpCode::RTI => 0b1000_0000_0000_0000,
            OpCode::ST => 0b0011_0000_0000_0000,
            OpCode::STI => 0b1011_0000_0000_0000,
            OpCode::STR => 0b0111_0000_0000_0000,
            OpCode::TRAP => 0b1111_0000_0000_0000,
            OpCode::UNKNOWN => 0, // Or some other fallback value
        }
    }
}

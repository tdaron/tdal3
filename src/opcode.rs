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
            0b001 => OpCode::ADD,
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

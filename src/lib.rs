use opcode::OpCode;

mod opcode;
const MEMORY_SIZE: usize = 65536; // 2^16 memory locations
const REGISTERS_COUNT: usize = 8;

macro_rules! get_bits {
    ($value:expr, $start:expr, $length:expr) => {
        ($value >> $start) & ((1 << $length) - 1)
    };
}

#[derive(Debug)]
struct Core {
    memory: [u16; MEMORY_SIZE],
    pc: u16,
    registers: [u16; REGISTERS_COUNT],
    conditions_code: [bool; 3],
    psr: u16,
    result: u16,
}

impl Core {
    pub fn new() -> Core {
        Core {
            result: 0,
            memory: [0; MEMORY_SIZE],
            pc: 0,
            registers: [0; REGISTERS_COUNT],
            conditions_code: [false; 3],
            psr: 0,
        }
    }
    fn setcc(&mut self) {
        if self.result == 0 {
            self.conditions_code[0] = false; // N
            self.conditions_code[1] = true; // Z
            self.conditions_code[2] = false; // P
            return;
        }
        if self.result as i16 > 0 {
            self.conditions_code[0] = false; // N
            self.conditions_code[2] = true; // P
            self.conditions_code[1] = false // Z
        } else {
            self.conditions_code[0] = true; // N
            self.conditions_code[2] = false; // P
            self.conditions_code[1] = false; // Z
        }
    }
    fn N(&self) -> bool {
        //TODO: Those registers should be set after
        // LD, LDI, LDR, LEA
        return self.conditions_code[0];
    }
    fn Z(&self) -> bool {
        return self.conditions_code[1];
    }
    fn P(&self) -> bool {
        return self.conditions_code[2];
    }

    pub fn extend_to_u16(&self, mut imm5: u16) -> u16 {
        if (imm5 & 0b10000) != 0 {
            // negative number
            imm5 = imm5 | 0b1111_1111_1110_0000;
        }
        return imm5;
    }
    pub fn exec_instruction(&mut self, inst: u16) {
        let op: OpCode = (inst >> 12).into();

        // Common operands. Might not be interesting to compute for some instructions.
        // Put here for brievty
        let dr = get_bits!(inst, 9, 3);

        match op {
            // ADD
            OpCode::ADD => {
                let sr1 = get_bits!(inst, 6, 3);
                let im = get_bits!(inst, 5, 1);
                // immediate mode
                if im == 1 {
                    let imm5 = get_bits!(inst, 0, 5);
                    let n = self.extend_to_u16(imm5); // needed to handle negatives properly. imm5 as u16 would not.
                    self.registers[dr as usize] = self.registers[sr1 as usize].wrapping_add(n);
                } else {
                    let sr2 = get_bits!(inst, 0, 3);
                    self.registers[dr as usize] =
                        self.registers[sr1 as usize].wrapping_add(self.registers[sr2 as usize]);
                }

                self.result = self.registers[dr as usize];
                self.setcc();
            }
            OpCode::AND => {
                let sr1 = get_bits!(inst, 6, 3);
                let im = get_bits!(inst, 5, 1);

                if im == 1 {
                    let imm5 = get_bits!(inst, 0, 5);
                    let n = self.extend_to_u16(imm5); // needed to handle negatives properly. imm5 as u16 would not
                    self.registers[dr as usize] = self.registers[sr1 as usize] & n;
                } else {
                    let sr2 = get_bits!(inst, 0, 3);
                    self.registers[dr as usize] =
                        self.registers[sr1 as usize] & self.registers[sr2 as usize];
                }
                self.result = self.registers[dr as usize];
                self.setcc();
            }
            OpCode::NOT => {
                let sr = get_bits!(inst, 6, 3);
                self.registers[dr as usize] = !self.registers[sr as usize];
                self.result = self.registers[dr as usize];
                self.setcc();
            }
            _ => {
                println!("Unimplemented opcode: {:?} ", op);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    pub fn test_init() {
        let c = Core::new();
        assert!(c.memory.len() == MEMORY_SIZE);
        assert!(c.pc == 0);
        assert!(c.registers.len() == REGISTERS_COUNT);
    }

    #[test]
    pub fn test_add() {
        //          ADD  R2  R7  IM 7
        //          R2 = R7 + 7
        let add_imm = 0b0001_010_111_1_00111;
        let mut c = Core::new();
        c.registers[7] = 3;
        c.exec_instruction(add_imm);
        assert_eq!(c.registers[2], 10);
        assert_eq!(c.N(), false);
        assert_eq!(c.Z(), false);
        assert_eq!(c.P(), true);

        //          ADD  R2  R2       R2
        //          R2 = R2 + R2
        let add = 0b0001_010_010_0_00_010;
        c.exec_instruction(add);
        assert_eq!(c.registers[2], 20);
        assert_eq!(c.P(), true);
        assert_eq!(c.N(), false);
        assert_eq!(c.Z(), false);

        // Test negatifs
        //          ADD  R2  R3      -5 = -16+11
        //          R2 = R3 + (-5)
        let add = 0b0001_010_011_1_11011;
        c.registers[3] = 2;
        c.exec_instruction(add);
        assert_eq!(c.registers[2] as i16, -3);
        assert_eq!(c.P(), false);
        assert_eq!(c.N(), true);
        assert_eq!(c.Z(), false);
    }

    #[test]
    pub fn test_and() {
        let mut c = Core::new();
        c.registers[7] = 3;
        c.registers[1] = 2;
        //          AND   R0  R7       R1
        //          R0 = R7 & R1
        let and = 0b0101_000_111_0_00_001;

        c.exec_instruction(and);
        assert_eq!(c.registers[0], 3 & 2);
        assert_eq!(c.N(), false);
        assert_eq!(c.Z(), false);
        assert_eq!(c.P(), true);

        //              AND  R0   R7    5
        //              R0 = R7 & 4
        let and_imm = 0b0101_000_111_1_00100;
        c.exec_instruction(and_imm);
        assert_eq!(c.registers[0], 3 & 4);
        assert_eq!(c.N(), false);
        assert_eq!(c.Z(), true);
        assert_eq!(c.P(), false);

        //              AND  R0   R7    -5
        //              R0 = R7 & -5
        let and_imm = 0b0101_000_111_1_11011;
        c.exec_instruction(and_imm);
        assert_eq!(c.registers[0] as i16, 3 as i16 & (-5));
        assert_eq!(c.N(), false);
        assert_eq!(c.Z(), false);
        assert_eq!(c.P(), true);
    }

    #[test]
    pub fn test_not() {
        let mut c = Core::new();
        c.registers[3] = 67;

        //          NOT   R2  R3
        //          R2 = ! R3
        let not = 0b1001_010_011_1_11111;
        c.exec_instruction(not);
        assert_eq!(c.registers[2], !67);
    }
}

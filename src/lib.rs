const MEMORY_SIZE: usize = 65536; // 2^16 memory locations
const REGISTERS_COUNT: usize = 8;
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
        }
        println!("Result: {}", self.result as i16);
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
        // ADD, AND, NOT
        return self.conditions_code[0];
    }
    fn Z(&self) -> bool {
        return self.conditions_code[1];
    }
    fn P(&self) -> bool {
        return self.conditions_code[2];
    }

    pub fn extend(&self, mut imm5: u16) -> u16 {
        if (imm5 & 0b10000) != 0 {
            // negative number
            imm5 = imm5 | 0b1111_1111_1110_0000;
        }
        return imm5;
    }
    pub fn exec_instruction(&mut self, instruction: u16) {
        let opcode = instruction >> 12;
        match opcode {
            // ADD
            0b001 => {
                let dr = (instruction >> 9) & 0b111;
                let sr1 = (instruction >> 6) & 0b111;
                let im = (instruction >> 5) & 1;
                // immediate mode
                if im == 1 {
                    let imm5 = (instruction) & 0b11111;
                    let n = self.extend(imm5);
                    self.registers[dr as usize] = self.registers[sr1 as usize].wrapping_add(n);
                } else {
                    let sr2 = (instruction) & 0b111;
                    self.registers[dr as usize] =
                        self.registers[sr1 as usize].wrapping_add(self.registers[sr2 as usize]);
                }

                self.result = self.registers[dr as usize];
                self.setcc();
            }
            _ => {
                println!("Unexpected opcode: {} ", opcode);
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
        let add_imm = 0b0001_010_111_1_00111;
        let mut c = Core::new();
        c.registers[7] = 3;
        c.exec_instruction(add_imm);
        assert_eq!(c.registers[2], 10);
        assert_eq!(c.N(), false);
        assert_eq!(c.Z(), false);
        assert_eq!(c.P(), true);

        //          ADD  R2  R2       R2
        let add = 0b0001_010_010_0_00_010;
        c.exec_instruction(add);
        assert_eq!(c.registers[2], 20);
        assert_eq!(c.P(), true);
        assert_eq!(c.N(), false);
        assert_eq!(c.Z(), false);

        // Test negatifs
        //          ADD  R2  R3      -5 = -16+11
        let add = 0b0001_010_011_1_11011;
        c.registers[3] = 2;
        c.exec_instruction(add);
        assert_eq!(c.registers[2] as i16, -3);
        assert_eq!(c.P(), false);
        assert_eq!(c.N(), true);
        assert_eq!(c.Z(), false);
    }
}

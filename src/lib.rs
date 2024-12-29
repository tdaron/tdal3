use opcode::OpCode;

mod opcode;
const MEMORY_SIZE: usize = 65536; // 2^16 memory locations
const REGISTERS_COUNT: usize = 8;

macro_rules! get_bits {
    ($value:expr, $start:expr, $length:expr) => {
        ($value >> $start) & ((1 << $length) - 1)
    };
}

macro_rules! extend_to_u16 {
    ($imm:expr, $size:expr) => {{
        let mask = 1 << ($size - 1);
        if ($imm & mask) != 0 {
            $imm | ((!0u16) << $size)
        } else {
            $imm
        }
    }};
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
            pc: 0x3000,
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

    fn exec_instruction(&mut self, inst: u16) -> Result<(), ()> {
        let op: OpCode = (inst).into();

        // Common operands. Might not be interesting to compute for some instructions.
        // Put here for brievty
        let dr = get_bits!(inst, 9, 3);

        let mut next_pc = self.pc + 1;
        let res = match op {
            // ADD
            OpCode::ADD => {
                let sr1 = get_bits!(inst, 6, 3);
                let im = get_bits!(inst, 5, 1);
                // immediate mode
                if im == 1 {
                    let imm5 = get_bits!(inst, 0, 5);
                    println!("Value as bits: {:016b}, {}", imm5, imm5 as i16);
                    let n = extend_to_u16!(imm5, 5); // needed to handle negatives properly. imm5 as u16 would not.
                    println!("Value as bits: {:016b}, {}", n, n as i16);
                    self.registers[dr as usize] = self.registers[sr1 as usize].wrapping_add(n);
                } else {
                    let sr2 = get_bits!(inst, 0, 3);
                    self.registers[dr as usize] =
                        self.registers[sr1 as usize].wrapping_add(self.registers[sr2 as usize]);
                }

                self.result = self.registers[dr as usize];
                self.setcc();
                Ok(())
            }
            OpCode::AND => {
                let sr1 = get_bits!(inst, 6, 3);
                let im = get_bits!(inst, 5, 1);

                if im == 1 {
                    let imm5 = get_bits!(inst, 0, 5);
                    let n = extend_to_u16!(imm5, 5); // needed to handle negatives properly. imm5 as u16 would not
                    self.registers[dr as usize] = self.registers[sr1 as usize] & n;
                } else {
                    let sr2 = get_bits!(inst, 0, 3);
                    self.registers[dr as usize] =
                        self.registers[sr1 as usize] & self.registers[sr2 as usize];
                }
                self.result = self.registers[dr as usize];
                self.setcc();
                Ok(())
            }
            OpCode::NOT => {
                let sr = get_bits!(inst, 6, 3);
                self.registers[dr as usize] = !self.registers[sr as usize];
                self.result = self.registers[dr as usize];
                self.setcc();
                Ok(())
            }
            OpCode::BR => {
                let n = get_bits!(inst, 11, 1) == 1;
                let z = get_bits!(inst, 10, 1) == 1;
                let p = get_bits!(inst, 9, 1) == 1;
                let pc_offset = extend_to_u16!(get_bits!(inst, 0, 9), 9);
                if (n & self.N()) | (z & self.Z()) | (p & self.P()) {
                    if (pc_offset as i16) < 0 {
                        next_pc -= pc_offset;
                    } else {
                        next_pc += pc_offset;
                    }
                }
                Ok(())
            }
            OpCode::JMP => {
                // RET is a special case of JMP with R7 as base_r
                let base_r = get_bits!(inst, 6, 3);
                next_pc = self.registers[base_r as usize];
                Ok(())
            }
            OpCode::JSR => {
                self.registers[7] = self.pc;
                let is_offset = get_bits!(inst, 11, 1) == 1;
                if is_offset {
                    let pc_offset = extend_to_u16!(get_bits!(inst, 0, 11), 11);
                    if (pc_offset as i16) < 0 {
                        next_pc -= pc_offset;
                    } else {
                        next_pc += pc_offset;
                    }
                } else {
                    self.pc = self.registers[get_bits!(inst, 6, 3) as usize]
                }
                Ok(())
            }
            _ => Err(()),
        };
        // if we reach the max value there must have been an error somewhere.
        // probably the user not ending with HALT.
        if self.pc == u16::MAX - 1 {
            return Err(());
        }
        self.pc = next_pc;
        res
    }

    pub fn load_obj(&mut self, obj: &[u16]) {
        let location = obj[0] as usize;

        let obj_data = &obj[1..];
        let end = location + obj_data.len();
        self.memory[location..end].copy_from_slice(obj_data);
    }
    pub fn run(&mut self) {
        loop {
            let instruction = self.memory[self.pc as usize];
            match self.exec_instruction(instruction) {
                Ok(()) => {}
                Err(()) => break,
            }
        }
        println!("Reached the end of the program.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    pub fn test_init() {
        let c = Core::new();
        assert!(c.memory.len() == MEMORY_SIZE);
        assert!(c.pc == 0x3000);
        assert!(c.registers.len() == REGISTERS_COUNT);
    }

    #[test]
    pub fn test_add() {
        //          ADD  R2  R7  IM 7
        //          R2 = R7 + 7
        let add_imm = 0b0001_010_111_1_00111;
        let mut c = Core::new();
        c.registers[7] = 3;
        let _ = c.exec_instruction(add_imm);
        assert_eq!(c.registers[2], 10);
        assert_eq!(c.N(), false);
        assert_eq!(c.Z(), false);
        assert_eq!(c.P(), true);

        //          ADD  R2  R2       R2
        //          R2 = R2 + R2
        let add = 0b0001_010_010_0_00_010;
        let _ = c.exec_instruction(add);
        assert_eq!(c.registers[2], 20);
        assert_eq!(c.P(), true);
        assert_eq!(c.N(), false);
        assert_eq!(c.Z(), false);

        // Test negatifs
        //          ADD  R2  R3      -5 = -16+11
        //          R2 = R3 + (-5)
        let add = 0b0001_010_011_1_11011;
        c.registers[3] = 2;
        let _ = c.exec_instruction(add);
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

        let _ = c.exec_instruction(and);
        assert_eq!(c.registers[0], 3 & 2);
        assert_eq!(c.N(), false);
        assert_eq!(c.Z(), false);
        assert_eq!(c.P(), true);

        //              AND  R0   R7    5
        //              R0 = R7 & 4
        let and_imm = 0b0101_000_111_1_00100;
        let _ = c.exec_instruction(and_imm);
        assert_eq!(c.registers[0], 3 & 4);
        assert_eq!(c.N(), false);
        assert_eq!(c.Z(), true);
        assert_eq!(c.P(), false);

        //              AND  R0   R7    -5
        //              R0 = R7 & -5
        let and_imm = 0b0101_000_111_1_11011;
        let _ = c.exec_instruction(and_imm);
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
        let _ = c.exec_instruction(not);
        assert_eq!(c.registers[2], !67);
    }

    #[test]
    pub fn test_load() {
        //                             ORIG      ADD   R2  R7    7      ADD    R2  R2       R2
        let basic_program: [u16; 3] = [0x3000, 0b0001_010_111_1_00111, 0b0001_010_010_0_00_010];
        let mut c = Core::new();
        c.load_obj(&basic_program);
        c.run();
        assert_eq!(c.registers[2], 14);
    }
    #[test]
    pub fn test_br() {
        let basic_program: [u16; 4] = [
            0x3000,
            // ADD  R2  R7     7
            0b0001_010_111_1_00111,
            // BR      p         1  (if result is positive, skip next instruction) (will happen)
            0b0000_0_0_1_000000001,
            // ADD  R2  R2       R2
            0b0001_010_010_0_00_010,
        ];
        let mut c = Core::new();
        c.load_obj(&basic_program);
        c.run();
        assert_eq!(c.registers[2], 7);

        let basic_program: [u16; 4] = [
            0x3000,
            // ADD  R2  R7     -2
            0b0001_010_111_1_11110,
            // BR      p         1  (if result is positive, skip next instruction) (will not happen)
            0b0000_0_0_1_000000001,
            // ADD  R2  R2       R2
            0b0001_010_010_0_00_010,
        ];
        let mut c = Core::new();
        c.load_obj(&basic_program);
        c.run();

        assert_eq!(c.registers[2] as i16, -4);
        let basic_program: [u16; 4] = [
            0x3000,
            // ADD  R2  R7     -2
            0b0001_010_111_1_11110,
            // BR  n             1  (if result is negative, skip next instruction) (will happen)
            0b0000_1_0_0_000000001,
            // ADD  R2  R2       R2
            0b0001_010_010_0_00_010,
        ];
        let mut c = Core::new();
        c.load_obj(&basic_program);
        c.run();
        assert_eq!(c.registers[2] as i16, -2);

        let basic_program: [u16; 4] = [
            0x3000,
            // ADD  R2  R7     2
            0b0001_010_111_1_00010,
            // BR  n             1  (if result is negative, skip next instruction) (will not happen)
            0b0000_1_0_0_000000001,
            // ADD  R2  R2       R2
            0b0001_010_010_0_00_010,
        ];
        let mut c = Core::new();
        c.load_obj(&basic_program);
        c.run();
        assert_eq!(c.registers[2] as i16, 4);

        let basic_program: [u16; 5] = [
            0x3000,
            // ADD  R2  R7     0
            0b0001_010_111_1_00000,
            // BR    z          1  (if result is zero, skip next instruction) (will happen)
            0b0000_0_1_0_000000001,
            // ADD  R2  R2       2
            0b0001_010_010_1_00010,
            // ADD  R2  R2     10
            0b0001_010_010_1_01010,
        ];
        let mut c = Core::new();
        c.load_obj(&basic_program);
        c.run();
        assert_eq!(c.registers[2] as i16, 10);

        let basic_program: [u16; 5] = [
            0x3000,
            // ADD  R2  R7     1
            0b0001_010_111_1_00001,
            // BR    z          1  (if result is zero, skip next instruction) (will not happen)
            0b0000_0_1_0_000000001,
            // ADD  R2  R2       2
            0b0001_010_010_1_00010,
            // ADD  R2  R2     10
            0b0001_010_010_1_01010,
        ];
        let mut c = Core::new();
        c.load_obj(&basic_program);
        c.run();
        assert_eq!(c.registers[2] as i16, 13);
    }
}

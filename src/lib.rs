use opcode::OpCode;
use wasm_bindgen::prelude::*;
pub mod assemble;
mod opcode;
mod parser;
#[cfg(target_arch = "wasm32")]
use js_sys;

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
#[wasm_bindgen]
pub struct Core {
    N: bool,
    P: bool,
    Z: bool,
    memory: [u16; MEMORY_SIZE],
    pub pc: u16,
    psr: u16,
    registers: [u16; REGISTERS_COUNT],
    result: u16,
    swap_sp: u16,
}

#[wasm_bindgen]
impl Core {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Core {
        let mut c = Core {
            result: 0,
            memory: [0; MEMORY_SIZE],
            pc: 0x0200,
            registers: [0; REGISTERS_COUNT],
            N: false,
            Z: false,
            P: false,
            psr: 0,
            swap_sp: 0xFE00, //Initial value of User Stack Pointer
        };
        c.registers[6] = 0x3000; // Supervisor Stack Pointer
        c
    }
    fn swap_stacks(&mut self) {
        let b = self.registers[6];
        self.registers[6] = self.swap_sp;
        self.swap_sp = b;
    }
    fn setcc(&mut self) {
        if self.result == 0 {
            self.N = false; // N
            self.Z = true; // Z
            self.P = false; // P
            return;
        }
        if self.result as i16 > 0 {
            self.N = false; // N
            self.P = true; // P
            self.Z = false // Z
        } else {
            self.N = true; // N
            self.P = false; // P
            self.Z = false; // Z
        }
    }
    // Returns the address we have been reading data from. OR zero
    fn exec_instruction(&mut self, inst: u16) -> u16 {
        let op: OpCode = (inst).into();
        // Common operands. Might not be interesting to compute for some instructions.
        // Put here for brievty
        let dr = get_bits!(inst, 9, 3);

        let mut next_pc = self.pc + 1;
        let mut address_read = 0;
        match op {
            // ADD
            OpCode::ADD => {
                let sr1 = get_bits!(inst, 6, 3);
                let im = get_bits!(inst, 5, 1);
                // immediate mode
                if im == 1 {
                    let imm5 = get_bits!(inst, 0, 5);
                    let n = extend_to_u16!(imm5, 5); // needed to handle negatives properly. imm5 as u16 would not.
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
                    let n = extend_to_u16!(imm5, 5); // needed to handle negatives properly. imm5 as u16 would not
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
            OpCode::BR => {
                let n = get_bits!(inst, 11, 1) == 1;
                let z = get_bits!(inst, 10, 1) == 1;
                let p = get_bits!(inst, 9, 1) == 1;
                let pc_offset = extend_to_u16!(get_bits!(inst, 0, 9), 9);
                if (n & self.N) | (z & self.Z) | (p & self.P) {
                    next_pc = self.pc.wrapping_add(pc_offset + 1);
                }
            }
            OpCode::JMP => {
                // RET is a special case of JMP with R7 as base_r
                let base_r = get_bits!(inst, 6, 3);
                next_pc = self.registers[base_r as usize];
            }
            OpCode::JSR => {
                self.registers[7] = self.pc + 1;
                let is_offset = get_bits!(inst, 11, 1) == 1;
                if is_offset {
                    let pc_offset = extend_to_u16!(get_bits!(inst, 0, 11), 11);
                    next_pc = self.pc.wrapping_add(pc_offset + 1);
                } else {
                    self.pc = self.registers[get_bits!(inst, 6, 3) as usize]
                }
            }
            OpCode::LD => {
                let offset = extend_to_u16!(get_bits!(inst, 0, 9), 9);
                address_read = self.pc.wrapping_add(offset + 1);
                self.registers[dr as usize] = self.memory[address_read as usize];
                self.result = self.registers[dr as usize];
                self.setcc();
            }
            OpCode::LDI => {
                let offset = extend_to_u16!(get_bits!(inst, 0, 9), 9);
                address_read = self.memory[self.pc.wrapping_add(offset + 1) as usize];
                self.registers[dr as usize] = self.memory[address_read as usize];
                self.result = self.registers[dr as usize];
                self.setcc();
            }
            OpCode::LDR => {
                let offset = extend_to_u16!(get_bits!(inst, 0, 6), 6);
                let base_r = get_bits!(inst, 6, 3);
                address_read = self.registers[base_r as usize].wrapping_add(offset);
                self.registers[dr as usize] = self.memory[address_read as usize];
                self.result = self.registers[dr as usize];
                self.setcc();
            }
            OpCode::LEA => {
                let offset = extend_to_u16!(get_bits!(inst, 0, 9), 9);
                self.registers[dr as usize] = self.pc.wrapping_add(offset + 1);
                self.result = self.registers[dr as usize];
                self.setcc();
            }
            OpCode::ST => {
                let sr = get_bits!(inst, 9, 3);
                let offset = extend_to_u16!(get_bits!(inst, 0, 9), 9);
                self.memory[self.pc.wrapping_add(offset + 1) as usize] =
                    self.registers[sr as usize];
            }
            OpCode::STI => {
                let sr = get_bits!(inst, 9, 3);
                let offset = extend_to_u16!(get_bits!(inst, 0, 9), 9);
                self.memory[self.memory[self.pc.wrapping_add(offset + 1) as usize] as usize] =
                    self.registers[sr as usize];
            }
            OpCode::STR => {
                let sr = get_bits!(inst, 9, 3);
                let base_r = get_bits!(inst, 6, 3);
                let offset = extend_to_u16!(get_bits!(inst, 0, 6), 6);
                self.memory[self.registers[base_r as usize].wrapping_add(offset) as usize] =
                    self.registers[sr as usize];
            }
            OpCode::TRAP => {
                let trapvect = get_bits!(inst, 0, 8);
                self.registers[7] = self.pc + 1;
                self.pc = self.memory[trapvect as usize];
            }
            OpCode::RTI => {
                // should check if in priviledge mode. Don't care for now.
                // R6 is the Stack Pointer
                // Those two operations are like a "pop"
                self.pc = self.memory[self.registers[6] as usize];
                self.registers[6] = self.registers[6] + 1;
                let tmp = self.registers[6];
                self.registers[6] = self.registers[6] + 1;
                self.psr = tmp;
            }
            _ => (),
        };
        self.pc = next_pc;
        address_read
    }

    pub fn load_obj(&mut self, obj: &[u16]) {
        let location = obj[0] as usize;

        let obj_data = &obj[1..];
        let end = location + obj_data.len();
        self.memory[location..end].copy_from_slice(obj_data);
        println!(
            "Loaded {} bytes at address {:#x}",
            obj_data.len() * 2,
            location
        );
    }
    // Returns the address that has been read from.
    pub fn step(&mut self) -> u16 {
        let instruction = self.memory[self.pc as usize];
        let read_address = self.exec_instruction(instruction);
        read_address
    }
    pub fn interrupt(&mut self, interrupt: u16) {
        // Load R6 with Supervisor Stack Pointer
        self.swap_stacks();

        self.registers[6] -= 2; // Reserving two spaces on the stack to store the PC AND PSR
        self.memory[self.registers[6] as usize] = self.pc;
        self.memory[self.registers[6] as usize + 1] = self.psr;
        // PSR and PC pushed onto SSP

        self.pc = self.memory[interrupt as usize]
        //TODO: Set privilege to Supervisor Mode
        //TODO: Set priority to level 4
    }
    pub fn registers_clone(&self) -> Vec<u16> {
        self.registers.into()
    }
    pub fn memory_clone(&self) -> Vec<u16> {
        self.memory.into()
    }
    #[cfg(target_arch = "wasm32")]
    pub unsafe fn memory_view(&self) -> js_sys::Uint16Array {
        js_sys::Uint16Array::view(&self.memory)
    }
    #[cfg(target_arch = "wasm32")]
    pub unsafe fn registers_view(&self) -> js_sys::Uint16Array {
        js_sys::Uint16Array::view(&self.registers)
    }

    // Getters from usage within WASM
    pub fn N(&self) -> bool {
        self.N
    }
    pub fn Z(&self) -> bool {
        self.Z
    }
    pub fn P(&self) -> bool {
        self.P
    }
    pub fn pc(&self) -> u16 {
        self.pc
    }

    pub fn dump_registers(&mut self) {
        for (i, r) in self.registers.iter().enumerate() {
            println!("R{}: {}\r", i, *r as i16);
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
        assert!(c.pc == 0x0200);
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
        assert_eq!(c.N, false);
        assert_eq!(c.Z, false);
        assert_eq!(c.P, true);

        //          ADD  R2  R2       R2
        //          R2 = R2 + R2
        let add = 0b0001_010_010_0_00_010;
        let _ = c.exec_instruction(add);
        assert_eq!(c.registers[2], 20);
        assert_eq!(c.P, true);
        assert_eq!(c.N, false);
        assert_eq!(c.Z, false);

        // Test negatifs
        //          ADD  R2  R3      -5 = -16+11
        //          R2 = R3 + (-5)
        let add = 0b0001_010_011_1_11011;
        c.registers[3] = 2;
        let _ = c.exec_instruction(add);
        assert_eq!(c.registers[2] as i16, -3);
        assert_eq!(c.P, false);
        assert_eq!(c.N, true);
        assert_eq!(c.Z, false);
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
        assert_eq!(c.N, false);
        assert_eq!(c.Z, false);
        assert_eq!(c.P, true);

        //              AND  R0   R7    5
        //              R0 = R7 & 4
        let and_imm = 0b0101_000_111_1_00100;
        let _ = c.exec_instruction(and_imm);
        assert_eq!(c.registers[0], 3 & 4);
        assert_eq!(c.N, false);
        assert_eq!(c.Z, true);
        assert_eq!(c.P, false);

        //              AND  R0   R7    -5
        //              R0 = R7 & -5
        let and_imm = 0b0101_000_111_1_11011;
        let _ = c.exec_instruction(and_imm);
        assert_eq!(c.registers[0] as i16, 3 as i16 & (-5));
        assert_eq!(c.N, false);
        assert_eq!(c.Z, false);
        assert_eq!(c.P, true);
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
        let basic_program: [u16; 3] = [0x0200, 0b0001_010_111_1_00111, 0b0001_010_010_0_00_010];
        let mut c = Core::new();
        c.load_obj(&basic_program);
        c.step();
        c.step();
        c.step();
        c.step();
        c.step();
        assert_eq!(c.registers[2], 14);
    }
    #[test]
    pub fn test_br() {
        let basic_program: [u16; 4] = [
            0x0200,
            // ADD  R2  R7     7
            0b0001_010_111_1_00111,
            // BR      p         1  (if result is positive, skip next instruction) (will happen)
            0b0000_0_0_1_000000001,
            // ADD  R2  R2       R2
            0b0001_010_010_0_00_010,
        ];
        let mut c = Core::new();
        c.load_obj(&basic_program);
        c.step();
        c.step();
        c.step();
        c.step();
        c.step();
        c.step();
        assert_eq!(c.registers[2], 7);

        let basic_program: [u16; 4] = [
            0x0200,
            // ADD  R2  R7     -2
            0b0001_010_111_1_11110,
            // BR      p         1  (if result is positive, skip next instruction) (will not happen)
            0b0000_0_0_1_000000001,
            // ADD  R2  R2       R2
            0b0001_010_010_0_00_010,
        ];
        let mut c = Core::new();
        c.load_obj(&basic_program);
        c.step();
        c.step();
        c.step();
        c.step();
        c.step();
        c.step();

        assert_eq!(c.registers[2] as i16, -4);
        let basic_program: [u16; 4] = [
            0x0200,
            // ADD  R2  R7     -2
            0b0001_010_111_1_11110,
            // BR  n             1  (if result is negative, skip next instruction) (will happen)
            0b0000_1_0_0_000000001,
            // ADD  R2  R2       R2
            0b0001_010_010_0_00_010,
        ];
        let mut c = Core::new();
        c.load_obj(&basic_program);
        c.step();
        c.step();
        c.step();
        c.step();
        c.step();
        c.step();

        assert_eq!(c.registers[2] as i16, -2);

        let basic_program: [u16; 4] = [
            0x0200,
            // ADD  R2  R7     2
            0b0001_010_111_1_00010,
            // BR  n             1  (if result is negative, skip next instruction) (will not happen)
            0b0000_1_0_0_000000001,
            // ADD  R2  R2       R2
            0b0001_010_010_0_00_010,
        ];
        let mut c = Core::new();
        c.load_obj(&basic_program);
        c.step();
        c.step();
        c.step();
        c.step();
        c.step();
        c.step();
        c.step();
        assert_eq!(c.registers[2] as i16, 4);

        let basic_program: [u16; 5] = [
            0x0200,
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
        c.step();
        c.step();
        c.step();
        c.step();
        c.step();
        c.step();
        c.step();
        c.step();
        c.step();
        c.step();
        assert_eq!(c.registers[2] as i16, 10);

        let basic_program: [u16; 5] = [
            0x0200,
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
        c.step();
        c.step();
        c.step();
        c.step();
        c.step();
        c.step();
        c.step();
        c.step();
        c.step();
        assert_eq!(c.registers[2] as i16, 13);
    }
}

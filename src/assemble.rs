use std::collections::HashMap;

use wasm_bindgen::prelude::*;

use crate::parser::{parse_lc3_file, Operand, OperandTypes, ParsedOpCode};

macro_rules! shrink_imm {
    ($value:expr, $size:expr) => {{
        // Shift the value to the left, discarding the higher bits, then shift back
        // to preserve the sign extension for two's complement
        ($value << (16 - $size)) >> (16 - $size)
    }};
}

pub fn emit_error(
    line_number: usize,
    file_content: &Vec<String>,
    error: &str,
) -> Result<(), String> {
    return Err(format!(
        "Error at line {}: {}\n --> {}",
        line_number,
        file_content.get(line_number).unwrap(),
        error
    ));
}

pub fn check_imm_bounds(
    line_number: usize,
    file_content: &Vec<String>,
    value: u16,
    sign: bool,
    size: u16,
) -> Result<(), String> {
    if ((value >> (size - 1)) & 1 == 1) == !sign {
        // If value is negative BUT it's sign was positive it means the user overflowed.

        let error = format!(
            "Value overflow. Max values are in the range of -{} and {}. (Limited to {} bytes)",
            (1 << (size - 1)) as i16,
            (u16::MAX >> (16 - size + 1)),
            size
        );

        emit_error(line_number, file_content, &error)?;
    }
    dbg!(value, value >> size);
    if (value >> size) != 0 && (value >> size) != (u16::MAX >> size) {
        // If raw value is in bounds
        let error = format!(
            "Value overflow. Max values are in the range of -{} and {}. (Limited to {} bytes)",
            (1 << (size - 1)) as i16,
            (u16::MAX >> (16 - size + 1)),
            size
        );

        emit_error(line_number, file_content, &error)?;
    }
    Ok(())
}

#[wasm_bindgen]
pub fn assemble_file(file_content: Vec<String>) -> Result<Vec<u16>, String> {
    let mut output: Vec<u16> = Vec::new();
    let parsed_file = parse_lc3_file(file_content.clone())?;
    output.push(parsed_file.orig);
    let mut symbol_table = HashMap::new();
    // First pass: Labels
    println!("First pass");
    parsed_file
        .instructions
        .iter()
        .enumerate()
        .filter(|(_, p)| p.is_some())
        .map(|(i, p)| (i, p.as_ref().unwrap()))
        .for_each(|(index, instruction)| {
            if let Some(label) = instruction.label.as_ref() {
                symbol_table.insert(label, index - 1); // ORIG counts as an index
                println!("Label {} at position {}", label, index);
            }
        });
    println!("Second pass");
    let fc = &file_content;
    for (ln, instruction) in parsed_file
        .instructions
        .iter()
        .enumerate()
        .filter(|(_, p)| p.is_some())
        .map(|(i, p)| (i + 1, p.as_ref().unwrap()))
    {
        match instruction.opcode {
            ParsedOpCode::ADD => {
                let mut assembled: u16 = crate::OpCode::ADD.into();
                println!("OP: {:#b}", assembled);
                dbg!(&instruction.operands);
                if instruction.operands.len() != 3 {
                    emit_error(ln, fc, "Add should have 3 operands.")?;
                }

                match instruction.operands.get(0).unwrap().operand_type {
                    OperandTypes::Register(reg) => {
                        assembled = assembled | ((reg as u16) << 9);
                    }
                    _ => emit_error(ln, fc, "First argument of ADD must be a register.")?, // TODO: INVALID DR
                }
                match instruction.operands.get(1).unwrap().operand_type {
                    OperandTypes::Register(reg) => {
                        assembled = assembled | ((reg as u16) << 6);
                    }
                    _ => emit_error(ln, fc, "Second argument of ADD must be a register.")?,
                }
                match instruction.operands.get(2).unwrap().operand_type {
                    OperandTypes::Register(reg) => {
                        assembled = assembled | reg as u16;
                    }
                    OperandTypes::Immediate { value, sign } => {
                        check_imm_bounds(ln, fc, value, sign, 5)?;
                        let value = shrink_imm!(value, 5);
                        assembled = assembled | (value);
                        assembled = assembled | (1 << 5);
                    }
                    _ => emit_error(
                        ln,
                        fc,
                        "Second argument of ADD must be a register or an immediate.",
                    )?,
                }
                output.push(assembled);
            }
            _ => {
                let e = format!(
                    "The instruction {:?} is still not implemented.",
                    instruction.opcode
                );
                emit_error(ln, fc, &e)?;
            }
        }
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_orig() -> Result<(), String> {
        let content = vec![".ORIG x3000"];
        let result = assemble_file(content.iter().map(|s| s.to_string()).collect())?;
        assert_eq!(result, [0x3000]);
        Ok(())
    }
    #[test]
    fn test_add() -> Result<(), String> {
        let content = vec![".ORIG x3000", "ADD R2, R2, #-5; testst", "ADD R2, R2, R2"];
        let result = assemble_file(content.iter().map(|s| s.to_string()).collect())?;
        for i in result.iter() {
            println!("{:#b}", i);
        }
        assert_eq!(
            result,
            [0x3000, 0b0001_010_010_1_11011, 0b0001_010_010_0_00_010]
        );
        Ok(())
    }
}

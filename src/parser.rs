use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::{char, space0, space1},
    combinator::{map, opt, value},
    multi::separated_list1,
    sequence::{delimited, preceded, terminated, tuple},
    IResult,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedOpCode {
    ADD,
    AND,
    BR,
    BRz,
    BRp,
    BRn,
    BRzp,
    BRzn,
    BRpn,
    JMP, // RET is also a JMP
    RET, // This is only here for parsing purpose. It will never be parsed from assembled code as it's a JMP
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

pub fn parse_opcode(input: &str) -> IResult<&str, ParsedOpCode> {
    alt((
        value(ParsedOpCode::ADD, tag("ADD")),
        value(ParsedOpCode::AND, tag("AND")),
        alt((
            value(ParsedOpCode::BR, tag("BR")),
            value(ParsedOpCode::BRz, tag("BRz")),
            value(ParsedOpCode::BRn, tag("BRn")),
            value(ParsedOpCode::BRp, tag("BRp")),
            value(ParsedOpCode::BRzn, tag("BRzn")),
            value(ParsedOpCode::BRzn, tag("BRnz")),
            value(ParsedOpCode::BRzp, tag("BRzp")),
            value(ParsedOpCode::BRzp, tag("BRpz")),
            value(ParsedOpCode::BRpn, tag("BRpn")),
            value(ParsedOpCode::BRpn, tag("BRnp")),
            value(ParsedOpCode::BR, tag("BRznp")),
            value(ParsedOpCode::BR, tag("BRzpn")),
            value(ParsedOpCode::BR, tag("BRnzp")),
            value(ParsedOpCode::BR, tag("BRnpz")),
            value(ParsedOpCode::BR, tag("BRpnz")),
            value(ParsedOpCode::BR, tag("BRpzn")),
        )),
        value(ParsedOpCode::JMP, tag("JMP")),
        value(ParsedOpCode::RET, tag("RET")),
        value(ParsedOpCode::JSR, tag("JSR")),
        value(ParsedOpCode::LD, tag("LD")),
        value(ParsedOpCode::LDI, tag("LDI")),
        value(ParsedOpCode::LDR, tag("LDR")),
        value(ParsedOpCode::LEA, tag("LEA")),
        value(ParsedOpCode::NOT, tag("NOT")),
        value(ParsedOpCode::RTI, tag("RTI")),
        value(ParsedOpCode::ST, tag("ST")),
        value(ParsedOpCode::STI, tag("STI")),
        value(ParsedOpCode::STR, tag("STR")),
        value(ParsedOpCode::TRAP, tag("TRAP")),
    ))(input)
}

// Label parser (e.g., "LABEL:")
fn label(input: &str) -> IResult<&str, &str> {
    terminated(
        take_while1(|c: char| c.is_alphanumeric() || c == '_'),
        char(':'),
    )(input)
}

// Register parser (e.g., "R1", "R2")
fn register(input: &str) -> IResult<&str, &str> {
    map(
        preceded(tag("R"), take_while1(|c: char| c.is_digit(10))),
        |digits: &str| {
            // Return the full register as a reference to the original string
            &input[0..(digits.len() + 1)] // This slices out the "R" + digits portion
        },
    )(input)
}

// Immediate value parser (e.g., "#5", "xF")
fn immediate(input: &str) -> IResult<&str, &str> {
    alt((
        preceded(tag("#"), take_while1(|c: char| c.is_digit(10))), // Decimal
        preceded(tag("x"), take_while1(|c: char| c.is_digit(16))), // Hexadecimal
    ))(input)
}

// Operand parser (register, immediate value, or label)
fn operand(input: &str) -> IResult<&str, &str> {
    alt((
        register,                                               // Match registers (e.g., "R1")
        immediate, // Match immediate values (e.g., "#5", "xF")
        take_while1(|c: char| c.is_alphanumeric() || c == '_'), // Match labels (e.g., "DONE")
    ))(input)
}

// Instruction parser using the opcode parser
fn instruction(input: &str) -> IResult<&str, (ParsedOpCode, Vec<&str>)> {
    let parse_operands = separated_list1(
        delimited(space0, char(','), space0), // Allow spaces around the comma
        operand,
    );
    tuple((parse_opcode, space1, parse_operands))(input)
        .map(|(remaining, (opcode, _, operands))| (remaining, (opcode, operands)))
}

fn comment(input: &str) -> IResult<&str, &str> {
    preceded(space0, preceded(char(';'), take_while1(|c| c != '\n')))(input) // Optionnal Comment
}

// Blank line parser: It matches a line with only whitespace or is empty
fn blank_line(input: &str) -> IResult<&str, &str> {
    // Match lines that are empty or contain only spaces
    alt((
        space0,  // Matches lines with only spaces
        tag(""), // Matches empty lines
    ))(input)
}

// Full-line parser: Optional label, instruction, and optional comment
fn lc3_line(
    input: &str,
) -> IResult<&str, Option<(Option<&str>, ParsedOpCode, Vec<&str>, Option<&str>)>> {
    // Ignore empty lines :)
    if let Ok((remaining, _)) = alt((comment, blank_line))(input) {
        if remaining.len() == 0 {
            return Ok((input, None));
        }
    }
    let mut label_and_instruction = tuple((
        opt(terminated(label, space1)), // Optional label
        instruction,                    // Instruction
        opt(comment),
    ));
    label_and_instruction(input).map(|(remaining, (label, (opcode, operands), comment))| {
        (remaining, Some((label, opcode, operands, comment)))
    })
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use super::*;
    #[test]
    pub fn test_parse_opcode() -> Result<(), Box<dyn Error>> {
        let (remainder, parsed) = parse_opcode("ADD")?;
        assert_eq!(parsed, ParsedOpCode::ADD);
        assert_eq!(remainder, "");

        let (remainder, parsed) = parse_opcode("AND R2, R3")?;
        assert_eq!(parsed, ParsedOpCode::AND);
        assert_eq!(remainder, " R2, R3");

        Ok(())
    }
    #[test]
    pub fn test_parse_full() -> Result<(), Box<dyn Error>> {
        let inputs = [
            "test: AND R2 , R3  , R4    ;Hello",
            "ADD R2, R2, xF",
            "; Cool comment Line :)",
            "     ",
            "ADD R2, R3, R4",
        ];
        for input in inputs {
            dbg!(lc3_line(input)?);
        }
        assert!(false);
        Ok(())
    }
}

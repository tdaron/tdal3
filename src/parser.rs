use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::{char, space0, space1},
    combinator::{map, opt, value},
    multi::separated_list1,
    sequence::{delimited, pair, preceded, terminated, tuple},
    AsChar, IResult,
};

// ParsedOpCode Enum (no change)
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
    JMP,
    RET,
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

// Operand Types
#[derive(Clone, Debug, PartialEq)]
pub enum OperandTypes {
    Register(u8),                         // Register type (R followed by u8)
    Immediate { value: u16, sign: bool }, // Immediate value (hexadecimal)
    Label(String),
}

// Operand struct
#[derive(Clone, Debug)]
pub struct Operand {
    pub operand_type: OperandTypes,
}

impl Operand {
    // From function to convert operand type and value into Operand struct
    pub fn from(op_type: &str, value: &str) -> Result<Self, String> {
        match op_type {
            "R" => value
                .parse::<u8>()
                .map(|reg| Self {
                    operand_type: OperandTypes::Register(reg),
                })
                .map_err(|e| format!("Invalid register value: {} - {}", value, e)),
            "#" => value
                .parse::<i16>()
                .map(|imm| Self {
                    operand_type: OperandTypes::Immediate {
                        value: imm as u16,
                        sign: value.contains("-"),
                    },
                })
                .map_err(|e| format!("Invalid immediate decimal value: {} - {}", value, e)),
            "x" => i16::from_str_radix(value.trim(), 16)
                .map(|imm| Self {
                    operand_type: OperandTypes::Immediate {
                        value: imm as u16,
                        sign: value.contains("-"),
                    },
                })
                .map_err(|e| format!("Invalid immediate hexadecimal value: {} - {}", value, e)),
            "LABEL" => Ok(Self {
                operand_type: OperandTypes::Label(value.to_string()),
            }),
            _ => Err(format!("Unknown operand type: {}", op_type)),
        }
    }
}

// Parsed Line struct (no change)
#[derive(Clone, Debug)]
pub struct ParsedLine {
    pub label: Option<String>,
    pub opcode: ParsedOpCode,
    pub operands: Vec<Operand>,
}

impl ParsedLine {
    pub fn from(label: Option<String>, opcode: ParsedOpCode, operands: Vec<Operand>) -> Self {
        Self {
            label,
            opcode,
            operands,
        }
    }
}

// Parsing OpCode (no change)
pub fn parse_opcode(input: &str) -> IResult<&str, ParsedOpCode> {
    alt((
        value(ParsedOpCode::ADD, tag("ADD")),
        value(ParsedOpCode::AND, tag("AND")),
        alt((
            value(ParsedOpCode::BR, tag("BRznp")),
            value(ParsedOpCode::BR, tag("BRzpn")),
            value(ParsedOpCode::BR, tag("BRnzp")),
            value(ParsedOpCode::BR, tag("BRnpz")),
            value(ParsedOpCode::BR, tag("BRpnz")),
            value(ParsedOpCode::BR, tag("BRpzn")),
            value(ParsedOpCode::BRzn, tag("BRzn")),
            value(ParsedOpCode::BRzn, tag("BRnz")),
            value(ParsedOpCode::BRzp, tag("BRzp")),
            value(ParsedOpCode::BRzp, tag("BRpz")),
            value(ParsedOpCode::BRpn, tag("BRpn")),
            value(ParsedOpCode::BRpn, tag("BRnp")),
            value(ParsedOpCode::BRz, tag("BRz")),
            value(ParsedOpCode::BRn, tag("BRn")),
            value(ParsedOpCode::BRp, tag("BRp")),
            value(ParsedOpCode::BR, tag("BR")),
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

// Label parser (no change)
fn label(input: &str) -> IResult<&str, &str> {
    terminated(
        take_while1(|c: char| c.is_alphanumeric() || c == '_'),
        opt(char(':')),
    )(input)
}

// Operand parser (updated to handle 3 operand types)
fn operand(input: &str) -> IResult<&str, (&str, &str)> {
    alt((
        pair(tag("R"), take_while1(|c: char| c.is_digit(10))), // Register
        pair(tag("#"), take_while1(|c: char| c.is_digit(10) || c == '-')), // Immediate decimal value
        pair(tag("x"), take_while1(|c: char| c.is_digit(16) || c == '-')), // Immediate hexadecimal value
        map(take_while1(|c: char| c.is_alphanum()), |label| {
            ("LABEL", label)
        }),
    ))(input)
}

// Instruction parser (no change)
fn instruction(input: &str) -> IResult<&str, (Option<&str>, ParsedOpCode, Vec<(&str, &str)>)> {
    let parse_operands = separated_list1(delimited(space0, char(','), space0), operand);
    tuple((
        alt((
            map(parse_opcode, |o| (None, o)),
            map(pair(label, preceded(space0, parse_opcode)), |(l, o)| {
                (Some(l), o)
            }),
        )),
        space1,
        parse_operands,
    ))(input)
    .map(|(remaining, ((label, opcode), _, operands))| (remaining, (label, opcode, operands)))
}

// Comment parser (no change)
fn comment(input: &str) -> IResult<&str, &str> {
    preceded(space0, preceded(char(';'), take_while1(|c| c != '\n')))(input)
}

// Blank line parser (no change)
fn blank_line(input: &str) -> IResult<&str, &str> {
    alt((space0, tag("")))(input)
}

fn orig(input: &str) -> IResult<&str, u16> {
    alt((
        // Handle .ORIG followed by a hexadecimal value (e.g., x65)
        map(
            preceded(
                tag(".ORIG "),
                preceded(tag("x"), take_while1(|c: char| c.is_digit(16))),
            ),
            |hex_str: &str| u16::from_str_radix(hex_str, 16).unwrap(),
        ),
        // // Handle .ORIG followed by a decimal value (e.g., #10)
        map(
            preceded(
                tag(".ORIG "),
                preceded(tag("#"), take_while1(|c: char| c.is_digit(10))),
            ),
            |dec_str: &str| dec_str.parse::<u16>().unwrap(),
        ),
    ))(input)
}

// Full line parser
fn lc3_line(input: &str) -> IResult<&str, Option<ParsedLine>> {
    if let Ok((remaining, _)) = alt((comment, blank_line))(input) {
        if remaining.is_empty() {
            return Ok((input, None));
        }
    }

    instruction(input).map(|(remaining, (label, opcode, operands))| {
        (
            remaining,
            Some(ParsedLine::from(
                label.map(|s| s.to_string()),
                opcode,
                operands
                    .iter()
                    .map(|(a, b)| Operand::from(a, b).unwrap()) // Only valid operands
                    .collect(),
            )),
        )
    })
}

// ParsedFile struct
#[derive(Debug)]
pub struct ParsedFile {
    pub instructions: Vec<Option<ParsedLine>>,
    pub orig: u16,
}

// Parsing LC-3 file (no change)
pub fn parse_lc3_file(file_content: Vec<String>) -> Result<ParsedFile, String> {
    if file_content.is_empty() {
        return Err("File is empty.".into());
    }

    let orig_value = &file_content[0];
    let orig = match orig(&orig_value) {
        Ok((_, data)) => data,
        Err(_) => return Err("The file should start with a .ORIG directive".into()),
    };

    let instructions: Vec<Option<ParsedLine>> = file_content
        .iter()
        .skip(1)
        .map(|line| match lc3_line(&line) {
            Ok((d, Some(instruction))) => {
                dbg!(d);
                Some(instruction)
            }
            Ok((_, None)) => None,
            Err(e) => {
                eprintln!("Error parsing line '{}': {}", line, e);
                None
            }
        })
        .collect();

    Ok(ParsedFile { instructions, orig })
}

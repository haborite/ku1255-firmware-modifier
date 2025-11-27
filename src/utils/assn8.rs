use std::collections::HashMap;
use std::fmt::Write as _;

/// Addressing space for operand.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AddrSpace {
    Null,
    Zero,
    Ram,
    Rom,
    Imm,
}

/// Operand spec for opcode table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OperandSpec {
    None,
    Fixed(&'static str), // "A", "R", "Z", "Y", "PFLAG", "RBANK"
    Addr,
    Imm,
    BitAddr,
}

/// One opcode entry (assembler view).
#[derive(Clone, Copy, Debug)]
struct OpcodeEntry {
    opcode: u8,          // high byte template
    mask: u16,           // mask for operand bits
    space: AddrSpace,    // operand addressing space
    mnemonic: &'static str,
    left: OperandSpec,
    right: OperandSpec,
}

// Direct port of libsn8.opcode_dict for encoding. (No flags/branch fns)
const OPCODES: &[OpcodeEntry] = &[
    OpcodeEntry { opcode: 0x00, mask: 0x0000, space: AddrSpace::Null, mnemonic: "NOP",
                  left: OperandSpec::None, right: OperandSpec::None },

    OpcodeEntry { opcode: 0x02, mask: 0x00ff, space: AddrSpace::Zero, mnemonic: "B0XCH",
                  left: OperandSpec::Fixed("A"), right: OperandSpec::Addr },
    OpcodeEntry { opcode: 0x03, mask: 0x00ff, space: AddrSpace::Zero, mnemonic: "B0ADD",
                  left: OperandSpec::Addr, right: OperandSpec::Fixed("A") },
    OpcodeEntry { opcode: 0x04, mask: 0x0000, space: AddrSpace::Null, mnemonic: "PUSH",
                  left: OperandSpec::None, right: OperandSpec::None },
    OpcodeEntry { opcode: 0x05, mask: 0x0000, space: AddrSpace::Null, mnemonic: "POP",
                  left: OperandSpec::None, right: OperandSpec::None },

    OpcodeEntry { opcode: 0x06, mask: 0x00ff, space: AddrSpace::Imm, mnemonic: "CMPRS",
                  left: OperandSpec::Fixed("A"), right: OperandSpec::Imm },
    OpcodeEntry { opcode: 0x07, mask: 0x00ff, space: AddrSpace::Ram, mnemonic: "CMPRS",
                  left: OperandSpec::Fixed("A"), right: OperandSpec::Addr },

    OpcodeEntry { opcode: 0x08, mask: 0x00ff, space: AddrSpace::Ram, mnemonic: "RRC",
                  left: OperandSpec::Addr, right: OperandSpec::None },
    OpcodeEntry { opcode: 0x09, mask: 0x00ff, space: AddrSpace::Ram, mnemonic: "RRCM",
                  left: OperandSpec::Addr, right: OperandSpec::None },
    OpcodeEntry { opcode: 0x0a, mask: 0x00ff, space: AddrSpace::Ram, mnemonic: "RLC",
                  left: OperandSpec::Addr, right: OperandSpec::None },
    OpcodeEntry { opcode: 0x0b, mask: 0x00ff, space: AddrSpace::Ram, mnemonic: "RLCM",
                  left: OperandSpec::Addr, right: OperandSpec::None },

    OpcodeEntry { opcode: 0x0d, mask: 0x0000, space: AddrSpace::Null, mnemonic: "MOVC",
                  left: OperandSpec::None, right: OperandSpec::None },
    OpcodeEntry { opcode: 0x0e, mask: 0x0000, space: AddrSpace::Null, mnemonic: "RET",
                  left: OperandSpec::None, right: OperandSpec::None },
    OpcodeEntry { opcode: 0x0f, mask: 0x0000, space: AddrSpace::Null, mnemonic: "RETI",
                  left: OperandSpec::None, right: OperandSpec::None },

    OpcodeEntry { opcode: 0x10, mask: 0x00ff, space: AddrSpace::Ram, mnemonic: "ADC",
                  left: OperandSpec::Fixed("A"), right: OperandSpec::Addr },
    OpcodeEntry { opcode: 0x11, mask: 0x00ff, space: AddrSpace::Ram, mnemonic: "ADC",
                  left: OperandSpec::Addr, right: OperandSpec::Fixed("A") },

    OpcodeEntry { opcode: 0x12, mask: 0x00ff, space: AddrSpace::Ram, mnemonic: "ADD",
                  left: OperandSpec::Fixed("A"), right: OperandSpec::Addr },
    OpcodeEntry { opcode: 0x13, mask: 0x00ff, space: AddrSpace::Ram, mnemonic: "ADD",
                  left: OperandSpec::Addr, right: OperandSpec::Fixed("A") },
    OpcodeEntry { opcode: 0x14, mask: 0x00ff, space: AddrSpace::Imm, mnemonic: "ADD",
                  left: OperandSpec::Fixed("A"), right: OperandSpec::Imm },

    OpcodeEntry { opcode: 0x15, mask: 0x00ff, space: AddrSpace::Ram, mnemonic: "INCS",
                  left: OperandSpec::Addr, right: OperandSpec::None },
    OpcodeEntry { opcode: 0x16, mask: 0x00ff, space: AddrSpace::Ram, mnemonic: "INCMS",
                  left: OperandSpec::Addr, right: OperandSpec::None },
    OpcodeEntry { opcode: 0x17, mask: 0x00ff, space: AddrSpace::Ram, mnemonic: "SWAP",
                  left: OperandSpec::Addr, right: OperandSpec::None },

    OpcodeEntry { opcode: 0x18, mask: 0x00ff, space: AddrSpace::Ram, mnemonic: "OR",
                  left: OperandSpec::Fixed("A"), right: OperandSpec::Addr },
    OpcodeEntry { opcode: 0x19, mask: 0x00ff, space: AddrSpace::Ram, mnemonic: "OR",
                  left: OperandSpec::Addr, right: OperandSpec::Fixed("A") },
    OpcodeEntry { opcode: 0x1a, mask: 0x00ff, space: AddrSpace::Imm, mnemonic: "OR",
                  left: OperandSpec::Fixed("A"), right: OperandSpec::Imm },

    OpcodeEntry { opcode: 0x1b, mask: 0x00ff, space: AddrSpace::Ram, mnemonic: "XOR",
                  left: OperandSpec::Fixed("A"), right: OperandSpec::Addr },
    OpcodeEntry { opcode: 0x1c, mask: 0x00ff, space: AddrSpace::Ram, mnemonic: "XOR",
                  left: OperandSpec::Addr, right: OperandSpec::Fixed("A") },
    OpcodeEntry { opcode: 0x1d, mask: 0x00ff, space: AddrSpace::Imm, mnemonic: "XOR",
                  left: OperandSpec::Fixed("A"), right: OperandSpec::Imm },

    OpcodeEntry { opcode: 0x1e, mask: 0x00ff, space: AddrSpace::Ram, mnemonic: "MOV",
                  left: OperandSpec::Fixed("A"), right: OperandSpec::Addr },
    OpcodeEntry { opcode: 0x1f, mask: 0x00ff, space: AddrSpace::Ram, mnemonic: "MOV",
                  left: OperandSpec::Addr, right: OperandSpec::Fixed("A") },

    OpcodeEntry { opcode: 0x20, mask: 0x00ff, space: AddrSpace::Ram, mnemonic: "SBC",
                  left: OperandSpec::Fixed("A"), right: OperandSpec::Addr },
    OpcodeEntry { opcode: 0x21, mask: 0x00ff, space: AddrSpace::Ram, mnemonic: "SBC",
                  left: OperandSpec::Addr, right: OperandSpec::Fixed("A") },

    OpcodeEntry { opcode: 0x22, mask: 0x00ff, space: AddrSpace::Ram, mnemonic: "SUB",
                  left: OperandSpec::Fixed("A"), right: OperandSpec::Addr },
    OpcodeEntry { opcode: 0x23, mask: 0x00ff, space: AddrSpace::Ram, mnemonic: "SUB",
                  left: OperandSpec::Addr, right: OperandSpec::Fixed("A") },
    OpcodeEntry { opcode: 0x24, mask: 0x00ff, space: AddrSpace::Imm, mnemonic: "SUB",
                  left: OperandSpec::Fixed("A"), right: OperandSpec::Imm },

    OpcodeEntry { opcode: 0x25, mask: 0x00ff, space: AddrSpace::Ram, mnemonic: "DECS",
                  left: OperandSpec::Addr, right: OperandSpec::None },
    OpcodeEntry { opcode: 0x26, mask: 0x00ff, space: AddrSpace::Ram, mnemonic: "DECMS",
                  left: OperandSpec::Addr, right: OperandSpec::None },
    OpcodeEntry { opcode: 0x27, mask: 0x00ff, space: AddrSpace::Ram, mnemonic: "SWAPM",
                  left: OperandSpec::Addr, right: OperandSpec::None },

    OpcodeEntry { opcode: 0x28, mask: 0x00ff, space: AddrSpace::Ram, mnemonic: "AND",
                  left: OperandSpec::Fixed("A"), right: OperandSpec::Addr },
    OpcodeEntry { opcode: 0x29, mask: 0x00ff, space: AddrSpace::Ram, mnemonic: "AND",
                  left: OperandSpec::Addr, right: OperandSpec::Fixed("A") },
    OpcodeEntry { opcode: 0x2a, mask: 0x00ff, space: AddrSpace::Imm, mnemonic: "AND",
                  left: OperandSpec::Fixed("A"), right: OperandSpec::Imm },

    OpcodeEntry { opcode: 0x2b, mask: 0x00ff, space: AddrSpace::Ram, mnemonic: "CLR",
                  left: OperandSpec::Addr, right: OperandSpec::None },

    OpcodeEntry { opcode: 0x2c, mask: 0x00ff, space: AddrSpace::Ram, mnemonic: "XCH",
                  left: OperandSpec::Fixed("A"), right: OperandSpec::Addr },

    OpcodeEntry { opcode: 0x2d, mask: 0x00ff, space: AddrSpace::Imm, mnemonic: "MOV",
                  left: OperandSpec::Fixed("A"), right: OperandSpec::Imm },

    OpcodeEntry { opcode: 0x2e, mask: 0x00ff, space: AddrSpace::Zero, mnemonic: "B0MOV",
                  left: OperandSpec::Fixed("A"), right: OperandSpec::Addr },
    OpcodeEntry { opcode: 0x2f, mask: 0x00ff, space: AddrSpace::Ram, mnemonic: "B0MOV",
                  left: OperandSpec::Addr, right: OperandSpec::Fixed("A") },

    OpcodeEntry { opcode: 0x32, mask: 0x00ff, space: AddrSpace::Imm, mnemonic: "B0MOV",
                  left: OperandSpec::Fixed("R"), right: OperandSpec::Imm },
    OpcodeEntry { opcode: 0x33, mask: 0x00ff, space: AddrSpace::Imm, mnemonic: "B0MOV",
                  left: OperandSpec::Fixed("Z"), right: OperandSpec::Imm },
    OpcodeEntry { opcode: 0x34, mask: 0x00ff, space: AddrSpace::Imm, mnemonic: "B0MOV",
                  left: OperandSpec::Fixed("Y"), right: OperandSpec::Imm },
    OpcodeEntry { opcode: 0x36, mask: 0x00ff, space: AddrSpace::Imm, mnemonic: "B0MOV",
                  left: OperandSpec::Fixed("PFLAG"), right: OperandSpec::Imm },
    OpcodeEntry { opcode: 0x37, mask: 0x00ff, space: AddrSpace::Imm, mnemonic: "B0MOV",
                  left: OperandSpec::Fixed("RBANK"), right: OperandSpec::Imm },

    // Bit operations: bit index in high byte, address in low byte
    OpcodeEntry { opcode: 0x40, mask: 0x00ff, space: AddrSpace::Ram, mnemonic: "BCLR",
                  left: OperandSpec::BitAddr, right: OperandSpec::None },
    OpcodeEntry { opcode: 0x48, mask: 0x00ff, space: AddrSpace::Ram, mnemonic: "BSET",
                  left: OperandSpec::BitAddr, right: OperandSpec::None },
    OpcodeEntry { opcode: 0x50, mask: 0x00ff, space: AddrSpace::Ram, mnemonic: "BTS0",
                  left: OperandSpec::BitAddr, right: OperandSpec::None },
    OpcodeEntry { opcode: 0x58, mask: 0x00ff, space: AddrSpace::Ram, mnemonic: "BTS1",
                  left: OperandSpec::BitAddr, right: OperandSpec::None },
    OpcodeEntry { opcode: 0x60, mask: 0x00ff, space: AddrSpace::Zero, mnemonic: "B0BCLR",
                  left: OperandSpec::BitAddr, right: OperandSpec::None },
    OpcodeEntry { opcode: 0x68, mask: 0x00ff, space: AddrSpace::Zero, mnemonic: "B0BSET",
                  left: OperandSpec::BitAddr, right: OperandSpec::None },
    OpcodeEntry { opcode: 0x70, mask: 0x00ff, space: AddrSpace::Zero, mnemonic: "B0BTS0",
                  left: OperandSpec::BitAddr, right: OperandSpec::None },
    OpcodeEntry { opcode: 0x78, mask: 0x00ff, space: AddrSpace::Zero, mnemonic: "B0BTS1",
                  left: OperandSpec::BitAddr, right: OperandSpec::None },

    // Jumps / Calls: 14-bit ROM address
    OpcodeEntry { opcode: 0x80, mask: 0x3fff, space: AddrSpace::Rom, mnemonic: "JMP",
                  left: OperandSpec::Addr, right: OperandSpec::None },
    OpcodeEntry { opcode: 0xc0, mask: 0x3fff, space: AddrSpace::Rom, mnemonic: "CALL",
                  left: OperandSpec::Addr, right: OperandSpec::None },
];

/// Parsed operand expression (before label resolution).
#[derive(Clone, Debug)]
enum OperandExpr {
    None,
    Number(u32),
    Symbol(String),
    Immediate(Box<OperandExpr>),         // #expr
    BitAddr(Box<OperandExpr>, u8),       // expr.bit
}

/// Data expression for DW.
#[derive(Clone, Debug)]
enum DataExpr {
    Number(u32),
    Symbol(String),
}

/// One parsed line.
#[derive(Clone, Debug)]
enum LineKind {
    Empty,
    Org(u32),
    Dw(Vec<DataExpr>),
    Instr {
        mnemonic: String,
        left: OperandExpr,
        right: OperandExpr,
    },
}

#[derive(Clone, Debug)]
struct ParsedLine {
    lineno: usize,
    kind: LineKind,
}

/// Evaluated operand (after symbols resolved).
#[derive(Clone, Debug)]
enum EvalOperand {
    None,
    Reg(&'static str),
    Address(u16),
    Imm(u16),
    BitAddr { addr: u16, bit: u8 },
}

const REG_A: &'static str = "A";
const REG_R: &'static str = "R";
const REG_Z: &'static str = "Z";
const REG_Y: &'static str = "Y";
const REG_PFLAG: &'static str = "PFLAG";
const REG_RBANK: &'static str = "RBANK";

fn is_reg_name(s: &str) -> Option<&'static str> {
    match s {
        "A" => Some(REG_A),
        "R" => Some(REG_R),
        "Z" => Some(REG_Z),
        "Y" => Some(REG_Y),
        "PFLAG" => Some(REG_PFLAG),
        "RBANK" => Some(REG_RBANK),
        _ => None,
    }
}

fn parse_number(tok: &str) -> Result<u32, String> {
    if let Some(rest) = tok.strip_prefix("0x").or_else(|| tok.strip_prefix("0X")) {
        u32::from_str_radix(rest, 16)
            .map_err(|e| format!("invalid hex literal {tok:?}: {e}"))
    } else if let Some(rest) = tok.strip_prefix("0b").or_else(|| tok.strip_prefix("0B")) {
        u32::from_str_radix(rest, 2)
            .map_err(|e| format!("invalid bin literal {tok:?}: {e}"))
    } else {
        tok.parse::<u32>()
            .map_err(|e| format!("invalid decimal literal {tok:?}: {e}"))
    }
}

fn parse_operand_expr(raw: &str) -> Result<OperandExpr, String> {
    let s = raw.trim();
    if s.is_empty() {
        return Ok(OperandExpr::None);
    }
    if let Some(rest) = s.strip_prefix('#') {
        return Ok(OperandExpr::Immediate(Box::new(parse_operand_expr(rest)?)));
    }
    if let Some(idx) = s.rfind('.') {
        let (base, bit_str) = s.split_at(idx);
        let bit_str = &bit_str[1..]; // skip '.'
        let bit: u8 = parse_number(bit_str)
            .map_err(|e| format!("bad bit index {bit_str:?}: {e}"))?
            as u8;
        if bit > 7 {
            return Err(format!("bit index out of range (0-7): {}", bit));
        }
        let base_expr = parse_operand_expr(base)?;
        return Ok(OperandExpr::BitAddr(Box::new(base_expr), bit));
    }
    // number or symbol
    if s.chars().all(|c| c.is_ascii_hexdigit() || c == 'x' || c == 'X' || c == 'b' || c == 'B')
        || s.chars().all(|c| c.is_ascii_digit())
    {
        if let Ok(v) = parse_number(s) {
            return Ok(OperandExpr::Number(v));
        }
    }
    Ok(OperandExpr::Symbol(s.to_string()))
}

fn parse_data_expr(raw: &str) -> Result<DataExpr, String> {
    let s = raw.trim();
    if s.is_empty() {
        return Err("empty data expression".into());
    }
    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        // Strings not implemented here; could be expanded to bytes for DB later.
        return Err("string literals in DW not supported in this minimal port".into());
    }
    if s.chars().all(|c| c.is_ascii_digit() || c == 'x' || c == 'X' || c == 'b' || c == 'B')
        || s.starts_with("0x")
        || s.starts_with("0X")
        || s.starts_with("0b")
        || s.starts_with("0B")
    {
        if let Ok(v) = parse_number(s) {
            return Ok(DataExpr::Number(v));
        }
    }
    Ok(DataExpr::Symbol(s.to_string()))
}

fn parse_line(lineno: usize, line: &str) -> Result<(Option<String>, ParsedLine), String> {
    // Strip comment
    let mut parts = line.splitn(2, ';');
    let code = parts.next().unwrap_or("").trim();
    if code.is_empty() {
        return Ok((None, ParsedLine { lineno, kind: LineKind::Empty }));
    }

    // Optional label: "label: rest..."
    let mut label: Option<String> = None;
    let mut rest = code;
    if let Some(colon_idx) = code.find(':') {
        let (left, right) = code.split_at(colon_idx);
        let name = left.trim();
        if !name.is_empty() {
            label = Some(name.to_string());
            rest = right[1..].trim(); // skip ':'
        }
    }

    if rest.is_empty() {
        return Ok((label, ParsedLine { lineno, kind: LineKind::Empty }));
    }

    // First token: mnemonic or directive
    let mut iter = rest.split_whitespace();
    let op_raw = iter.next().unwrap();
    let op_upper = op_raw.to_ascii_uppercase();
    let rest_after_op = iter.collect::<Vec<_>>().join(" ");

    match op_upper.as_str() {
        "ORG" => {
            let expr = rest_after_op.trim();
            if expr.is_empty() {
                return Err(format!("line {lineno}: ORG requires an address"));
            }
            let val = parse_number(expr)?;
            Ok((label, ParsedLine { lineno, kind: LineKind::Org(val) }))
        }
        "DW" => {
            let mut data = Vec::new();
            for item in rest_after_op.split(',') {
                if item.trim().is_empty() {
                    continue;
                }
                data.push(parse_data_expr(item)?);
            }
            Ok((label, ParsedLine { lineno, kind: LineKind::Dw(data) }))
        }
        // Other directives not implemented; treat them as errors
        ".CHIP" | "CHIP" |
        ".DATA" | ".ALIGN" |
        "INCLUDE" | "INCLUDEBIN" |
        "DB" | "DS" => {
            Err(format!("line {lineno}: directive {op_raw} not supported in this minimal assembler"))
        }
        _ => {
            // Instruction
            let operands_str = rest_after_op;
            let (left_expr, right_expr) = if operands_str.is_empty() {
                (OperandExpr::None, OperandExpr::None)
            } else {
                let mut parts = operands_str.splitn(2, ',');
                let left_raw = parts.next().unwrap_or("").trim();
                let right_raw = parts.next().unwrap_or("").trim();
                let left = if left_raw.is_empty() {
                    OperandExpr::None
                } else {
                    parse_operand_expr(left_raw)?
                };
                let right = if right_raw.is_empty() {
                    OperandExpr::None
                } else {
                    parse_operand_expr(right_raw)?
                };
                (left, right)
            };
            Ok((
                label,
                ParsedLine {
                    lineno,
                    kind: LineKind::Instr {
                        mnemonic: op_upper,
                        left: left_expr,
                        right: right_expr,
                    },
                },
            ))
        }
    }
}

fn eval_operand(expr: &OperandExpr, labels: &HashMap<String, u16>) -> Result<EvalOperand, String> {
    match expr {
        OperandExpr::None => Ok(EvalOperand::None),
        OperandExpr::Number(v) => {
            if *v > 0xffff {
                Err(format!("numeric operand too large: 0x{v:08x}"))
            } else {
                Ok(EvalOperand::Address(*v as u16))
            }
        }
        OperandExpr::Symbol(name) => {
            if let Some(reg) = is_reg_name(name.as_str()) {
                Ok(EvalOperand::Reg(reg))
            } else if let Some(addr) = labels.get(name) {
                Ok(EvalOperand::Address(*addr))
            } else {
                Err(format!("undefined symbol: {name}"))
            }
        }
        OperandExpr::Immediate(inner) => {
            let inner_eval = eval_operand(inner, labels)?;
            let val = match inner_eval {
                EvalOperand::Address(a) => a,
                EvalOperand::Imm(a) => a,
                _ => return Err(format!("immediate must be numeric or label, got {inner_eval:?}")),
            };
            Ok(EvalOperand::Imm(val))
        }
        OperandExpr::BitAddr(base, bit) => {
            let base_eval = eval_operand(base, labels)?;
            let addr = match base_eval {
                EvalOperand::Address(a) => a,
                _ => {
                    return Err(format!(
                        "bit address base must be numeric or label, got {base_eval:?}"
                    ))
                }
            };
            Ok(EvalOperand::BitAddr { addr, bit: *bit })
        }
    }
}

fn matches_spec(spec: OperandSpec, op: &EvalOperand) -> bool {
    use EvalOperand::*;
    match spec {
        OperandSpec::None => matches!(op, None),
        OperandSpec::Fixed(name) => matches!(op, Reg(r) if r == &name),
        OperandSpec::Addr => matches!(op, Address(_) | BitAddr { .. }),
        OperandSpec::Imm => matches!(op, Imm(_)),
        OperandSpec::BitAddr => matches!(op, BitAddr { .. }),
    }
}

/// Assemble SN8 source code into a 0x3000-word (0x6000-byte) binary image.
pub fn assemble_sn8(source: &str) -> Result<Vec<u8>, String> {
    // Parse lines (once) and collect labels in first pass.
    let mut parsed_lines: Vec<ParsedLine> = Vec::new();
    let mut labels: HashMap<String, u16> = HashMap::new();
    let mut addr: u16 = 0;

    for (idx, raw_line) in source.lines().enumerate() {
        let lineno = idx + 1;
        let (label_opt, pline) = parse_line(lineno, raw_line)?;
        if let Some(label) = label_opt {
            if labels.contains_key(&label) {
                return Err(format!("line {lineno}: duplicate label {label:?}"));
            }
            labels.insert(label, addr);
        }

        match &pline.kind {
            LineKind::Empty => { /* no effect on addr */ }
            LineKind::Org(val) => {
                if *val > 0x3fff {
                    return Err(format!("line {lineno}: ORG address out of range: 0x{val:08x}"));
                }
                addr = *val as u16;
            }
            LineKind::Dw(vs) => {
                let new_addr = addr as usize + vs.len();
                if new_addr > 0x4000 {
                    return Err(format!("line {lineno}: DW causes address overflow"));
                }
                addr = new_addr as u16;
            }
            LineKind::Instr { .. } => {
                if addr >= 0x4000 {
                    return Err(format!("line {lineno}: instruction address overflow"));
                }
                addr = addr.wrapping_add(1);
            }
        }

        parsed_lines.push(pline);
    }

    // Build instruction lookup: mnemonic -> candidates
    let mut instr_map: HashMap<&'static str, Vec<&'static OpcodeEntry>> = HashMap::new();
    for op in OPCODES {
        instr_map.entry(op.mnemonic).or_default().push(op);
    }

    // Second pass: encode into ROM
    let mut rom: Vec<u16> = vec![0; 0x3000];
    let mut addr: u16 = 0;

    for pline in &parsed_lines {
        let lineno = pline.lineno;
        match &pline.kind {
            LineKind::Empty => {}
            LineKind::Org(val) => {
                addr = *val as u16;
            }
            LineKind::Dw(items) => {
                for item in items {
                    let val_u32 = match item {
                        DataExpr::Number(v) => *v,
                        DataExpr::Symbol(name) => {
                            *labels
                                .get(name)
                                .ok_or_else(|| format!("line {lineno}: undefined symbol in DW: {name}"))?
                                as u32
                        }
                    };
                    if val_u32 > 0xffff {
                        return Err(format!("line {lineno}: DW value too large: 0x{val_u32:08x}"));
                    }
                    if (addr as usize) >= rom.len() {
                        // Outside the 0x3000-word output window: match Python's silent truncation
                        // by simply ignoring writes beyond the end.
                        continue;
                    }
                    rom[addr as usize] = val_u32 as u16;
                    addr = addr.wrapping_add(1);
                }
            }
            LineKind::Instr { mnemonic, left, right } => {
                let candidates = instr_map
                    .get(mnemonic.as_str())
                    .ok_or_else(|| format!("line {lineno}: unknown instruction {mnemonic:?}"))?;

                let left_eval = eval_operand(left, &labels)?;
                let right_eval = eval_operand(right, &labels)?;

                let mut encoded: Option<u16> = None;
                let mut last_err = String::new();

                'outer: for entry in candidates {
                    let left_is_fixed = matches!(entry.left, OperandSpec::Fixed(_));
                    let right_is_fixed = matches!(entry.right, OperandSpec::Fixed(_));

                    if !matches_spec(entry.left, &left_eval) || !matches_spec(entry.right, &right_eval) {
                        continue;
                    }

                    // Choose the operand that carries the numeric value (if any)
                    let operand_src = if !left_is_fixed && !matches!(left_eval, EvalOperand::None) {
                        Some(&left_eval)
                    } else if !right_is_fixed && !matches!(right_eval, EvalOperand::None) {
                        Some(&right_eval)
                    } else {
                        None
                    };

                    let mut opcode_word: u16 = (entry.opcode as u16) << 8;

                    if let Some(opv) = operand_src {
                        match opv {
                            EvalOperand::BitAddr { addr, bit } => {
                                opcode_word |= (*bit as u16) << 8;
                                let val = *addr as u16;
                                let masked = val & entry.mask;
                                if masked != val {
                                    last_err = format!(
                                        "line {lineno}: operand too large for {}: 0x{val:04x}",
                                        entry.mnemonic
                                    );
                                    continue 'outer;
                                }
                                opcode_word |= masked;
                            }
                            EvalOperand::Address(val) | EvalOperand::Imm(val) => {
                                let masked = *val & entry.mask;
                                if masked != *val {
                                    last_err = format!(
                                        "line {lineno}: operand too large for {}: 0x{val:04x}",
                                        entry.mnemonic
                                    );
                                    continue 'outer;
                                }
                                opcode_word |= masked;
                            }
                            _ => {
                                last_err = format!(
                                    "line {lineno}: unsupported operand kind for {}",
                                    entry.mnemonic
                                );
                                continue 'outer;
                            }
                        }
                    }

                    encoded = Some(opcode_word);
                    break;
                }

                let word = encoded.ok_or_else(|| {
                    if last_err.is_empty() {
                        format!(
                            "line {lineno}: no opcode suitable for {} {:?}, {:?}",
                            mnemonic, left_eval, right_eval
                        )
                    } else {
                        last_err.clone()
                    }
                })?;

                if (addr as usize) < rom.len() {
                    rom[addr as usize] = word;
                }
                addr = addr.wrapping_add(1);
            }
        }
    }

    // Convert ROM words to little-endian bytes.
    let mut out = Vec::with_capacity(rom.len() * 2);
    for w in rom {
        out.push((w & 0xff) as u8);
        out.push((w >> 8) as u8);
    }
    Ok(out)
}

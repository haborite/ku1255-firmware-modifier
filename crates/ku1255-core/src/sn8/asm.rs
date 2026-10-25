// Port of assn8.py (single source file; INCLUDE / INCLUDEBIN are not supported).

use std::collections::{BTreeMap, HashMap};

use super::config::{Chip, CodeOptionValues, parse_int};
use super::opcode::{Arg, OPCODES, Space};

const ROM_WORDS: u32 = 0x3000;
const AUTO_LABEL: &str = "@@";
const AUTO_LABEL_FORWARD: &str = "@F";
const AUTO_LABEL_BACKWARD: &str = "@B";

#[derive(Clone, Debug, PartialEq)]
enum Tok {
    Code,
    Data,
    Align,
    Identifier(String),
    Str(String),
    Instruction(String),
    BitSelector(u8),
    Number(i64),
    Eol,
    CodeOptionsBegin,
    CodeOptionsEnd,
    CodeOption,
    RelativeAddress(i64),
    ByteSelector(char),
    Word(&'static str), // DB DW DS EQU ORG CHIP ENDP INCLUDE INCLUDEBIN
    Hash,
    Comma,
    Colon,
}

const TOKEN_WORDS: [&str; 9] = [
    "DB",
    "DW",
    "DS",
    "EQU",
    "ORG",
    "CHIP",
    "ENDP",
    "INCLUDE",
    "INCLUDEBIN",
];

struct Token {
    tok: Tok,
    line: usize,
}

fn is_hex(c: u8) -> bool {
    c.is_ascii_hexdigit()
}

/// Length of the longest prefix of `s` made of bytes matching `pred`.
fn span(s: &[u8], pred: impl Fn(u8) -> bool) -> usize {
    s.iter().take_while(|c| pred(**c)).count()
}

/// `0([Xx][0-9a-fA-F]+|[0-9a-fA-F]*[Hh])`
fn match_hex(s: &[u8]) -> Option<(usize, i64)> {
    if s.first() != Some(&b'0') {
        return None;
    }
    let rest = &s[1..];
    if matches!(rest.first(), Some(b'x' | b'X')) {
        let n = span(&rest[1..], is_hex);
        if n > 0 {
            let text = std::str::from_utf8(&rest[1..1 + n]).ok()?;
            return Some((2 + n, i64::from_str_radix(text, 16).ok()?));
        }
    }
    // [0-9a-fA-F]*[Hh]: H is not a hex digit, so backtracking cannot help.
    let n = span(rest, is_hex);
    if !matches!(rest.get(n), Some(b'h' | b'H')) {
        return None;
    }
    let text = std::str::from_utf8(&s[..1 + n]).ok()?;
    Some((2 + n, i64::from_str_radix(text, 16).ok()?))
}

/// `0([Bb][01]+|[01]*[Bb])`
fn match_bin(s: &[u8]) -> Option<(usize, i64)> {
    if s.first() != Some(&b'0') {
        return None;
    }
    let rest = &s[1..];
    if matches!(rest.first(), Some(b'b' | b'B')) {
        let n = span(&rest[1..], |c| c == b'0' || c == b'1');
        if n > 0 {
            let text = std::str::from_utf8(&rest[1..1 + n]).ok()?;
            return Some((2 + n, i64::from_str_radix(text, 2).ok()?));
        }
    }
    let n = span(rest, |c| c == b'0' || c == b'1');
    if matches!(rest.get(n), Some(b'b' | b'B')) {
        let text = std::str::from_utf8(&s[..1 + n]).ok()?;
        return Some((2 + n, i64::from_str_radix(text, 2).ok()?));
    }
    None
}

/// Lexer following ply's master regex order: function rules in definition
/// order, then string rules by decreasing regex length, then literals.
fn tokenize(source: &str) -> Result<Vec<Token>, String> {
    let s = source.as_bytes();
    let mut tokens = Vec::new();
    let mut line = 1;
    let mut i = 0;
    let mut push = |tok, line| tokens.push(Token { tok, line });
    while i < s.len() {
        let rest = &s[i..];
        let c = rest[0];
        // t_ignore (CR is ignored too, so CRLF sources work)
        if c == b' ' || c == b'\t' || c == b'\r' {
            i += 1;
            continue;
        }
        // BYTE_SELECTOR \$[HML]
        if c == b'$' && matches!(rest.get(1), Some(b'H' | b'M' | b'L')) {
            push(Tok::ByteSelector(rest[1] as char), line);
            i += 2;
            continue;
        }
        // RELATIVE_ADDRESS \$([+-][0-9]+)?
        if c == b'$' {
            let mut len = 1;
            let mut value = 0;
            if matches!(rest.get(1), Some(b'+' | b'-')) {
                let n = span(&rest[2..], |c| c.is_ascii_digit());
                if n > 0 {
                    len = 2 + n;
                    let text = std::str::from_utf8(&rest[1..len]).unwrap();
                    // Python int(x, 0): leading zeros are rejected.
                    if text[1..].len() > 1
                        && text[1..].starts_with('0')
                        && text[1..].bytes().any(|b| b != b'0')
                    {
                        return Err(format!("line {line}: invalid relative address {text:?}"));
                    }
                    value = text.parse::<i64>().map_err(|e| e.to_string())?;
                }
            }
            push(Tok::RelativeAddress(value), line);
            i += len;
            continue;
        }
        // QUOTED_STRING "(\\.|[^\\"\n])*"
        if c == b'"' {
            let mut j = 1;
            let mut value = String::new();
            let mut closed = false;
            while j < rest.len() {
                match rest[j] {
                    b'"' => {
                        closed = true;
                        j += 1;
                        break;
                    }
                    b'\n' => break,
                    b'\\' if j + 1 < rest.len() => {
                        value.push(match rest[j + 1] {
                            b'n' => '\n',
                            b't' => '\t',
                            b'r' => '\r',
                            b'0' => '\0',
                            other => other as char,
                        });
                        j += 2;
                    }
                    b'\\' => break,
                    _ => {
                        let ch = source[i + j..].chars().next().unwrap();
                        value.push(ch);
                        j += ch.len_utf8();
                    }
                }
            }
            if closed {
                push(Tok::Str(value), line);
                i += j;
                continue;
            }
        }
        // IDENTIFIER [a-zA-Z_@][a-zA-Z_0-9@]*
        if c.is_ascii_alphabetic() || c == b'_' || c == b'@' {
            let n = span(rest, |c| {
                c.is_ascii_alphanumeric() || c == b'_' || c == b'@'
            });
            let word = std::str::from_utf8(&rest[..n]).unwrap();
            let tok = if OPCODES.iter().any(|op| op.mnemonic == word) {
                Tok::Instruction(word.to_string())
            } else if let Some(w) = TOKEN_WORDS.iter().find(|w| **w == word) {
                Tok::Word(w)
            } else {
                Tok::Identifier(word.to_string())
            };
            push(tok, line);
            i += n;
            continue;
        }
        // BIT_SELECTOR \.[0-7]
        if c == b'.' && matches!(rest.get(1), Some(b'0'..=b'7')) {
            push(Tok::BitSelector(rest[1] - b'0'), line);
            i += 2;
            continue;
        }
        if let Some((n, value)) = match_hex(rest).or_else(|| match_bin(rest)) {
            push(Tok::Number(value), line);
            i += n;
            continue;
        }
        // DEC_NUMBER
        if c.is_ascii_digit() {
            let n = span(rest, |c| c.is_ascii_digit());
            let text = std::str::from_utf8(&rest[..n]).unwrap();
            push(
                Tok::Number(
                    text.parse()
                        .map_err(|_| format!("line {line}: number too large"))?,
                ),
                line,
            );
            i += n;
            continue;
        }
        // CHAR '(.)'
        if c == b'\'' && rest.len() >= 3 && rest[1] != b'\n' && rest[2] == b'\'' {
            if !rest[1].is_ascii() {
                return Err(format!("line {line}: non-ascii character literal"));
            }
            push(Tok::Number(rest[1] as i64), line);
            i += 3;
            continue;
        }
        // EOL \n+
        if c == b'\n' {
            let n = span(rest, |c| c == b'\n');
            push(Tok::Eol, line);
            line += n;
            i += n;
            continue;
        }
        // String rules, longest regex first.
        let string_rules: [(&str, Option<Tok>); 7] = [
            ("//{{SONIX_CODE_OPTION", Some(Tok::CodeOptionsBegin)),
            ("//}}SONIX_CODE_OPTION", Some(Tok::CodeOptionsEnd)),
            (".Code_Option", Some(Tok::CodeOption)),
            (".ALIGN", Some(Tok::Align)),
            (".CODE", Some(Tok::Code)),
            (".DATA", Some(Tok::Data)),
            (";", None), // t_ignore_COMMENT ;.*
        ];
        if let Some((text, tok)) = string_rules
            .iter()
            .find(|(text, _)| rest.starts_with(text.as_bytes()))
        {
            match tok {
                Some(tok) => {
                    push(tok.clone(), line);
                    i += text.len();
                }
                None => i += span(rest, |c| c != b'\n'),
            }
            continue;
        }
        let tok = match c {
            b'#' => Tok::Hash,
            b',' => Tok::Comma,
            b':' => Tok::Colon,
            _ => {
                let ch = source[i..].chars().next().unwrap();
                return Err(format!("line {line}: Illegal character {ch:?}"));
            }
        };
        push(tok, line);
        i += 1;
    }
    Ok(tokens)
}

#[derive(Clone, Debug, PartialEq)]
enum Value {
    /// Magic identifier "A", for instruction matching.
    RegA,
    Address(i64),
    BitAddress(i64, u8),
    Immediate(i64),
    NoOperand,
    /// Undefined identifier, possibly a forward label reference.
    Undefined(String),
}

impl Value {
    fn address_value(&self, what: &str) -> Result<i64, String> {
        match self {
            Value::Address(v) | Value::BitAddress(v, _) | Value::Immediate(v) => Ok(*v),
            other => Err(format!("{what}: {other:?} has no value")),
        }
    }
}

struct Allocator {
    /// Uninterrupted free ranges `[length, start]`, smallest first.
    free: Vec<[i64; 2]>,
}

impl Allocator {
    fn new(start: i64, stop: i64) -> Self {
        Allocator {
            free: vec![[stop - start + 1, start]],
        }
    }

    fn allocate(&mut self, length: i64) -> Result<i64, String> {
        let idx = self
            .free
            .iter()
            .position(|[free_length, _]| length <= *free_length)
            .ok_or("No large-enough free memory chunk")?;
        let [free_length, free_start] = self.free[idx];
        if length < free_length {
            self.free[idx] = [free_length - length, free_start + length];
        } else {
            self.free.remove(idx);
        }
        self.free.sort();
        Ok(free_start)
    }

    fn allocate_at(&mut self, at: i64, length: i64) -> Result<(), String> {
        let stop = at + length;
        let idx = self
            .free
            .iter()
            .position(|[l, s]| *s <= at && stop <= s + l)
            .ok_or("No large-enough free memory chunk")?;
        let [free_length, free_start] = self.free[idx];
        let free_stop = free_start + free_length;
        if free_start < at {
            self.free[idx][0] = at - free_start;
            if stop < free_stop {
                self.free.push([free_stop - stop, stop]);
            }
        } else if stop < free_stop {
            self.free[idx] = [free_stop - stop, stop];
        } else {
            self.free.remove(idx);
        }
        self.free.sort();
        Ok(())
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Section {
    Top,
    Code,
    Data,
    Ended,
}

struct Assembler {
    tokens: Vec<Token>,
    pos: usize,
    chip: Option<Chip>,
    chip_identifiers: HashMap<String, Value>,
    identifiers: HashMap<String, Value>,
    label_referers: BTreeMap<String, Vec<i64>>,
    ram_allocator: Option<Allocator>,
    rom: HashMap<i64, i64>,
    address: i64,
    partial_word: bool,
}

impl Assembler {
    fn peek(&self) -> Option<&Tok> {
        self.tokens.get(self.pos).map(|t| &t.tok)
    }

    fn peek_at(&self, offset: usize) -> Option<&Tok> {
        self.tokens.get(self.pos + offset).map(|t| &t.tok)
    }

    fn line(&self) -> usize {
        self.tokens
            .get(self.pos)
            .or_else(|| self.tokens.last())
            .map(|t| t.line)
            .unwrap_or(0)
    }

    fn next(&mut self) -> Option<Tok> {
        let tok = self.tokens.get(self.pos).map(|t| t.tok.clone());
        self.pos += 1;
        tok
    }

    fn syntax_error(&self) -> String {
        match self.peek() {
            Some(tok) => format!("line {}: Syntax error, unexpected {tok:?}", self.line()),
            None => format!("line {}: Syntax error, unexpected end of file", self.line()),
        }
    }

    fn expect_eol(&mut self) -> Result<(), String> {
        match self.peek() {
            Some(Tok::Eol) => {
                self.pos += 1;
                Ok(())
            }
            _ => Err(self.syntax_error()),
        }
    }

    fn get_identifier(&self, name: &str) -> Option<Value> {
        self.chip_identifiers
            .get(name)
            .or_else(|| self.identifiers.get(name))
            .cloned()
    }

    fn resolve(&self, name: &str) -> Result<Value, String> {
        self.get_identifier(name)
            .ok_or_else(|| format!("line {}: Undefined identifier: {name:?}", self.line()))
    }

    fn write(&mut self, value: i64) -> Result<(), String> {
        if !(0..=0xffff).contains(&value) {
            return Err(format!(
                "line {}: Value out of bounds: {value:x}",
                self.line()
            ));
        }
        if !(0..=0x3fff).contains(&self.address) {
            return Err(format!(
                "line {}: Address out of bounds: {:x}",
                self.line(),
                self.address
            ));
        }
        if self.rom.contains_key(&self.address) {
            return Err(format!(
                "line {}: Redefining program address {:x}",
                self.line(),
                self.address
            ));
        }
        self.rom.insert(self.address, value);
        self.address += 1;
        self.partial_word = false;
        Ok(())
    }

    fn resolve_label(&mut self, name: &str, address: i64) -> Result<(), String> {
        let referrers = self.label_referers.remove(name).unwrap_or_default();
        if !referrers.is_empty() && address & 0x3fff != address {
            return Err(format!("Label {name:?} address out of range: {address:x}"));
        }
        for referrer in referrers {
            *self.rom.get_mut(&referrer).expect("referrer was written") |= address;
        }
        Ok(())
    }

    fn run(&mut self) -> Result<(), String> {
        let mut section = Section::Top;
        while let Some(tok) = self.peek().cloned() {
            if section == Section::Ended && tok != Tok::Eol {
                return Err(self.syntax_error());
            }
            match tok {
                Tok::Eol => self.pos += 1,
                Tok::Word("CHIP") => {
                    self.pos += 1;
                    let Some(Tok::Identifier(name)) = self.next() else {
                        self.pos -= 1;
                        return Err(self.syntax_error());
                    };
                    self.expect_eol()?;
                    self.set_chip(&name)?;
                    section = Section::Top;
                }
                Tok::CodeOptionsBegin => {
                    self.pos += 1;
                    self.expect_eol()?;
                    self.code_options()?;
                    section = Section::Top;
                }
                Tok::Code => {
                    self.pos += 1;
                    self.expect_eol()?;
                    section = Section::Code;
                }
                Tok::Data => {
                    self.pos += 1;
                    self.expect_eol()?;
                    section = Section::Data;
                }
                Tok::Word("ENDP") => {
                    self.pos += 1;
                    if self.peek().is_some() {
                        self.expect_eol()?;
                    }
                    section = Section::Ended;
                }
                Tok::Word("INCLUDE" | "INCLUDEBIN") => {
                    return Err(format!("line {}: Imports are disabled", self.line()));
                }
                _ => match section {
                    Section::Code => self.emitable()?,
                    Section::Data => self.declaration()?,
                    _ => return Err(self.syntax_error()),
                },
            }
        }
        self.label_referers.remove(AUTO_LABEL_BACKWARD);
        if !self.label_referers.is_empty() {
            let names: Vec<_> = self.label_referers.keys().cloned().collect();
            return Err(format!("Undefined labels:\n  {}", names.join("\n  ")));
        }
        Ok(())
    }

    fn set_chip(&mut self, name: &str) -> Result<(), String> {
        if self.chip.is_some() {
            return Err(format!("line {}: Redefining chip type.", self.line()));
        }
        let chip = Chip::builtin(name)?;
        if chip.name.to_ascii_lowercase() != name.to_ascii_lowercase() {
            return Err(format!("Inconsistent chip config file for {name:?}"));
        }
        for (_, option) in &chip.code_options {
            if let CodeOptionValues::Fixed(value) = option.values {
                self.rom.insert(option.address as i64, value as i64);
            }
        }
        for (address, name) in &chip.rom {
            self.chip_identifiers
                .insert(name.clone(), Value::Address(*address as i64));
        }
        for range in &chip.ram {
            for (address, name) in &range.symbols {
                let value = match address.split_once('.') {
                    Some((word, bit)) => {
                        Value::BitAddress(parse_int(word)? as i64, parse_int(bit)? as u8)
                    }
                    None => Value::Address(parse_int(address)? as i64),
                };
                self.chip_identifiers.insert(name.clone(), value);
            }
        }
        let mut allocator = Allocator::new(chip.ram_start as i64, chip.ram_stop as i64);
        for (start, stop) in &chip.ram_reserved.ranges {
            allocator.allocate_at(*start as i64, *stop as i64 - *start as i64)?;
        }
        self.ram_allocator = Some(allocator);
        self.chip = Some(chip);
        Ok(())
    }

    fn code_options(&mut self) -> Result<(), String> {
        loop {
            match self.next() {
                Some(Tok::Eol) => {}
                Some(Tok::CodeOptionsEnd) => return self.expect_eol(),
                Some(Tok::CodeOption) => {
                    let Some(Tok::Identifier(name)) = self.next() else {
                        self.pos -= 1;
                        return Err(self.syntax_error());
                    };
                    let value_name = match self.next() {
                        Some(Tok::Identifier(v) | Tok::Str(v)) => v,
                        _ => {
                            self.pos -= 1;
                            return Err(self.syntax_error());
                        }
                    };
                    self.expect_eol()?;
                    let chip = self.chip.as_ref().ok_or("Code option before CHIP")?;
                    let (_, option) = chip
                        .code_options
                        .iter()
                        .find(|(n, _)| *n == name)
                        .ok_or_else(|| format!("Unknown code option {name:?}"))?;
                    let CodeOptionValues::Named(values) = &option.values else {
                        return Err(format!("Code option {name:?} is not settable"));
                    };
                    // Python builds {name: value}: the last value with a given name wins.
                    let value = values
                        .iter()
                        .rev()
                        .find(|(_, n)| *n == value_name)
                        .map(|(v, _)| *v as i64)
                        .ok_or_else(|| {
                            format!("Unknown value {value_name:?} for code option {name:?}")
                        })?;
                    let address = option.address as i64;
                    let mask = option.mask as i64;
                    let word = self
                        .rom
                        .get_mut(&address)
                        .ok_or_else(|| format!("No code option word at 0x{address:04x}"))?;
                    if *word & mask != 0 {
                        return Err("Duplicate code option declaration".into());
                    }
                    *word |= value << mask.trailing_zeros();
                }
                _ => {
                    self.pos -= 1;
                    return Err(self.syntax_error());
                }
            }
        }
    }

    /// `address : NUMBER | resolved_identifier | resolved_identifier BYTE_SELECTOR`,
    /// optionally followed by BIT_SELECTOR (`bit_address`).
    fn address(&mut self) -> Result<Value, String> {
        let base = match self.next() {
            Some(Tok::Number(n)) => Value::Address(n),
            Some(Tok::Identifier(name)) => {
                let value = self.resolve(&name)?;
                if let Some(Tok::ByteSelector(sel)) = self.peek().cloned() {
                    self.pos += 1;
                    byte_select(&value, sel)?
                } else {
                    value
                }
            }
            _ => {
                self.pos -= 1;
                return Err(self.syntax_error());
            }
        };
        self.bit_suffix(base)
    }

    fn bit_suffix(&mut self, base: Value) -> Result<Value, String> {
        let Some(Tok::BitSelector(bit)) = self.peek().cloned() else {
            return Ok(base);
        };
        self.pos += 1;
        match base {
            Value::BitAddress(..) => Err(format!(
                "line {}: {base:?} is already a bit address.",
                self.line()
            )),
            Value::Address(v) => Ok(Value::BitAddress(v, bit)),
            other => Err(format!(
                "line {}: Cannot make a bit address from {other:?}",
                self.line()
            )),
        }
    }

    fn operand(&mut self) -> Result<Value, String> {
        match self.peek().cloned() {
            Some(Tok::Number(_)) => self.address(),
            Some(Tok::Identifier(name)) => {
                match self.peek_at(1) {
                    Some(Tok::BitSelector(_) | Tok::ByteSelector(_)) => self.address(),
                    _ => {
                        // operand : IDENTIFIER (may be a forward reference)
                        self.pos += 1;
                        match self.get_identifier(&name) {
                            Some(value) => Ok(value),
                            None if name == AUTO_LABEL_BACKWARD => Err(format!(
                                "line {}: Undefined identifier: {name:?}",
                                self.line()
                            )),
                            None => Ok(Value::Undefined(name)),
                        }
                    }
                }
            }
            Some(Tok::Hash) => {
                self.pos += 1;
                match self.address()? {
                    Value::Address(v) => Ok(Value::Immediate(v)),
                    _ => Err(format!(
                        "line {}: Wrong identifier type for immediate",
                        self.line()
                    )),
                }
            }
            Some(Tok::RelativeAddress(offset)) => {
                self.pos += 1;
                Ok(Value::Address(self.address + offset))
            }
            _ => Err(self.syntax_error()),
        }
    }

    fn data_list(&mut self) -> Result<Vec<i64>, String> {
        let mut data = Vec::new();
        loop {
            match self.next() {
                Some(Tok::Identifier(name)) => {
                    data.push(self.resolve(&name)?.address_value(&name)?)
                }
                Some(Tok::Number(n)) => data.push(n),
                Some(Tok::Str(s)) => data.extend(s.chars().map(|c| c as i64)),
                _ => {
                    self.pos -= 1;
                    return Err(self.syntax_error());
                }
            }
            if self.peek() == Some(&Tok::Comma) {
                self.pos += 1;
            } else {
                return Ok(data);
            }
        }
    }

    fn emitable(&mut self) -> Result<(), String> {
        let line = self.line();
        match self.next() {
            Some(Tok::Word("ORG")) => {
                let address = match self.next() {
                    Some(Tok::Number(n)) => n,
                    Some(Tok::Identifier(name)) => match self.resolve(&name)? {
                        Value::Address(v) => v,
                        _ => return Err(format!("line {line}: Wrong identifier type for ORG")),
                    },
                    _ => {
                        self.pos -= 1;
                        return Err(self.syntax_error());
                    }
                };
                self.expect_eol()?;
                self.address = address;
                self.partial_word = false;
            }
            Some(Tok::Align) => {
                let Some(Tok::Number(alignment)) = self.next() else {
                    self.pos -= 1;
                    return Err(self.syntax_error());
                };
                self.expect_eol()?;
                if alignment == 0 {
                    return Err(format!("line {line}: Alignment must not be zero"));
                }
                self.partial_word = false;
                let remainder = self.address.rem_euclid(alignment);
                if remainder != 0 {
                    self.address = (self.address.div_euclid(alignment) + 1) * alignment;
                }
            }
            Some(Tok::Identifier(name)) if self.peek() == Some(&Tok::Colon) => {
                self.pos += 1;
                self.label(name)?;
            }
            Some(Tok::Word("DB")) => {
                let data = self.data_list()?;
                self.expect_eol()?;
                let mut data = data.into_iter().peekable();
                let mut pending = Vec::new();
                if self.partial_word {
                    self.address -= 1;
                    let data_a = self.rom.remove(&self.address).unwrap_or(0);
                    if !(0..=0xff).contains(&data_a) {
                        return Err(format!("line {line}: Value out of bounds: {data_a:x}"));
                    }
                    pending.push(data_a);
                }
                let mut all = pending.into_iter().chain(&mut data);
                while let Some(data_a) = all.next() {
                    match all.next() {
                        Some(data_r) => self.write(data_a | (data_r << 8))?,
                        None => {
                            self.write(data_a)?;
                            self.partial_word = true;
                        }
                    }
                }
            }
            Some(Tok::Word("DW")) => {
                let data = self.data_list()?;
                self.expect_eol()?;
                for value in data {
                    self.write(value)?;
                }
            }
            Some(Tok::Instruction(name)) => {
                let mut left = Value::NoOperand;
                let mut right = Value::NoOperand;
                if self.peek() != Some(&Tok::Eol) && self.peek().is_some() {
                    left = self.operand()?;
                    if self.peek() == Some(&Tok::Comma) {
                        self.pos += 1;
                        right = self.operand()?;
                    }
                }
                if self.peek().is_some() {
                    self.expect_eol()?;
                }
                self.write_instruction(line, &name, left, right)?;
            }
            _ => {
                self.pos -= 1;
                return Err(self.syntax_error());
            }
        }
        Ok(())
    }

    fn label(&mut self, name: String) -> Result<(), String> {
        if name == AUTO_LABEL_FORWARD || name == AUTO_LABEL_BACKWARD {
            return Err(format!("Reserved label name {name:?}"));
        }
        let address = Value::Address(self.address);
        if name != AUTO_LABEL {
            if let Some(existing) = self.get_identifier(&name) {
                if existing != address {
                    return Err(format!("line {}: Redefining label {name:?}", self.line()));
                }
            }
        }
        let is_auto = name == AUTO_LABEL;
        // Resolve any forward reference.
        self.resolve_label(
            if is_auto { AUTO_LABEL_FORWARD } else { &name },
            self.address,
        )?;
        // Prepare for any back reference.
        let stored = if is_auto {
            AUTO_LABEL_BACKWARD.to_string()
        } else {
            name
        };
        self.identifiers.insert(stored, address);
        self.partial_word = false;
        Ok(())
    }

    fn declaration(&mut self) -> Result<(), String> {
        let Some(Tok::Identifier(name)) = self.next() else {
            self.pos -= 1;
            return Err(self.syntax_error());
        };
        let value = match self.next() {
            Some(Tok::Word("EQU")) => self.address()?,
            Some(Tok::Word("DS")) => {
                let Some(Tok::Number(count)) = self.next() else {
                    self.pos -= 1;
                    return Err(self.syntax_error());
                };
                if count < 1 {
                    return Err(format!(
                        "line {}: Trying to allocate less than one byte.",
                        self.line()
                    ));
                }
                let allocator = self.ram_allocator.as_mut().ok_or("DS before CHIP")?;
                Value::Address(allocator.allocate(count)?)
            }
            _ => {
                self.pos -= 1;
                return Err(self.syntax_error());
            }
        };
        self.expect_eol()?;
        // Redefinition only warns in Python; chip identifiers keep precedence anyway.
        self.identifiers.insert(name, value);
        Ok(())
    }

    fn write_instruction(
        &mut self,
        line: usize,
        name: &str,
        mut left: Value,
        right: Value,
    ) -> Result<(), String> {
        for op in OPCODES.iter().filter(|op| op.mnemonic == name) {
            if op.space == Space::Rom {
                if let Value::Undefined(label) = &left {
                    // Remember address for label, and use placeholder.
                    self.label_referers
                        .entry(label.clone())
                        .or_default()
                        .push(self.address);
                    left = Value::Address(0);
                }
            }
            if matches!(op.space, Space::Ram | Space::Zro) {
                for value in [&left, &right] {
                    if let Value::Undefined(id) = value {
                        return Err(format!("line {line}: Undefined identifier: {id:?}"));
                    }
                }
            }
            let fits = |arg: Arg, value: &Value| match arg {
                Arg::Fixed("A") => *value == Value::RegA,
                Arg::Fixed(reg) => self.get_identifier(reg).as_ref() == Some(value),
                Arg::None => *value == Value::NoOperand,
                Arg::Address => matches!(value, Value::Address(_)),
                Arg::BitAddress => matches!(value, Value::BitAddress(..)),
                Arg::Immediate => matches!(value, Value::Immediate(_)),
            };
            if !(fits(op.left, &left) && fits(op.right, &right)) {
                continue;
            }
            let mut word = (op.key as i64) << 8;
            let operand = if !matches!(op.left, Arg::Fixed(_)) && left != Value::NoOperand {
                Some(&left)
            } else if !matches!(op.right, Arg::Fixed(_)) && right != Value::NoOperand {
                Some(&right)
            } else {
                None
            };
            if let Some(operand) = operand {
                let (value, bit) = match operand {
                    Value::BitAddress(v, bit) => (*v, Some(*bit)),
                    other => (other.address_value(name)?, None),
                };
                if let Some(bit) = bit {
                    word |= (bit as i64) << 8;
                }
                let masked = value & op.mask as i64;
                if masked != value {
                    return Err(format!(
                        "line {line}: Operand too large for instruction {name:?}: {value:x}"
                    ));
                }
                word |= masked;
            }
            return self.write(word);
        }
        Err(format!(
            "line {line}: No opcode suitable for {name:?} {left:?}, {right:?}"
        ))
    }
}

/// Assemble SN8 source into a 0x3000-word little-endian ROM image.
pub fn assemble(source: &str) -> Result<Vec<u8>, String> {
    let mut asm = Assembler {
        tokens: tokenize(source)?,
        pos: 0,
        chip: None,
        chip_identifiers: HashMap::from([("A".to_string(), Value::RegA)]),
        identifiers: HashMap::new(),
        label_referers: BTreeMap::new(),
        ram_allocator: None,
        rom: HashMap::new(),
        address: 0,
        partial_word: false,
    };
    asm.run()?;
    let mut image = Vec::with_capacity(ROM_WORDS as usize * 2);
    for address in 0..ROM_WORDS as i64 {
        let word = asm.rom.get(&address).copied().unwrap_or(0) as u16;
        image.extend_from_slice(&word.to_le_bytes());
    }
    Ok(image)
}

fn byte_select(value: &Value, selector: char) -> Result<Value, String> {
    let v = match value {
        Value::BitAddress(..) => return Err("Cannot select a byte from a bit address.".into()),
        other => other.address_value("byte selector")?,
    };
    Ok(Value::Address(match selector {
        'L' => v & 0xff,
        'M' => (v >> 8) & 0xff,
        _ => (v >> 16) & 0xff,
    }))
}

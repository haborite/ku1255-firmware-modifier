// Port of dissn8.py ("walker" method, without --dot output).

use std::collections::{BTreeMap, HashMap};

use super::config::{Chip, CodeOptionValues, parse_int};
use super::opcode::{self, Arg, Flow, Space};

const COMMENT_POSITION: usize = 5; // 5th tab

type Function = Option<String>;

/// Map preserving insertion order of keys (Python dict / defaultdict semantics).
#[derive(Default)]
struct OrderedMultiMap {
    keys: Vec<u32>,
    values: HashMap<u32, Vec<u32>>,
}

impl OrderedMultiMap {
    fn push(&mut self, key: u32, value: u32) {
        self.values
            .entry(key)
            .or_insert_with(|| {
                self.keys.push(key);
                Vec::new()
            })
            .push(value);
    }

    fn pop(&mut self, key: u32) -> Option<Vec<u32>> {
        let values = self.values.remove(&key)?;
        self.keys.retain(|k| *k != key);
        Some(values)
    }

    fn contains(&self, key: u32) -> bool {
        self.values.contains_key(&key)
    }

    fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    fn items(&self) -> impl Iterator<Item = (u32, &Vec<u32>)> {
        self.keys.iter().map(|k| (*k, &self.values[k]))
    }
}

struct RamSymbols {
    name: Option<String>,
    bits: BTreeMap<u8, Option<String>>,
}

struct RamRange {
    start: u32,
    stop: u32,
    symbols: HashMap<u32, RamSymbols>,
}

struct Disassembler {
    ram_ranges: Vec<RamRange>,
    rom_symbols: HashMap<u32, String>,
    entry_stack: Vec<(u32, Function)>,
    callers: OrderedMultiMap,
    jumpers: OrderedMultiMap,
    ram_usage: BTreeMap<String, Vec<(u32, &'static str)>>,
    functions: HashMap<String, u32>,
    line_owner: HashMap<u32, Function>,
}

fn as_printable(value: u16) -> char {
    let value = (value & 0xff) as u8;
    if (0x20..0x7f).contains(&value) {
        value as char
    } else {
        '.'
    }
}

fn data_word(value: u16) -> String {
    format!(
        "DW\t0x{value:04x}\t; {}{}",
        as_printable(value >> 8),
        as_printable(value)
    )
}

fn expanded_len(text: &str) -> usize {
    text.chars().fold(0, |col, c| {
        if c == '\t' {
            (col / 8 + 1) * 8
        } else {
            col + 1
        }
    })
}

/// Returns `prefix` and `suffix` so that `suffix` starts at tabstop `position`.
fn tabstop(prefix: &str, position: usize, suffix: &str) -> String {
    let tabcount = position.saturating_sub(expanded_len(prefix) / 8);
    let sep = if tabcount > 0 {
        "\t".repeat(tabcount)
    } else {
        " ".to_string()
    };
    format!("{prefix}{sep}{suffix}")
}

fn function_name(function: &Function) -> &str {
    function.as_deref().unwrap_or("None")
}

impl Disassembler {
    fn new(chip: &Chip) -> Result<Self, String> {
        let mut ram_ranges = Vec::new();
        for range in &chip.ram {
            let mut symbols: HashMap<u32, RamSymbols> = HashMap::new();
            for (address, name) in &range.symbols {
                let name = if name.is_empty() {
                    None
                } else {
                    Some(name.clone())
                };
                let (address, bit) = match address.split_once('.') {
                    Some((a, b)) => (a, Some(b)),
                    None => (address.as_str(), None),
                };
                let entry = symbols
                    .entry(parse_int(address)?)
                    .or_insert_with(|| RamSymbols {
                        name: None,
                        bits: BTreeMap::new(),
                    });
                match bit {
                    None => entry.name = name,
                    Some(bit) => {
                        entry.bits.insert(parse_int(bit)? as u8, name);
                    }
                }
            }
            ram_ranges.push(RamRange {
                start: range.start,
                stop: range.stop,
                symbols,
            });
        }
        let mut line_owner = HashMap::new();
        let mut functions = HashMap::new();
        let mut entry_stack = Vec::new();
        for (address, name) in &chip.callees {
            line_owner.insert(*address, Some(name.clone()));
            functions.insert(name.clone(), *address);
            entry_stack.push((*address, Some(name.clone())));
        }
        Ok(Disassembler {
            ram_ranges,
            rom_symbols: chip.rom.iter().cloned().collect(),
            entry_stack,
            callers: OrderedMultiMap::default(),
            jumpers: OrderedMultiMap::default(),
            ram_usage: BTreeMap::new(),
            functions,
            line_owner,
        })
    }

    fn function_address(&self, function: &Function) -> Result<u32, String> {
        function
            .as_ref()
            .and_then(|name| self.functions.get(name).copied())
            .ok_or_else(|| format!("Unknown function {:?}", function_name(function)))
    }

    fn name_and_offset(&self, address: u32) -> Result<String, String> {
        let function = self
            .line_owner
            .get(&address)
            .ok_or_else(|| format!("No owner for address 0x{address:04x}"))?;
        let offset = address as i64 - self.function_address(function)? as i64;
        let sign = if offset < 0 { '-' } else { '+' };
        Ok(format!(
            "{}{sign}0x{:x}",
            function_name(function),
            offset.unsigned_abs()
        ))
    }

    fn jump(&mut self, address: u32, operand: u32, function: &Function) -> Result<(), String> {
        self.entry_stack.push((operand, function.clone()));
        if operand == address + 1 {
            // Delay jump
            return Ok(());
        }
        self.jumpers.push(operand, address);
        if !self.rom_symbols.contains_key(&operand) {
            let offset = match function {
                // systematic disassembler, force absolute labels
                None => -1,
                Some(_) => operand as i64 - self.function_address(function)? as i64,
            };
            let label = if offset >= 0 {
                format!("{}_{offset:04x}", function_name(function))
            } else {
                format!("_label_{operand:04x}")
            };
            self.rom_symbols.insert(operand, label);
        }
        Ok(())
    }

    fn call(&mut self, address: u32, operand: u32, function: &Function) -> Result<(), String> {
        let callee = self
            .rom_symbols
            .entry(operand)
            .or_insert_with(|| format!("func_{operand:04x}"))
            .clone();
        match self.functions.get(&callee) {
            Some(&known) if known != operand => {
                return Err(format!(
                    "{callee} declared at 0x{known:04x} and 0x{operand:04x}"
                ));
            }
            Some(_) => {}
            None => {
                self.functions.insert(callee.clone(), operand);
            }
        }
        self.entry_stack.push((operand, Some(callee)));
        self.entry_stack.push((address + 1, function.clone()));
        self.callers.push(operand, address);
        Ok(())
    }

    fn disassemble(
        &mut self,
        address: u32,
        instruction: u16,
        function: &Function,
    ) -> Result<String, String> {
        let bincode = (instruction >> 8) as u8;
        let (key, is_bit) = if bincode >= 0x80 {
            (bincode & 0xc0, false)
        } else if bincode >= 0x40 {
            (bincode & 0xf8, true)
        } else {
            (bincode, false)
        };
        let Some(op) = opcode::by_key(key) else {
            self.entry_stack.push((address + 1, None));
            return Ok(format!("DW\t0x{instruction:04x}\t; ILLEGAL OPCODE"));
        };
        if op.space == Space::Nul {
            if op.flow == Flow::Next {
                self.entry_stack.push((address + 1, function.clone()));
            }
            return Ok(op.mnemonic.to_string());
        }
        let operand = (instruction & op.mask) as u32;
        match op.flow {
            Flow::NoNext => {}
            Flow::Next => self.entry_stack.push((address + 1, function.clone())),
            Flow::Branch => {
                self.entry_stack.push((address + 1, function.clone()));
                self.entry_stack.push((address + 2, function.clone()));
            }
            Flow::Jump => self.jump(address, operand, function)?,
            Flow::Call => self.call(address, operand, function)?,
        }
        let mut symbol: Option<String>;
        let mut operand_text: String;
        match op.space {
            Space::Rom => {
                symbol = self.rom_symbols.get(&operand).cloned();
                operand_text = format!("0x{operand:04x}");
            }
            Space::Imm => {
                symbol = None;
                operand_text = format!("#0x{operand:02x}");
            }
            _ => {
                symbol = None;
                let mut bit_symbols: BTreeMap<u8, Option<String>> = BTreeMap::new();
                for range in &self.ram_ranges {
                    if range.start <= address && address <= range.stop {
                        if let Some(entry) = range.symbols.get(&operand) {
                            if symbol.is_none() {
                                symbol = entry.name.clone();
                            }
                            for (bit, name) in &entry.bits {
                                bit_symbols.entry(*bit).or_insert_with(|| name.clone());
                            }
                        }
                    }
                }
                operand_text = format!("0x{operand:02x}");
                if is_bit {
                    let bit = bincode & 0x7;
                    match bit_symbols.get(&bit).cloned().flatten() {
                        Some(bit_symbol) => symbol = Some(bit_symbol),
                        None => match &mut symbol {
                            None => operand_text += &format!(".{bit}"),
                            Some(s) => *s += &format!(".{bit}"),
                        },
                    }
                }
            }
        }
        let symbol = symbol.unwrap_or_else(|| {
            if op.flow == Flow::Jump && operand == address + 1 {
                "$+1".to_string()
            } else {
                operand_text
            }
        });
        if matches!(op.space, Space::Zro | Space::Ram) {
            self.ram_usage
                .entry(symbol.clone())
                .or_default()
                .push((address, op.mode));
        }
        let caption: Vec<&str> = [op.left, op.right]
            .iter()
            .filter_map(|arg| match arg {
                Arg::None => None,
                Arg::Fixed(name) => Some(*name),
                _ => Some(symbol.as_str()),
            })
            .collect();
        let mut text = op.mnemonic.to_string();
        if !caption.is_empty() {
            text += "\t";
            text += &caption.join(", ");
        }
        Ok(text)
    }

    fn walk(&mut self, rom: &mut BTreeMap<u32, u16>) -> Result<BTreeMap<u32, String>, String> {
        let mut disassembled = BTreeMap::new();
        while let Some((address, mut function)) = self.entry_stack.pop() {
            let Some(instruction) = rom.remove(&address) else {
                continue; // Already disassembled
            };
            let owner = self
                .line_owner
                .entry(address)
                .or_insert_with(|| function.clone());
            if *owner != function {
                function = owner.clone();
            }
            if instruction == 0x03ce || instruction == 0x13ce {
                // B0ADD PCL, A / ADD PCL, A: a jump table follows, composed of
                // JMP, CALL and NOP. It ends at the first unexpected instruction.
                let mut found = false;
                let mut sub_address = address + 1;
                while let Some(&word) = rom.get(&sub_address) {
                    if word & 0x8000 == 0 && word != 0 {
                        break;
                    }
                    found = true;
                    if !disassembled.contains_key(&sub_address) {
                        self.entry_stack.push((sub_address, function.clone()));
                    }
                    sub_address += 1;
                }
                if !found {
                    return Err(format!("Empty jump table at 0x{address:04x}"));
                }
            }
            let text = self.disassemble(address, instruction, &function)?;
            disassembled.insert(address, text);
        }
        // Convert any non-disassembled word into a data word, ignoring runs of
        // more than 6 null words for readability.
        let mut chunks: Vec<(u32, Vec<u16>)> = Vec::new();
        let mut next_address = None;
        for (&address, &value) in rom.iter() {
            if Some(address) != next_address {
                chunks.push((address, Vec::new()));
            }
            next_address = Some(address + 1);
            chunks.last_mut().unwrap().1.push(value);
        }
        for (mut chunk_address, values) in chunks {
            let mut i = 0;
            while i < values.len() {
                let value = values[i];
                let count = values[i..].iter().take_while(|v| **v == value).count();
                if value != 0 || count < 7 {
                    for offset in 0..count as u32 {
                        disassembled.insert(chunk_address + offset, data_word(value));
                    }
                }
                chunk_address += count as u32;
                i += count;
            }
        }
        Ok(disassembled)
    }
}

/// Disassemble a raw ROM image, producing source accepted by [`super::assemble`].
pub fn disassemble(image: &[u8], chip: &Chip) -> Result<String, String> {
    let mut out = String::new();
    let mut d = Disassembler::new(chip)?;

    // Note: only look for duplicate ram symbols, not rom.
    let mut reverse: HashMap<&str, &str> = HashMap::new();
    let mut flat: BTreeMap<u32, (Vec<Option<String>>, BTreeMap<u8, Vec<Option<String>>>)> =
        BTreeMap::new();
    for range in &chip.ram {
        for (address, name) in &range.symbols {
            let (word, bit) = match address.split_once('.') {
                Some((a, b)) => (a, Some(b)),
                None => (address.as_str(), None),
            };
            let word = parse_int(word)?;
            let name = if name.is_empty() {
                // An empty name un-defines a symbol.
                None
            } else {
                if let Some(other) = reverse.insert(name, address) {
                    if other != address {
                        return Err(format!(
                            "Name {name:?} is used for different addresses: {other:?} and {address:?}"
                        ));
                    }
                }
                Some(name.clone())
            };
            let (names, bits) = flat.entry(word).or_default();
            match bit {
                None => names.push(name),
                Some(bit) => bits.entry(parse_int(bit)? as u8).or_default().push(name),
            }
        }
    }

    out += &format!("CHIP\t{}\n", chip.name);

    let mut rom: BTreeMap<u32, u16> = BTreeMap::new();
    for (i, word) in image.chunks_exact(2).enumerate() {
        let address = chip.rom_start + i as u32;
        if address > chip.rom_stop {
            break;
        }
        rom.insert(address, u16::from_le_bytes([word[0], word[1]]));
    }

    out += "//{{SONIX_CODE_OPTION\n";
    for (name, option) in &chip.code_options {
        let CodeOptionValues::Named(values) = &option.values else {
            continue;
        };
        let Some(&word) = rom.get(&option.address) else {
            continue;
        };
        let mut mask = option.mask;
        let mut value = word as u32 & mask;
        while mask & 1 == 0 {
            mask >>= 1;
            value >>= 1;
        }
        let value_name = values
            .iter()
            .find(|(v, _)| *v == value)
            .map(|(_, n)| n)
            .ok_or_else(|| format!("Unknown value {value} for code option {name}"))?;
        out += &tabstop(
            &format!("\t.Code_Option\t{name}"),
            5,
            &format!("\"{value_name}\"\n"),
        );
    }
    out += "//}}SONIX_CODE_OPTION\n";

    let disassembled = d.walk(&mut rom)?;

    out += ".DATA\n";
    let reserved = |name: &Option<String>| match name {
        Some(n) => chip.ram_reserved_names.contains(n),
        None => false,
    };
    for (address, (names, bits)) in &flat {
        for name in names {
            if !reserved(name) {
                out += &format!("{}\tEQU\t0x{address:02x}\n", function_name(name));
            }
        }
        for (bit, names) in bits {
            for name in names {
                if !reserved(name) {
                    out += &format!("{}\tEQU\t0x{address:02x}.{bit}\n", function_name(name));
                }
            }
        }
    }
    if !d.ram_usage.is_empty() {
        out += "; RAM address usage\n";
        for (symbol, accessors) in &d.ram_usage {
            out += &format!("; {symbol}:\n");
            let mut accessors = accessors.clone();
            accessors.sort();
            for (accessor, mode) in accessors {
                out += &format!(";\t{mode} {}\n", d.name_and_offset(accessor)?);
            }
        }
    }

    out += ".CODE\n";
    let mut comments: HashMap<u32, String> = chip.comments.iter().cloned().collect();
    for reserved in chip.rom_reserved.iter() {
        comments
            .entry(reserved)
            .or_insert_with(|| "Reserved".to_string());
    }
    let mut next_key = None;
    for (&key, text) in &disassembled {
        let is_reserved = chip.rom_reserved.contains(key);
        if Some(key) != next_key {
            if is_reserved {
                out += ";";
            }
            out += &format!("ORG 0x{key:04x}\n");
        }
        if key & 0xf == 0 {
            out += &tabstop("", COMMENT_POSITION, &format!("; 0x{key:04x}\n"));
        }
        next_key = Some(key + 1);
        if let Some(symbol) = d.rom_symbols.get(&key).cloned() {
            out += &tabstop(&format!("{symbol}:"), COMMENT_POSITION, ";");
            if let Some(mut callers) = d.callers.pop(key) {
                callers.sort();
                let names = callers
                    .iter()
                    .map(|a| d.name_and_offset(*a))
                    .collect::<Result<Vec<_>, _>>()?;
                out += &format!(" Called from {}", names.join(", "));
            }
            if let Some(mut jumpers) = d.jumpers.pop(key) {
                jumpers.sort();
                let names = jumpers
                    .iter()
                    .map(|a| d.name_and_offset(*a))
                    .collect::<Result<Vec<_>, _>>()?;
                out += &format!(" Jumped from {}", names.join(", "));
            }
            out += "\n";
        } else if d.callers.contains(key) || d.jumpers.contains(key) {
            return Err(format!("Unlabelled call/jump target 0x{key:04x}"));
        }
        let mut line = format!("{}\t{text}", if is_reserved { ";" } else { "" });
        if let Some(comment) = comments.get(&key) {
            line = tabstop(&line, COMMENT_POSITION, &format!("; {comment}"));
        }
        out += &line;
        out += "\n";
    }
    for (title, kind, map) in [
        ("; Unknown calls:\n", "Called", &d.callers),
        ("; Unknown jumps:\n", "Jumped", &d.jumpers),
    ] {
        if map.is_empty() {
            continue;
        }
        out += title;
        for (address, sources) in map.items() {
            let symbol = &d.rom_symbols[&address];
            let names = sources
                .iter()
                .map(|a| d.name_and_offset(*a))
                .collect::<Result<Vec<_>, _>>()?;
            out += &format!("ORG 0x{address:04x}\n");
            out += &format!("{symbol}: ; {kind} from {}\n", names.join(", "));
            out += &format!("\tJMP\t{symbol}\n");
        }
    }
    out += "ENDP\n";
    Ok(out)
}

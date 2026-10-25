// Port of `parseConfig` and `MultiRange` from libsn8.py.

const SN8F2288_CFG: &str = include_str!("sn8f2288.cfg");

/// Set of half-open `[start, stop)` ranges.
#[derive(Clone, Debug, Default)]
pub struct MultiRange {
    pub ranges: Vec<(u32, u32)>,
}

impl MultiRange {
    fn parse(definition: &str, min_boundary: u32, max_boundary: u32) -> Result<Self, String> {
        let mut ranges = Vec::new();
        for span in definition.split(',') {
            if let Some((start, stop)) = span.split_once('-') {
                let start = if start.is_empty() {
                    min_boundary
                } else {
                    parse_int(start)?
                };
                let stop = if stop.is_empty() {
                    max_boundary
                } else {
                    parse_int(stop)? + 1
                };
                ranges.push((start, stop));
            } else {
                let start = parse_int(span)?;
                ranges.push((start, start + 1));
            }
        }
        Ok(MultiRange { ranges })
    }

    pub fn contains(&self, value: u32) -> bool {
        self.ranges
            .iter()
            .any(|&(start, stop)| start <= value && value < stop)
    }

    pub fn iter(&self) -> impl Iterator<Item = u32> + '_ {
        self.ranges.iter().flat_map(|&(start, stop)| start..stop)
    }
}

#[derive(Clone, Debug)]
pub enum CodeOptionValues {
    /// Named values of an actual option.
    Named(Vec<(u32, String)>),
    /// Hard-coded word (mask == 0).
    Fixed(u32),
}

#[derive(Clone, Debug)]
pub struct CodeOption {
    pub address: u32,
    pub mask: u32,
    pub values: CodeOptionValues,
}

/// RAM symbols valid while executing code within `start..=stop`.
#[derive(Clone, Debug)]
pub struct RamRange {
    pub priority: u8,
    pub start: u32,
    pub stop: u32,
    /// Raw `(address[.bit], name)` pairs, in file order.
    pub symbols: Vec<(String, String)>,
}

#[derive(Clone, Debug)]
pub struct Chip {
    pub name: String,
    pub rom_start: u32,
    pub rom_stop: u32,
    pub ram_start: u32,
    pub ram_stop: u32,
    pub rom_reserved: MultiRange,
    pub ram_reserved: MultiRange,
    pub code_options: Vec<(String, CodeOption)>,
    pub comments: Vec<(u32, String)>,
    pub callees: Vec<(u32, String)>,
    /// `[rom]` updated with `[callee]`.
    pub rom: Vec<(u32, String)>,
    /// Sorted: shorter ranges first, `[ram-reserved]` last.
    pub ram: Vec<RamRange>,
    /// Names declared in `[ram-reserved]`.
    pub ram_reserved_names: Vec<String>,
}

impl Chip {
    /// Look up a chip definition bundled with this crate, by (case-insensitive) name.
    pub fn builtin(name: &str) -> Result<Chip, String> {
        match name.to_ascii_lowercase().as_str() {
            "sn8f2288" => Chip::parse(SN8F2288_CFG),
            _ => Err(format!("Unknown chip: {name}")),
        }
    }

    pub fn sn8f2288() -> Chip {
        Chip::parse(SN8F2288_CFG).expect("bundled sn8f2288.cfg must be valid")
    }

    pub fn parse(text: &str) -> Result<Chip, String> {
        let sections = parse_ini(text)?;
        let section = |name: &str| sections.iter().find(|(n, _)| n == name).map(|(_, s)| s);

        let chip_section = section("chip").ok_or("Missing [chip] section")?;
        let get = |key: &str| {
            chip_section
                .iter()
                .find(|(k, _)| k == key)
                .map(|(_, v)| v.as_str())
                .ok_or_else(|| format!("Missing '{key}' in [chip]"))
        };
        let name = get("name")?.to_string();
        let rom_start = parse_int(get("rom_start")?)?;
        let rom_stop = parse_int(get("rom_stop")?)?;
        let ram_start = parse_int(get("ram_start")?)?;
        let ram_stop = parse_int(get("ram_stop")?)?;
        let rom_reserved =
            MultiRange::parse(get("rom_reserved").unwrap_or(""), rom_start, rom_stop)?;
        let ram_reserved =
            MultiRange::parse(get("ram_reserved").unwrap_or(""), ram_start, ram_stop)?;

        let mut chip = Chip {
            name,
            rom_start,
            rom_stop,
            ram_start,
            ram_stop,
            rom_reserved,
            ram_reserved,
            code_options: Vec::new(),
            comments: Vec::new(),
            callees: Vec::new(),
            rom: Vec::new(),
            ram: Vec::new(),
            ram_reserved_names: Vec::new(),
        };

        for (section_name, items) in &sections {
            match section_name.as_str() {
                "chip" => {}
                "comment" => chip.comments = int_keyed(items)?,
                "rom" => chip.rom = int_keyed(items)?,
                "callee" => chip.callees = int_keyed(items)?,
                "code-option" => {
                    for (option_name, definition) in items {
                        let mut parts = definition.splitn(3, ' ');
                        let (Some(address), Some(mask), Some(value_names)) =
                            (parts.next(), parts.next(), parts.next())
                        else {
                            return Err(format!("Bad code option definition: {definition}"));
                        };
                        let address = parse_int(address)?;
                        let mask = parse_int(mask)?;
                        let values = if mask != 0 {
                            let mut named = Vec::new();
                            for pair in value_names.split(' ') {
                                let (value, name) = pair
                                    .split_once('=')
                                    .ok_or_else(|| format!("Bad code option value: {pair}"))?;
                                upsert(&mut named, parse_int(value)?, name.to_string());
                            }
                            CodeOptionValues::Named(named)
                        } else {
                            CodeOptionValues::Fixed(parse_int(value_names)?)
                        };
                        upsert(
                            &mut chip.code_options,
                            option_name.clone(),
                            CodeOption {
                                address,
                                mask,
                                values,
                            },
                        );
                    }
                }
                name if name.starts_with("ram") => {
                    let mut priority = 0;
                    let (start, stop) = if let Some(range) = name.strip_prefix("ram@") {
                        let (start, stop) = range
                            .split_once('-')
                            .ok_or_else(|| format!("Bad ram section name: {name}"))?;
                        (parse_int(start)?, parse_int(stop)?)
                    } else {
                        if name != "ram" && name != "ram-reserved" {
                            return Err(format!("Unknown ram section: {name}"));
                        }
                        if name == "ram-reserved" {
                            // Low priority, to allow overriding in [ram] section.
                            priority = 1;
                            chip.ram_reserved_names =
                                items.iter().map(|(_, v)| v.clone()).collect();
                        }
                        (rom_start, rom_stop)
                    };
                    chip.ram.push(RamRange {
                        priority,
                        start,
                        stop,
                        symbols: items.clone(),
                    });
                }
                _ => {}
            }
        }
        // Shorter ranges first, overriding larger maps (stable sort, as Python).
        chip.ram
            .sort_by_key(|r| (r.priority, r.stop as i64 - r.start as i64));
        for (address, name) in chip.callees.clone() {
            upsert(&mut chip.rom, address, name);
        }
        Ok(chip)
    }
}

/// Insert or replace, keeping the position of an existing key (Python dict semantics).
fn upsert<K: PartialEq, V>(list: &mut Vec<(K, V)>, key: K, value: V) {
    if let Some(entry) = list.iter_mut().find(|(k, _)| *k == key) {
        entry.1 = value;
    } else {
        list.push((key, value));
    }
}

fn int_keyed(items: &[(String, String)]) -> Result<Vec<(u32, String)>, String> {
    let mut out = Vec::new();
    for (key, value) in items {
        upsert(&mut out, parse_int(key)?, value.clone());
    }
    Ok(out)
}

/// Equivalent of Python `int(text, 0)` for non-negative values.
pub(crate) fn parse_int(text: &str) -> Result<u32, String> {
    let t = text.trim().replace('_', "");
    let lower = t.to_ascii_lowercase();
    let (digits, radix) = if let Some(rest) = lower.strip_prefix("0x") {
        (rest, 16)
    } else if let Some(rest) = lower.strip_prefix("0b") {
        (rest, 2)
    } else if let Some(rest) = lower.strip_prefix("0o") {
        (rest, 8)
    } else {
        if lower.len() > 1 && lower.starts_with('0') && lower.bytes().any(|b| b != b'0') {
            return Err(format!("invalid literal for int() with base 0: {text:?}"));
        }
        (lower.as_str(), 10)
    };
    u32::from_str_radix(digits, radix)
        .map_err(|_| format!("invalid literal for int() with base 0: {text:?}"))
}

/// Minimal configparser-compatible reader (no interpolation, no continuation lines).
fn parse_ini(text: &str) -> Result<Vec<(String, Vec<(String, String)>)>, String> {
    let mut sections: Vec<(String, Vec<(String, String)>)> = Vec::new();
    for (lineno, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        if raw.starts_with([' ', '\t']) {
            return Err(format!(
                "line {}: continuation lines are not supported",
                lineno + 1
            ));
        }
        if let Some(name) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
            if sections.iter().any(|(n, _)| n == name) {
                return Err(format!("line {}: duplicate section [{name}]", lineno + 1));
            }
            sections.push((name.to_string(), Vec::new()));
            continue;
        }
        let (_, items) = sections
            .last_mut()
            .ok_or_else(|| format!("line {}: option outside of a section", lineno + 1))?;
        let split = line
            .find(['=', ':'])
            .ok_or_else(|| format!("line {}: missing '=' in {line:?}", lineno + 1))?;
        let key = line[..split].trim_end().to_string();
        let value = line[split + 1..].trim().to_string();
        if value.contains('%') {
            return Err(format!(
                "line {}: '%' interpolation is not supported",
                lineno + 1
            ));
        }
        if items.iter().any(|(k, _)| *k == key) {
            return Err(format!("line {}: duplicate option {key:?}", lineno + 1));
        }
        items.push((key, value));
    }
    Ok(sections)
}

use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

/// Replace leading tabs with N spaces and other tabs with a single space.
fn convert_leading_tabs_to_spaces(line: &str, tab_as_spaces: usize) -> String {
    let mut result = String::new();
    result.reserve(line.len() + 8); // small reserve to avoid frequent reallocations

    let mut chars = line.chars().peekable();

    // Expand leading tabs
    while let Some(&c) = chars.peek() {
        if c == '\t' {
            chars.next();
            result.push_str(&" ".repeat(tab_as_spaces));
        } else {
            break;
        }
    }

    // Convert remaining tabs to one space
    for c in chars {
        if c == '\t' {
            result.push(' ');
        } else {
            result.push(c);
        }
    }

    result
}


/// Return the first non-whitespace token in a line.
fn first_word(line: &str) -> &str {
    line.split_whitespace().next().unwrap_or("")
}


/// Format an assembly file:
/// - strip comments,
/// - normalize indentation and tabs,
/// - align mnemonics/operands,
/// - optionally insert blank lines before func_* labels,
/// - align trailing semicolons.
pub fn format_asm_file(in_path: &str, out_path: &str) -> io::Result<()> {
    let in_path = Path::new(in_path);
    if !in_path.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Input file not found: {}", in_path.display()),
        ));
    }

    // --------------------------------------------------------------
    // 1. Read the entire input file
    // --------------------------------------------------------------
    let mut input = String::new();
    fs::File::open(in_path)?.read_to_string(&mut input)?;

    // --------------------------------------------------------------
    // 2. Preprocess lines: remove comments, normalize tabs/spaces
    // --------------------------------------------------------------
    let mut cleaned_lines = Vec::new();
    cleaned_lines.reserve(input.lines().count());

    for raw_line in input.lines() {
        let mut line = raw_line.to_string();

        // Skip full-line comments
        if line.trim_start().starts_with(';') {
            continue;
        }

        // Remove inline comments starting from ';'
        if let Some(pos) = line.find(';') {
            line.truncate(pos);
        }

        // Convert tabs and trim trailing spaces
        line = convert_leading_tabs_to_spaces(&line, 4);
        let trimmed = line.trim_end();

        if !trimmed.is_empty() {
            cleaned_lines.push(trimmed.to_string());
        }
    }

    if cleaned_lines.is_empty() {
        // If nothing remains, write an empty output file.
        fs::File::create(out_path)?; // truncate/create
        println!("Warning: no lines left after processing.");
        println!("Empty output written to: {}", out_path);
        return Ok(());
    }

    // --------------------------------------------------------------
    // 3. Parse instruction structure and compute alignment widths
    // --------------------------------------------------------------
    #[derive(Debug)]
    struct StructuredLine {
        raw: String,
        indent: String,
        mnemonic: String,
        op1: String,
        op2: Option<String>,
        candidate: bool,
    }

    let mut structured = Vec::with_capacity(cleaned_lines.len());
    let mut candidate_widths = Vec::new();
    candidate_widths.reserve(cleaned_lines.len());

    for line in &cleaned_lines {
        // Count leading spaces
        let indent_len = line.chars().take_while(|c| *c == ' ').count();
        let indent = " ".repeat(indent_len);
        let rest = line[indent_len..].trim();

        if rest.is_empty() {
            structured.push(StructuredLine {
                raw: line.clone(),
                indent,
                mnemonic: String::new(),
                op1: String::new(),
                op2: None,
                candidate: false,
            });
            continue;
        }

        let parts: Vec<&str> = rest.split_whitespace().collect();
        if parts.len() < 2 {
            // Lines without "mnemonic operand" are not alignment candidates
            structured.push(StructuredLine {
                raw: line.clone(),
                indent,
                mnemonic: String::new(),
                op1: String::new(),
                op2: None,
                candidate: false,
            });
            continue;
        }

        let mnemonic = parts[0].to_string();
        let operand_str = parts[1..].join(" ");

        let (op1, op2) = if let Some(pos) = operand_str.find(',') {
            let left = operand_str[..pos].trim().to_string();
            let right = operand_str[pos + 1..].trim().to_string();
            let op2 = if right.is_empty() { None } else { Some(right) };
            (left, op2)
        } else {
            (operand_str.trim().to_string(), None)
        };

        let width = indent_len + mnemonic.len();
        candidate_widths.push(width);

        structured.push(StructuredLine {
            raw: line.clone(),
            indent,
            mnemonic,
            op1,
            op2,
            candidate: true,
        });
    }

    // --------------------------------------------------------------
    // 4. Align mnemonics and operands into columns
    // --------------------------------------------------------------
    let aligned_lines: Vec<String> = if !candidate_widths.is_empty() {
        let max_width = *candidate_widths.iter().max().unwrap();
        let operand_col = max_width + 1;

        structured
            .into_iter()
            .map(|item| {
                if !item.candidate {
                    return item.raw;
                }

                let base_width = item.indent.len() + item.mnemonic.len();
                let spaces_between = operand_col.saturating_sub(base_width).max(1);

                let mut line = String::with_capacity(item.indent.len()
                    + item.mnemonic.len()
                    + spaces_between
                    + item.op1.len()
                    + item.op2.as_ref().map(|s| s.len() + 8).unwrap_or(0));

                line.push_str(&item.indent);
                line.push_str(&item.mnemonic);
                line.push_str(&" ".repeat(spaces_between));
                line.push_str(&item.op1);

                if let Some(op2) = item.op2 {
                    line.push(',');
                    if item.op1.len() < 6 {
                        line.push_str(&" ".repeat(6 - item.op1.len()));
                    }
                    line.push_str(&op2);
                }

                line
            })
            .collect()
    } else {
        // No alignment candidates, just reuse cleaned lines
        cleaned_lines
    };

    // --------------------------------------------------------------
    // 5. Insert blank lines before func_* labels when appropriate
    // --------------------------------------------------------------
    const NO_BLANK_PREV: [&str; 4] = ["RET", "DW", "CALL", "JMP"];
    const SECOND_PREV_OK: [&str; 5] = ["CMPRS", "B0BTS0", "B0BTS1", "BTS0", "BTS1"];

    let mut with_blank_lines = Vec::with_capacity(aligned_lines.len() * 2);

    for (idx, line) in aligned_lines.iter().enumerate() {
        let stripped = line.trim();

        let is_func_label = stripped.starts_with("func_")
            && stripped.ends_with(':')
            && !stripped.contains(' ');

        if is_func_label {
            let mut need_blank = true;

            if idx == 0 {
                // Do not insert a blank line before the very first line
                need_blank = false;
            } else {
                let prev_first = first_word(&aligned_lines[idx - 1]);
                let cond1 = !NO_BLANK_PREV.contains(&prev_first);

                let mut cond2 = false;
                if NO_BLANK_PREV.contains(&prev_first) && idx >= 2 {
                    let prev2_first = first_word(&aligned_lines[idx - 2]);
                    cond2 = SECOND_PREV_OK.contains(&prev2_first);
                }

                // If previous or second-previous lines match specific patterns,
                // we skip inserting a blank line.
                if cond1 || cond2 {
                    need_blank = false;
                }
            }

            if need_blank {
                with_blank_lines.push(String::new());
            }
        }

        with_blank_lines.push(line.clone());
    }

    // --------------------------------------------------------------
    // 6. Align trailing semicolons
    // --------------------------------------------------------------
    let mut max_len = 0usize;
    for line in &with_blank_lines {
        if !line.is_empty() && line.len() > max_len {
            max_len = line.len();
        }
    }

    // --------------------------------------------------------------
    // 7. Write the final result directly via BufWriter
    // --------------------------------------------------------------
    let out_file = fs::File::create(out_path)?;
    let mut writer = io::BufWriter::new(out_file);

    for line in with_blank_lines {
        if line.is_empty() {
            writer.write_all(b"\n")?;
        } else {
            let spaces = max_len.saturating_sub(line.len()) + 1;
            writer.write_all(line.as_bytes())?;
            writer.write_all(&vec![b' '; spaces])?;
            writer.write_all(b";\n")?;
        }
    }

    writer.flush()?;
    println!("Saved to: {}", out_path);

    Ok(())
}

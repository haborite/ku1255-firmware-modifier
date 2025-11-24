use serde::Deserialize;
use std::fs;
use std::io::{self, BufRead, BufReader, Write, Read};
use std::path::Path;

/// JSON diff file format:
/// {
///   "ops": [
///     { "op": "copy",   "from": 123 },
///     { "op": "insert", "code": "MOV A,#0x20" },
///     ...
///   ]
/// }
#[derive(Debug, Deserialize)]
struct DiffOps {
    ops: Vec<Op>,
}

#[derive(Debug, Deserialize)]
struct Op {
    op: String,
    #[serde(default)]
    from: Option<usize>,    // 1-based line number in Origin.asm (code part)
    #[serde(default)]
    code: Option<String>,   // literal code line for "insert"
}

/// Split a line into (code, comment) by the first ';' character.
/// - `code` does NOT include ';'
/// - `comment` includes ';' and the rest (or empty if no ';')
fn split_code_comment(line: &str) -> (String, String) {
    if let Some(idx) = line.find(';') {
        let (code, comment) = line.split_at(idx);
        (code.to_string(), comment.to_string())
    } else {
        (line.to_string(), String::new())
    }
}

/// Read only the "code part" (before ';') from Origin.asm.
/// Each element corresponds to a line (including empty code lines).
fn read_codes<P: AsRef<Path>>(path: P) -> io::Result<Vec<String>> {
    let file = fs::File::open(path)?;
    let reader = io::BufReader::new(file);

    let mut codes = Vec::new();
    for line in reader.lines() {
        let line = line?; // no trailing newline
        let (code, _comment) = split_code_comment(&line);
        codes.push(code);
    }
    Ok(codes)
}

/// Apply the JSON operation list to the A-code list and build B-code list.
/// This mirrors the Python `build_b_codes` logic:
/// - "copy": copy line from A by 1-based index
/// - "insert": insert given code string as-is.
fn build_b_codes(a_codes: &[String], ops: &[Op]) -> io::Result<Vec<String>> {
    let mut b_codes = Vec::with_capacity(a_codes.len());

    for op in ops {
        match op.op.as_str() {
            "copy" => {
                let from = op.from.ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Missing 'from' field in op: {:?}", op),
                    )
                })?;

                if from == 0 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Invalid 'from' (must be >= 1): {}", from),
                    ));
                }

                let idx = from - 1;
                if idx >= a_codes.len() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("'from' line {} is out of range for A (len = {})", from, a_codes.len()),
                    ));
                }

                b_codes.push(a_codes[idx].clone());
            }
            "insert" => {
                let code = op.code.clone().unwrap_or_default();
                b_codes.push(code);
            }
            other => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Unknown op type: {}", other),
                ));
            }
        }
    }

    Ok(b_codes)
}


/// - origin_path : A (Origin.asm)
/// - diff_path   : diff.json
/// - comments_path: comments.txt
/// - out_path    : output (Modified.asm)
pub fn apply_diff_files(
    origin_path: &str,
    diff_path: &str,
    comments_path: &str,
    out_path: &str,
) -> io::Result<()> {
    // 1. Read A code part
    let a_codes = read_codes(origin_path)?;

    // 2. Load JSON operation list
    let mut diff_json_str = String::new();
    fs::File::open(diff_path)?.read_to_string(&mut diff_json_str)?;
    let diff_ops: DiffOps = serde_json::from_str(&diff_json_str).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Failed to parse diff JSON: {}", e),
        )
    })?;
    let ops = diff_ops.ops;

    // 3. Build B codes from A + ops
    let b_codes = build_b_codes(&a_codes, &ops)?;

    // 4. Read comments.txt (one line per output line)
    let comments_file = fs::File::open(comments_path)?;
    let comments_reader = BufReader::new(comments_file);
    let mut comments = Vec::new();

    for line in comments_reader.lines() {
        // Python: ln.rstrip("\n") → `lines()` already strips newline
        let line = line?;
        comments.push(line);
    }

    // 5. Check line count consistency
    if b_codes.len() != comments.len() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "Number of code lines after ops ({}) != number of comment lines ({})",
                b_codes.len(),
                comments.len()
            ),
        ));
    }

    // 6. Combine code + comment and write to output
    let out_file = fs::File::create(out_path)?;
    let mut writer = io::BufWriter::new(out_file);

    for (code, comment) in b_codes.iter().zip(comments.iter()) {
        writer.write_all(code.as_bytes())?;
        writer.write_all(comment.as_bytes())?;
        writer.write_all(b"\n")?;
    }

    writer.flush()?;
    println!("apply_diff: wrote {}", out_path);
    Ok(())
}


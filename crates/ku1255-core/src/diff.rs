use serde::Deserialize;
use std::fs;
use std::io;

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
fn read_codes(origin: &str) -> Vec<String> {
    origin.lines().map(|line| split_code_comment(line).0).collect()
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
    let output = apply_diff(
        &fs::read_to_string(origin_path)?,
        &fs::read_to_string(diff_path)?,
        &fs::read_to_string(comments_path)?,
    )?;
    fs::write(out_path, output)?;
    println!("apply_diff: wrote {}", out_path);
    Ok(())
}

/// Same as [`apply_diff_files`], on strings.
pub fn apply_diff(origin: &str, diff_json: &str, comments: &str) -> io::Result<String> {
    // 1. Read A code part
    let a_codes = read_codes(origin);

    // 2. Load JSON operation list
    let diff_ops: DiffOps = serde_json::from_str(diff_json).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Failed to parse diff JSON: {}", e),
        )
    })?;

    // 3. Build B codes from A + ops
    let b_codes = build_b_codes(&a_codes, &diff_ops.ops)?;

    // 4. comments.txt (one line per output line)
    let comments: Vec<&str> = comments.lines().collect();

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

    // 6. Combine code + comment
    let mut out = String::new();
    for (code, comment) in b_codes.iter().zip(comments) {
        out.push_str(code);
        out.push_str(comment);
        out.push('\n');
    }
    Ok(out)
}

use std::collections::HashMap;
use std::fs;
use std::io;

/// Error type for template rendering
#[derive(Debug)]
pub enum TemplateError {
    UnclosedPlaceholder(usize),
    EmptyPlaceholder(usize),
    InvalidFormat(String),
    UnknownKind(String),
    MissingEnumChoice { name: String },
    EnumIndexOutOfRange { name: String, idx: usize, len: usize },
}

impl std::fmt::Display for TemplateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TemplateError::UnclosedPlaceholder(pos) => {
                write!(f, "Unclosed placeholder starting at position {}", pos)
            }
            TemplateError::EmptyPlaceholder(pos) => {
                write!(f, "Empty placeholder at position {}", pos)
            }
            TemplateError::InvalidFormat(msg) => write!(f, "Invalid placeholder format: {}", msg),
            TemplateError::UnknownKind(k) => write!(f, "Unknown placeholder kind: {}", k),
            TemplateError::MissingEnumChoice { name } => {
                write!(f, "Enum choice index not provided for '{}'", name)
            }
            TemplateError::EnumIndexOutOfRange { name, idx, len } => write!(
                f,
                "Enum index {} for '{}' out of range (choices = {})",
                idx, name, len
            ),
        }
    }
}

impl std::error::Error for TemplateError {}

/// Render a template string containing ${...} placeholders.
///
/// - `s_values`: override values for `s` type (free string). If absent, default in template is used.
/// - `e_choices`: index (0-based) to pick one of the enum choices for `e` type.
///
/// Example placeholders:
///   ${s/fn_key/af}
///   ${e/tp_sp5/NOP/CALL func_07b7}
pub fn render_template(
    input: &str,
    s_values: &HashMap<String, String>,
    e_choices: &HashMap<String, usize>,
) -> Result<String, TemplateError> {
    let mut out = String::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if i + 1 < bytes.len() && bytes[i] == b'$' && bytes[i + 1] == b'{' {
            let start = i; // position of '$'
            // find the closing '}'
            let mut j = i + 2;
            let mut end_opt = None;
            while j < bytes.len() {
                if bytes[j] == b'}' {
                    end_opt = Some(j);
                    break;
                }
                j += 1;
            }
            let end = match end_opt {
                Some(pos) => pos,
                None => return Err(TemplateError::UnclosedPlaceholder(start)),
            };

            // Extract inside "${ ... }"
            let inner = &input[(i + 2)..end].trim();
            if inner.is_empty() {
                return Err(TemplateError::EmptyPlaceholder(start));
            }

            let replacement = render_placeholder(inner, s_values, e_choices)?;
            out.push_str(&replacement);

            i = end + 1; // move past '}'
        } else {
            out.push(bytes[i] as char);
            i += 1;
        }
    }

    Ok(out)
}

/// Handle one placeholder body, e.g. "s/fn_key/af" or "e/tp_sp5/NOP/CALL func_07b7"
fn render_placeholder(
    inner: &str,
    s_values: &HashMap<String, String>,
    e_choices: &HashMap<String, usize>,
) -> Result<String, TemplateError> {
    let parts: Vec<&str> = inner.split('/').collect();
    if parts.len() < 3 {
        return Err(TemplateError::InvalidFormat(inner.to_string()));
    }

    let kind = parts[0];
    let name = parts[1].to_string();

    match kind {
        "s" => {
            // s / <name> / <default>
            let default_value = parts[2..].join("/"); // allow '/' inside default by re-joining
            if let Some(v) = s_values.get(&name) {
                Ok(v.clone())
            } else {
                Ok(default_value)
            }
        }
        "e" => {
            // e / <name> / choice0 / choice1 / ...
            if parts.len() < 4 {
                return Err(TemplateError::InvalidFormat(format!(
                    "Enum placeholder '{}' must have at least one choice",
                    inner
                )));
            }
            let choices: Vec<&str> = parts[2..].to_vec();
            let idx = e_choices
                .get(&name)
                .ok_or_else(|| TemplateError::MissingEnumChoice { name: name.clone() })?;
            if *idx >= choices.len() {
                return Err(TemplateError::EnumIndexOutOfRange {
                    name,
                    idx: *idx,
                    len: choices.len(),
                });
            }
            Ok(choices[*idx].to_string())
        }
        other => Err(TemplateError::UnknownKind(other.to_string())),
    }
}

pub fn render_template_file(
    in_path: &str,
    out_path: &str,
    s_values: &HashMap<String, String>,
    e_choices: &HashMap<String, usize>,
) -> io::Result<()> {
    let input = fs::read_to_string(in_path)?;
    let rendered = render_template(&input, s_values, e_choices)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
    fs::write(out_path, rendered)?;
    Ok(())
}

//! Data files bundled into the executable, so the app does not depend on its
//! working directory or install location.

use std::io;

use include_dir::{Dir, include_dir};

pub static BOARDS: Dir = include_dir!("$CARGO_MANIFEST_DIR/boards");
pub static LOGICAL_LAYOUTS: Dir = include_dir!("$CARGO_MANIFEST_DIR/logical_layouts");
pub static EXAMPLES: Dir = include_dir!("$CARGO_MANIFEST_DIR/examples");

pub const GENERAL_SETTING: &str = include_str!("../settings/general_setting.csv");
pub const MEDIA_KEY_USAGE_NAMES: &str = include_str!("../settings/media_key_usage_names.csv");
pub const DIFF_JSON: &str = include_str!("../template/diff.json");
pub const COMMENTS: &str = include_str!("../template/comments.txt");
const URL: &str = include_str!("../settings/url.txt");

/// URL of the official Lenovo firmware installer.
pub fn firmware_url() -> &'static str {
    URL.lines().next().unwrap_or("").trim()
}

/// `(file name, contents)` of the files with extension `ext` directly in `dir`,
/// sorted by file name.
pub fn text_files<'a>(dir: &'a Dir, ext: &str) -> io::Result<Vec<(String, &'a str)>> {
    let mut files = Vec::new();
    for file in dir.files() {
        if file.path().extension().is_some_and(|e| e == ext) {
            let name = file.path().file_name().unwrap_or_default().to_string_lossy().into_owned();
            let text = file.contents_utf8().ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, format!("{name} is not UTF-8"))
            })?;
            files.push((name, text));
        }
    }
    files.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(files)
}

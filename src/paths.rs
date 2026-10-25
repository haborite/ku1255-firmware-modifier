//! Writable locations: the per-user data directory of the app.
//!
//! - Windows: `%LOCALAPPDATA%\ku1255-firmware-modifier`
//! - macOS:   `~/Library/Application Support/ku1255-firmware-modifier`
//! - Linux:   `~/.local/share/ku1255-firmware-modifier`
//!
//! On Windows a `data` folder next to the exe (shipped in the portable zip)
//! takes its place, so the app leaves nothing in the user profile.
//! `KU1255_DATA_DIR` overrides both (e.g. to work on firmware files in the repository).

use std::fs;
use std::io;
use std::path::PathBuf;

use crate::resources;

const APP_DIR: &str = "ku1255-firmware-modifier";

pub fn data_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os("KU1255_DATA_DIR") {
        return PathBuf::from(dir);
    }
    #[cfg(target_os = "windows")]
    if let Some(dir) = portable_data_dir() {
        return dir;
    }
    dirs::data_local_dir()
        .map(|d| d.join(APP_DIR))
        .unwrap_or_else(|| PathBuf::from("."))
}

/// `data` next to the exe, if it exists.
#[cfg(target_os = "windows")]
fn portable_data_dir() -> Option<PathBuf> {
    let dir = std::env::current_exe().ok()?.parent()?.join("data");
    dir.is_dir().then_some(dir)
}

/// Downloaded official installer and intermediate firmware files.
pub fn firmware_dir() -> io::Result<PathBuf> {
    let dir = data_dir().join("firmware");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Custom firmware template, used instead of the bundled `template/diff.json`
/// when `diff.json` exists here (`comments.txt` falls back to the bundled one).
pub fn custom_template_dir() -> PathBuf {
    data_dir().join("template")
}

/// Example configs, copied from the bundled ones on first use.
/// Existing files are left untouched so users can edit them.
pub fn examples_dir() -> io::Result<PathBuf> {
    let dir = data_dir().join("examples");
    fs::create_dir_all(&dir)?;
    for file in resources::EXAMPLES.files() {
        let target = dir.join(file.path());
        if !target.exists() {
            fs::write(target, file.contents())?;
        }
    }
    Ok(dir)
}

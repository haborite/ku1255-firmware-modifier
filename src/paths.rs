//! Writable locations: the per-user data directory of the app.
//!
//! - Windows: `%LOCALAPPDATA%\ku1255-firmware-modifier`
//! - macOS:   `~/Library/Application Support/ku1255-firmware-modifier`
//! - Linux:   `~/.local/share/ku1255-firmware-modifier`
//!
//! `KU1255_DATA_DIR` overrides it (e.g. to work on firmware files in the repository).

use std::fs;
use std::io;
use std::path::PathBuf;

use crate::resources;

const APP_DIR: &str = "ku1255-firmware-modifier";

pub fn data_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os("KU1255_DATA_DIR") {
        return PathBuf::from(dir);
    }
    dirs::data_local_dir()
        .map(|d| d.join(APP_DIR))
        .unwrap_or_else(|| PathBuf::from("."))
}

/// Downloaded official installer and intermediate firmware files.
pub fn firmware_dir() -> io::Result<PathBuf> {
    let dir = data_dir().join("firmware");
    fs::create_dir_all(&dir)?;
    Ok(dir)
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

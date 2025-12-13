use std::fs::{self, File};
use std::io::Write;
use dioxus::prelude::*;
use std::path::Path;

// Constants (fixed)
pub const SN8_OFFSET: usize = 472208; // 0x73490
pub const SN8_SIZE: usize = 24576;    // 24 KB
pub const XOR_KEY: u8 = 0x5A;         // XOR key
const ORG_INSTALLER_PATH: &str = "firmware/tp_compact_usb_kb_with_trackpoint_fw.exe";

// Load or download Lenovo official firmware installer
pub async fn load_or_download_firmware_installer(exe_url_cloned: &str) -> Vec<u8>  {
    let firmware_path = Path::new(ORG_INSTALLER_PATH);
    if firmware_path.exists() {
        println!("Firmware found at {}. Loading from disk...", ORG_INSTALLER_PATH);
        return fs::read(firmware_path).unwrap_or_else(|err| {
            eprintln!("Error reading firmware: {}", err);
            vec![]
        });
    }
    println!("Firmware not found. Downloading from {}...", exe_url_cloned);
    match reqwest::get(exe_url_cloned).await {
        Ok(resp) => match resp.bytes().await {
            Ok(bytes) => {
                if let Err(err) = fs::File::create(firmware_path)
                    .and_then(|mut file| file.write_all(&bytes))
                {
                    eprintln!("Failed to save firmware to {}: {}", ORG_INSTALLER_PATH, err);
                } else {
                    println!("Firmware downloaded and saved to {}", ORG_INSTALLER_PATH);
                }
                bytes.to_vec()
            }
            Err(err) => {
                eprintln!("Failed to read response body: {}", err);
                vec![]
            }
        },
        Err(err) => {
            eprintln!("Failed to download firmware: {}", err);
            vec![]
        }
    }
}

/// Extract and save decrypted firmware to a file.
pub fn extract_fw_from_installer_to_file(
    installer: &[u8],
    out_path: &str,
) -> Result<(), String> {
    let decrypted = extract_fw_from_installer_to_vec(installer)?;
    write_binary(out_path, &decrypted)
}

/// Extract and decrypt SN8 firmware from installer binary.
pub fn extract_fw_from_installer_to_vec(
    installer: &[u8],
) -> Result<Vec<u8>, String> {
    if installer.len() < SN8_OFFSET + SN8_SIZE {
        return Err(format!(
            "Installer too small: need {} bytes, have {} bytes",
            SN8_OFFSET + SN8_SIZE,
            installer.len()
        ));
    }

    let decrypted: Vec<u8> = installer[SN8_OFFSET .. SN8_OFFSET + SN8_SIZE]
        .iter()
        .map(|b| b ^ XOR_KEY)
        .collect();

    Ok(decrypted)
}

/// Write raw bytes to a file.
fn write_binary(path: &str, data: &[u8]) -> Result<(), String> {
    let mut f = File::create(path)
        .map_err(|e| format!("Failed to create {}: {}", path, e))?;
    f.write_all(data)
        .map_err(|e| format!("Failed to write {}: {}", path, e))?;
    Ok(())
}

// Build a new installer binary with XOR-encrypted firmware inserted.
//
// Equivalent to the Python script:
// - Take the first SN8_SIZE bytes of fw_plain
// - XOR with XOR_KEY

/*
pub fn build_installer_with_fw(
    fw_plain: &[u8],
    original_installer: &[u8],
) -> Result<Vec<u8>, String> {
    if fw_plain.len() < SN8_SIZE {
        return Err(format!(
            "fw_plain too small: need {} bytes, have {} bytes",
            SN8_SIZE,
            fw_plain.len()
        ));
    }

    if original_installer.len() < SN8_OFFSET + SN8_SIZE {
        return Err(format!(
            "Installer too small: need {} bytes, have {} bytes",
            SN8_OFFSET + SN8_SIZE,
            original_installer.len()
        ));
    }

    let mut modified = original_installer.to_vec();

    let dst = &mut modified[SN8_OFFSET .. SN8_OFFSET + SN8_SIZE];
    let src = &fw_plain[..SN8_SIZE];

    for (d, s) in dst.iter_mut().zip(src.iter()) {
        *d = *s ^ XOR_KEY;
    }

    Ok(modified)
}
*/
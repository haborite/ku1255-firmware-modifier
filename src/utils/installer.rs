use std::fs::File;
use std::io::Write;

// Constants (fixed)
pub const SN8_OFFSET: usize = 472208; // 0x73490
pub const SN8_SIZE: usize = 24576;    // 24 KB
pub const XOR_KEY: u8 = 0x5A;         // XOR key


/// Write raw bytes to a file.
pub fn write_binary(path: &str, data: &[u8]) -> Result<(), String> {
    let mut f = File::create(path)
        .map_err(|e| format!("Failed to create {}: {}", path, e))?;
    f.write_all(data)
        .map_err(|e| format!("Failed to write {}: {}", path, e))?;
    Ok(())
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

/// Extract and save decrypted firmware to a file.
pub fn extract_fw_from_installer_to_file(
    installer: &[u8],
    out_path: &str,
) -> Result<(), String> {
    let decrypted = extract_fw_from_installer_to_vec(installer)?;
    write_binary(out_path, &decrypted)
}

/// Build a new installer binary with XOR-encrypted firmware inserted.
///
/// Equivalent to the Python script:
/// - Take the first SN8_SIZE bytes of fw_plain
/// - XOR with XOR_KEY
/// - Overwrite installer[SN8_OFFSET .. SN8_OFFSET + SN8_SIZE]
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

// Build installer and save it to a file.
/*
pub fn build_installer_with_fw_to_file(
    fw_plain: &[u8],
    original_installer: &[u8],
    out_path: &str,
) -> Result<(), String> {
    let modified = build_installer_with_fw(fw_plain, original_installer)?;
    if let Err(err) = File::create(out_path)
        .and_then(|mut file| file.write_all(&modified))
    {
        eprintln!("Failed to save modified firmware installer to {}: {}", out_path, err);
        return Err(format!("Failed to save modified firmware installer: {}", err));
    }
    println!("Modified firmware installer successfully saved to {}", out_path);
    Ok(())
}

*/
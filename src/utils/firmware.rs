use dioxus::prelude::{Signal, Resource, ReadableExt, WritableExt};
use crate::models::Config;

use crate::utils::diff::apply_diff_files;
use crate::utils::format::format_asm_file;
use crate::utils::commands::{run_dissn8, run_assn8, run_flashsn8_gui};
use crate::utils::lenovo_fw_installer::extract_fw_from_installer_to_file;
use crate::utils::mod_asm::modify_asm_file;

// Firmware
const ORG_BIN_PATH: &str = "firmware/fw_org.bin";
const MOD_BIN_PATH: &str = "firmware/fw_mod.bin";
const ORG_ASM_PATH: &str = "firmware/fw_org.asm";
const FMT_ASM_PATH: &str = "firmware/fw_fmt.asm";
const TMP_ASM_PATH: &str = "firmware/fw_tmp.asm";
const MOD_ASM_PATH: &str = "firmware/fw_mod.asm";
const DIFF_PATH: &str = "template/diff.json";
const COMMENTS_PATH: &str = "template/comments.txt";


pub fn build_and_install_mod_fw (
    config: &Config,
    fw_installer_future: Resource<Vec<u8>>,
    error_msg: &mut Signal<Option<String>>
) {
    if let Some(msg) = validate_mod_key_position(config) {
        error_msg.set(Some(msg));
        return;
    }

    build_mod_fw(config, fw_installer_future).unwrap_or_else(|err| {
        error_msg.set(Some(format!("Failed to build modified firmware: {}", err)));
        return;
    });

    install_mod_fw(error_msg);
}


fn validate_mod_key_position(config: &Config) -> Option<String> {
    for (k, v) in &config.layer0 {
        if *v == 231 {
            if config.layer1.get(&k) != Some(&231) {
                return Some("The 'Mod' key position must be same on the Main and 2nd layers.".into());
            }
        }
    }
    None
}


fn build_mod_fw(
    config: &Config,
    fw_installer_future: Resource<Vec<u8>>
) -> Result<(), String> {

    let Some(original_binary) = &*fw_installer_future.read_unchecked() else {
        return Err("Firmware binary not loaded.".into());
    };

    extract_fw_from_installer_to_file(original_binary, ORG_BIN_PATH)?;
    run_dissn8(ORG_BIN_PATH, ORG_ASM_PATH)
        .map_err(|e| format!("dissn8 failed: {}", e))?;
    format_asm_file(ORG_ASM_PATH, FMT_ASM_PATH)
        .map_err(|e| format!("Failed to format ASM: {}", e))?;
    apply_diff_files(FMT_ASM_PATH, DIFF_PATH, COMMENTS_PATH, TMP_ASM_PATH)
        .map_err(|e| format!("Failed to apply diff: {}", e))?;
    modify_asm_file(config, TMP_ASM_PATH, MOD_ASM_PATH)
        .map_err(|e| format!("Failed to modify ASM: {}", e))?;
    run_assn8(MOD_ASM_PATH, MOD_BIN_PATH)
        .map_err(|e| format!("assn8 failed: {}", e))?;

    Ok(())
}


fn install_mod_fw(error_msg: &mut Signal<Option<String>> ) {
    run_flashsn8_gui(MOD_BIN_PATH, ORG_BIN_PATH).unwrap_or_else(|err| {
        error_msg.set(Some(format!("Failed to launch flashsn8: {}", err)));
        return;
    });
}


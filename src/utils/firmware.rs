use std::collections::{HashMap, BTreeMap};
use dioxus::prelude::{Signal, Resource, ReadableExt, WritableExt};
use std::fs;
use std::path::{Path};
use std::io;
use std::io::{Write};
use crate::models::MacroKey;

use crate::utils::template::render_template_file;
use crate::utils::diff::apply_diff_files;
use crate::utils::format::format_asm_file;
use crate::utils::commands::{run_dissn8, run_assn8, run_flashsn8_gui};
use crate::utils::installer::{
    extract_fw_from_installer_to_file,
};

const ORG_INSTALLER_PATH: &str = "firmware/tp_compact_usb_kb_with_trackpoint_fw.exe";

const ORG_BIN_PATH: &str = "firmware/fw_org.bin";
const MOD_BIN_PATH: &str = "firmware/fw_mod.bin";

const ORG_ASM_PATH: &str = "firmware/fw_org.asm";
const FMT_ASM_PATH: &str = "firmware/fw_fmt.asm";
const TMP_ASM_PATH: &str = "firmware/fw_tmp.asm";
const MOD_ASM_PATH: &str = "firmware/fw_mod.asm";

const DIFF_PATH: &str = "template/diff.json";
const COMMENTS_PATH: &str = "template/comments.txt";


fn validate_mod_key_position(
    layout0: Signal<BTreeMap<u8, Option<u8>>>,
    layout1: Signal<BTreeMap<u8, Option<u8>>>,
) -> Option<String> {
    for (k, v) in layout0() {
        if v == Some(231) {
            if layout1().get(&k) != Some(&Some(231)) {
                return Some("The 'Mod' key position must be same on the Main and 2nd layers.".into());
            }
        }
    }
    None
}


fn build_mod_fw(
    firmware_future: Resource<Vec<u8>>,
    layout0: Signal<BTreeMap<u8, Option<u8>>>,
    layout1: Signal<BTreeMap<u8, Option<u8>>>,
    fn_id: Signal<u8>,
    tp_sensitivity: Signal<u32>,
    macro_key_map: Signal<BTreeMap<u8, MacroKey>>,
    media_key_map: Signal<BTreeMap<u8, u16>>,
    enable_middle_click: Signal<bool>,
) -> Result<(), String> {

    let Some(original_binary) = &*firmware_future.read_unchecked() else {
        return Err("Firmware binary not loaded.".into());
    };

    let _r = extract_fw_from_installer_to_file(original_binary, ORG_BIN_PATH)?;
    let _r = run_dissn8(ORG_BIN_PATH, ORG_ASM_PATH)
        .map_err(|e| format!("dissn8 failed: {}", e))?;
    let _r = format_asm_file(ORG_ASM_PATH, FMT_ASM_PATH)
        .map_err(|e| format!("Failed to format ASM: {}", e))?;
    let _r = apply_diff_files(FMT_ASM_PATH, DIFF_PATH, COMMENTS_PATH, TMP_ASM_PATH)
        .map_err(|e| format!("Failed to apply diff: {}", e))?;
    let _r = modify_asm_file(TMP_ASM_PATH, MOD_ASM_PATH, &layout0(), &layout1(), fn_id(), tp_sensitivity(), &macro_key_map(), &media_key_map(), enable_middle_click())
        .map_err(|e| format!("Failed to modify ASM: {}", e))?;
    let _r = run_assn8(MOD_ASM_PATH, MOD_BIN_PATH)
        .map_err(|e| format!("assn8 failed: {}", e))?;

    Ok(())
}

pub fn install_firmware_by_flashsn8(
    id_layout_l0: Signal<BTreeMap<u8, Option<u8>>>,
    id_layout_l1: Signal<BTreeMap<u8, Option<u8>>>,
    firmware_future: Resource<Vec<u8>>,
    fn_id: Signal<u8>,
    tp_sensitivity: Signal<u32>,
    macro_key_map: Signal<BTreeMap<u8, MacroKey>>,
    media_key_map: Signal<BTreeMap<u8, u16>>,
    enable_middle_click: Signal<bool>,
    error_msg: &mut Signal<Option<String>>,    
) {
    if let Some(msg) = validate_mod_key_position(id_layout_l0, id_layout_l1) {
        error_msg.set(Some(msg));
        return;
    }

    let _r = build_mod_fw(firmware_future, id_layout_l0, id_layout_l1, fn_id, tp_sensitivity, macro_key_map, media_key_map, enable_middle_click).unwrap_or_else(|err| {
        error_msg.set(Some(format!("Failed to build modified firmware: {}", err)));
        return;
    });

    run_flashsn8_gui(MOD_BIN_PATH, ORG_BIN_PATH).unwrap_or_else(|err| {
        error_msg.set(Some(format!("Failed to launch flashsn8: {}", err)));
        return;
    });

}

pub async fn load_or_download_firmware(exe_url_cloned: &str) -> Vec<u8>  {
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


fn modify_asm_file(
    in_path: &str,
    out_path: &str,
    layout0: &BTreeMap<u8, Option<u8>>,
    layout1: &BTreeMap<u8, Option<u8>>,
    fn_id: u8,
    tp_sensitivity: u32,
    macro_key_map: &BTreeMap<u8, MacroKey>,
    media_key_map: &BTreeMap<u8, u16>,
    enable_middle_click: bool,
) -> io::Result<()> {

    // Prepare s_values and e_choices
    let mut s_values = HashMap::new();
    let mut e_choices = HashMap::new();

    // Replace Function key ID
    s_values.insert("fn_id".to_string(), format!("{:02x}", fn_id));

    // Key layout mapping
    let mut map1: BTreeMap<u8, Option<u8>> = BTreeMap::new();
    for (pos, code) in layout1.iter() {
        map1.insert(*pos, *code);
    }
    for (pos0, code0) in layout0.iter() {
        if let Some(code1) = map1.get(pos0) {
            s_values.insert(
                format!("{:02x}", pos0),
                format!("{:02x}{:02x}", code1.unwrap_or(0), code0.unwrap_or(0)),
            );
        } else {
            eprintln!("Warning: pos {pos0} not found in layout1");
        }
    }

    // Trackpoint accelaration switches    
    let accel_switches: [u8; 4] = match tp_sensitivity {
        2 => [1, 0, 0, 0],
        3 => [1, 1, 0, 0],
        4 => [1, 1, 1, 0],
        5 => [1, 1, 1, 1],
        _ => [0, 0, 0, 0],
    };
    for (i, accel_switch) in accel_switches.into_iter().enumerate() {
        e_choices.insert(format!("tp_accel_{}", i), accel_switch as usize);
    }

    // Macro Key
    for (trigger_key_id, macro_key) in macro_key_map.iter() {
        let media_key_id = macro_key.key_id;
        let mut mod_key_bits = 0;
        if macro_key.left_ctrl {mod_key_bits += 1};
        if macro_key.left_shift {mod_key_bits += 2};
        if macro_key.left_alt {mod_key_bits += 4};
        if macro_key.left_gui {mod_key_bits += 8};
        if macro_key.right_ctrl {mod_key_bits += 16};
        if macro_key.right_shift {mod_key_bits += 32};
        if macro_key.right_alt {mod_key_bits += 64};
        if macro_key.right_gui {mod_key_bits += 128};
        s_values.insert(format!("macro_{:02x}", trigger_key_id), format!("{:02x}{:02x}", mod_key_bits, media_key_id));
    }

    // Media Key
    for (trigger_key_id, media_key_id) in media_key_map.iter() {
        s_values.insert(format!("media_{:02x}", trigger_key_id), format!("{:04x}", media_key_id));
    }

    // Enable middle click
    e_choices.insert("mclick".to_string(), if enable_middle_click {1} else {0});

    let _r = render_template_file(in_path, out_path, &s_values, &e_choices)?;
    Ok(())
}

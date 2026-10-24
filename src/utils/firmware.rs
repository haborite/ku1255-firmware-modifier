use std::collections::{HashMap, BTreeMap};
use dioxus::prelude::{Signal, ReadSignal, Resource, ReadableExt};
use std::borrow::Cow;
use std::fs;
use std::path::PathBuf;
use crate::models::{Board, MacroKey, TrackPointSpeedSettings};
use crate::{paths, resources};

use ku1255_core::template::render_template;
use ku1255_core::diff::apply_diff;
use ku1255_core::format::format_asm;
use ku1255_core::installer::extract_fw_from_installer_to_vec;
use ku1255_core::flash::{check_image, CheckedImage};
use ku1255_core::sn8;

const INSTALLER_NAME: &str = "tp_compact_usb_kb_with_trackpoint_fw.exe";

/// Path of the custom `diff.json`, if the user has placed one.
pub fn custom_diff_path() -> Option<PathBuf> {
    let path = paths::custom_template_dir().join("diff.json");
    path.is_file().then_some(path)
}

/// `(diff.json, comments.txt)`: the custom ones in the data directory if present,
/// otherwise the bundled ones.
fn template_files() -> Result<(Cow<'static, str>, Cow<'static, str>), String> {
    let Some(diff_path) = custom_diff_path() else {
        return Ok((Cow::Borrowed(resources::DIFF_JSON), Cow::Borrowed(resources::COMMENTS)));
    };
    let read = |path: PathBuf| {
        fs::read_to_string(&path).map_err(|e| format!("Failed to read {}: {}", path.display(), e))
    };
    let diff = read(diff_path)?;
    let comments_path = paths::custom_template_dir().join("comments.txt");
    let comments = if comments_path.is_file() {
        Cow::Owned(read(comments_path)?)
    } else {
        Cow::Borrowed(resources::COMMENTS)
    };
    Ok((Cow::Owned(diff), comments))
}


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
    single_mod_key_id: Signal<u8>,
    trackpoint_speed_settings: Signal<TrackPointSpeedSettings>,
    macro_key_map: Signal<BTreeMap<u8, MacroKey>>,
    media_key_map: Signal<BTreeMap<u8, u16>>,
    enable_middle_click: Signal<bool>,
    selected_board: ReadSignal<Board>,
) -> Result<CheckedImage, String> {

    let Some(original_binary) = &*firmware_future.read_unchecked() else {
        return Err("Firmware binary not loaded.".into());
    };

    // Intermediate files are kept in the data directory for firmware development.
    let dir = paths::firmware_dir().map_err(|e| format!("Failed to create firmware directory: {}", e))?;
    let save = |name: &str, data: &[u8]| {
        let path = dir.join(name);
        fs::write(&path, data).map_err(|e| format!("Failed to write {}: {}", path.display(), e))
    };

    let original_fw = extract_fw_from_installer_to_vec(original_binary)?;
    save("fw_org.bin", &original_fw)?;
    let original_asm = sn8::disassemble(&original_fw, &sn8::Chip::sn8f2288())
        .map_err(|e| format!("Disassembly failed: {}", e))?;
    save("fw_org.asm", original_asm.as_bytes())?;
    let formatted_asm = format_asm(&original_asm);
    save("fw_fmt.asm", formatted_asm.as_bytes())?;
    let (diff_json, comments) = template_files()?;
    let template_asm = apply_diff(&formatted_asm, &diff_json, &comments)
        .map_err(|e| format!("Failed to apply diff: {}", e))?;
    save("fw_tmp.asm", template_asm.as_bytes())?;
    let modified_asm = modify_asm(
        &template_asm,
        &layout0(), 
        &layout1(), 
        fn_id(),
        single_mod_key_id(),
        &trackpoint_speed_settings(), 
        &macro_key_map(), 
        &media_key_map(), 
        enable_middle_click(),
        &selected_board(),
    )
        .map_err(|e| format!("Failed to modify ASM: {}", e))?;
    save("fw_mod.asm", modified_asm.as_bytes())?;
    let modified_fw = sn8::assemble(&modified_asm)
        .map_err(|e| format!("Assembly failed: {}", e))?;
    save("fw_mod.bin", &modified_fw)?;

    check_image(&modified_fw, &original_fw)
}

/// Build the modified firmware and run the offline safety checks.
/// On success the image is ready to be flashed.
pub fn prepare_firmware(
    id_layout_l0: Signal<BTreeMap<u8, Option<u8>>>,
    id_layout_l1: Signal<BTreeMap<u8, Option<u8>>>,
    firmware_future: Resource<Vec<u8>>,
    fn_id: Signal<u8>,
    single_mod_key_id: Signal<u8>,
    trackpoint_speed_settings: Signal<TrackPointSpeedSettings>,
    macro_key_map: Signal<BTreeMap<u8, MacroKey>>,
    media_key_map: Signal<BTreeMap<u8, u16>>,
    enable_middle_click: Signal<bool>,
    selected_board: ReadSignal<Board>,
) -> Result<CheckedImage, String> {
    if let Some(msg) = validate_mod_key_position(id_layout_l0, id_layout_l1) {
        return Err(msg);
    }
    build_mod_fw(
        firmware_future,
        id_layout_l0,
        id_layout_l1,
        fn_id,
        single_mod_key_id,
        trackpoint_speed_settings,
        macro_key_map,
        media_key_map,
        enable_middle_click,
        selected_board,
    )
    .map_err(|err| format!("Failed to build modified firmware: {}", err))
}

pub async fn load_or_download_firmware(exe_url_cloned: &str) -> Vec<u8>  {
    let firmware_path = match paths::firmware_dir() {
        Ok(dir) => dir.join(INSTALLER_NAME),
        Err(err) => {
            eprintln!("Failed to create firmware directory: {}", err);
            return vec![];
        }
    };
    if firmware_path.exists() {
        println!("Firmware found at {}. Loading from disk...", firmware_path.display());
        return fs::read(&firmware_path).unwrap_or_else(|err| {
            eprintln!("Error reading firmware: {}", err);
            vec![]
        });
    }
    println!("Firmware not found. Downloading from {}...", exe_url_cloned);
    let response = match reqwest::get(exe_url_cloned).await.and_then(|r| r.error_for_status()) {
        Ok(resp) => resp,
        Err(err) => {
            eprintln!("Failed to download firmware: {}", err);
            return vec![];
        }
    };
    match response.bytes().await {
        Ok(bytes) => {
            // Only cache what looks like the installer, so a bad download is retried next time.
            if extract_fw_from_installer_to_vec(&bytes).is_ok() {
                match fs::write(&firmware_path, &bytes) {
                    Ok(()) => println!("Firmware downloaded and saved to {}", firmware_path.display()),
                    Err(err) => eprintln!("Failed to save firmware to {}: {}", firmware_path.display(), err),
                }
            }
            bytes.to_vec()
        }
        Err(err) => {
            eprintln!("Failed to read response body: {}", err);
            vec![]
        }
    }
}


fn modify_asm(
    template: &str,
    layout0: &BTreeMap<u8, Option<u8>>,
    layout1: &BTreeMap<u8, Option<u8>>,
    fn_id: u8,
    single_mod_key_id: u8,
    trackpoint_speed_settings: &TrackPointSpeedSettings,
    macro_key_map: &BTreeMap<u8, MacroKey>,
    media_key_map: &BTreeMap<u8, u16>,
    enable_middle_click: bool,
    selected_board: &Board,
) -> Result<String, String> {

    // Prepare s_values and e_choices
    let mut s_values = HashMap::new();
    let mut e_choices = HashMap::new();

    // Replace Function key ID
    s_values.insert("fn_id".to_string(), format!("{:02x}", fn_id));

    // Replace Key ID when the 'Mod' key is singly pressed
    s_values.insert("single_mod_key_id".to_string(), format!("{:02x}", single_mod_key_id));
    e_choices.insert(
        "single_mod_key_id_enabled".to_string(), 
        if single_mod_key_id == 0 {0} else {1}
    );

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

    // Trackpoint speed settings
    for (i, coeffs) in trackpoint_speed_settings.coeffs.iter().enumerate() {
        for (j, coeff) in coeffs.iter().enumerate() {
            let accelaration_value = coeff - j as i8;
            let accelaration_value_shifted: u8 = if accelaration_value < 0 {
                let a = accelaration_value as i16 + 256;
                a as u8
            } else {
                accelaration_value as u8
            };
            s_values.insert(format!("tp{:01}_{:02}", i+1, j), format!("0x00{:02x}", accelaration_value_shifted));
        }
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

    // Set keymask
    let mut kms: [u8; 16] = [0; 16];
    for &addr in selected_board.map_address.iter().flatten().flatten() {
        let row = (addr >> 4) as usize;
        let col = (addr & 0x07) as u32;
        kms[row] |= 1u8 << col;
    }
    for (i, km) in kms.into_iter().enumerate() {
        s_values.insert(format!("km_{:01x}", i), format!("00{:02x}", km));
    }

    render_template(template, &s_values, &e_choices).map_err(|e| e.to_string())
}

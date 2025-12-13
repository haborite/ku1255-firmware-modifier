use std::collections::{HashMap, BTreeMap};
use std::io;
use crate::utils::template::render_template_file;
use crate::models::Config;

pub fn modify_asm_file(
    config: &Config,
    in_path: &str,
    out_path: &str,
) -> io::Result<()> {

    // Prepare s_values and e_choices
    let mut s_values = HashMap::new();
    let mut e_choices = HashMap::new();

    // Replace Function key ID
    s_values.insert("fn_id".to_string(), format!("{:02x}", config.fn_id));

    // Key layout mapping
    let mut map1: BTreeMap<u32, u8> = BTreeMap::new();
    for (pos, code) in config.layer1.iter() {
        map1.insert(*pos, *code);
    }
    for (pos0, code0) in config.layer0.iter() {
        if let Some(code1) = map1.get(pos0) {
            s_values.insert(
                format!("{:06}", pos0),
                format!("{:02x}{:02x}", code1, code0),
            );
        } else {
            eprintln!("Warning: pos {pos0} not found in layout1");
        }
    }

    // Trackpoint accelaration switches    
    let accel_switches: [u8; 4] = match config.tp_sensitivity {
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
    for (trigger_key_id, macro_key) in config.macro_key_map.iter() {
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
    for (trigger_key_id, media_key_id) in config.media_key_map.iter() {
        s_values.insert(format!("media_{:02x}", trigger_key_id), format!("{:04x}", media_key_id));
    }

    // Enable middle click
    e_choices.insert("mclick".to_string(), if config.enable_middle_click {1} else {0});

    let _r = render_template_file(in_path, out_path, &s_values, &e_choices)?;
    Ok(())
}

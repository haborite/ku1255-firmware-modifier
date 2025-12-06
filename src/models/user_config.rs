use serde::{Serialize, Deserialize};
use serde_json::{to_writer_pretty, from_reader};
use std::{collections::HashMap};
use std::path::Path;
use std::fs::File;
use std::io::{self, BufReader, BufWriter};
use crate::models::{PhysicalLayout, LogicalLayout, KeyboardSpec};

const GENERAL_SETTING_PATH: &str = "settings/general_setting.csv";

// Version of the config file format
const CURRENT_CONFIG_VERSION: u32 = 2;

// Default values
const DEFAULT_FN_ID: u8 = 0xaf;
const DEFAULT_TP_SENSITIVITY: u32 = 1;

const MACRO_KEY_TRIGGER_IDS: [u8;24] = [
    0xE8, 0xE9, 0xEA, 0xEB, 0xEC, 0xED, 0xEE, 0xEF,
    0xF0, 0xF1, 0xF2, 0xF3, 0xF4, 0xF5, 0xF6, 0xF7,
    0xF8, 0xF9, 0xFA, 0xFB, 0xFC, 0xFD, 0xFE, 0xFF,
];
const MEDIA_KEY_TRIGGER_IDS: [u8;11] = [
    0xD5, 0xD6, 0xD7, 0xD8, 0xD9, 0xDA, 0xDB, 0xDC,
    0xDD, 0xDE, 0xDF,
];

/*
fn default_fn_id() -> u8 { 0xaf }

fn default_macro_key_map() -> HashMap<u8, MacroKey> {
    let mut map = HashMap::new();
    for &id in MACRO_KEY_TRIGGER_IDS.iter() {
        map.insert(id, MacroKey::new());
    }
    map
}

fn default_media_key_map() -> HashMap<u8, u16> {
    let mut map = HashMap::new();
    for &id in MEDIA_KEY_TRIGGER_IDS.iter() {
        map.insert(id, 0);
    }
    map
}
*/

// All user-setting Configuration
// #[serde(default)]
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
#[serde(default)]
pub struct UserConfig {
    pub config_version: u32,
    pub physical_layout_name: String,
    pub logical_layout_name: String,
    pub layer0: HashMap<u32, u8>,
    pub layer1: HashMap<u32, u8>,
    pub fn_id: u8,
    pub tp_sensitivity: u32,
    pub macro_key_map: HashMap<u8, MacroKey>,
    pub media_key_map: HashMap<u8, u16>,
}


impl Default for UserConfig {
    fn default() -> Self {
        UserConfig {
            config_version: CURRENT_CONFIG_VERSION,
            physical_layout_name: String::new(),
            logical_layout_name: String::new(),
            layer0: {
                let general_setting_path = Path::new(GENERAL_SETTING_PATH);
                if let Ok((id_map, _un)) = KeyboardSpec::load_general_settings(general_setting_path){
                    id_map
                } else {
                    HashMap::new()
                } 
            },
            layer1: {
                let general_setting_path = Path::new(GENERAL_SETTING_PATH);
                if let Ok((id_map, _un)) = KeyboardSpec::load_general_settings(general_setting_path){
                    id_map
                } else {
                    HashMap::new()
                } 
            },
            fn_id: DEFAULT_FN_ID,
            tp_sensitivity: DEFAULT_TP_SENSITIVITY,
            macro_key_map: {
                let mut map = HashMap::new();
                for &id in MACRO_KEY_TRIGGER_IDS.iter() {
                    map.insert(id, MacroKey::new());
                }
                map
            },
            media_key_map: {
                let mut map = HashMap::new();
                for &id in MEDIA_KEY_TRIGGER_IDS.iter() {
                    map.insert(id, 0);
                }
                map
            },
        }
    }
}


impl UserConfig {

    pub fn new() -> UserConfig {
        UserConfig::default()
    }

    pub fn load_from_file(filepath_str: &str) -> io::Result<UserConfig> {
        let filepath = Path::new(filepath_str);
        let file = File::open(filepath)?;
        let reader = BufReader::new(file);
        let loaded_config: UserConfig = from_reader(reader)?;
        println!("Load config from file: {:?}", loaded_config);
        Ok(loaded_config)
    }

    pub fn update_from_file(&mut self, filepath: &Path) -> io::Result<()> {
        let file = File::open(filepath)?;
        let reader = BufReader::new(file);
        let loaded_config: UserConfig = from_reader(reader)?;
        *self = loaded_config;
        Ok(())
    }

    pub fn save_to_file(&self, filepath: &Path) -> io::Result<()> {
        let file = File::create(filepath)?;
        let writer = BufWriter::new(file);
        to_writer_pretty(writer, &self)?;
        Ok(())
    }

    pub fn update_physical_layout_name(&mut self, new_name: &str) {
        self.physical_layout_name = new_name.to_string();
    }

    pub fn update_logical_layout_name(&mut self, new_name: &str) {
        self.logical_layout_name = new_name.to_string();
    }

    pub fn get_physical_layout(&self, keyboard_spec: &KeyboardSpec) -> PhysicalLayout {
        keyboard_spec.avail_physical_layouts.iter().find(|l| l.name == self.physical_layout_name)
            .unwrap_or(keyboard_spec.avail_physical_layouts.get(0).unwrap()).clone()
    }

    pub fn get_logical_layout(&self, keyboard_spec: &KeyboardSpec) -> LogicalLayout {
        keyboard_spec.avail_logical_layouts.iter().find(|l| l.name == self.logical_layout_name)
            .unwrap_or(keyboard_spec.avail_logical_layouts.get(0).unwrap()).clone()
    }

    pub fn get_id_layout(&self, layer_number: u8) -> &HashMap<u32, u8> {
        if layer_number == 0 { &self.layer0 } else { &self.layer1 }
    }

    pub fn update_layer(&mut self, layer_number: u8, address: u32, new_id: u8) {
        if layer_number == 0 {
            self.layer0.insert(address, new_id);
        } else {
            self.layer1.insert(address, new_id);
        }
    }

    pub fn copy_layer0_to_layer1(&mut self) {
        self.layer1 = self.layer0.clone();
    }

    pub fn update_tp_sensitivity(&mut self, new_tp_sensitivity: u32) {
        self.tp_sensitivity = new_tp_sensitivity;
    }

    pub fn update_fn_id(&mut self, new_fn_id: u8) {
        self.fn_id = new_fn_id;
    }

    pub fn get_media_key_id(&self, trigger_key_id: u8) -> u16 {
        *self.media_key_map.get(&trigger_key_id).unwrap_or(&0)
    }

    pub fn update_media_key_map(&mut self, trigger_key_id: u8, new_media_key_id: u16) {
        self.media_key_map.insert(
            trigger_key_id,
            new_media_key_id as u16,
        );
    }

    pub fn get_macro_key(&self, trigger_key_id: u8) -> MacroKey {
        self.macro_key_map.get(&trigger_key_id).unwrap_or(&MacroKey::new()).clone()
    }

    pub fn update_left_ctrl(&mut self, trigger_key_id: u8, left_ctrl: bool) {
        self.macro_key_map.get_mut(&trigger_key_id).unwrap().left_ctrl = left_ctrl;
    }

    pub fn update_right_ctrl(&mut self, trigger_key_id: u8, right_ctrl: bool) {
        self.macro_key_map.get_mut(&trigger_key_id).unwrap().right_ctrl = right_ctrl;
    }

    pub fn update_left_shift(&mut self, trigger_key_id: u8, left_shift: bool) {
        self.macro_key_map.get_mut(&trigger_key_id).unwrap().left_shift = left_shift;
    }

    pub fn update_right_shift(&mut self, trigger_key_id: u8, right_shift: bool) {
        self.macro_key_map.get_mut(&trigger_key_id).unwrap().right_shift = right_shift;
    }

    pub fn update_left_alt(&mut self, trigger_key_id: u8, left_alt: bool) {
        self.macro_key_map.get_mut(&trigger_key_id).unwrap().left_alt = left_alt;
    }

    pub fn update_right_alt(&mut self, trigger_key_id: u8, right_alt: bool) {
        self.macro_key_map.get_mut(&trigger_key_id).unwrap().right_alt = right_alt;
    }

    pub fn update_left_gui(&mut self, trigger_key_id: u8, left_gui: bool) {
        self.macro_key_map.get_mut(&trigger_key_id).unwrap().left_gui = left_gui;
    }

    pub fn update_right_gui(&mut self, trigger_key_id: u8, right_gui: bool) {
        self.macro_key_map.get_mut(&trigger_key_id).unwrap().right_gui = right_gui;
    }

    pub fn update_macro_key_id(&mut self, trigger_key_id: u8, new_key_id: u8) {
        self.macro_key_map.get_mut(&trigger_key_id).unwrap().key_id = new_key_id;        
    }

}


// Combination Key Mode
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct MacroKey {
    pub key_id: u8,
    pub left_ctrl: bool,
    pub left_shift: bool,
    pub left_alt: bool,
    pub left_gui: bool,
    pub right_ctrl: bool,
    pub right_shift: bool,
    pub right_alt: bool,
    pub right_gui: bool,
}

impl MacroKey {
    pub fn new() -> MacroKey {
        MacroKey {
            key_id: 0,
            left_ctrl: false,
            left_shift: false,
            left_alt: false,
            left_gui: false,
            right_ctrl: false,
            right_shift: false,
            right_alt: false,
            right_gui: false,
        }
    }
}









use std::collections::BTreeMap;
use serde::{Serialize, Deserialize};

mod general_setting;
pub use general_setting::*;

// Default values
const DEFAULT_TP_SENSITIVITY: u32 = 1;
const DEFAULT_FN_ID: u8 = 0xaf;

const MACRO_KEY_TRIGGER_IDS: [u8;24] = [
    0xE8, 0xE9, 0xEA, 0xEB, 0xEC, 0xED, 0xEE, 0xEF,
    0xF0, 0xF1, 0xF2, 0xF3, 0xF4, 0xF5, 0xF6, 0xF7,
    0xF8, 0xF9, 0xFA, 0xFB, 0xFC, 0xFD, 0xFE, 0xFF,
];
const MEDIA_KEY_TRIGGER_IDS: [u8;11] = [
    0xD5, 0xD6, 0xD7, 0xD8, 0xD9, 0xDA, 0xDB, 0xDC,
    0xDD, 0xDE, 0xDF,
];

#[derive(Clone, PartialEq, Debug)]
pub struct Board {
    pub board_name: String,
    pub board_label: String,
    pub default_logical_layout_name: String,
    pub map_widths: Vec<Vec<u16>>, 
    pub map_address: Vec<Vec<Option<u32>>>,
}

/*
impl Board {
    pub fn new() -> Board {
        Board {
            board_name: String::new(),
            board_label: String::new(),
            default_logical_layout_name: String::new(),
            map_widths: vec![vec![]],
            map_address: vec![vec![]],
        }
    }
}
*/

#[derive(Clone, PartialEq)]
pub struct LogicalLayout {
    pub layout_name: String,
    pub layout_label: String,
    pub map_key_label: BTreeMap<u8, KeyLabel>,
}

/*
impl LogicalLayout {
    pub fn new() -> LogicalLayout {
        LogicalLayout {
            layout_name: String::new(),
            layout_label: String::new(),
            map_key_label: BTreeMap::new(),
        }
    }
}
*/

#[derive(Clone, PartialEq)]
pub struct KeyLabel {
    pub usage_name: String,
    pub default: String,
    pub shifted: String, 
}

impl KeyLabel {
    pub fn new() -> KeyLabel {
        KeyLabel {
            usage_name: String::new(),
            default: String::new(),
            shifted: String::new(),
        }
    }
}

pub fn default_fn_id() -> u8 { DEFAULT_FN_ID }

pub fn default_tp_sensitivity() -> u32 { DEFAULT_TP_SENSITIVITY }


pub fn default_macro_key_map() -> BTreeMap<u8, MacroKey> {
    MACRO_KEY_TRIGGER_IDS
        .iter()
        .map(|tk| (*tk, MacroKey::new()))
        .collect::<BTreeMap<u8, MacroKey>>()
}

pub fn default_media_key_map() -> BTreeMap<u8, u16> {
    MEDIA_KEY_TRIGGER_IDS
        .iter()
        .map(|tk| (*tk, 0))
        .collect::<BTreeMap<u8, u16>>()
}

#[derive(Serialize, Deserialize, )]
pub struct Config {
    pub config_version: u32,
    pub physical_layout_name: String,
    pub logical_layout_name: String,
    pub layer0: Vec<[u32; 2]>,
    pub layer1: Vec<[u32; 2]>,
    #[serde(default = "default_fn_id")]
    pub fn_id: u8,
    #[serde(default = "default_tp_sensitivity")]
    pub tp_sensitivity: u32,
    #[serde(default = "default_macro_key_map")]
    pub macro_key_map: BTreeMap<u8, MacroKey>, 
    #[serde(default = "default_media_key_map")]
    pub media_key_map: BTreeMap<u8, u16>,
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
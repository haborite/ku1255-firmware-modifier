use std::collections::BTreeMap;
use serde::{Serialize, Deserialize};

mod general_setting;
pub use general_setting::*;

// Default values
const DEFAULT_FN_ID: u8 = 0xaf;
const DEFAULT_SINGLE_MOD_KEY_ID: u8 = 0x00;

const MACRO_KEY_TRIGGER_IDS: [u8;24] = [
    0xE8, 0xE9, 0xEA, 0xEB, 0xEC, 0xED, 0xEE, 0xEF,
    0xF0, 0xF1, 0xF2, 0xF3, 0xF4, 0xF5, 0xF6, 0xF7,
    0xF8, 0xF9, 0xFA, 0xFB, 0xFC, 0xFD, 0xFE, 0xFF,
];
const MEDIA_KEY_TRIGGER_IDS: [u8;11] = [
    0xD5, 0xD6, 0xD7, 0xD8, 0xD9, 0xDA, 0xDB, 0xDC,
    0xDD, 0xDE, 0xDF,
];

const TP_ACCEL_COEFFS: [[i8;43];9] = 
    [
        [
             0,     1,     1,     1,     2,     2,     3,     3,     4,     4,     4,     5,
             5,     6,     6,     7,     7,     8,     8,     9,     9,     9,    10,    10,
            11,    11,    12,    12,    13,    13,    13,    14,    14,    15,    15,    16,
            16,    17,    17,    18,    18,    18,    19,
        ],
        [
             0,     1,     1,     2,     2,     3,     3,     4,     4,     5,     5,     6,
             7,     7,     8,     8,     9,     9,    10,    10,    11,    12,    12,    13,
            13,    14,    14,    15,    15,    16,    16,    17,    18,    18,    19,    19,
            20,    20,    21,    21,    22,    23,    23,
        ],
        [
             0,     1,     2,     2,     3,     4,     4,     5,     6,     7,     7,     8,
             9,    10,    10,    11,    12,    13,    13,    14,    15,    16,    16,    17,
            18,    19,    19,    20,    21,    22,    22,    23,    24,    25,    25,    26,
            27,    28,    28,    29,    30,    31,    32,
        ],
        [
             0,     1,     2,     3,     3,     4,     5,     6,     7,     8,     9,     9,
            10,    11,    12,    13,    14,    14,    15,    16,    17,    18,    19,    20,
            20,    21,    22,    23,    24,    25,    26,    26,    27,    28,    29,    30,
            31,    31,    32,    33,    34,    35,    36,
        ],
        [
             0,     1,     2,     3,     4,     5,     6,     7,     8,     9,    10,    11,
            12,    13,    14,    15,    16,    17,    18,    19,    20,    21,    22,    23,
            24,    25,    26,    27,    28,    29,    30,    31,    32,    33,    34,    35,
            36,    37,    38,    39,    40,    41,    42,
        ],
        [
             0,     1,     3,     4,     5,     6,     8,     9,    10,    11,    13,    14,
            15,    16,    18,    19,    20,    21,    23,    24,    25,    26,    28,    29,
            30,    31,    33,    34,    35,    36,    38,    39,    40,    41,    43,    44,
            45,    46,    48,    49,    50,    51,    53,
        ],
        [
             0,     1,     3,     5,     6,     8,     9,    11,    12,    14,    16,    17,
            19,    20,    22,    23,    25,    26,    28,    29,    31,    33,    34,    36,
            37,    39,    40,    42,    43,    45,    47,    48,    50,    51,    53,    54,
            56,    57,    59,    60,    62,    64,    65,
        ],
        [
             0,     1,     4,     6,     7,     9,    11,    13,    15,    17,    19,    20,
            22,    24,    26,    28,    30,    31,    33,    35,    37,    39,    41,    43,
            44,    46,    48,    50,    52,    54,    56,    57,    59,    61,    63,    65,
            67,    68,    70,    72,    74,    76,    78,
        ],
        [
             0,     1,     4,     6,     8,    10,    12,    14,    16,    18,    21,    23,
            25,    27,    29,    31,    33,    35,    37,    39,    41,    43,    45,    47,
            49,    51,    53,    55,    57,    59,    62,    64,    66,    68,    70,    72,
            74,    76,    78,    80,    82,    84,    86,
        ]
    ];


#[derive(Clone, PartialEq, Debug)]
pub struct Board {
    pub board_name: String,
    pub board_label: String,
    pub default_logical_layout_name: String,
    pub map_widths: Vec<Vec<u16>>, 
    pub map_address: Vec<Vec<Option<u8>>>,
    pub initial_id_map: BTreeMap<u8, Option<u8>>,
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
pub fn default_single_mod_key_id() -> u8 { DEFAULT_SINGLE_MOD_KEY_ID }


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

pub fn default_enable_middle_click() -> bool { false }

pub fn default_tp_accel_coeffs() -> TrackPointSpeedSettings {
    TrackPointSpeedSettings { coeffs: TP_ACCEL_COEFFS.iter().map(|row| row.to_vec()).collect() }
}

#[derive(Serialize, Deserialize, )]
pub struct Config {
    pub config_version: u32,
    pub physical_layout_name: String,
    pub logical_layout_name: String,
    pub layer0: BTreeMap<u8, Option<u8>>, 
    pub layer1: BTreeMap<u8, Option<u8>>, 
    #[serde(default = "default_fn_id")]
    pub fn_id: u8,
    #[serde(default = "default_single_mod_key_id")]
    pub single_mod_key_id: u8,
    #[serde(default = "default_tp_accel_coeffs")]
    pub trackpoint_speed_settings: TrackPointSpeedSettings,
    #[serde(default = "default_macro_key_map")]
    pub macro_key_map: BTreeMap<u8, MacroKey>, 
    #[serde(default = "default_media_key_map")]
    pub media_key_map: BTreeMap<u8, u16>,
    #[serde(default = "default_enable_middle_click")]
    pub enable_middle_click: bool,
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

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum Tab {
    Keyboard,
    MacroKey,
    MediaKey,
    Trackpoint,
    KeyMatrix,
    Others,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct TrackPointSpeedSettings {
    /// 9 speed settings, each with 43 tilt levels (0..=42)
    pub coeffs: Vec<Vec<i8>>,
}
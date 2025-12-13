use std::collections::BTreeMap;
use serde::{Serialize, Deserialize};

mod general_setting;
pub use general_setting::*;

pub mod config;
pub use config::*;



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
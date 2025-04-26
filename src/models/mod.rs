use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Clone, PartialEq)]
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
    pub map_key_label: HashMap<u8, KeyLabel>,
}

/*
impl LogicalLayout {
    pub fn new() -> LogicalLayout {
        LogicalLayout {
            layout_name: String::new(),
            layout_label: String::new(),
            map_key_label: HashMap::new(),
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

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub config_version: u32,
    pub physical_layout_name: String,
    pub logical_layout_name: String,
    pub layer0: Vec<[u32; 2]>,
    pub layer1: Vec<[u32; 2]>,
    pub tp_sensitivity: u32,
}

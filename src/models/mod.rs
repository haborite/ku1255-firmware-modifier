use std::{collections::HashMap};

mod user_config;
mod keyboard_spec;
pub use user_config::*;
pub use keyboard_spec::*;

/// PhysicalLayout Model
#[derive(Clone, PartialEq, Debug)]
pub struct PhysicalLayout {
    pub name: String,
    pub label: String,
    pub default_logical_layout_name: String,
    pub map_widths: Vec<Vec<u16>>, 
    pub map_address: Vec<Vec<Option<u32>>>,
}


// Logical Layout Model
#[derive(Clone, PartialEq, Debug)]
pub struct LogicalLayout {
    pub name: String,
    pub label: String,
    pub map_key_label: HashMap<u8, KeyLabel>,
}


// Key Label Model
#[derive(Clone, PartialEq, Debug)]
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


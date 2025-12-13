use dioxus::prelude::*;
use std::fs::File;
use std::collections::BTreeMap;
use std::io::{self, BufWriter};
use std::path::Path;
use serde_json::{to_writer_pretty, from_reader};
use crate::models::{GeneralSeitting, LogicalLayout, MacroKey, Board};
use crate::utils::firmware;
use serde::{Serialize, Deserialize};
use rfd::FileDialog;
use std::sync::Arc;

// Default values
const CONFIG_VERSION: u32 = 3;
const DEFAULT_CONFIG_PATH: &str = "example/__default__.json";
const DEFAULT_PHYSICAL_LAYOUT_NAME: &str = "0B47190";
const DEFAULT_LOGICAL_LAYOUT_NAME: &str = "US_English";
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

pub fn default_enable_middle_click() -> bool { false }

#[derive(Store, Serialize, Deserialize)]
pub struct Config {
    pub config_version: u32,
    pub physical_layout_name: String,
    pub logical_layout_name: String,
    pub layer0: BTreeMap<u32, u8>,
    pub layer1: BTreeMap<u32, u8>,
    #[serde(default = "default_fn_id")]
    pub fn_id: u8,
    #[serde(default = "default_tp_sensitivity")]
    pub tp_sensitivity: u32,
    #[serde(default = "default_macro_key_map")]
    pub macro_key_map: BTreeMap<u8, MacroKey>, 
    #[serde(default = "default_media_key_map")]
    pub media_key_map: BTreeMap<u8, u16>,
    #[serde(default = "default_enable_middle_click")]
    pub enable_middle_click: bool,
}


impl Config {

    pub fn new() -> Config {
        Config {
            config_version: CONFIG_VERSION,
            physical_layout_name: DEFAULT_PHYSICAL_LAYOUT_NAME.to_string(),
            logical_layout_name: DEFAULT_LOGICAL_LAYOUT_NAME.to_string(),
            layer0: BTreeMap::new(),
            layer1: BTreeMap::new(),
            fn_id: DEFAULT_FN_ID,
            tp_sensitivity: DEFAULT_TP_SENSITIVITY,
            macro_key_map: BTreeMap::new(),
            media_key_map: BTreeMap::new(),
            enable_middle_click: false,
        }
    }

    pub fn create_from_file(filepath: &Path) -> io::Result<Config> {
        let filepath = Path::new(filepath);
        let file = File::open(filepath)?;
        let config: Config = from_reader(file)?;
        Ok(config)
    }

    pub fn create_default() -> io::Result<Config> {
        let filepath = Path::new(DEFAULT_CONFIG_PATH);
        Self::create_from_file(filepath)
    }

    pub fn create_from_file_dialog() -> Config {
        let config_dir = std::env::current_exe().unwrap().parent().unwrap().join("examples");
        let file = FileDialog::new()
            .add_filter("Config files", &["json"])
            .set_directory(config_dir)
            .set_title("Select config file")
            .pick_file();
        match file {
            Some(path) => {
                match Config::create_from_file(&path) {
                    Ok(config) => config,
                    Err(e) => {
                        println!("Failed to load config file: {}", e);
                        Config::new()
                    }
                }
            },
            None => {
                println!("file not selected");
                Config::new()
            },
        }
    }

    pub fn load_from_file(&mut self, filepath: &Path) -> io::Result<Config> {
        let file = File::open(filepath)?;
        let config: Config = from_reader(file)?;
        Ok(config)
    }

    pub fn load_from_file_dialog(&mut self) {
        let config_dir = std::env::current_exe().unwrap().parent().unwrap().join("examples");
        let file = FileDialog::new()
            .add_filter("Config files", &["json"])
            .set_directory(config_dir)
            .set_title("Select config file")
            .pick_file();
        match file {
            Some(path) => {
                if let Err(e) = self.load_from_file(&path) {
                    eprintln!("Failed to load config file: {}", e);
                };
            },
            None => println!("file not selected"),
        }
    }

    pub fn save_to_file(&self, filepath: &Path) -> io::Result<()> {
        let file = File::create(filepath)?;
        let writer = BufWriter::new(file);
        to_writer_pretty(writer, &self)?;
        Ok(())
    }

    pub fn save_to_file_dialog(&self) {
        let save_path = FileDialog::new()
            .add_filter("JSON files", &["json"])
            .set_directory(std::env::current_exe().unwrap().parent().unwrap().join("examples"))
            .set_file_name("config.json")
            .set_title("Set config filepath")
            .save_file();
        match save_path {
            Some(path) => {
                let _ = self.save_to_file(&path);
                println!("Config file has been saved to: {}", path.display());
            },
            None => println!("Cancel"),
        }
    }

    pub fn copy_layer0_to_layer1(&mut self) {
        self.layer1 = self.layer0.clone();
    }

    pub fn physical_layout<'a, 'b>(&'b self, general_setting: &'a Arc<GeneralSeitting>) -> &'a Board {
        general_setting.avail_boards
            .iter()
            .find(|b| b.board_name == self.physical_layout_name)
            .unwrap_or(general_setting.avail_boards.get(0).unwrap())
    }

    pub fn logical_layout<'a, 'b>(&'b self, general_setting: &'a Arc<GeneralSeitting>) -> &'a LogicalLayout {
        general_setting.avail_logical_layouts
            .iter()
            .find(|l| l.layout_name == self.logical_layout_name)
            .unwrap_or(general_setting.avail_logical_layouts.get(0).unwrap())
    }

    pub fn update_physical_layout(
        &mut self,
        general_setting: Arc<GeneralSeitting>, 
        new_physical_layout_name: &str
    ) {
        self.physical_layout_name = new_physical_layout_name.to_string();
        let avail_boards = &general_setting.avail_boards;
        let avail_logical_layouts = &general_setting.avail_logical_layouts;
        let new_physical_layout = avail_boards
            .iter()
            .find(|b| b.board_name == new_physical_layout_name)
            .unwrap_or(avail_boards.get(0).unwrap());
        let new_logical_layout_name = &new_physical_layout.default_logical_layout_name;
        let new_logical_layout = avail_logical_layouts
            .iter()
            .find(|l| &l.layout_name == new_logical_layout_name)
            .unwrap_or(avail_logical_layouts.get(0).unwrap()); 
        self.logical_layout_name = new_logical_layout.layout_name.to_string();
    }

    pub fn update_logical_layout(&mut self, new_logical_layout_name: &str) {
        self.logical_layout_name = new_logical_layout_name.to_string();
    }

    pub fn update_fn_id(&mut self, new_fn_id: u8) {
        self.fn_id = new_fn_id;        
    }

    pub fn update_tp_sensitivity(&mut self, new_tp_sensitivity: u32) {
        self.tp_sensitivity = new_tp_sensitivity;
    }

    pub fn update_enable_middle_click(&mut self, new_flag: bool) {
        self.enable_middle_click = new_flag;
    }

    pub fn update_left_ctrl(&mut self, trigger_id: u8, new_flag: bool) {
        self.macro_key_map.get_mut(&trigger_id).unwrap().left_ctrl = new_flag;
    }

    pub fn update_left_shift(&mut self, trigger_id: u8, new_flag: bool) {
        self.macro_key_map.get_mut(&trigger_id).unwrap().left_shift = new_flag;
    }

    pub fn update_left_alt(&mut self, trigger_id: u8, new_flag: bool) {
        self.macro_key_map.get_mut(&trigger_id).unwrap().left_alt = new_flag;
    }
    
    pub fn update_left_gui(&mut self, trigger_id: u8, new_flag: bool) {
        self.macro_key_map.get_mut(&trigger_id).unwrap().left_gui = new_flag;
    }

    pub fn update_right_ctrl(&mut self, trigger_id: u8, new_flag: bool) {
        self.macro_key_map.get_mut(&trigger_id).unwrap().right_ctrl = new_flag;
    }
    
    pub fn update_right_shift(&mut self, trigger_id: u8, new_flag: bool) {
        self.macro_key_map.get_mut(&trigger_id).unwrap().right_shift = new_flag;
    }
    
    pub fn update_right_alt(&mut self, trigger_id: u8, new_flag: bool) {
        self.macro_key_map.get_mut(&trigger_id).unwrap().right_alt = new_flag;
    }
    
    pub fn update_right_gui(&mut self, trigger_id: u8, new_flag: bool) {
        self.macro_key_map.get_mut(&trigger_id).unwrap().right_gui = new_flag;
    }

    pub fn update_media_key_map(&mut self, trigger_key_id: u8, new_media_key_id: u16) {
        self.media_key_map.insert(trigger_key_id, new_media_key_id);
    }

    pub fn install_firmware(
        &self,
        fw_installer_future: Resource<Vec<u8>>,
        error_msg: &mut Signal<Option<String>>,    
    ) {
        firmware::build_and_install_mod_fw(self, fw_installer_future, error_msg);
    }

}

/*

    // Board variables
    let avail_board_cloned = general_setting.avail_boards.clone();
    let selected_board_name = use_signal(|| general_setting.avail_boards.get(0).unwrap().board_name.clone() );
    let selected_board: Memo<Board> = use_memo(move || {
        avail_board_cloned.iter().find(|b| b.board_name == selected_board_name())
            .unwrap_or(avail_board_cloned.get(0).unwrap()).clone()
    });
    
    // Logical layout variables
    let logical_layouts_cloned = general_setting.avail_logical_layouts.clone();
    let selected_logical_layout_name = use_signal(|| { selected_board().default_logical_layout_name });
    let selected_logical_layout: Memo<LogicalLayout>  = use_memo(move || {
        logical_layouts_cloned.iter().find(|l| l.layout_name == selected_logical_layout_name())
            .unwrap_or(logical_layouts_cloned.get(0).unwrap()).clone()
    });

*/
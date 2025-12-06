use std::{collections::HashMap};
use std::path::Path;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

use crate::models::{PhysicalLayout, KeyLabel, LogicalLayout};

// Constants
const GENERAL_SETTING_PATH: &str = "settings/general_setting.csv";
const MEDIA_KEY_USAGE_NAMES_PATH: &str = "settings/media_key_usage_names.csv";
const BOARDS_DIR:  &str = "boards";
const LOGICAL_LAYOUT_DIR:  &str = "logical_layouts";
const EXE_URL_SETTING_PATH: &str = "settings/url.txt";

#[derive(PartialEq, Clone)]
pub struct KeyboardSpec {
    pub initial_id_map: HashMap<u32, u8>,
    pub avail_hid_usage_names: HashMap<u8, String>,
    pub avail_media_key_usage_names: HashMap<u16, String>,
    pub avail_physical_layouts: Vec<PhysicalLayout>,
    pub avail_logical_layouts: Vec<LogicalLayout>,
    pub official_firmware_url: String,
}


impl KeyboardSpec {

    pub fn get_media_key_usage_name(&self, media_key_id: u16) -> String {
        self.avail_media_key_usage_names.get(&media_key_id).unwrap().to_string()
    }

    pub fn load_from_files() -> io::Result<KeyboardSpec> {
        
        let general_setting_path = Path::new(GENERAL_SETTING_PATH);
        let media_key_setting_path = Path::new(MEDIA_KEY_USAGE_NAMES_PATH);
        let boards_dir_path = Path::new(BOARDS_DIR);
        let logical_layouts_dir_path = Path::new(LOGICAL_LAYOUT_DIR);
        let official_firmware_url_path = Path::new(EXE_URL_SETTING_PATH);

        let (id_map, usage_names) = KeyboardSpec::load_general_settings(general_setting_path)?;
        let media_key_usage_names = KeyboardSpec::load_media_key_settings(media_key_setting_path)?;
        let avail_physical_layouts = KeyboardSpec::load_boards(boards_dir_path, general_setting_path)?;
        let avail_logical_layouts = KeyboardSpec::load_logical_layouts(logical_layouts_dir_path, general_setting_path)?;
        let official_firmware_url = KeyboardSpec::load_url(official_firmware_url_path)?;

        Ok(KeyboardSpec {
            initial_id_map: id_map,
            avail_physical_layouts,
            avail_logical_layouts,
            avail_media_key_usage_names: media_key_usage_names,
            avail_hid_usage_names: usage_names,
            official_firmware_url: official_firmware_url,
        })

    }

    pub fn load_general_settings(general_setting_path: &Path) -> io::Result<(HashMap<u32, u8>, HashMap<u8, String>)> {

        let file = File::open(general_setting_path)?;
        let mut rdr = csv::Reader::from_reader(BufReader::new(file));
        let mut id_map = HashMap::new();
        let mut usage_names = HashMap::new();
        for record in rdr.records() {
            let record = record?;
            let id_str = record.get(0).unwrap_or("").trim();
            let id = u8::from_str_radix(id_str, 16).unwrap_or(0);
            let usage_name = record.get(1).unwrap_or("").trim();
            let address_str = record.get(2).unwrap_or("").trim();
            if let Some(address) = u32::from_str_radix(address_str, 16).ok() {
                id_map.insert(address, id);
            };
            usage_names.insert(id, usage_name.to_string());
        }
        Ok((id_map, usage_names))
    }

    pub fn load_media_key_settings(media_key_setting_path: &Path) -> io::Result<HashMap<u16, String>> {
        
        let file = File::open(media_key_setting_path)?;
        let mut rdr = csv::Reader::from_reader(BufReader::new(file));
        let mut media_key_usage_names = HashMap::new();
        for record in rdr.records() {
            let record = record?;
            let media_key_str = record.get(0).unwrap_or("").trim();
            let media_key = u16::from_str_radix(media_key_str, 16).unwrap_or(0);
            let label = record.get(1).unwrap_or("").trim();
            media_key_usage_names.insert(media_key, label.to_string());
        }
        Ok(media_key_usage_names)
    }

    pub fn load_address2id(general_config_path: &Path) -> io::Result<HashMap<u32, u8>> {
        let file = File::open(general_config_path)?;
        let mut rdr = csv::Reader::from_reader(BufReader::new(file));
        let mut id_map = HashMap::new();

        for record in rdr.records() {
            let record = record?;
            let id_str = record.get(0).unwrap_or("").trim();
            let Ok(id) = u8::from_str_radix(id_str, 16) else {
                println!("Cannot convert '{}' to hex number", id_str);
                break
            };
            let address_str = record.get(2).unwrap_or("").trim();
            if let Some(address) = u32::from_str_radix(address_str, 16).ok() {
                id_map.insert(address, id);
            };
        }
        Ok(id_map)
    }

    pub fn load_board(board_config_path: &Path, general_config_path: &Path) -> io::Result<PhysicalLayout> {
        let file = File::open(board_config_path)?;
        let reader = BufReader::new(file);

        enum Section {
            None,
            Name,
            Label,
            DefaultLogicalLayout,
            KeyId,
            Width,
        }

        let mut section = Section::None;
        let mut name = "".to_string();
        let mut label = "".to_string();
        let mut default_logical_layout_name = "".to_string();
        let mut map_ids: Vec<Vec<Option<u8>>> = Vec::new();
        let mut map_widths: Vec<Vec<u16>> = Vec::new();

        for line in reader.lines() {
            let line = line?;
            let trimmed = line.split('#').next().unwrap_or("").trim();

            if trimmed.is_empty() {
                continue;
            }

            match trimmed {
                "[name]" => {
                    section = Section::Name;
                    continue;
                }
                "[label]" => {
                    section = Section::Label;
                    continue;
                }
                "[default_logical_layout_name]" => {
                    section = Section::DefaultLogicalLayout;
                    continue;                
                }
                "[key_id]" => {
                    section = Section::KeyId;
                    continue;
                }
                "[key_width]" => {
                    section = Section::Width;
                    continue;
                }
                _ => {}
            }

            let tokens: Vec<&str> = trimmed.split(',').map(|s| s.trim()).collect();

            match section {
                Section::Name => {
                    if let Some(s) = tokens.get(0).copied() {
                        name = s.to_string();
                    }
                }
                Section::Label => {
                    if let Some(s) = tokens.get(0).copied() {
                        label = s.to_string();
                    }
                }
                Section::DefaultLogicalLayout => {
                    if let Some(s) = tokens.get(0).copied() {
                        default_logical_layout_name = s.to_string();
                    }
                }
                Section::KeyId => {
                    let row = tokens
                        .into_iter()
                        .map(|s| {
                            if s.is_empty() {
                                None
                            } else {
                                u8::from_str_radix(s, 16).ok()
                            }
                        })
                        .collect::<Vec<_>>();
                    map_ids.push(row);
                }
                Section::Width => {
                    let row = tokens
                        .into_iter()
                        .map(|s| s.parse::<u16>())
                        .collect::<Result<Vec<_>, _>>()
                        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                    map_widths.push(row);
                }
                Section::None => {
                    // Ignore
                }
            }
        }

        if map_ids.len() != map_widths.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Row count mismatch between key IDs and widths",
            ));
        }

        let address2id = KeyboardSpec::load_address2id(general_config_path)?;

        let map_address = map_ids.into_iter().map(|v|{
            v.into_iter().map(|id_opt|{
                if let Some(id) = id_opt {
                    address2id.iter().find_map(|(k, v)| if *v == id { Some(*k) } else { None })
                } else {
                    None
                }
            }).collect()
        }).collect();

        Ok(PhysicalLayout {
            name,
            label,
            default_logical_layout_name,
            map_widths,
            map_address,
        })
    }

    pub fn load_boards(dir: &Path, general_config_path: &Path) -> io::Result<Vec<PhysicalLayout>> {
        let mut cfg_files = Vec::new();
        let mut cfgs = Vec::new();
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                if let Some(ext_found) = path.extension() {
                    if ext_found == "cfg" {
                        cfg_files.push(path);
                    }
                }
            }
        }
        for cfg_filepath in cfg_files {
            if let Ok(cfg) = KeyboardSpec::load_board(&cfg_filepath, general_config_path) {
                cfgs.push(cfg);
            };
        }
        Ok(cfgs)
    }

    pub fn load_usage_names(general_config_path: &Path) -> io::Result<HashMap<u8, String>> {
        let file = File::open(general_config_path)?;
        let mut rdr = csv::Reader::from_reader(BufReader::new(file));
        let mut usage_name_map = HashMap::new();

        for record in rdr.records() {
            let record = record?;
            let id_str = record.get(0).unwrap_or("").trim();
            let Ok(id) = u8::from_str_radix(id_str, 16) else {
                println!("Cannot convert '{}' to hex number", id_str);
                break
            };
            let usage_name = record.get(1).unwrap_or("").trim();
            usage_name_map.insert(id, usage_name.to_string());
        }
        Ok(usage_name_map)
    }

    pub fn load_logical_layout(logical_layout_path: &Path, general_config_path: &Path) -> io::Result<LogicalLayout> {
        let file = File::open(logical_layout_path)?;
        let mut rdr = csv::Reader::from_reader(BufReader::new(file));
        let mut map_key_label = HashMap::new();

        let usage_names = KeyboardSpec::load_usage_names(general_config_path)?;

        for record in rdr.records() {
            let record = record?;
            let id_str = record.get(0).unwrap_or("").trim();
            if id_str.is_empty() {
                continue;
            }

            let id = u8::from_str_radix(id_str, 16).unwrap_or(0);
            let default = record.get(1).unwrap_or("").trim().to_string();
            let shifted = record.get(2).unwrap_or("").trim().to_string();

            let usage_name = usage_names.get(&id).map_or("", |v| v).to_string();
            map_key_label.insert(id, KeyLabel{ usage_name, default, shifted } );
        }

        let basename = logical_layout_path.file_stem().unwrap().to_str().unwrap();

        Ok(LogicalLayout{
            name: basename.to_string(),
            label: basename.to_string().replace("_", " / "),
            map_key_label
        })
    }

    pub fn load_logical_layouts(dir: &Path, general_config_path: &Path) -> io::Result<Vec<LogicalLayout>> {
        let mut cfg_files = Vec::new();
        let mut cfgs = Vec::new();
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                if let Some(ext_found) = path.extension() {
                    if ext_found == "csv" {
                        cfg_files.push(path);
                    }
                }
            }
        }
        for cfg_filepath in cfg_files {
            if let Ok(cfg) = KeyboardSpec::load_logical_layout(&cfg_filepath, general_config_path) {
                cfgs.push(cfg);
            };
        }
        Ok(cfgs)
    }

    pub fn load_url(filepath: &Path) -> io::Result<String> {
        let file = File::open(filepath)?;
        let mut reader = BufReader::new(file);
        let mut line = String::new();
        reader.read_line(&mut line)?;
        Ok(line.trim().to_string())
    }

}



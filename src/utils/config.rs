// mod crate::models;
use std::error::Error;
use std::fs::File;
use std::collections::HashMap;
use std::io::{self, BufRead, BufReader, BufWriter};
use std::path::Path;
// use serde::{Serialize, Deserialize};
use serde_json::{to_writer_pretty, from_reader};
use crate::models::{Board, KeyLabel, LogicalLayout, Config};


pub fn load_general(path: &Path) -> Result<(HashMap<u32, u8>, Vec<u8>, Vec<String>), Box<dyn Error>> {
    let file = File::open(path)?;
    let mut rdr = csv::Reader::from_reader(BufReader::new(file));
    let mut id_map = HashMap::new();
    let mut id_list = Vec::<u8>::new();
    let mut usage_names = Vec::<String>::new();

    for record in rdr.records() {
        let record = record?;
        let id_str = record.get(0).unwrap_or("").trim();
        let id = u8::from_str_radix(id_str, 16).map_err(|e| format!("Invalid hex ID '{}': {}", id_str, e))?;
        let usage_name = record.get(1).unwrap_or("").trim();
        let address_str = record.get(2).unwrap_or("").trim();
        if let Some(address) = u32::from_str_radix(address_str, 16).ok() {
            id_map.insert(address, id);
        };
        usage_names.push(usage_name.to_string());
        id_list.push(id);
    }
    Ok((id_map, id_list, usage_names))
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


pub fn load_board(board_config_path: &Path, general_config_path: &Path) -> io::Result<Board> {
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
    let mut board_name = "".to_string();
    let mut board_label = "".to_string();
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
            "[board_name]" => {
                section = Section::Name;
                continue;
            }
            "[board_label]" => {
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
                    board_name = s.to_string();
                }
            }
            Section::Label => {
                if let Some(s) = tokens.get(0).copied() {
                    board_label = s.to_string();
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

    let address2id = load_address2id(general_config_path)?;

    let map_address = map_ids.into_iter().map(|v|{
        v.into_iter().map(|id_opt|{
            if let Some(id) = id_opt {
                address2id.iter().find_map(|(k, v)| if *v == id { Some(*k) } else { None })
            } else {
                None
            }
        }).collect()
    }).collect();

    Ok(Board {
        board_name,
        board_label,
        default_logical_layout_name,
        map_widths,
        map_address,
    })
}


pub fn load_boards(dir: &Path, general_config_path: &Path) -> Vec<Board> {
    let mut cfg_files = Vec::new();
    let mut cfgs = Vec::new();
    for entry in std::fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
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
        if let Ok(cfg) = load_board(&cfg_filepath, general_config_path) {
            cfgs.push(cfg);
        };
    }
    cfgs
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


pub fn load_logical_layout(logical_layout_path: &Path, general_config_path: &Path) -> Result<LogicalLayout, Box<dyn Error>> {
    let file = File::open(logical_layout_path)?;
    let mut rdr = csv::Reader::from_reader(BufReader::new(file));
    let mut map_key_label = HashMap::new();

    let usage_names = load_usage_names(general_config_path)?;

    for record in rdr.records() {
        let record = record?;
        let id_str = record.get(0).unwrap_or("").trim();
        if id_str.is_empty() {
            continue;
        }

        let id = u8::from_str_radix(id_str, 16).map_err(|e| format!("Invalid hex ID '{}': {}", id_str, e))?;
        let default = record.get(1).unwrap_or("").trim().to_string();
        let shifted = record.get(2).unwrap_or("").trim().to_string();

        let usage_name = usage_names.get(&id).map_or("", |v| v).to_string();
        map_key_label.insert(id, KeyLabel{ usage_name, default, shifted } );
    }

    let basename = logical_layout_path.file_stem().unwrap().to_str().unwrap();

    Ok(LogicalLayout{
        layout_name: basename.to_string(),
        layout_label: basename.to_string().replace("_", " / "),
        map_key_label
    })
}


pub fn load_logical_layouts(dir: &Path, general_config_path: &Path) -> Vec<LogicalLayout> {
    let mut cfg_files = Vec::new();
    let mut cfgs = Vec::new();
    for entry in std::fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
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
        if let Ok(cfg) = load_logical_layout(&cfg_filepath, general_config_path) {
            cfgs.push(cfg);
        };
    }
    cfgs
}


pub fn load_config(filepath: &Path)
    -> io::Result<(String, String, HashMap<u32, u8>, HashMap<u32, u8>, u8, u32)> 
{
    let file = File::open(filepath)?;
    let config: Config = from_reader(file)?;
    let layer0: HashMap<u32, u8> = config.layer0.into_iter().map(|[key, value]| (key, value as u8)).collect();
    let layer1: HashMap<u32, u8> = config.layer1.into_iter().map(|[key, value]| (key, value as u8)).collect();
    let fn_id = config.fn_id;
    let tp_sensitivity = config.tp_sensitivity;
    Ok((
        config.physical_layout_name,
        config.logical_layout_name,
        layer0,
        layer1,
        fn_id,
        tp_sensitivity,
    ))
}

pub fn save_config(
    filepath: &Path,
    physical_layout_name: &str,
    logical_layout_name: &str,
    id_layout_l0: &HashMap<u32, u8>,
    id_layout_l1: &HashMap<u32, u8>,
    fn_id: u8,
    tp_sensitivity: u32
) -> io::Result<()> {
    let layer0: Vec<[u32; 2]> = id_layout_l0.iter()
        .map(|(&key, &value)| [key, value as u32])
        .collect();
    let layer1: Vec<[u32; 2]> = id_layout_l1.iter()
        .map(|(&key, &value)| [key, value as u32])
        .collect();
    let config = Config {
        config_version: 2,
        physical_layout_name: physical_layout_name.to_string(),
        logical_layout_name: logical_layout_name.to_string(),
        layer0,
        layer1,
        fn_id,
        tp_sensitivity,
    };
    let file = File::create(filepath)?;
    let writer = BufWriter::new(file);
    to_writer_pretty(writer, &config)?;
    Ok(())
}


pub fn load_url(filepath: &Path) -> io::Result<String> {
    let file = File::open(filepath)?;
    let mut reader = BufReader::new(file);
    let mut line = String::new();
    reader.read_line(&mut line)?;
    Ok(line.trim().to_string())
}
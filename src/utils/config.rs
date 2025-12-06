// mod crate::models;
// use std::error::Error;
use std::fs::File;
use std::collections::BTreeMap;
use std::io::{self, BufRead, BufReader, BufWriter};
use std::path::Path;
// use serde::{Serialize, Deserialize};
use serde_json::{to_writer_pretty, from_reader};
use crate::models::{Config, MacroKey};



pub fn load_config(filepath: &Path)
    -> io::Result<(String, String, BTreeMap<u32, u8>, BTreeMap<u32, u8>, u8, u32, BTreeMap<u8, MacroKey>, BTreeMap<u8, u16>)> 
{
    let file = File::open(filepath)?;
    let config: Config = from_reader(file)?;
    let layer0: BTreeMap<u32, u8> = config.layer0.into_iter().map(|[key, value]| (key, value as u8)).collect();
    let layer1: BTreeMap<u32, u8> = config.layer1.into_iter().map(|[key, value]| (key, value as u8)).collect();
    let fn_id = config.fn_id;
    let tp_sensitivity = config.tp_sensitivity;
    let macro_key_map = config.macro_key_map;
    let media_key_map = config.media_key_map;
    Ok((
        config.physical_layout_name,
        config.logical_layout_name,
        layer0,
        layer1,
        fn_id,
        tp_sensitivity,
        macro_key_map,
        media_key_map,
    ))
}

pub fn save_config(
    filepath: &Path,
    physical_layout_name: &str,
    logical_layout_name: &str,
    id_layout_l0: &BTreeMap<u32, u8>,
    id_layout_l1: &BTreeMap<u32, u8>,
    fn_id: u8,
    tp_sensitivity: u32,
    macro_key_map: &BTreeMap<u8, MacroKey>,
    media_key_map: &BTreeMap<u8, u16>,
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
        macro_key_map: macro_key_map.clone(),
        media_key_map: media_key_map.clone(),
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
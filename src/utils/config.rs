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
    -> io::Result<(String, String, BTreeMap<u8, Option<u8>>, BTreeMap<u8, Option<u8>>, u8, u32, BTreeMap<u8, MacroKey>, BTreeMap<u8, u16>, bool)> 
{
    let file = File::open(filepath)?;
    let config: Config = from_reader(file)?;
    println!("{:?}", config.layer0);
    Ok((
        config.physical_layout_name,
        config.logical_layout_name,
        config.layer0,
        config.layer1,
        config.fn_id,
        config.tp_sensitivity,
        config.macro_key_map,
        config.media_key_map,
        config.enable_middle_click,
    ))
}

pub fn save_config(
    filepath: &Path,
    physical_layout_name: &str,
    logical_layout_name: &str,
    layer0: &BTreeMap<u8, Option<u8>>,
    layer1: &BTreeMap<u8, Option<u8>>,
    fn_id: u8,
    tp_sensitivity: u32,
    macro_key_map: &BTreeMap<u8, MacroKey>,
    media_key_map: &BTreeMap<u8, u16>,
    enable_middle_click: bool,
) -> io::Result<()> {
    let config = Config {
        config_version: 4,
        physical_layout_name: physical_layout_name.to_string(),
        logical_layout_name: logical_layout_name.to_string(),
        layer0: layer0.clone(),
        layer1: layer1.clone(),
        fn_id,
        tp_sensitivity,
        macro_key_map: macro_key_map.clone(),
        media_key_map: media_key_map.clone(),
        enable_middle_click,
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
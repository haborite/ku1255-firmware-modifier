use std::{collections::BTreeMap};
use std::io;

use include_dir::Dir;

use crate::models::{Board, KeyLabel, LogicalLayout};
use crate::resources;

#[derive(PartialEq, Clone)]
pub struct GeneralSeitting {
    pub initial_id_map: BTreeMap<u8, Option<u8>>,
    pub avail_hid_usage_names: BTreeMap<u8, String>,
    pub avail_media_key_usage_names: BTreeMap<u16, String>,
    pub avail_boards: Vec<Board>,
    pub avail_logical_layouts: Vec<LogicalLayout>,
    pub official_firmware_url: String,
}


impl GeneralSeitting {

    pub fn get_media_key_usage_name(&self, media_key_id: u16) -> String {
        self.avail_media_key_usage_names.get(&media_key_id).unwrap().to_string()
    }

    /// Load the settings bundled into the executable.
    pub fn load_embedded() -> io::Result<GeneralSeitting> {
        let (id_map, usage_names) = GeneralSeitting::load_general_settings(resources::GENERAL_SETTING)?;
        let media_key_usage_names = GeneralSeitting::load_media_key_settings(resources::MEDIA_KEY_USAGE_NAMES)?;
        let avail_boards = GeneralSeitting::load_boards(&resources::BOARDS)?;
        let avail_logical_layouts = GeneralSeitting::load_logical_layouts(&resources::LOGICAL_LAYOUTS, &usage_names)?;
        let official_firmware_url = resources::firmware_url().to_string();

        Ok(GeneralSeitting {
            initial_id_map: id_map,
            avail_boards,
            avail_logical_layouts,
            avail_media_key_usage_names: media_key_usage_names,
            avail_hid_usage_names: usage_names,
            official_firmware_url: official_firmware_url,
        })

    }

    pub fn load_general_settings(general_setting: &str) -> io::Result<(BTreeMap<u8, Option<u8>>, BTreeMap<u8, String>)> {

        let mut rdr = csv::Reader::from_reader(general_setting.as_bytes());
        let mut id_map = BTreeMap::new();
        let mut usage_names = BTreeMap::new();
        for record in rdr.records() {
            let record = record?;
            let id_str = record.get(0).unwrap_or("").trim();
            let id = u8::from_str_radix(id_str, 16).unwrap_or(0);
            let usage_name = record.get(1).unwrap_or("").trim();
            let address_str = record.get(2).unwrap_or("").trim();
            if let Some(address) = u8::from_str_radix(address_str, 16).ok() {
                id_map.insert(address, Some(id));
            };
            usage_names.insert(id, usage_name.to_string());
        }
        Ok((id_map, usage_names))
    }

    pub fn load_media_key_settings(media_key_setting: &str) -> io::Result<BTreeMap<u16, String>> {
        
        let mut rdr = csv::Reader::from_reader(media_key_setting.as_bytes());
        let mut media_key_usage_names = BTreeMap::new();
        for record in rdr.records() {
            let record = record?;
            let media_key_str = record.get(0).unwrap_or("").trim();
            let media_key = u16::from_str_radix(media_key_str, 16).unwrap_or(0);
            let label = record.get(1).unwrap_or("").trim();
            media_key_usage_names.insert(media_key, label.to_string());
        }
        Ok(media_key_usage_names)
    }

    pub fn load_board(board_config: &str) -> io::Result<Board> {

        enum Section {
            None,
            Name,
            Label,
            DefaultLogicalLayout,
            KeyId,
            KeyAddress,
            Width,
        }

        let mut section = Section::None;
        let mut board_name = "".to_string();
        let mut board_label = "".to_string();
        let mut default_logical_layout_name = "".to_string();
        let mut map_ids: Vec<Vec<Option<u8>>> = Vec::new();
        let mut map_address: Vec<Vec<Option<u8>>> = Vec::new();
        let mut map_widths: Vec<Vec<u16>> = Vec::new();

        for line in board_config.lines() {
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
                "[key_address]" => {
                    section = Section::KeyAddress;
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
                Section::KeyAddress => {
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
                    map_address.push(row);
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

        if map_address.len() != map_widths.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Row count mismatch between key addresses and widths",
            ));
        }

        let initial_id_map: BTreeMap<u8, Option<u8>> = map_address
            .iter().flatten()
            .zip(map_ids.iter().flatten())
            .filter_map(|(addr_opt, id_opt)| {
                addr_opt.map(|addr| (addr, *id_opt))
            })
            .collect();

        Ok(Board {
            board_name,
            board_label,
            default_logical_layout_name,
            map_widths,
            map_address,
            initial_id_map,
        })
    }

    /// `*.cfg` files of `dir`, sorted by file name (the first one is the default board).
    pub fn load_boards(dir: &Dir) -> io::Result<Vec<Board>> {
        let mut cfgs = Vec::new();
        for (name, text) in resources::text_files(dir, "cfg")? {
            match GeneralSeitting::load_board(text) {
                Ok(cfg) => cfgs.push(cfg),
                Err(e) => eprintln!("Skipping board {name}: {e}"),
            }
        }
        Ok(cfgs)
    }

    pub fn load_logical_layout(basename: &str, logical_layout: &str, usage_names: &BTreeMap<u8, String>) -> io::Result<LogicalLayout> {
        let mut rdr = csv::Reader::from_reader(logical_layout.as_bytes());
        let mut map_key_label = BTreeMap::new();

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

        Ok(LogicalLayout{
            layout_name: basename.to_string(),
            layout_label: basename.to_string().replace("_", " / "),
            map_key_label
        })
    }

    /// `*.csv` files of `dir`, sorted by file name.
    pub fn load_logical_layouts(dir: &Dir, usage_names: &BTreeMap<u8, String>) -> io::Result<Vec<LogicalLayout>> {
        let mut cfgs = Vec::new();
        for (name, text) in resources::text_files(dir, "csv")? {
            let basename = name.strip_suffix(".csv").unwrap_or(&name);
            match GeneralSeitting::load_logical_layout(basename, text, usage_names) {
                Ok(cfg) => cfgs.push(cfg),
                Err(e) => eprintln!("Skipping logical layout {name}: {e}"),
            }
        }
        Ok(cfgs)
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_embedded_settings() {
        let settings = GeneralSeitting::load_embedded().unwrap();
        let boards: Vec<_> = settings.avail_boards.iter().map(|b| b.board_name.as_str()).collect();
        // The first board is the default selection.
        assert_eq!(boards, ["0B47190", "0B47208", "0B47221"]);
        let layouts: Vec<_> = settings.avail_logical_layouts.iter().map(|l| l.layout_name.as_str()).collect();
        assert_eq!(layouts, ["JP_Japanese", "UK_English", "US_English"]);
        for board in &settings.avail_boards {
            assert!(layouts.contains(&board.default_logical_layout_name.as_str()), "{}", board.board_name);
        }
        assert_eq!(settings.avail_hid_usage_names.len(), 256);
        assert!(settings.official_firmware_url.starts_with("https://"));
    }
}

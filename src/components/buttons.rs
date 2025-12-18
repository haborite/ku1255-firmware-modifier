use dioxus::prelude::*;
use std::collections::BTreeMap;
use rfd::FileDialog;
use crate::models::{MacroKey, Board, LogicalLayout};
use crate::utils::{
    install_firmware_by_flashsn8,
    load_config,
    save_config,
};

#[component]
pub fn ButtonCopyLayer(
    id_layout_l0: Signal<BTreeMap<u32, u8>>,
    id_layout_l1: Signal<BTreeMap<u32, u8>>,    
) -> Element {
    rsx! {
        div { class: "relative inline-flex",
            button {
                class: "px-4 py-2 bg-gray-500 text-white rounded shadow hover:bg-gray-600",
                onclick: move |_| { id_layout_l1.set(id_layout_l0().clone()) },
                "Copy layer from Main to 2nd"
            }
        }
    }
}

#[component]
pub fn ButtonInstall(
    id_layout_l0: Signal<BTreeMap<u32, u8>>,
    id_layout_l1: Signal<BTreeMap<u32, u8>>,
    firmware_future: Resource<Vec<u8>>,
    fn_id: Signal<u8>,
    tp_sensitivity: Signal<u32>,
    macro_key_map: Signal<BTreeMap<u8, MacroKey>>,
    media_key_map: Signal<BTreeMap<u8, u16>>,
    enable_middle_click: Signal<bool>,
    error_msg: Signal<Option<String>>,
) -> Element {
    rsx! {
        button {
            class: "px-4 py-2 bg-blue-500 text-white rounded shadow hover:bg-blue-600",
            onclick: move |_| {
                install_firmware_by_flashsn8(
                    id_layout_l0,
                    id_layout_l1,
                    firmware_future,
                    fn_id,
                    tp_sensitivity,
                    macro_key_map,
                    media_key_map,
                    enable_middle_click,
                    &mut error_msg,
                );
            },
            "Install firmware"              
        }
    }
}


#[component]
pub fn ButtonLoad(
    selected_board_name: Signal<String>,
    selected_logical_layout_name: Signal<String>,
    id_layout_l0: Signal<BTreeMap<u32, u8>>,
    id_layout_l1: Signal<BTreeMap<u32, u8>>,
    fn_id: Signal<u8>,
    tp_sensitivity: Signal<u32>,
    macro_key_map: Signal<BTreeMap<u8, MacroKey>>,
    media_key_map: Signal<BTreeMap<u8, u16>>,
    enable_middle_click: Signal<bool>,
) -> Element {
    rsx! {
        button {
            class: "px-4 py-2 bg-green-500 text-white rounded shadow hover:bg-green-600",
            onclick: move |_| {
                let file = FileDialog::new()
                .add_filter("Config files", &["json"])
                .set_directory(std::env::current_exe().unwrap().parent().unwrap().join("examples"))
                .set_title("Select key-remapping file")
                .pick_file();
                match file {
                    Some(path) => {
                        if let Ok((
                            loaded_board_name,
                            loaded_logical_layout_name,
                            loaded_id_layout_l0,
                            loaded_id_layout_l1,
                            loaded_fn_id,
                            loaded_tp_sensitivity,
                            loaded_macro_key_map,
                            loaded_media_key_map,
                            loaded_enable_middle_click
                        )) = load_config(&path) {
                            selected_board_name.set(loaded_board_name);
                            selected_logical_layout_name.set(loaded_logical_layout_name);
                            id_layout_l0.set(loaded_id_layout_l0);
                            id_layout_l1.set(loaded_id_layout_l1);
                            fn_id.set(loaded_fn_id);
                            tp_sensitivity.set(loaded_tp_sensitivity);
                            macro_key_map.set(loaded_macro_key_map);
                            media_key_map.set(loaded_media_key_map);
                            enable_middle_click.set(loaded_enable_middle_click);
                        };
                    },
                    None => println!("file not selected"),
                }
            },
            "Load config"
        }
    }
}

#[component]
pub fn ButtonSave(
    selected_board: ReadSignal<Board>,
    selected_logical_layout: Memo<LogicalLayout>,
    id_layout_l0: ReadSignal<BTreeMap<u32, u8>>,
    id_layout_l1: ReadSignal<BTreeMap<u32, u8>>,
    fn_id: ReadSignal<u8>,
    tp_sensitivity: ReadSignal<u32>,
    macro_key_map: ReadSignal<BTreeMap<u8, MacroKey>>,
    media_key_map: ReadSignal<BTreeMap<u8, u16>>,
    enable_middle_click: ReadSignal<bool>,
) -> Element {
    rsx! {
        button {
            class: "px-4 py-2 bg-green-500 text-white rounded shadow hover:bg-green-600",
            onclick: move |_| {
                let save_path = FileDialog::new()
                    .add_filter("JSON files", &["json"])
                    .set_directory(std::env::current_exe().unwrap().parent().unwrap().join("examples"))
                    .set_file_name("config.json")
                    .set_title("Set config filepath")
                    .save_file();
                match save_path {
                    Some(path) => {
                        println!("Config file has been saved to: {}", path.display());
                        let _ = save_config(
                            &path,
                            &selected_board().board_name,
                            &selected_logical_layout().layout_name,
                            &id_layout_l0(),
                            &id_layout_l1(),
                            fn_id(),
                            tp_sensitivity(),
                            &macro_key_map(),
                            &media_key_map(),
                            enable_middle_click(),
                        );
                    },
                    None => println!("Cancel"),
                }
            },
            "Save config"
        }
    }
}
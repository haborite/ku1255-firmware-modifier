use std::path::Path;
use std::sync::Arc;
use std::collections::BTreeMap;

mod components;
mod models;
mod utils;

use dioxus::prelude::*;
use components::{
    SelectBoard,
    SelectLogicalLayout,
    ButtonCopyLayer,
    ButtonInstall,
    ButtonLoad,
    ButtonSave,
    ErrorMessage,
    Keyboard,
    SliderTPSensitivity,
    SelectFnID,
    MacroKeySetting,
    MediaKeySetting,
};

use models::{
    Board, LogicalLayout, GeneralSeitting, MacroKey, 
    default_fn_id, default_tp_sensitivity, default_macro_key_map, default_media_key_map,
};
use utils::{load_url, load_or_download_firmware};

// Assets
const FAVICON: Asset = asset!("/public/favicon.ico");
const MAIN_CSS: Asset = asset!("/public/styling/main.css");
const TAILWIND_CSS: Asset = asset!("/public/tailwind.css");

// Constants
const EXE_URL_SETTING_PATH: &str = "settings/url.txt";


fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {

    let general_setting = GeneralSeitting::load_from_files().unwrap();

    // Firmware to be patched
    let exe_url = load_url(Path::new(EXE_URL_SETTING_PATH)).unwrap();
    let firmware_future = use_resource({move || {
        let exe_url_cloned = exe_url.clone();
        async move {
            load_or_download_firmware(&exe_url_cloned).await
        }
    }});

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        MainWindow { general_setting, firmware_future }
    }
}

#[component]
pub fn MainWindow(
    general_setting: GeneralSeitting,
    firmware_future: Resource<Vec<u8>>,
) -> Element {

    // General setting 
    let general_setting = Arc::new(general_setting);

    // Error message
    let error_msg: Signal<Option<String>> = use_signal(|| None);

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

    // ID Layout variables
    let initial_id_cloned = general_setting.initial_id_map.clone();
    let id_layout_l0 = use_signal(|| initial_id_cloned);
    let id_layout_l1 = use_signal(|| id_layout_l0().clone());

    // Other variables
    let fn_id = use_signal(default_fn_id);
    let tp_sensitivity = use_signal(default_tp_sensitivity);
    let macro_key_map: Signal<BTreeMap<u8, MacroKey>> = use_signal(default_macro_key_map);
    let media_key_map: Signal<BTreeMap<u8, u16>> = use_signal(default_media_key_map);


    rsx! {
        if let Some(msg) = error_msg() {
            ErrorMessage { msg, error_msg }
        }

        div { class: "min-h-screen bg-gray-600 text-slate-100",
            div { class: "mx-auto w-full p-4 space-y-4",

                div { class: "w-full bg-gray-800 p-4 rounded shadow flex flex-wrap items-end gap-4",
                    div { class: "flex flex-wrap items-center gap-2",
                        label { class: "text-sm text-gray-200", "Keyboard" }
                        SelectBoard { 
                            general_setting: general_setting.clone(),
                            selected_board_name, 
                            selected_logical_layout_name, 
                            selected_board, 
                        }
                        label { class: "text-sm text-gray-200", "Language" }
                        SelectLogicalLayout {
                            general_setting: general_setting.clone(),
                            selected_logical_layout_name,
                            selected_logical_layout
                        }
                    }
                    div { class: "flex items-center gap-2 ml-auto",
                        ButtonCopyLayer { id_layout_l0, id_layout_l1 }
                        ButtonLoad { 
                            selected_board_name,
                            selected_logical_layout_name,
                            id_layout_l0,
                            id_layout_l1,
                            fn_id,
                            tp_sensitivity,
                            macro_key_map,
                            media_key_map
                        }
                        ButtonSave {
                            selected_board,
                            selected_logical_layout,
                            id_layout_l0,
                            id_layout_l1,
                            fn_id,
                            tp_sensitivity,
                            macro_key_map,
                            media_key_map
                        }
                        ButtonInstall {
                            id_layout_l0,
                            id_layout_l1,
                            firmware_future,
                            fn_id,
                            tp_sensitivity,
                            macro_key_map,
                            media_key_map,
                            error_msg
                        }
                    }
                }

                div { class: "flex gap-4",
                    div { class: "bg-black rounded flex flex-col",
                        Keyboard {
                            general_setting: general_setting.clone(),
                            layer_number: 0,
                            board: selected_board().clone(),
                            logical_layout: selected_logical_layout().clone(),
                            id_layout_l0: id_layout_l0,
                            id_layout_l1: id_layout_l1,
                        }
                        Keyboard {
                            general_setting: general_setting.clone(),
                            layer_number: 1,
                            board: selected_board().clone(),
                            logical_layout: selected_logical_layout().clone(),
                            id_layout_l0: id_layout_l0,
                            id_layout_l1: id_layout_l1,
                        }
                    }
                    div { class: "flex flex-col gap-4",
                        div { class: "bg-black rounded",
                            SliderTPSensitivity { tp_sensitivity }
                        }
                        div { class: "flex flex-col bg-black rounded max-h-[calc(100vh-282px)]",
                            h2 { class: "text-xl font-bold text-center py-2", "Macro keys" },
                            div { class: "px-6 overflow-y-auto",
                                MacroKeySetting {
                                    general_setting: general_setting.clone(),
                                    map_key_label: selected_logical_layout().map_key_label.clone(),
                                    macro_key_map: macro_key_map.clone(),
                                }
                            }
                        }
                    }
                    div { class: "flex flex-col gap-4", 
                        div { class: "px-6 bg-black rounded pb-6",
                            SelectFnID {
                                general_setting: general_setting.clone(),
                                fn_id,
                                map_key_label: selected_logical_layout().map_key_label.clone(),
                            }
                        }
                        div { class: "flex flex-col bg-black rounded max-h-[calc(100vh-220px)]",
                            h2 { class: "text-xl font-bold text-center py-2", "Media keys" },
                            div { class: "px-6 overflow-y-auto",
                                MediaKeySetting {
                                    general_setting: general_setting.clone(),
                                    media_key_map: media_key_map.clone(),
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

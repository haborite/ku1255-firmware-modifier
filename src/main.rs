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
    ButtonTab,
    ErrorMessage,
    Keyboard,
    SelectFnID,
    SelectSingleModKeyID,
    MacroKeySetting,
    MediaKeySetting,
    TrackPointSpeedChart,
    TrackPointSpeedTableEditor,
    TrackPointSpeedPresetButtons,
};




use models::{
    Board, LogicalLayout, GeneralSeitting, MacroKey, 
    default_fn_id, default_macro_key_map, default_media_key_map, default_enable_middle_click,
    default_tp_accel_coeffs, default_single_mod_key_id
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
    // let initial_id_cloned = selected_board().initial_id_map.clone();
    let id_layout_l0 = use_signal(|| initial_id_cloned);
    let id_layout_l1 = use_signal(|| id_layout_l0().clone());

    // Other variables
    let fn_id = use_signal(default_fn_id);
    let single_mod_key_id = use_signal(default_single_mod_key_id);
    let macro_key_map: Signal<BTreeMap<u8, MacroKey>> = use_signal(default_macro_key_map);
    let media_key_map: Signal<BTreeMap<u8, u16>> = use_signal(default_media_key_map);
    let mut enable_middle_click: Signal<bool> = use_signal(default_enable_middle_click);
    let trackpoint_speed_settings: Signal<models::TrackPointSpeedSettings> = use_signal(default_tp_accel_coeffs);

    // UI switch
    let current_tab = use_signal(|| models::Tab::Keyboard);

    rsx! {
        if let Some(msg) = error_msg() {
            ErrorMessage { msg, error_msg }
        }

        div { class: "min-h-screen bg-gray-600 text-slate-100",

            div { class: "mx-auto w-full p-4 space-y-4",

                div { class: "flex gap-4 min-h-screen w-full",

                    // Left side bar
                    div { class: "w-min bg-gray-900 p-4 rounded shadow flex flex-col gap-4",
                        hr {}
                        ButtonTab { tabname: "⌨ Key mapping", tabkind: models::Tab::Keyboard, current_tab },
                        ButtonTab { tabname: "⚙ Macro keys", tabkind: models::Tab::MacroKey, current_tab },
                        ButtonTab { tabname: "⚙ Media keys", tabkind: models::Tab::MediaKey, current_tab },
                        ButtonTab { tabname: "⚙ Trackpoint", tabkind: models::Tab::Trackpoint, current_tab },
                        // ButtonTab { tabname: "⚙ Key matrix", tabkind: models::Tab::KeyMatrix, current_tab }
                        ButtonTab { tabname: "⚙ Other settings", tabkind: models::Tab::Others, current_tab }
                        hr {}
                        ButtonLoad {
                            selected_board_name,
                            selected_logical_layout_name,
                            id_layout_l0,
                            id_layout_l1,
                            fn_id,
                            single_mod_key_id,
                            trackpoint_speed_settings,
                            macro_key_map,
                            media_key_map,
                            enable_middle_click,
                        }
                        ButtonSave {
                            selected_board,
                            selected_logical_layout,
                            id_layout_l0,
                            id_layout_l1,
                            fn_id,
                            single_mod_key_id,
                            trackpoint_speed_settings,
                            macro_key_map,
                            media_key_map,
                            enable_middle_click,
                        }
                        ButtonInstall {
                            id_layout_l0,
                            id_layout_l1,
                            firmware_future,
                            fn_id,
                            single_mod_key_id,
                            trackpoint_speed_settings,
                            macro_key_map,
                            media_key_map,
                            enable_middle_click,
                            selected_board,
                            error_msg,
                        }
                    }

                    // Wrapper 
                    div { class: "gap-4 space-y-4 w-full min-h-screen",

                        // Board and Language select bar
                        div { class: "w-full bg-gray-900 p-4 rounded shadow flex flex-wrap items-end gap-4",
                            div { class: "flex flex-wrap items-center gap-4",
                                label { class: "text-gray-200", "Keyboard:" }
                                SelectBoard {
                                    general_setting: general_setting.clone(),
                                    selected_board_name,
                                    selected_logical_layout_name,
                                    selected_board,
                                }
                                div { class: "w-4" }
                                label { class: "text-gray-200", "Language:" }
                                SelectLogicalLayout {
                                    general_setting: general_setting.clone(),
                                    selected_logical_layout_name,
                                    selected_logical_layout,
                                }
                            }
                        }
                        
                        // Main content
                        div { class: "bg-gray-900 rounded flex flex-col p-4 min-h-screen gap-4 overflow-auto",
                            { match current_tab() {
                                    models::Tab::Keyboard => {
                                        rsx! {
                                            Keyboard {
                                                general_setting: general_setting.clone(),
                                                layer_number: 0,
                                                board: selected_board().clone(),
                                                logical_layout: selected_logical_layout().clone(),
                                                id_layout_l0,
                                                id_layout_l1,
                                            }
                                            Keyboard {
                                                general_setting: general_setting.clone(),
                                                layer_number: 1,
                                                board: selected_board().clone(),
                                                logical_layout: selected_logical_layout().clone(),
                                                id_layout_l0,
                                                id_layout_l1,
                                            }
                                            div { class: "flex items-center px-4",
                                                ButtonCopyLayer { id_layout_l0, id_layout_l1 }
                                            }
                                        }                                        
                                    },
                                    models::Tab::MacroKey => {
                                        rsx!{
                                            // div { class: "px-6 overflow-y-auto",
                                                MacroKeySetting {
                                                    general_setting: general_setting.clone(),
                                                    map_key_label: selected_logical_layout().map_key_label.clone(),
                                                    macro_key_map: macro_key_map.clone(),
                                                }
                                            // }
                                        }
                                    },
                                    models::Tab::MediaKey => {
                                        rsx!{
                                            // div { class: "px-6 overflow-y-auto",
                                                MediaKeySetting {
                                                    general_setting: general_setting.clone(),
                                                    media_key_map: media_key_map.clone(),
                                                }
                                            // }
                                        }
                                    },
                                    models::Tab::Trackpoint => {
                                        rsx!{
                                            div { class: "flex gap-12 px-4",
                                                div { class: "flex flex-col space-y-2 min-w-[600px]",
                                                    TrackPointSpeedPresetButtons { 
                                                        tp_data: trackpoint_speed_settings, 
                                                        default_coeffs: default_tp_accel_coeffs().coeffs.clone()
                                                    }
                                                    TrackPointSpeedChart {
                                                        tp_data: trackpoint_speed_settings,
                                                    }
                                                }
                                                div { class: "flex flex-col space-y-2",
                                                    TrackPointSpeedTableEditor {
                                                        tp_data: trackpoint_speed_settings,
                                                    }
                                                    label { class: "text-s text-gray-100 items-right",
                                                        "\"Speed 5\" will be used if the driver is not installed in OS (e.g. macOS)"
                                                    }
                                                }
                                            }

                                        }
                                    },
                                    models::Tab::KeyMatrix => {rsx!{}},
                                    models::Tab::Others => {
                                        rsx! {
                                            div { class: "grid grid-cols-[max-content,max-content] gap-x-6 gap-y-3 items-center",
                                                label { class: "text-lg", "● Fn / Media trigger:" }
                                                SelectFnID {
                                                    general_setting: general_setting.clone(),
                                                    fn_id,
                                                    map_key_label: selected_logical_layout().map_key_label.clone(),
                                                }

                                                label { class: "text-lg", "● Single 'Mod' key behavior:" }
                                                SelectSingleModKeyID {
                                                    general_setting: general_setting.clone(),
                                                    single_mod_key_id,
                                                    map_key_label: selected_logical_layout().map_key_label.clone(),
                                                }

                                                label { class: "text-lg", "● Force to disable Thinkpad preferred scrolling:" }
                                                input {
                                                    r#type: "checkbox",
                                                    class: "justify-self-start",
                                                    checked: enable_middle_click(),
                                                    onchange: move |evt| {
                                                        enable_middle_click.set(evt.checked());
                                                    },
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

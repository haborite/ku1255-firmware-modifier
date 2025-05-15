/// Define a components module that contains all shared components for our app.
mod components;
mod models;
mod utils;

use dioxus::prelude::*;
use components::{
    SelectBoard,
    SelectLogicalLayout,
    ButtonInstall,
    ButtonLoad,
    ButtonSave,
    ErrorMessage,
    Keyboard,
    SliderTPSensitivity,
};
use std::path::Path;
use std::io::Write;
use std::fs;

use models::{Board, LogicalLayout};
use utils::{
    load_url,
    load_general,
    load_boards,
    load_logical_layouts,
};

// The macro returns an `Asset` type that will display as the path to the asset in the browser or a local path in desktop bundles.
const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/styling/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

// Constants
const GENERAL_SETTING_PATH: &str = "settings/general_setting.csv";
const BOARDS_DIR:  &str = "boards";
const LOGICAL_LAYOUT_DIR:  &str = "logical_layouts";
const EXE_PATH: &str = "firmware/tp_compact_usb_kb_with_trackpoint_fw.exe";
const EXE_URL_SETTING_PATH: &str = "settings/url.txt";

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        MainWindow {}
    }
}

#[component]
pub fn MainWindow() -> Element {

    // Error message
    let exe_url = load_url(Path::new(EXE_URL_SETTING_PATH)).unwrap();
    let error_msg: Signal<Option<String>> = use_signal(|| None);

    // Firmware to be patched
    let firmware_future = use_resource( move || {
        let exe_url_cloned = exe_url.clone();
        async move {
            let firmware_path = Path::new(EXE_PATH);
            if firmware_path.exists() {
                println!("Firmware found at {}. Loading from disk...", EXE_PATH);
                return fs::read(firmware_path).unwrap_or_else(|err| {
                    eprintln!("Error reading firmware: {}", err);
                    vec![]
                });
            }
            println!("Firmware not found. Downloading from {}...", exe_url_cloned);
            match reqwest::get(exe_url_cloned).await {
                Ok(resp) => match resp.bytes().await {
                    Ok(bytes) => {
                        if let Err(err) = fs::File::create(firmware_path)
                            .and_then(|mut file| file.write_all(&bytes))
                        {
                            eprintln!("Failed to save firmware to {}: {}", EXE_PATH, err);
                        } else {
                            println!("Firmware downloaded and saved to {}", EXE_PATH);
                        }
                        bytes.to_vec()
                    }
                    Err(err) => {
                        eprintln!("Failed to read response body: {}", err);
                        vec![]
                    }
                },
                Err(err) => {
                    eprintln!("Failed to download firmware: {}", err);
                    vec![]
                }
            }
        }
    });

    // Paths
    let boards_dir = Path::new(BOARDS_DIR);
    let logical_layouts_dir = Path::new(LOGICAL_LAYOUT_DIR);
    let general_setting_path = Path::new(GENERAL_SETTING_PATH);

    // Load const data
    let (id_layout_original, id_list, usage_names) = load_general(general_setting_path).unwrap();

    // Config list
    let boards = load_boards(boards_dir, general_setting_path);
    let boards_cloned = boards.clone();
    let logical_layouts = load_logical_layouts(logical_layouts_dir, general_setting_path);
    let logical_layouts_cloned = logical_layouts.clone();

    // Board variables
    let selected_board_name = use_signal(|| boards_cloned.get(0).unwrap().board_name.clone() );
    let selected_board: Memo<Board> = use_memo(move || {
        boards_cloned.iter().find(|b| b.board_name == selected_board_name())
            .unwrap_or(boards_cloned.get(0).unwrap()).clone()
    });

    // Logical layout variables
    let selected_logical_layout_name = use_signal(|| {
        selected_board().default_logical_layout_name
    });
    let selected_logical_layout: Memo<LogicalLayout>  = use_memo(move || {
        logical_layouts_cloned.iter().find(|l| l.layout_name == selected_logical_layout_name())
            .unwrap_or(logical_layouts_cloned.get(0).unwrap()).clone()
    });

    // ID Layout variables
    let id_layout_l0 = use_signal(|| id_layout_original.clone());
    let mut id_layout_l1 = use_signal(|| id_layout_l0().clone());

    // TrackPoint sensitivity variables
    let tp_sensitivity = use_signal(|| 1 );

    rsx! {
        if let Some(msg) = error_msg() {
            ErrorMessage { msg, error_msg }
        }
        div { class: "max-w-min min-w-max flex flex-col p-4 space-y-4",
            div { class: "w-full bg-gray-700 p-4 rounded shadow flex space-x-4",
                label {"Keyboard: "}
                SelectBoard { selected_board_name, selected_logical_layout_name, selected_board, boards }
                label {"Language: "}
                SelectLogicalLayout { selected_logical_layout_name, selected_logical_layout, logical_layouts }
                div { class: "flex space-x-2 justify-end",
                    ButtonLoad { selected_board_name, selected_logical_layout_name, id_layout_l0, id_layout_l1, tp_sensitivity }
                    ButtonSave { selected_board, selected_logical_layout, id_layout_l0, id_layout_l1, tp_sensitivity }
                    ButtonInstall { id_layout_l0, id_layout_l1, firmware_future, tp_sensitivity, error_msg }
                }
            }
            div { class: "flex flex-1 space-x-4",
                div { class: "flex flex-col flex-1 space-y-4",
                    Keyboard {
                        layer_number: 0,
                        id_list: id_list.clone(),
                        usage_names: usage_names.clone(),
                        board: selected_board().clone(),
                        logical_layout: selected_logical_layout().clone(),
                        id_layout: id_layout_l0,
                        id_layout_original: id_layout_original.clone(),
                    }
                    Keyboard {
                        layer_number: 1,
                        id_list: id_list.clone(),
                        usage_names: usage_names.clone(),
                        board: selected_board().clone(),
                        logical_layout: selected_logical_layout().clone(),
                        id_layout: id_layout_l1,
                        id_layout_original: id_layout_l0().clone(),
                    }
                    div { class: "flex flex-1 space-x-4",
                        button {
                            class: "w-80 px-1 py-1 bg-gray-500 text-white rounded shadow hover:bg-gray-600",
                            onclick: move |_| {id_layout_l1.set(id_layout_l0().clone())},
                            "Copy from Main layer to 2nd layer"
                        }
                    }
                }
                div { class: "flex flex-col flex-1 space-y-6",
                    SliderTPSensitivity { tp_sensitivity }
                }            
            }
        }
        div {
            class: "p-4 space-y-4",
        }
    }
}
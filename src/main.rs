/// Define a components module that contains all shared components for our app.
mod components;
mod models;
mod utils;

use dioxus::prelude::*;
use components::{Keyboard, ErrorMessage};
use std::path::Path;
use std::io::Write;
use std::fs;
use std::process::Command;
use rfd::FileDialog;
use models::{Board, LogicalLayout};
use utils::{
    load_url,
    load_general,
    load_boards,
    load_logical_layouts,
    load_config,
    save_config,
    patch_firmware,
    extract_fw_bin,
};

// The macro returns an `Asset` type that will display as the path to the asset in the browser or a local path in desktop bundles.
const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/styling/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

// CONST
const GENERAL_SETTING_PATH: &str = "settings/general_setting.csv";
const BOARDS_DIR:  &str = "boards";
const LOGICAL_LAYOUT_DIR:  &str = "logical_layouts";
const EXE_PATH: &str = "firmware/tp_compact_usb_kb_with_trackpoint_fw.exe";
const EXE_URL_SETTING_PATH: &str = "settings/url.txt";
const MOD_EXE_PATH: &str = "firmware/mod_fw.exe";
const MOD_BIN_PATH: &str = "firmware/mod_fw.bin";
const FLASHER_PATH: &str = "firmware/flashsn8";

fn main() {
    // init_logger(Level::DEBUG).expect("failed to init logger");
    dioxus::launch(App);
}

/// Components should be annotated with `#[component]` to support props, better error messages, and autocomplete
#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        BoardSelector {}
    }
}

#[component]
pub fn BoardSelector() -> Element {

    // Error message
    let exe_url = load_url(Path::new(EXE_URL_SETTING_PATH)).unwrap();
    let mut error_msg: Signal<Option<String>> = use_signal(|| None);

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
    let mut selected_board_name = use_signal(|| boards_cloned.get(0).unwrap().board_name.clone() );
    let selected_board: Memo<Board> = use_memo(move || {
        boards_cloned.iter().find(|b| b.board_name == selected_board_name())
            .unwrap_or(boards_cloned.get(0).unwrap()).clone()
    });

    // Logical layout variables
    let mut selected_logical_layout_name = use_signal(|| {
        selected_board().default_logical_layout_name
    });
    let selected_logical_layout: Memo<LogicalLayout>  = use_memo(move || {
        logical_layouts_cloned.iter().find(|l| l.layout_name == selected_logical_layout_name())
            .unwrap_or(logical_layouts_cloned.get(0).unwrap()).clone()
    });

    // ID Layout variables
    let mut id_layout_l0 = use_signal(|| id_layout_original.clone());
    let mut id_layout_l1 = use_signal(|| id_layout_l0().clone());

    // TrackPoint sensitivity variables
    let mut tp_sensitivity = use_signal(|| 1 );

    rsx! {
        if let Some(msg) = error_msg() {
            ErrorMessage {msg, error_msg}
        }
        div { class: "max-w-min min-w-max flex flex-col p-4 space-y-4",
            div { class: "w-full bg-gray-700 p-4 rounded shadow flex space-x-4",
                label {"Keyboard: "}
                select {
                    style: format!("width: 250px;"),
                    class: "bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500",
                    id: "board-select",
                    value: selected_board_name,
                    onchange: move |evt| {
                        selected_board_name.set(evt.value());
                        selected_logical_layout_name.set(selected_board().default_logical_layout_name);
                    },
                    { boards.iter().map(|b|{
                        rsx!(option { value: b.board_name.clone(), label: b.board_label.clone() })
                    })}
                }
                label {"Language: "}
                select {
                    style: format!("width: 250px;"),
                    class: "bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500",
                    id: "board-select",
                    value: selected_logical_layout_name,
                    onmounted: move |_| {
                        selected_logical_layout_name.set(selected_logical_layout().layout_name)
                    },
                    onchange: move |evt| {
                        selected_logical_layout_name.set(evt.value());
                    },
                    { logical_layouts.iter().map(|l|{
                        rsx!(option { value: l.layout_name.clone(), label: l.layout_label.clone() })
                    })}
                }
                div { class: "flex space-x-2 justify-end",
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
                                        loaded_tp_sensitivity,
                                    )) = load_config(&path) {
                                        selected_board_name.set(loaded_board_name);
                                        selected_logical_layout_name.set(loaded_logical_layout_name);
                                        id_layout_l0.set(loaded_id_layout_l0);
                                        id_layout_l1.set(loaded_id_layout_l1);
                                        tp_sensitivity.set(loaded_tp_sensitivity);
                                    };
                                },
                                None => println!("file not selected"),
                            }
                        },
                        "Load config"
                    }
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
                                        tp_sensitivity(),
                                    );
                                },
                                None => println!("Cancel"),
                            }
                        },
                        "Save config"
                    }
                    button {
                        class: "px-4 py-2 bg-blue-500 text-white rounded shadow hover:bg-blue-600",
                        onclick: move |_| {
                            for (k, v) in id_layout_l0() {
                                if v == 231 {
                                    if id_layout_l1().get(&k).unwrap() != &231 {
                                        error_msg.set(Some(
                                            "The 'Mod' key position must be same on the Main and 2nd layers.".to_string()
                                        ));
                                        return;
                                    }
                                }
                            }
                            let Some(original_binary) = &*firmware_future.read_unchecked() else {
                                eprintln!("Original firmware binary is missing. Cannot apply patch.");
                                return;
                            };
                            let modified_exe_bin = match patch_firmware(original_binary, &id_layout_l0(), &id_layout_l1(), tp_sensitivity()) {
                                Ok(bin) => bin,
                                Err(err) => {
                                    eprintln!("Failed to modify firmware binary: {}", err);
                                    return;
                                }
                            };

                            if cfg!(target_os = "macos") || cfg!(target_os = "linux") {
                                let modified_fw_bin = extract_fw_bin(&modified_exe_bin);
                                if let Err(err) = fs::File::create(MOD_BIN_PATH)
                                    .and_then(|mut file| file.write_all(&modified_fw_bin))
                                {
                                    eprintln!("Failed to save modified firmware binary to {}: {}", MOD_BIN_PATH, err);
                                    return;
                                }
                                println!("Modified firmware binary successfully saved to {}", MOD_BIN_PATH);
                                let status = Command::new(FLASHER_PATH).arg(MOD_BIN_PATH).status();
                                match status {
                                    Ok(status) if status.success() => println!("flashsn8 completed successfully."),
                                    Ok(status) => eprintln!("flashsn8 failed with exit code: {}", status),
                                    Err(err) => eprintln!("Failed to execute flashsn8: {}", err),
                                }
                            } else if cfg!(target_os = "windows") {
                                if let Err(err) = fs::File::create(MOD_EXE_PATH)
                                    .and_then(|mut file| file.write_all(&modified_exe_bin))
                                {
                                    eprintln!("Failed to save modified firmware installer to {}: {}", MOD_EXE_PATH, err);
                                    return;
                                }
                                println!("Modified firmware installer successfully saved to {}", MOD_EXE_PATH);
                                match Command::new(MOD_EXE_PATH).spawn() {
                                    Ok(_) => println!("Launched modified firmware executable."),
                                    Err(err) => eprintln!("Failed to launch modified firmware: {}", err),
                                }
                            } else {
                                eprintln!("Unsupported OS.");
                            }
                        },
                        "Install firmware"              
                    }
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
                        /*
                        button {
                            class: "w-60 px-1 py-1 bg-gray-500 text-white rounded shadow hover:bg-gray-600",
                            onclick: move |_| {},
                            "Key rollover test"
                        }
                        */
                    }
                }
                div { class: "flex flex-col flex-1 space-y-6",
                    div {
                        class: "w-full max-w-md mx-auto p-6 space-y-6",
                        h2 { class: "text-xl font-bold text-center", "TrackPoint Speed" },
                        div {
                            class: "flex items-center justify-center space-x-8",
                            div {
                                class: "flex flex-col items-start",
                                input {
                                    r#type: "range",
                                    min: 1,
                                    max: 5,
                                    step: 1,
                                    value: tp_sensitivity,
                                    onchange: move |evt| {
                                        tp_sensitivity.set(u32::from_str_radix(&evt.value(), 10).unwrap());
                                    },
                                },
                            },
                            span {
                                class: "text-xl w-24 text-center",
                                {
                                    let n = tp_sensitivity();
                                    match n {
                                        1 => "1 (default)".to_string(),
                                        _ => n.to_string()
                                    }
                                }
                            }
                        }
                    },
                    // textarea {
                    //     class: "flex-1 p-2 rounded resize-none bg-gray-700",
                    //     readonly: true,
                    //     value: "console",
                    // }
                }            
            }
        }

        div {
            class: "p-4 space-y-4",
        }
    }
}
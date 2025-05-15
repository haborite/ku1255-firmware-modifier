use dioxus::prelude::*;
use std::collections::HashMap;
use rfd::FileDialog;
use crate::models::{Board, LogicalLayout};
use crate::utils::{
    install_firmware_by_flashsn8,
    install_firmware_by_lenovo_installer,
    load_config,
    save_config,
};

#[component]
pub fn ButtonInstall(
    id_layout_l0: Signal<HashMap<u32, u8>>,
    id_layout_l1: Signal<HashMap<u32, u8>>,
    firmware_future: Resource<Vec<u8>>,
    tp_sensitivity: Signal<u32>,
    error_msg: Signal<Option<String>>,
) -> Element {

    // Install button submenu state
    let mut show_install_menu = use_signal(|| false);

    rsx! {
        div { class: "relative inline-flex",
            button {
                class: "px-4 py-2 bg-blue-500 text-white rounded-l shadow hover:bg-blue-600",
                onclick: move |_| {
                    if cfg!(target_os = "windows") {
                        install_firmware_by_lenovo_installer(
                            &id_layout_l0, &id_layout_l1, &firmware_future, &tp_sensitivity, &mut error_msg,    
                        );
                    } else {
                        install_firmware_by_flashsn8(
                            &id_layout_l0, &id_layout_l1, &firmware_future, &tp_sensitivity, &mut error_msg,    
                        );
                    }
                },
                "Install firmware"              
            }
            button { 
                class: "px-2 py-2 bg-blue-500 text-white rounded-r shadow hover:bg-blue-600 flex items-center",
                onclick: move |_| { show_install_menu.set(!show_install_menu()); },
                "▼"
            }
            {
                show_install_menu().then(|| rsx!(
                    InstallSubMenu {
                        show_install_menu,
                        id_layout_l0,
                        id_layout_l1,
                        firmware_future,
                        tp_sensitivity,
                        error_msg,
                    }
                ))
            }
        }
    }
}

#[component]
fn InstallSubMenu(
    show_install_menu: Signal<bool>,
    id_layout_l0: Signal<HashMap<u32, u8>>,
    id_layout_l1: Signal<HashMap<u32, u8>>,
    firmware_future: Resource<Vec<u8>>,
    tp_sensitivity: Signal<u32>,
    error_msg: Signal<Option<String>>,
) -> Element {

    rsx! {
        div {
            class: "absolute right-0 mt-12 w-48 bg-white border border-gray-200 rounded shadow-lg z-10",
            ul { class: "text-sm text-gray-700",
                li {
                    div {
                        class: "block px-4 py-2 hover:bg-gray-100",
                        onclick: move |_| {
                            show_install_menu.set(!show_install_menu());
                            install_firmware_by_lenovo_installer(
                                &id_layout_l0, &id_layout_l1, &firmware_future, &tp_sensitivity, &mut error_msg,    
                            );
                        },
                        "by Lenovo installer"
                    }
                }
                li {
                    div {
                        class: "block px-4 py-2 hover:bg-gray-100",
                        onclick: move |_| {
                            show_install_menu.set(!show_install_menu());
                            install_firmware_by_flashsn8(
                                &id_layout_l0, &id_layout_l1, &firmware_future, &tp_sensitivity, &mut error_msg,    
                            );
                        },
                        "by FlashSN8"
                    }
                }
            }
        }
    }

}

#[component]
pub fn ButtonLoad(
    selected_board_name: Signal<String>,
    selected_logical_layout_name: Signal<String>,
    id_layout_l0: Signal<HashMap<u32, u8>>,
    id_layout_l1: Signal<HashMap<u32, u8>>,
    tp_sensitivity: Signal<u32>,
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
    }
}

#[component]
pub fn ButtonSave(
    selected_board: ReadOnlySignal<Board>,
    selected_logical_layout: Memo<LogicalLayout>,
    id_layout_l0: ReadOnlySignal<HashMap<u32, u8>>,
    id_layout_l1: ReadOnlySignal<HashMap<u32, u8>>,
    tp_sensitivity: ReadOnlySignal<u32>,
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
                            tp_sensitivity(),
                        );
                    },
                    None => println!("Cancel"),
                }
            },
            "Save config"
        }
    }
}
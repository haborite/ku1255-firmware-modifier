use std::sync::Arc;

mod components;
mod models;
mod utils;

use dioxus::prelude::*;
use components::{
    SelectBoard, SelectLogicalLayout, ButtonCopyLayer, ButtonInstall, ButtonLoad,
    ButtonSave, ErrorMessage, Keyboard, SliderTPSensitivity, SelectFnID,
    MacroKeySetting, MediaKeySetting, EnableMiddleClick
};

use models::{ GeneralSeitting, Config };
use utils::{load_or_download_firmware_installer};

// Assets
const FAVICON: Asset = asset!("/public/favicon.ico");
const MAIN_CSS: Asset = asset!("/public/styling/main.css");
const TAILWIND_CSS: Asset = asset!("/public/tailwind.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {

    // General setting 
    let general_setting = GeneralSeitting::load_from_files().unwrap();
    let general_setting = Arc::new(general_setting);

    // User-defined config
    let config = use_store(||{
        match Config::create_default() {
            Ok(config) => config,
            Err(err) => {
                eprintln!("Failed to load default config. {}", err);
                Config::create_from_file_dialog()
            }
        }
    });

    // Firmware to be patched
    let general_setting_cloned = general_setting.clone();
    let fw_installer_future = use_resource({move || {
        let exe_url_cloned = general_setting_cloned.official_firmware_url.clone();
        async move {
            load_or_download_firmware_installer(&exe_url_cloned).await
        }
    }});

    // Error message
    let error_msg: Signal<Option<String>> = use_signal(|| None);

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        MainWindow {
            general_setting: general_setting.clone(),
            config,
            fw_installer_future,
            error_msg
        }
    }
}

#[component]
pub fn MainWindow(
    general_setting: Arc<GeneralSeitting>,
    config: Store<Config>,
    fw_installer_future: Resource<Vec<u8>>,
    error_msg: Signal<Option<String>>
) -> Element {

    rsx! {

        if let Some(msg) = error_msg() {
            ErrorMessage { msg, error_msg }
        }

        div { class: "min-h-screen bg-gray-600 text-slate-100",
            div { class: "mx-auto w-full p-4 space-y-4",

                div { class: "w-full bg-gray-800 p-4 rounded shadow flex flex-wrap items-end gap-4",
                    div { class: "flex flex-wrap items-center gap-2",
                        SelectBoard { general_setting: general_setting.clone(), config }
                        SelectLogicalLayout { general_setting: general_setting.clone(), config }
                    }
                    div { class: "flex items-center gap-2 ml-auto",
                        ButtonCopyLayer { config }
                        ButtonLoad { config }
                        ButtonSave { config }
                        ButtonInstall { config, fw_installer_future, error_msg }
                    }
                }

                div { class: "flex gap-4",
                    div { class: "bg-black rounded flex flex-col",
                        Keyboard { general_setting: general_setting.clone(), config, layer_number: 0 }
                        Keyboard { general_setting: general_setting.clone(), config, layer_number: 1 }
                    }
                    div { class: "flex flex-col gap-4",
                        div { class: "flex gap-4",
                            div { class: "bg-black rounded flex flex-1", 
                                SliderTPSensitivity { config }
                            }
                            div { class: "bg-black rounded flex flex-1",
                                EnableMiddleClick { config }
                            }
                        }
                        div { class: "flex flex-col bg-black rounded max-h-[calc(100vh-282px)]",
                            h2 { class: "text-xl font-bold text-center py-2", "Macro keys" },
                            div { class: "px-6 overflow-y-auto",
                                MacroKeySetting { general_setting: general_setting.clone(), config }
                            }
                        }
                    }
                    div { class: "flex flex-col gap-4", 
                        div { class: "px-6 bg-black rounded pb-6",
                            SelectFnID { general_setting: general_setting.clone(), config }
                        }
                        div { class: "flex flex-col bg-black rounded max-h-[calc(100vh-220px)]",
                            h2 { class: "text-xl font-bold text-center py-2", "Media keys" },
                            div { class: "px-6 overflow-y-auto",
                                MediaKeySetting { general_setting: general_setting.clone(), config }
                            }
                        }
                    }
                }
            }
        }
    }
}

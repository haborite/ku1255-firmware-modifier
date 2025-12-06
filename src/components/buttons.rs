use dioxus::prelude::*;
use rfd::FileDialog;
use crate::models::{KeyboardSpec, UserConfig};
use crate::utils::{
    install_firmware_by_flashsn8,
};

#[component]
pub fn ButtonInstall(
    keyboard_spec: ReadOnlySignal<KeyboardSpec>,
    user_config: ReadOnlySignal<UserConfig>,
    firmware_future: Resource<Vec<u8>>,
    error_msg: Signal<Option<String>>,
) -> Element {

    rsx! {
        div { class: "relative inline-flex",
            button {
                class: "px-4 py-2 bg-blue-500 text-white rounded-l shadow hover:bg-blue-600",
                onclick: move |_| {
                    install_firmware_by_flashsn8(
                        user_config, firmware_future, error_msg,    
                    );
                },
                "Install firmware"              
            }
        }
    }
}


#[component]
pub fn ButtonLoad(
    keyboard_spec: ReadOnlySignal<KeyboardSpec>,
    user_config: Signal<UserConfig>
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
                println!("{:?}", file);
                match file {
                    Some(path) => {
                        println!("Config file selected: {}", path.display());
                        let user_config_mut = &mut user_config.write();
                        if let Err(e) = user_config_mut.update_from_file(&path) {
                            eprintln!("Failed to load config file: {e}");
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
    keyboard_spec: ReadOnlySignal<KeyboardSpec>,
    user_config: Signal<UserConfig>
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
                        println!("Config file to be saved: {}", path.display());
                        let _ = user_config.read().save_to_file(&path);
                    },
                    None => println!("Cancel"),
                }
            },
            "Save config"
        }
    }
}
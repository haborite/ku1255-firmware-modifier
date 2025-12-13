use dioxus::prelude::*;
use dioxus::prelude::{Signal, Resource, ReadableExt, WritableExt};
use crate::models::Config;

#[component]
pub fn ButtonCopyLayer( config: WriteStore<Config> ) -> Element {
    rsx! {
        button {
            class: "px-4 py-2 bg-gray-500 text-white rounded shadow hover:bg-gray-600",
            onclick: move |_| config.write().copy_layer0_to_layer1() ,
            "Copy layer from Main to 2nd"
        }
    }
}

#[component]
pub fn ButtonLoad( config: WriteStore<Config> ) -> Element {
    rsx! {
        button {
            class: "px-4 py-2 bg-green-500 text-white rounded shadow hover:bg-green-600",
            onclick: move |_| config.write().load_from_file_dialog(),
            "Load config"
        }
    }
}

#[component]
pub fn ButtonSave( config: ReadStore<Config> ) -> Element {
    rsx! {
        button {
            class: "px-4 py-2 bg-green-500 text-white rounded shadow hover:bg-green-600",
            onclick: move |_| config.read().save_to_file_dialog(),
            "Save config"
        }
    }
}

#[component]
pub fn ButtonInstall(
    config: ReadStore<Config>,
    fw_installer_future: Resource<Vec<u8>>,
    error_msg: Signal<Option<String>>,
) -> Element {
    rsx! {
        button { class: "px-4 py-2 bg-blue-500 text-white rounded shadow hover:bg-blue-600",
            onclick: move |_| config.read().install_firmware(fw_installer_future, &mut error_msg),
            "Install firmware"              
        }
    }
}

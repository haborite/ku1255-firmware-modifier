mod components;
mod models;
mod utils;

use dioxus::prelude::*;
use components::{
    SelectPhysicalLayout,
    SelectLogicalLayout,
    ButtonInstall,
    ButtonLoad,
    ButtonSave,
    ErrorMessage,
    Keyboard,
    SliderTPSensitivity,
    SelectFnKeyID,
    MacroKeySetting,
    MediaKeySetting,
};

use models::{KeyboardSpec, UserConfig};
use utils::load_or_download_firmware;

// Assets
const FAVICON: Asset = asset!("/public/favicon.ico");
const MAIN_CSS: Asset = asset!("/public/styling/main.css");
const TAILWIND_CSS: Asset = asset!("/public/tailwind.css");

// Default User Config Path
const DEFAULT_USER_CONFIG_PATH: &str = "examples/__default__.json";


fn main() {

    println!("LAUNCHED Main");
    dioxus::launch(App);
}

#[component]
fn App() -> Element {

    println!("LAUNCHED App");

    let keyboard_spec = KeyboardSpec::load_from_files().unwrap();
    let default_user_config = UserConfig::load_from_file(DEFAULT_USER_CONFIG_PATH).unwrap_or(UserConfig::default());

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        MainWindow { keyboard_spec, default_user_config }
    }

}

#[component]
pub fn MainWindow(
    keyboard_spec: KeyboardSpec,
    default_user_config: UserConfig,
) -> Element {

    println!("LAUNCHED MainWindow");

    // Keyboard spec signal
    let keyboard_spec = use_signal(|| keyboard_spec);

    // User config signal
    let mut user_config = use_signal(|| default_user_config);

    // Error message
    let error_msg: Signal<Option<String>> = use_signal(|| None);

    // Firmware to be patched
    let firmware_future = use_resource({move || {
        println!("Getting future");
        let exe_url = keyboard_spec.read().official_firmware_url.clone();
        async move {
            load_or_download_firmware(&exe_url).await
        }
    }});

    // Selected board
    // let selected_name = use_signal(|| keyboard_spec.read().avail_physical_layouts.get(0).unwrap().name.clone() );
    // let selected_board: Memo<PhysicalLayout> = use_memo(move || {
    //     keyboard_spec.read().avail_physical_layouts.iter().find(|b| b.name == selected_name())
    //         .unwrap_or(keyboard_spec.read().avail_physical_layouts.get(0).unwrap()).clone()
    // });

    // Selected logical layout
    // let selected_logical_layout_name = use_signal(|| {selected_board().default_logical_layout_name});
    // let selected_logical_layout: Memo<LogicalLayout>  = use_memo(move || {
    //     keyboard_spec.read().avail_logical_layouts.iter().find(|l| l.layout_name == selected_logical_layout_name())
    //         .unwrap_or(keyboard_spec.read().avail_logical_layouts.get(0).unwrap()).clone()
    // });

    rsx! {
        if let Some(msg) = error_msg() {
            ErrorMessage { msg, error_msg }
        }
        div { class: "max-w-min min-w-max flex flex-col p-4 space-y-4",
            div { class: "w-full bg-gray-700 p-4 rounded shadow flex space-x-4",
                label {"Keyboard: "}
                // SelectPhysicalLayout { selected_name, selected_logical_layout_name, selected_board, keyboard_spec }
                SelectPhysicalLayout { keyboard_spec, user_config }
                label {"Language: "}
                // SelectLogicalLayout { selected_logical_layout_name, selected_logical_layout, keyboard_spec }
                SelectLogicalLayout { keyboard_spec, user_config }
                div { class: "flex space-x-2 justify-end",
                    // ButtonLoad { selected_name, selected_logical_layout_name, id_layout_l0, id_layout_l1, fn_id, tp_sensitivity, macro_key_map, app_key_map }
                    // ButtonSave { selected_board, selected_logical_layout, id_layout_l0, id_layout_l1, fn_id, tp_sensitivity, macro_key_map, app_key_map }
                    // ButtonInstall { id_layout_l0, id_layout_l1, firmware_future, fn_id, tp_sensitivity, error_msg }
                    ButtonLoad    { keyboard_spec, user_config }
                    ButtonSave    { keyboard_spec, user_config }
                    ButtonInstall { keyboard_spec, user_config, firmware_future, error_msg }
                }
            }
            div { class: "flex flex-1 space-x-4",
                div { class: "flex flex-col flex-1 space-y-4",
                    Keyboard { layer_number: 0, user_config, keyboard_spec }
                    Keyboard { layer_number: 1, user_config, keyboard_spec }
                    div { class: "flex flex-1 space-x-4",
                        button {
                            class: "w-80 px-1 py-1 bg-gray-500 text-white rounded shadow hover:bg-gray-600",
                            onclick: move |_| { user_config.write().copy_layer0_to_layer1(); },
                            "Copy from Main layer to 2nd layer"
                        }
                    }
                }
                div { class: "flex flex-col flex-1 space-y-6",
                    SliderTPSensitivity { user_config }
                    SelectFnKeyID { keyboard_spec, user_config }
                    //     id_list,
                    //     usage_names,
                    //     key_id: fn_id,
                    //     map_key_label: selected_logical_layout().clone().map_key_label,
                    // }
                }      
            }
            div {
                MacroKeySetting { keyboard_spec, user_config }
            }
            div {
                MediaKeySetting { keyboard_spec, user_config }
            }
        }
        div {
            class: "p-4 space-y-4",
        }
    }
}
use dioxus::prelude::*;
use crate::models::{KeyboardSpec, UserConfig};

/// Application Key Setting Component
/// Receives:
/// id_list: Vec<u8>: Key ID list
/// usage_names: Vec<String>: Usage names corresponding to key IDs
/// trigger_ids: Vec<u8>: Trigger key IDs for application keys
/// application_keys: HashMap<u8, ApplicationKey>: How key ids (u8) are converted to application keys
/// List of slect box for application key
#[component]
pub fn MediaKeySetting(
    keyboard_spec: ReadOnlySignal<KeyboardSpec>,
    user_config: Signal<UserConfig>
) -> Element {
    rsx! {
        div {
            class: "flex flex-col space-y-4",
            {
                user_config.read().media_key_map.keys().map(|&trigger_key_id| {
                    rsx!(
                        div {
                            class: "flex items-center space-x-4",
                            label {
                                class: "w-32",
                                r#for: format!("Trigger Key ID: {:02X}", trigger_key_id)
                            }
                            SelectAppKeyID { trigger_key_id, keyboard_spec, user_config }
                        }
                    )
                })
            }
        }
    }
}

#[component]
pub fn SelectAppKeyID(
    trigger_key_id: u8,
    keyboard_spec: ReadOnlySignal<KeyboardSpec>,
    user_config: Signal<UserConfig>
) -> Element {
    rsx!{
        div {
            class: "w-full max-w-md mx-auto p-6 space-y-6",
            h2 { class: "text-xl font-bold text-center", "Function key" },
            select {
                class: "w-full p-2 border border-gray-300 rounded mb-4 text-gray-700",
                id: "options",
                value: keyboard_spec.read().get_media_key_usage_name(
                    user_config.read().get_media_key_id(trigger_key_id)
                ),
                onchange: move |evt| {
                    let new_id: u16 = evt.value().clone().parse().unwrap();
                    user_config.write().update_media_key_map(trigger_key_id, new_id);
                }
            }
        }
    }
}

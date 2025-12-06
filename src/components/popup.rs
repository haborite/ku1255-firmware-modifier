use dioxus::prelude::*;
use std::collections::HashMap;
use crate::models::{KeyLabel, UserConfig, KeyboardSpec};
use std::rc::Rc;

#[component]
pub fn Popup(
    layer_number: u8,
    user_config: Signal<UserConfig>,
    keyboard_spec: ReadOnlySignal<KeyboardSpec>,
    selected_address: Signal<Option<u32>>,
    map_key_label: ReadOnlySignal<HashMap::<u8, KeyLabel>>,
) -> Element {

    let mut input_ref = use_signal::<Option<Rc<MountedData>>>(|| None);
    use_effect(move || {
        if let Some(input) = input_ref.read().clone() {
            spawn(async move {
                _ = input.set_focus(true).await;
            });
        }
    });

    let id_layout = user_config.read().get_id_layout(layer_number).clone();
    let avail_hid_usage_names = keyboard_spec.read().avail_hid_usage_names.clone();
    let key_labels = map_key_label();

    rsx! {
        { if let Some(key_address) = selected_address() {
            let selected_id = id_layout.get(&selected_address().unwrap_or(0)).copied().unwrap_or(0);
            rsx! {
                div {
                    class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
                    id: "overlay",                    
                    div { 
                        class: "bg-white p-6 rounded-lg shadow-lg w-80",
                        label {
                            onmounted: move |cx| {input_ref.set(Some(cx.data()))},
                            class: "block mb-2 text-sm font-medium text-gray-700",
                            r#for: "options",
                            "Select a key to be assigned for {key_address}: "
                        }
                        select {
                            class: "w-full p-2 border border-gray-300 rounded mb-4 text-gray-700",
                            id: "options",
                            value: selected_id,
                            onchange: move |evt| {
                                let new_id: u8 = evt.value().clone().parse().unwrap();
                                user_config.write().update_layer(layer_number, key_address, new_id);
                                // let new_id: u8 = evt.value().clone().parse().unwrap();
                                // let mut id_layout_clone = id_layout().clone();
                                // id_layout_clone.insert(key_address, new_id);
                                // id_layout.set(id_layout_clone);
                                selected_address.set(None);
                            },
                            {
                                avail_hid_usage_names
                                    .iter()
                                    .map(|(&kid, name)| {
                                        let (label, class) = match key_labels.get(&kid) {
                                            Some(ks) if !ks.default.is_empty() => {
                                                let label = if ks.shifted.is_empty() {
                                                    ks.default.clone()
                                                } else {
                                                    format!("{} and {}", ks.default, ks.shifted)
                                                };
                                                (label, "text-gray-700")
                                            }
                                            _ => (
                                                format!("{{ {:02X}: {} }}", kid, name),
                                                "text-gray-400",
                                            ),
                                        };

                                        rsx! {
                                            option {
                                                class: class,
                                                value: kid,
                                                label: label,
                                                selected: kid == selected_id,
                                            }
                                        }
                                    })
                            }
                        }
                        button {
                            class: "w-full bg-red-600 text-white py-2 rounded hover:bg-red-700",
                            id: "submitBtn",
                            onclick: move |_evt| {
                                selected_address.set(None);
                            },
                            "Cancel"
                        }
                    }
                }
            }
        } else {
            rsx!{}
        }}
    }
}

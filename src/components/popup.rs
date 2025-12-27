use dioxus::prelude::*;
use std::collections::BTreeMap;
use crate::models::{GeneralSeitting, KeyLabel};
use std::rc::Rc;
use std::sync::Arc;

#[component]
pub fn Popup(
    general_setting: Arc<GeneralSeitting>,
    layer_number: u8,
    selected_address: Signal<Option<u8>>,
    id_layout_l0: Signal<BTreeMap<u8, Option<u8>>>,
    id_layout_l1: Signal<BTreeMap<u8, Option<u8>>>,
    map_key_label: BTreeMap::<u8, KeyLabel>,
) -> Element {

    let mut id_layout = { if layer_number == 0 { id_layout_l0 } else { id_layout_l1 } };

    let mut input_ref = use_signal::<Option<Rc<MountedData>>>(|| None);
    use_effect(move || {
        if let Some(input) = input_ref.read().clone() {
            spawn(async move {
                _ = input.set_focus(true).await;
            });
        }
    });

    rsx! {
        { if let Some(key_address) = selected_address() {
            let selected_id = id_layout().get(&selected_address().unwrap_or(0)).copied().unwrap_or(None);
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
                                let mut id_layout_mut = id_layout.write();
                                id_layout_mut.insert(key_address, Some(new_id));
                                if (new_id == 231) && (layer_number == 0) {
                                    let mut id_layout_l1_mut = id_layout_l1.write();
                                    id_layout_l1_mut.insert(key_address, Some(new_id));
                                }
                                selected_address.set(None);
                            },
                            {
                                general_setting.avail_hid_usage_names.iter().map(|(key_id, usage_name)|{
                                    let (label, style) = match map_key_label.get(&key_id) {
                                        None => ("".to_string(), "text-gray-700".to_string()),
                                        Some(ks) => {
                                            if ks.default == "" {
                                                (
                                                    format!("{{ {:02X}: {} }}", key_id, usage_name),
                                                    "text-gray-400".to_string()
                                                )                                                
                                            } else { 
                                                if ks.shifted == "" {
                                                    (
                                                        format!("{}", ks.default),
                                                        "text-gray-700".to_string()
                                                    )
                                                } else {
                                                    (
                                                        format!("{} and {}", ks.default, ks.shifted),
                                                        "text-gray-700".to_string()
                                                    )
                                                }
                                            }
                                        },
                                    };
                                    let selected_flag = if Some(*key_id) == selected_id {true} else {false};
                                    rsx!(
                                        option {
                                            class: style,
                                            value: *key_id,
                                            label: label,
                                            selected: selected_flag,
                                        }
                                    )                                   
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

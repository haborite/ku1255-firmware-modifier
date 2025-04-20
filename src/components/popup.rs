use dioxus::prelude::*;
use std::collections::HashMap;
use crate::models::KeyLabel;
use std::rc::Rc;

#[component]
pub fn Popup(
    selected_address: Signal<Option<u32>>,
    id_list: Vec<u8>,
    usage_names: Vec<String>,
    id_layout: Signal<HashMap<u32, u8>>,
    map_key_label: HashMap::<u8, KeyLabel>,
) -> Element {

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
            // let selecte_id = use_signal(|| id_layout().get(&selected_address().unwrap_or(0)).copied().unwrap_or(0));
            let selecte_id = id_layout().get(&selected_address().unwrap_or(0)).copied().unwrap_or(0);
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
                            value: selecte_id,
                            // onmounted: move |cx| {input_ref.set(Some(cx.data()))},
                            onchange: move |evt| {
                                let new_id: u8 = evt.value().clone().parse().unwrap();
                                let mut id_layout_clone = id_layout().clone();
                                id_layout_clone.insert(key_address, new_id);
                                id_layout.set(id_layout_clone);
                                selected_address.set(None);
                            },
                            {
                                id_list.into_iter().enumerate().map(|(idx, kid)|{
                                    let (label, style) = match map_key_label.get(&kid) {
                                        None => ("".to_string(), "text-gray-700".to_string()),
                                        Some(ks) => {
                                            if ks.default == "" {
                                                (
                                                    format!("{{ {:02X}: {} }}", kid, usage_names[idx]),
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
                                    let selected_flag = if kid == selecte_id {true} else {false};
                                    rsx!(
                                        option {
                                            class: style,
                                            value: kid,
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

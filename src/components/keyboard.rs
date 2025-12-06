use dioxus::prelude::*;
use std::sync::Arc;
use std::collections::BTreeMap;
use crate::models::{Board, GeneralSeitting, KeyLabel, LogicalLayout};
use crate::components::Popup;

#[component]
pub fn Keyboard(
    general_setting: Arc<GeneralSeitting>,
    layer_number: u8,
    // id_list: Vec<u8>,
    // usage_names: Vec<String>,
    board: Board,
    logical_layout: LogicalLayout,
    id_layout: Signal<BTreeMap<u32, u8>>,
    // id_layout_original: BTreeMap<u32, u8>,
) -> Element {
    
    let mut selected_address = use_signal(|| None as Option<u32>);

    rsx! {
        div { class: "p-4 space-y-4",
            div { class: "text-xl font-bold", { if layer_number == 0 { "Main Layer" } else { "2nd Layer" } }
            }
            div { 
                class: "space-y-1",
                {   
                    let al = board.map_address.clone();
                    let wl = board.map_widths.clone();
                    al.into_iter().zip(wl.into_iter()).map(|row| rsx! {
                    div { class: "flex space-x-1",
                        { row.0.into_iter().zip(row.1.into_iter()).map(|(add_opt, width)| {
                            let width_str = width.to_string();
                            let (id_opt, id_opt_org) = match add_opt {
                                Some(address) => (
                                    id_layout().get(&address).copied(),
                                    general_setting.initial_id_map.get(&address).copied()
                                ),
                                None => (None, None),
                            };
                            let key_label = match &id_opt {
                                Some(kid) => {
                                    match logical_layout.map_key_label.get(&kid) {
                                        Some(kl) => kl.clone(),
                                        None => KeyLabel::new(),
                                    }
                                },
                                None => KeyLabel::new(),
                            };
                            let key_default = key_label.default.clone();
                            let key_shifted = key_label.shifted.clone();

                            if let Some(kid) = id_opt {
                                let border_color = match kid {
                                    0   => "border-gray-500",
                                    231 => "border-rose-300",
                                    _   => { if kid == id_opt_org.unwrap() {""} else { if layer_number == 0 {"border-sky-300"} else {"border-green-300"}}}
                                };
                                let text_color = match kid {
                                    0   => "text-gray-500",
                                    231 => "text-rose-300",
                                    _   => { if kid == id_opt_org.unwrap() {""} else { if layer_number == 0 {"text-sky-300"} else {"text-green-300"}}}
                                };
                                rsx! {
                                    button {
                                        style: format!("width: {}px;", width_str),
                                        class: format!(
                                            "border {} px-2 py-1 h-10 text-xs flex flex-col items-center justify-center text-center break-words whitespace-normal text-[10px] leading-tight hover:bg-gray-600",
                                            border_color
                                        ),
                                        onclick: move |_| selected_address.set(add_opt),
                                        {   
                                            if key_shifted != "" {
                                                rsx! {
                                                    span { class: format!("text-gray-500 text-[10px]"), "{key_shifted}" }
                                                    span { class: format!("{}", text_color), "{key_default}" }
                                                }
                                            } else {
                                                rsx! {
                                                    span { class: format!("{}", text_color), "{key_default}" }
                                                }
                                            }
                                        }
                                    }
                                }
                            } else {
                                rsx! {
                                    button {
                                        style: format!("width: {}px;", width_str),
                                        class: format!(
                                            "invisible border px-2 py-1 h-10 flex flex-col items-center justify-center"
                                        )
                                    }
                                }
                            }

                        })}
                    }})
                }
            }
            Popup {
                general_setting,
                selected_address,
                id_layout,
                map_key_label: logical_layout.map_key_label,
            }
        }
    }
}

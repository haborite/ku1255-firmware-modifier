use dioxus::prelude::*;
use std::collections::HashMap;
use crate::models::{Board, LogicalLayout, KeyLabel};
use crate::components::Popup;

#[component]
pub fn Keyboard(
    layer_number: u8,
    id_list: Vec<u8>,
    usage_names: Vec<String>,
    board: Board,
    logical_layout: LogicalLayout,
    id_layout: Signal<HashMap<u32, u8>>,
    id_layout_original: HashMap<u32, u8>,
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
                                    id_layout_original.get(&address).copied()
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
                                let border_color = if kid == 231 {"border-rose-300"} else {
                                    if kid == id_opt_org.unwrap() {""} else {
                                        if layer_number == 0 {"border-sky-300"} else {"border-green-300"}
                                    }
                                };
                                let text_color = if kid == 231 {"text-rose-300"} else {
                                    if kid == id_opt_org.unwrap() {""} else {
                                        if layer_number == 0 {"text-sky-300"} else {"text-green-300"}
                                    }
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
                selected_address,
                id_list,
                usage_names,
                id_layout,
                map_key_label: logical_layout.map_key_label,
            }
        }
    }
}

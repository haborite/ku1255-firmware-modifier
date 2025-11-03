use dioxus::prelude::*;
use crate::models::{Board, LogicalLayout, KeyLabel};
use std::collections::HashMap;

#[component]
pub fn SelectBoard(
    selected_board_name: Signal<String>,
    selected_logical_layout_name: Signal<String>,
    selected_board: Memo<Board>,
    boards: Vec<Board>,
) -> Element {
    rsx!{
        select {
            style: format!("width: 250px;"),
            class: "bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500",
            id: "board-select",
            value: selected_board_name,
            onchange: move |evt| {
                selected_board_name.set(evt.value());
                selected_logical_layout_name.set(selected_board().default_logical_layout_name);
            },
            { boards.iter().map(|b|{
                rsx!(option { value: b.board_name.clone(), label: b.board_label.clone() })
            })}
        }
    }
}

#[component]
pub fn SelectLogicalLayout(
    selected_logical_layout_name: Signal<String>,
    selected_logical_layout: Memo<LogicalLayout>,
    logical_layouts: Vec<LogicalLayout>,
) -> Element {
    rsx!{
        select {
            style: format!("width: 250px;"),
            class: "bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500",
            id: "board-select",
            value: selected_logical_layout_name,
            onmounted: move |_| {
                selected_logical_layout_name.set(selected_logical_layout().layout_name)
            },
            onchange: move |evt| {
                selected_logical_layout_name.set(evt.value());
            },
            { logical_layouts.iter().map(|l|{
                rsx!(option { value: l.layout_name.clone(), label: l.layout_label.clone() })
            })}
        }
    }
}



#[component]
pub fn SelectFnID(
    id_list: Vec<u8>,
    usage_names: Vec<String>,
    fn_id: Signal<u8>,
    map_key_label: HashMap::<u8, KeyLabel>,
) -> Element {
    rsx!{
        div {
            class: "w-full max-w-md mx-auto p-6 space-y-6",
            h2 { class: "text-xl font-bold text-center", "Function key" },
            select {
                class: "w-full p-2 border border-gray-300 rounded mb-4 text-gray-700",
                id: "options",
                value: fn_id(),
                onchange: move |evt| {
                    let new_id: u8 = evt.value().clone().parse().unwrap();
                    fn_id.set(new_id);
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
                        let selected_flag = if kid == fn_id() {true} else {false};
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
        }
    }
}
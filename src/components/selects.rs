use dioxus::prelude::*;
use std::sync::Arc;
use crate::models::{Board, GeneralSeitting, KeyLabel, LogicalLayout};
use std::collections::BTreeMap;

#[component]
pub fn SelectBoard(
    general_setting: Arc<GeneralSeitting>,
    selected_board_name: Signal<String>,
    selected_logical_layout_name: Signal<String>,
    selected_board: Memo<Board>,
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
            { general_setting.avail_boards.iter().map(|b|{
                rsx!(option { value: b.board_name.clone(), label: b.board_label.clone() })
            })}
        }
    }
}

#[component]
pub fn SelectLogicalLayout(
    general_setting: Arc<GeneralSeitting>,
    selected_logical_layout_name: Signal<String>,
    selected_logical_layout: Memo<LogicalLayout>,
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
            { general_setting.avail_logical_layouts.iter().map(|l|{
                rsx!(option { value: l.layout_name.clone(), label: l.layout_label.clone() })
            })}
        }
    }
}



#[component]
pub fn SelectFnID(
    general_setting: Arc<GeneralSeitting>,
    fn_id: Signal<u8>,
    map_key_label: BTreeMap::<u8, KeyLabel>,
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
                        let selected_flag = if *key_id == fn_id() {true} else {false};
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
        }
    }
}
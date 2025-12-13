use dioxus::prelude::*;
use std::sync::Arc;
use crate::models::{Config, ConfigStoreExt, GeneralSeitting};

#[component]
pub fn SelectBoard(
    general_setting: Arc<GeneralSeitting>,
    config: Store<Config>,
) -> Element {
    rsx!{
        label { class: "text-sm text-gray-200", "Keyboard" }
        select {
            style: format!("width: 250px;"),
            class: "bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500",
            id: "board-select",
            value: config.physical_layout_name(),
            onchange: move |evt| {
                // config.physical_layout_name.write().set(evt.value());
                // config.selected_logical_layout_name.set(selected_board().default_logical_layout_name);
                config.write().update_physical_layout(general_setting.clone(), &evt.value());
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
    config: Store<Config>,
) -> Element {
    rsx!{
        label { class: "text-sm text-gray-200", "Language" }
        select {
            style: format!("width: 250px;"),
            class: "bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500",
            id: "board-select",
            value: config.logical_layout_name(),
            // onmounted: move |_| {
            //     selected_logical_layout_name.set(selected_logical_layout().layout_name)
            // },
            onchange: move |evt| config.write().update_logical_layout(&evt.value()),
            { general_setting.avail_logical_layouts.iter().map(|l|{
                rsx!(option { value: l.layout_name.clone(), label: l.layout_label.clone() })
            })}
        }
    }
}



#[component]
pub fn SelectFnID(
    general_setting: Arc<GeneralSeitting>,
    config: Store<Config>,
) -> Element {
    let fn_id = config.fn_id();
    rsx!{
        div {
            class: "min-w-[6rem]",
            h2 { class: "text-xl py-4 font-bold text-center", "Fn / Media trigger" },
            select {
                class: "w-full px-2 py-1 border border-gray-300 rounded text-gray-700 text-sm",
                id: "options",
                value: fn_id(),
                onchange: move |evt| {
                    let new_id: u8 = evt.value().parse().unwrap();
                    config.write().update_fn_id(new_id);
                },
                {
                    general_setting.avail_hid_usage_names.iter().map(|(key_id, usage_name)|{
                        let (label, style) = match config.read().logical_layout(&general_setting).map_key_label.get(&key_id) {
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
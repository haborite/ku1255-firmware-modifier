use dioxus::prelude::*;
use crate::models::{KeyboardSpec, UserConfig};

#[component]
pub fn SelectPhysicalLayout(
    // selected_name: Signal<String>,
    // selected_logical_layout_name: Signal<String>,
    // selected_board: Memo<PhysicalLayout>,
    keyboard_spec: ReadOnlySignal<KeyboardSpec>,
    user_config: Signal<UserConfig>,
) -> Element {
    rsx!{
        select {
            style: format!("width: 250px;"),
            class: "bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500",
            id: "board-select",
            // value: selected_name,
            value: user_config.read().physical_layout_name.clone(),
            onchange: move |evt| {
                let user_config_mut = &mut user_config.write();
                user_config_mut.update_physical_layout_name(&evt.value());
                let pl = user_config_mut.get_physical_layout(&keyboard_spec.read());
                user_config_mut.update_logical_layout_name(&pl.default_logical_layout_name);
                // selected_logical_layout_name.set(selected_board().default_logical_layout_name);
            },
            { keyboard_spec.read().avail_physical_layouts.iter().map(|b|{
                rsx!(option { value: b.name.clone(), label: b.label.clone() })
            })}
        }
    }
}

#[component]
pub fn SelectLogicalLayout(
    // selected_logical_layout_name: Signal<String>,
    // selected_logical_layout: Memo<LogicalLayout>,
    keyboard_spec: ReadOnlySignal<KeyboardSpec>,
    user_config: Signal<UserConfig>
) -> Element {
    rsx!{
        select {
            style: format!("width: 250px;"),
            class: "bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500",
            id: "board-select",
            // value: selected_logical_layout_name,
            value: user_config.read().logical_layout_name.clone(),
            /*
            onmounted: move |_| {
                // selected_logical_layout_name.set(selected_logical_layout().layout_name)
                let user_config_mut = &mut user_config.write();
                user_config_mut.update_logical_layout_name(
                    &keyboard_spec.read().avail_logical_layouts.iter()
                        .find(|l| l.layout_name == user_config.read().logical_layout_name)
                        .unwrap_or(&keyboard_spec.read().avail_logical_layouts.get(0).unwrap())
                        .layout_name
                );
            },
            */
            onchange: move |evt| {
                // selected_logical_layout_name.set(evt.value());
                let user_config_mut = &mut user_config.write();
                user_config_mut.update_logical_layout_name(&evt.value());
            },
            { keyboard_spec.read().avail_logical_layouts.iter().map(|l|{
                rsx!(option { value: l.name.clone(), label: l.label.clone() })
            })}
        }
    }
}



#[component]
pub fn SelectFnKeyID(
    keyboard_spec: ReadOnlySignal<KeyboardSpec>,
    user_config: Signal<UserConfig>
    // id_list: ReadOnlySignal<Vec<u8>>,
    // usage_names: ReadOnlySignal<Vec<String>>,
    // key_id: Signal<u8>,
    // map_key_label: HashMap::<u8, KeyLabel>,
) -> Element {

    let avail_hid_usage_names = keyboard_spec.read().avail_hid_usage_names.clone();
    let selected_logical_layout = user_config.read().get_logical_layout(&keyboard_spec.read());
    let map_key_label = selected_logical_layout.map_key_label;

    rsx!{
        div {
            class: "w-full max-w-md mx-auto p-6 space-y-6",
            h2 { class: "text-xl font-bold text-center", "Function key" },
            select {
                class: "w-full p-2 border border-gray-300 rounded mb-4 text-gray-700",
                id: "options",
                value: user_config.read().fn_id,
                onchange: move |evt| {
                    let new_id: u8 = evt.value().clone().parse().unwrap();
                    user_config.write().update_fn_id(new_id);
                },
                {
                    avail_hid_usage_names
                        .iter()
                        .map(|(&kid, name)| {
                            let (label, class) = match map_key_label.get(&kid) {
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
                                    selected: kid == user_config.read().fn_id,
                                }
                            }
                        })
                }
            }
        }
    }
}
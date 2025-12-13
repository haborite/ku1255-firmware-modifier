use dioxus::prelude::*;
use crate::components::Popup;
use crate::models::{Config, ConfigStoreExt, GeneralSeitting, KeyLabel};
use std::sync::Arc;

#[component]
pub fn Keyboard(
    general_setting: Arc<GeneralSeitting>,
    config: Store<Config>,
    layer_number: u8,
) -> Element {
    let id_layout = {
        if layer_number == 0 {
            config.layer0()
        } else {
            config.layer1()
        }
    };

    let mut selected_address = use_signal(|| None as Option<u32>);

    rsx! {
        div { class: "p-4 space-y-4",
            div { class: "text-xl font-bold", { if layer_number == 0 { "Main Layer" } else { "2nd Layer" } } }
            div {
                class: "space-y-1",
                {
                    let config_read = config.read();
                    let physical_layout = config_read.physical_layout(&general_setting);
                    physical_layout.map_address.clone()
                        .into_iter()
                        .zip(physical_layout.map_widths.iter())
                        .map(|row| 
                            rsx! {
                                div { class: "flex space-x-1",
                                    {
                                        row.0
                                            .into_iter()
                                            .zip(row.1.iter())
                                            .map(|(add_opt, width)| {
                                                let general_setting = general_setting.clone();
                                                let width_str = width.to_string();
                                                let (id_opt, id_opt_org) = match add_opt {
                                                    Some(address) => (
                                                        id_layout().get(&address).copied(),
                                                        if layer_number == 0 {
                                                            general_setting.initial_id_map.get(&address).copied()
                                                        } else {
                                                            config.read().layer0.get(&address).copied()
                                                        }
                                                    ),
                                                    None => (None, None),
                                                };
                                                let key_label = match &id_opt {
                                                    Some(kid) => {
                                                        match config.read().logical_layout(&general_setting).map_key_label.get(&kid) {
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
                                                        0        => "border-gray-500",
                                                        1..213   => if kid == id_opt_org.unwrap() {""} else { "border-sky-300" }
                                                        213..224 => if kid == id_opt_org.unwrap() {""} else { "border-violet-300" }
                                                        224..231 => if kid == id_opt_org.unwrap() {""} else { "border-sky-300" }
                                                        231      => "border-rose-300",
                                                        _        => if kid == id_opt_org.unwrap() {""} else { "border-green-300" }
                                                    };
                                                    let text_color = match kid {
                                                        0        => "text-gray-500",
                                                        1..213   => if kid == id_opt_org.unwrap() {""} else { "text-sky-300" }
                                                        213..224 => if kid == id_opt_org.unwrap() {""} else { "text-violet-300" }
                                                        224..231 => if kid == id_opt_org.unwrap() {""} else { "text-sky-300" }
                                                        231      => "text-rose-300",
                                                        _        => if kid == id_opt_org.unwrap() {""} else { "text-green-300" }
                                                    };
                                                    rsx! {
                                                        button {
                                                            style: format!("width: {}px;", width_str),
                                                            class: format!(
                                                                "border {} px-2 py-1 h-10 text-xs flex flex-col items-center justify-center text-center break-words whitespace-normal text-[10px] leading-tight hover:bg-gray-600",
                                                                border_color
                                                            ),
                                                            onclick: move |_| selected_address.set(add_opt.clone()),
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
                                            })
                                    }
                                }
                            }
                    )
                }
            }
            Popup {
                general_setting: general_setting.clone(),
                config,
                layer_number,
                selected_address,
            }
        }
    }
}

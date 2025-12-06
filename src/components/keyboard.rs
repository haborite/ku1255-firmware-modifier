use dioxus::prelude::*;
use crate::models::{UserConfig, KeyboardSpec, KeyLabel};
use crate::components::Popup;

#[component]
pub fn Keyboard(
    layer_number: u8,
    user_config: Signal<UserConfig>,
    keyboard_spec: ReadOnlySignal<KeyboardSpec>,
) -> Element {

    // println!("Keyboard here");
    
    let mut selected_address = use_signal(|| None as Option<u32>);
    // let selected_physical_layout = user_config.read().get_physical_layout(&keyboard_spec.read());
    // let selected_logical_layout = user_config.read().get_logical_layout(&keyboard_spec.read());

    rsx! {
        div { class: "p-4 space-y-4",
            div { class: "text-xl font-bold", { if layer_number == 0 { "Main Layer" } else { "2nd Layer" } }
            }
            div { 
                class: "space-y-1",
                {
                    let physical_layout = user_config.read().get_physical_layout(&keyboard_spec.read());
                    // println!("{:?}", physical_layout);
                    let addr_list = physical_layout.map_address;
                    // println!("{:?}", addr_list);
                    let width_list = physical_layout.map_widths;
                    // println!("{:?}", width_list);
                    addr_list.into_iter().zip(width_list.into_iter()).map(|row| rsx! {
                        div { class: "flex space-x-1",
                            { row.0.into_iter().zip(row.1.into_iter()).map(|(add_opt, width)| {
                                // println!("{:?}, {:?}", add_opt, width);
                                let width_str = width.to_string();
                                let (id_opt, id_opt_org) = match add_opt {
                                    Some(address) => (
                                        user_config.read().layer0.get(&address).copied(),
                                        keyboard_spec.read().initial_id_map.get(&address).copied()
                                    ),
                                    None => (None, None),
                                };
                                let key_label = match &id_opt {
                                    Some(kid) => {
                                        match user_config.read().get_logical_layout(&keyboard_spec.read()).map_key_label.get(&kid) {
                                            Some(kl) => kl.clone(),
                                            None => KeyLabel::new(),
                                        }
                                    },
                                    None => KeyLabel::new(),
                                };
                                let key_default = key_label.default.clone();
                                let key_shifted = key_label.shifted.clone();

                                // println!("id_opt: {:?}", id_opt);

                                if let Some(kid) = id_opt {
                                    // println!("kid: {}", kid);
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
                        }
                    })
                }
            }
            Popup {
                layer_number,
                user_config,
                keyboard_spec,
                selected_address,
                map_key_label: user_config.read().get_logical_layout(&keyboard_spec.read()).map_key_label,
            }
        }
    }
}

use dioxus::prelude::*;
use crate::models::{KeyboardSpec, UserConfig};
// use crate::components::selects::SelectKeyID;


/// Combination Key Setting Component
/// Set of 8 boolean boxes (indicating left Ctrl, left Shift, left Alt, left GUI, right Ctrl, right Shift, right Alt, right GUI) and a select box for key ID
#[component]
pub fn MacroKeySetting(
    keyboard_spec: ReadOnlySignal<KeyboardSpec>,
    user_config: Signal<UserConfig>
) -> Element {
    rsx! {
        div {
            class: "flex flex-col space-y-4",
            {
                user_config.read().macro_key_map.keys().map(|&trigger_id| {
                    rsx!(
                        div {
                            class: "flex items-center space-x-4",
                            label {
                                class: "w-32",
                                r#for: format!("Trigger Key ID: {:02X}", trigger_id)
                            }
                            div {
                                class: "flex items-center space-x-2",
                                input {
                                    r#type: "checkbox",
                                    checked: user_config.read().get_macro_key(trigger_id).left_ctrl,
                                    onchange: move |evt| { user_config.write().update_left_ctrl(trigger_id, evt.checked()); },
                                }
                                span { "LCtrl" }
                                input {
                                    r#type: "checkbox",
                                    checked: user_config.read().get_macro_key(trigger_id).left_shift,
                                    onchange: move |evt| { user_config.write().update_left_shift(trigger_id, evt.checked()); },
                                }
                                span { "LShift" }
                                input {
                                    r#type: "checkbox",
                                    checked: user_config.read().get_macro_key(trigger_id).left_alt,
                                    onchange: move |evt| { user_config.write().update_left_alt(trigger_id, evt.checked()); },
                                }
                                span { "LAlt" }
                                input {
                                    r#type: "checkbox",
                                    checked: user_config.read().get_macro_key(trigger_id).left_gui,
                                    onchange: move |evt| { user_config.write().update_left_gui(trigger_id, evt.checked()); },
                                }
                                span { "LGui" }
                                input {
                                    r#type: "checkbox",
                                    checked: user_config.read().get_macro_key(trigger_id).right_ctrl,
                                    onchange: move |evt| { user_config.write().update_right_ctrl(trigger_id, evt.checked()); },
                                }
                                span { "RCtrl" }
                                input {
                                    r#type: "checkbox",
                                    checked: user_config.read().get_macro_key(trigger_id).right_shift,
                                    onchange: move |evt| { user_config.write().update_right_shift(trigger_id, evt.checked()); },
                                }
                                span { "RShift" }
                                input {
                                    r#type: "checkbox",
                                    checked: user_config.read().get_macro_key(trigger_id).right_alt,
                                    onchange: move |evt| { user_config.write().update_right_alt(trigger_id, evt.checked()); },
                                }
                                span { "RAlt" }
                                input {
                                    r#type: "checkbox",
                                    checked: user_config.read().get_macro_key(trigger_id).right_gui,
                                    onchange: move |evt| { user_config.write().update_right_gui(trigger_id, evt.checked()); },
                                }
                                span { "RGui" }
                                SelectMacroKeyID { trigger_id, keyboard_spec, user_config }
                            }
                        }
                    )
            }) }
        }
    }
}

#[component]
pub fn SelectMacroKeyID(
    trigger_id: u8,
    keyboard_spec: ReadOnlySignal<KeyboardSpec>,
    user_config: Signal<UserConfig>
) -> Element {
    rsx!{
        div {
            class: "w-full max-w-md mx-auto p-6 space-y-6",
            h2 { class: "text-xl font-bold text-center", "Function key" },
            select {
                class: "w-full p-2 border border-gray-300 rounded mb-4 text-gray-700",
                id: "options",
                value: user_config.read().get_macro_key(trigger_id).key_id,
                onchange: move |evt| {
                    let new_id: u8 = evt.value().clone().parse().unwrap();
                    user_config.write().update_macro_key_id(trigger_id, new_id);
                },
                {
                    keyboard_spec.read().avail_hid_usage_names
                        .iter()
                        .map(|(&kid, name)| {
                            let (label, class) = match user_config.read().get_logical_layout(&keyboard_spec.read()).map_key_label.get(&kid) {
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
                                    selected: kid == user_config.read().get_macro_key(trigger_id).key_id,
                                }
                            }
                        })
                }
            }
        }
    }
}

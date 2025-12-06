use dioxus::prelude::*;
use std::collections::BTreeMap;
use std::sync::Arc;
use crate::models::GeneralSeitting;

/// Application Key Setting Component
#[component]
pub fn MediaKeySetting(
    general_setting: Arc<GeneralSeitting>,
    media_key_map: Signal<BTreeMap<u8, u16>>,
) -> Element {
    rsx! {
        div {
            class: "flex flex-col space-y-4",
            {
                media_key_map().iter().map(|(trigger_key_id, media_key_id)| {
                    let label = format!("Media {:02}", trigger_key_id - 212);
                    rsx!(
                        div {
                            class: "flex items-center space-x-4",
                            h3 { class: "text-l font-bold text-center", {label} },
                            SelectMediaKeyID {
                                general_setting: general_setting.clone(),
                                trigger_key_id: *trigger_key_id,
                                media_key_id: *media_key_id,
                                media_key_map
                            }
                        }
                    )
                })
            }
        }
    }
}

#[component]
pub fn SelectMediaKeyID(
    general_setting: Arc<GeneralSeitting>,
    trigger_key_id: u8,
    media_key_id: u16,
    media_key_map: Signal<BTreeMap<u8, u16>>,
) -> Element {
    rsx!{
        div {
            class: "w-full max-w-md mx-auto p-6 space-y-6",
            select {
                class: "w-full p-2 border border-gray-300 rounded mb-4 text-gray-700",
                id: "options",
                value: general_setting.avail_media_key_usage_names.get(&media_key_id).unwrap().clone(),
                onchange: move |evt| {
                    let new_media_key_id: u16 = evt.value().clone().parse().unwrap();
                    let mut media_key_map_mut = media_key_map.write();
                    media_key_map_mut.insert(trigger_key_id, new_media_key_id);
                },
                {
                    general_setting.avail_media_key_usage_names
                        .iter()
                        .map(|(&avail_media_key_id, media_key_usage_name)| {
                            rsx! {
                                option {
                                    class: "text-gray-700",
                                    value: avail_media_key_id,
                                    label: media_key_usage_name.clone(),
                                    selected: avail_media_key_id == media_key_id,
                                }
                            }
                        })
                }
            }
        }
    }
}

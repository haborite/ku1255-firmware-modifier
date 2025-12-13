use dioxus::prelude::*;
use std::sync::Arc;
use crate::models::{Config, ConfigStoreExt, GeneralSeitting};

/// Application Key Setting Component
#[component]
pub fn MediaKeySetting(
    general_setting: Arc<GeneralSeitting>,
    config: Store<Config>,
    // media_key_map: Signal<BTreeMap<u8, u16>>,
) -> Element {
    let media_key_map = config.media_key_map();
    rsx! {
        div {
            class: "flex flex-col space-y-4",
            {
                media_key_map().iter().map(|(trigger_key_id, media_key_id)| {
                    let label = format!("Media {:02}", trigger_key_id - 212);
                    rsx!(
                        div {
                            class: "flex gap-4 py-2",
                            span {
                                class: "text-sm font-semibold text-right whitespace-nowrap",
                                {label}
                            },
                            SelectMediaKeyID {
                                general_setting: general_setting.clone(),
                                config,
                                trigger_key_id: *trigger_key_id,
                                media_key_id: *media_key_id,
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
    config: Store<Config>,
    trigger_key_id: u8,
    media_key_id: u16,
    // media_key_map: Signal<BTreeMap<u8, u16>>,
) -> Element {
    rsx!{
        div {
            class: "min-w-[12rem]",
            select {
                class: "w-full px-2 py-1 border border-gray-300 rounded text-gray-700 text-sm",
                id: "options",
                value: general_setting.avail_media_key_usage_names.get(&media_key_id).unwrap().clone(),
                onchange: move |evt| {
                    let new_media_key_id: u16 = evt.value().parse().unwrap();
                    config.write().update_media_key_map(trigger_key_id, new_media_key_id); 
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

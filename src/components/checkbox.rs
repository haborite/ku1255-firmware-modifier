use dioxus::prelude::*;
use crate::models::Config;
use crate::models::config::ConfigStoreExt;

#[component]
pub fn EnableMiddleClick(config: Store<Config>) -> Element {
    rsx! {
        div { class: "w-full p-6 space-y-6",
            h2 { class: "text-xl font-bold text-center flex-wrap",
                "Enable middle",
                br {},
                "button click",
            }
            div { class: "flex justify-center",
                input {
                    r#type: "checkbox",
                    checked: config.enable_middle_click(),
                    onchange: move |evt| config.write().update_enable_middle_click(evt.checked()),
                }
            }
        }
    }
}
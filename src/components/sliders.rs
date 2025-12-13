use dioxus::prelude::*;
use crate::models::Config;

#[component]
pub fn SliderTPSensitivity(
    config: Store<Config>,
) -> Element {
    rsx! (
        div {
            class: "w-full max-w-md mx-auto p-6 space-y-6",
            h2 { class: "text-xl font-bold text-center", "TrackPoint Speed" },
            div {
                class: "flex items-center justify-center space-x-8",
                div {
                    class: "flex flex-col items-start",
                    input {
                        r#type: "range",
                        min: 1,
                        max: 5,
                        step: 1,
                        value: config.read().tp_sensitivity,
                        onchange: move |evt| {
                            let new_sensitivity = u32::from_str_radix(&evt.value(), 10).unwrap();
                            config.write().update_tp_sensitivity(new_sensitivity);
                        },
                    },
                },
                span {
                    class: "text-xl w-24 text-center",
                    {
                        let n = config.read().tp_sensitivity;
                        match n {
                            1 => "1 (default)".to_string(),
                            _ => n.to_string()
                        }
                    }
                }
            }
        },
    )
}
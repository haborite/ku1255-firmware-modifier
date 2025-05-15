use dioxus::prelude::*;

#[component]
pub fn SliderTPSensitivity(
    tp_sensitivity: Signal<u32>,
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
                        value: tp_sensitivity,
                        onchange: move |evt| {
                            tp_sensitivity.set(u32::from_str_radix(&evt.value(), 10).unwrap());
                        },
                    },
                },
                span {
                    class: "text-xl w-24 text-center",
                    {
                        let n = tp_sensitivity();
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
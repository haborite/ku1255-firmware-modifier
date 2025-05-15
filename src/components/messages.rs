use dioxus::prelude::*;

#[component]
pub fn ErrorMessage(msg: String, error_msg: Signal<Option<String>>) -> Element {
    rsx! {
        div { class: "fixed inset-0 flex items-center justify-center bg-black bg-opacity-50 z-50",
            div { 
                class: "bg-red-100 border border-red-400 text-red-700 px-6 py-4 rounded-xl shadow-lg max-w-md w-full relative",
                strong { class: "text-lg font-semibold", "Error" }
                p {
                    class: "mt-2",
                    { msg }
                }
                button {
                    class: "absolute top-2 right-2 text-red-500 hover:text-red-700",
                    id: "errorMessage",
                    onclick: move |_evt| {
                        error_msg.set(None);
                    },
                    "close"
                }
            }
        }
    }
}



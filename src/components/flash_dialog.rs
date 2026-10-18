use dioxus::prelude::*;
use futures_util::StreamExt;
use ku1255_core::flash::{self, CheckedImage, Event};
use crate::utils::HidTransport;

#[derive(Clone, PartialEq)]
enum FlashState {
    Ready,
    Running,
    Succeeded,
    Failed(String),
}

enum Update {
    Event(Event),
    Done(Result<(), String>),
}

#[component]
pub fn FlashDialog(flash_job: Signal<Option<CheckedImage>>) -> Element {
    let mut state = use_signal(|| FlashState::Ready);
    let mut log: Signal<Vec<String>> = use_signal(|| {
        vec![
            "✓ Firmware safety check PASSED.".to_string(),
            "Make sure that the keyboard is plugged in, then press Start.".to_string(),
        ]
    });
    let mut progress = use_signal(|| (0usize, 1usize));

    let start = move |_| {
        let Some(image) = flash_job() else { return };
        state.set(FlashState::Running);
        log.set(Vec::new());
        progress.set((0, 1));

        let (tx, mut rx) = futures_channel::mpsc::unbounded::<Update>();
        std::thread::spawn(move || {
            let result = HidTransport::open().and_then(|mut transport| {
                flash::flash(&mut transport, &image, &mut |event| {
                    let _ = tx.unbounded_send(Update::Event(event));
                })
            });
            let _ = tx.unbounded_send(Update::Done(result));
        });
        spawn(async move {
            while let Some(update) = rx.next().await {
                match update {
                    Update::Event(Event::Message(msg)) => log.write().push(msg),
                    Update::Event(Event::Progress { sent, last }) => progress.set((sent, last.max(1))),
                    Update::Done(Ok(())) => {
                        progress.set((1, 1));
                        state.set(FlashState::Succeeded);
                    }
                    Update::Done(Err(msg)) => {
                        log.write().push(format!("Error: {msg}"));
                        state.set(FlashState::Failed(msg));
                    }
                }
            }
        });
    };

    let (sent, last) = progress();
    let percent = sent * 100 / last;
    let (banner_class, banner) = match state() {
        FlashState::Ready => ("text-slate-100", "Make sure that the keyboard is plugged in.".to_string()),
        FlashState::Running => ("text-yellow-300", "IN PROGRESS. DO NOT unplug the keyboard.".to_string()),
        FlashState::Succeeded => (
            "text-green-400",
            "✓ Flashing finished. Unplug and reconnect the keyboard.".to_string(),
        ),
        FlashState::Failed(_) => (
            "text-red-400",
            "✖ Flashing failed. You can retry, or restore with the official Lenovo installer.".to_string(),
        ),
    };
    let running = state() == FlashState::Running;

    rsx! {
        div { class: "fixed inset-0 flex items-center justify-center bg-black bg-opacity-50 z-50",
            div { class: "bg-gray-900 text-slate-100 p-6 rounded-xl shadow-lg w-[36rem] flex flex-col gap-3",
                strong { class: "text-lg", "Install firmware" }
                p { class: "font-bold {banner_class}", "{banner}" }
                div { class: "w-full h-3 bg-gray-700 rounded",
                    div { class: "h-3 bg-indigo-500 rounded", style: "width: {percent}%" }
                }
                pre { class: "bg-black text-xs p-2 h-48 overflow-y-auto whitespace-pre-wrap rounded",
                    { log().join("\n") }
                }
                div { class: "flex justify-end gap-2",
                    if !matches!(state(), FlashState::Succeeded) {
                        button {
                            class: "px-4 py-2 bg-indigo-900 text-white rounded shadow hover:bg-indigo-600 border border-white disabled:opacity-50",
                            disabled: running,
                            onclick: start,
                            if matches!(state(), FlashState::Failed(_)) { "Retry" } else { "Start" }
                        }
                    }
                    button {
                        class: "px-4 py-2 bg-gray-800 text-white rounded shadow hover:bg-gray-700 border border-white disabled:opacity-50",
                        disabled: running,
                        onclick: move |_| flash_job.set(None),
                        if matches!(state(), FlashState::Succeeded) { "Finish" } else { "Cancel" }
                    }
                }
            }
        }
    }
}

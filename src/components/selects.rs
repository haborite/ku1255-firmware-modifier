use dioxus::prelude::*;
use crate::models::{Board, LogicalLayout};

#[component]
pub fn SelectBoard(
    selected_board_name: Signal<String>,
    selected_logical_layout_name: Signal<String>,
    selected_board: Memo<Board>,
    boards: Vec<Board>,
) -> Element {
    rsx!{
        select {
            style: format!("width: 250px;"),
            class: "bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500",
            id: "board-select",
            value: selected_board_name,
            onchange: move |evt| {
                selected_board_name.set(evt.value());
                selected_logical_layout_name.set(selected_board().default_logical_layout_name);
            },
            { boards.iter().map(|b|{
                rsx!(option { value: b.board_name.clone(), label: b.board_label.clone() })
            })}
        }
    }
}

#[component]
pub fn SelectLogicalLayout(
    selected_logical_layout_name: Signal<String>,
    selected_logical_layout: Memo<LogicalLayout>,
    logical_layouts: Vec<LogicalLayout>,
) -> Element {
    rsx!{
        select {
            style: format!("width: 250px;"),
            class: "bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500",
            id: "board-select",
            value: selected_logical_layout_name,
            onmounted: move |_| {
                selected_logical_layout_name.set(selected_logical_layout().layout_name)
            },
            onchange: move |evt| {
                selected_logical_layout_name.set(evt.value());
            },
            { logical_layouts.iter().map(|l|{
                rsx!(option { value: l.layout_name.clone(), label: l.layout_label.clone() })
            })}
        }
    }
}
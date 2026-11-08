#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use dioxus_native::prelude::*;
use dioxus_native::{LogicalSize, WindowAttributes};

const INITIAL_NOTE: &str =
    "日本語の入力を試してください。\n変換中の文字と確定後の文字を確認します。";

fn main() {
    let window = WindowAttributes::default()
        .with_title("QuantumView - Blitz")
        .with_inner_size(LogicalSize::new(1000.0, 720.0));
    dioxus_native::launch_cfg(app, vec![], vec![Box::new(window)]);
}

fn app() -> Element {
    let mut note = use_signal(|| INITIAL_NOTE.to_string());
    let chars = note.read().chars().count();

    rsx! {
        style { {include_str!("../../../ui/style.css")} }
        main { class: "app",
            header {
                p { class: "eyebrow", "QuantumView / rendering probe" }
                h1 { "デスクトップUIの比較" }
                p { "日本語入力・選択・スクロール・リサイズを確認します。" }
            }
            section { class: "editor",
                label { r#for: "note", "日本語メモ" }
                textarea {
                    id: "note",
                    spellcheck: false,
                    value: "{note}",
                    oninput: move |event| note.set(event.value()),
                }
                div { class: "toolbar",
                    button {
                        r#type: "button",
                        onclick: move |_| note.set(String::new()),
                        "クリア"
                    }
                    span { class: "count", "{chars} 文字" }
                }
            }
            section { class: "cards",
                for i in 1..=100 {
                    article { class: "card", key: "{i}",
                        h2 { "項目 {i}" }
                        p { "HTML / CSSで描くローカルUI。スクロールと文字表示を確認します。" }
                    }
                }
            }
        }
    }
}

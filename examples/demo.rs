use dioxus::prelude::*;
use dioxus_library_template::*;

fn main() {
    dioxus::launch(app);
}

fn app() -> Element {
    let mut dropdown_open = use_signal(|| false);
    let mut tooltip_open = use_signal(|| false);
    let mut modal_open = use_signal(|| false);

    rsx! {
        PortalProvider { 
            style: "width: 100vw; height: 100vh; font-family: system-ui, -apple-system, Segoe UI, Roboto, Helvetica, Arial, \"Segoe UI Emoji\"; background: #f7f7fb;".to_string(),

            div { style: "padding: 32px; display: flex; gap: 32px; flex-wrap: wrap;",

                // ---------- Dropdown Demo ----------
                div { style: "min-width: 320px; padding: 16px; background: white; border: 1px solid #e5e7eb; border-radius: 12px; box-shadow: 0 1px 2px rgba(0,0,0,.04);",
                    h2 { style: "margin: 0 0 12px; font-size: 14px; color: #6b7280; text-transform: uppercase; letter-spacing: .06em;", "Dropdown" }
                    Portal { open: *dropdown_open.read(), layer: 10,
                        PortalAnchor {
                            button {
                                onclick: move |_| dropdown_open.set(true),
                                style: "padding: 8px 12px; border-radius: 8px; background: #111827; color: white; border: none; cursor: pointer;",
                                "メニューを開く"
                            }
                        }
                        PortalOverlay {
                            // 背景クリックで閉じる半透明オーバーレイ
                            div {
                                onclick: move |_| dropdown_open.set(false),
                                style: "position: absolute; inset: 0; background: rgba(0,0,0,0.25);",
                            }
                        }
                        PortalContent {
                            div { style: "min-width: 180px; padding: 8px; background: white; border: 1px solid #e5e7eb; border-radius: 10px; box-shadow: 0 10px 30px rgba(0,0,0,.10);",
                                ul { style: "list-style: none; padding: 0; margin: 0;",
                                    li { style: item_style(), "新規作成" }
                                    li { style: item_style(), "開く" }
                                    li { style: item_style(), "保存" }
                                    li { style: item_style(), "設定" }
                                }
                            }
                        }
                    }
                }

                // ---------- Tooltip Demo ----------
                div { style: "min-width: 320px; padding: 16px; background: white; border: 1px solid #e5e7eb; border-radius: 12px; box-shadow: 0 1px 2px rgba(0,0,0,.04);",
                    h2 { style: "margin: 0 0 12px; font-size: 14px; color: #6b7280; text-transform: uppercase; letter-spacing: .06em;", "Tooltip" }
                    Portal { open: *tooltip_open.read(), layer: 20,
                        PortalAnchor {
                            span {
                                onmouseenter: move |_| tooltip_open.set(true),
                                onmouseleave: move |_| tooltip_open.set(false),
                                style: "display: inline-flex; align-items: center; gap: 8px; padding: 6px 10px; border-radius: 8px; border: 1px dashed #9ca3af; color: #111827; background: #f9fafb;",
                                "ホバーで表示",
                                span { style: "font-weight: 700; color: #2563eb;", "(?)" }
                            }
                        }
                        PortalContent { style: "pointer-events: none;",
                            div { style: "padding: 8px 10px; background: #111827; color: white; font-size: 12px; border-radius: 8px; box-shadow: 0 10px 24px rgba(0,0,0,.18);",
                                "このテキストはアンカーの位置に追従します"
                            }
                        }
                    }
                }

                // ---------- Modal (no anchor) Demo ----------
                div { style: "flex: 1; min-width: 320px; padding: 16px; background: white; border: 1px solid #e5e7eb; border-radius: 12px; box-shadow: 0 1px 2px rgba(0,0,0,.04);",
                    h2 { style: "margin: 0 0 12px; font-size: 14px; color: #6b7280; text-transform: uppercase; letter-spacing: .06em;", "Modal" }
                    button {
                        onclick: move |_| modal_open.set(true),
                        style: "padding: 8px 12px; border-radius: 8px; background: #2563eb; color: white; border: none; cursor: pointer;",
                        "モーダルを開く"
                    }
                    Portal { open: *modal_open.read(), layer: 30,
                        // アンカーを置かない → Provider 内でセンタリングを指定
                        vertical_alignment: Alignment::Center,
                        horizontal_alignment: Alignment::Center,

                        PortalOverlay {
                            div {
                                onclick: move |_| modal_open.set(false),
                                style: "position: absolute; inset: 0; background: rgba(15,23,42,0.45); backdrop-filter: blur(2px);",
                            }
                        }
                        PortalContent {
                                div { style: "width: 360px; max-width: calc(100vw - 32px); padding: 16px; border-radius: 12px; background: white; border: 1px solid #e5e7eb; box-shadow: 0 24px 60px rgba(0,0,0,.25); display: flex; flex-direction: column; gap: 12px;",
                                h3 { style: "margin: 0; font-size: 16px;", "ダイアログ" }
                                p { style: "margin: 0; color: #374151;", "アンカーなしのポータル。Provider を基準にセンター表示。" }
                                div { style: "display: flex; justify-content: flex-end; gap: 8px;",
                                    button { onclick: move |_| modal_open.set(false), style: "padding: 8px 12px; border-radius: 8px; background: #e5e7eb; border: none; cursor: pointer;", "キャンセル" }
                                    button { onclick: move |_| modal_open.set(false), style: "padding: 8px 12px; border-radius: 8px; background: #111827; color: white; border: none; cursor: pointer;", "OK" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn item_style() -> String {
    "padding: 8px 10px; border-radius: 8px; cursor: pointer; color: #111827;".to_string()
        + ":hover{background:#f3f4f6;}"
}

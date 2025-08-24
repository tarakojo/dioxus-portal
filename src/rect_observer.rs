//! Utility that observes an element's rectangle (`getBoundingClientRect` equivalent) and notifies Rust.
//!
//! - The `RectObserver` component hooks JS `ResizeObserver` and `scroll`/`resize` events on its own
//!   DOM element, throttling via rAF while sending rectangles.
//! - Observation handles are managed by a JS-side registry (`REG_KEY`), ensuring proper start/stop
//!   on mount/unmount.
//! - The received rectangle is propagated upward via the `on_rect_changed` callback.
use dioxus_lib::core::use_drop;
use dioxus_lib::html::geometry::Pixels;
use dioxus_lib::{document, prelude::*};
use euclid::{Point2D, Size2D};
use serde::Deserialize;

pub type Rect = euclid::Rect<f64, Pixels>;

/// Properties for `RectObserver`.
/// Sends rectangles to `on_rect_changed`. `style`/`attributes` are applied to the wrapping `div`.
#[derive(Props, PartialEq, Debug, Clone)]
pub struct RectObserverProps {
    #[props(default)]
    pub on_rect_changed: Callback<Rect>,

    #[props(default)]
    pub style: String,

    #[props(extends = GlobalAttributes)]
    attributes: Vec<Attribute>,

    children: Element,
}

/// Component that starts/stops observing its own element and sends changes to Rust.
#[component]
pub fn RectObserver(props: RectObserverProps) -> Element {
    let id = use_memo(|| alloc_id());

    {
        let id = id();
        use_effect(move || {
            let js_code = js_code_of_start_observer(&id);
            let mut eval = document::eval(&js_code);

            // JS -> Rust receive loop
            spawn(async move {
                while let Ok(val) = eval.recv::<ObserverReport>().await {
                    (props.on_rect_changed)(val.into());
                }
            });
        });
    }

    // Stop observing on unmount
    {
        let id = id();
        use_drop(move || {
            let js_code = js_code_of_stop_observer(&id);
            document::eval(&js_code);
        });
    }

    rsx! {
        div {
            id: id,
            style: props.style,
            ..props.attributes,
            {props.children}
        }
    }
}

const ID_PREFIX: &str = "dioxus-portal-rect-observer-";
const REG_KEY: &str = "dioxus-portal-rect-observers";

static NEXT_ID: GlobalSignal<u64> = Signal::global(|| 0);

fn alloc_id() -> String {
    let n = {
        let mut w = NEXT_ID.write();
        *w += 1;
        *w
    };
    format!("{ID_PREFIX}{}", n)
}

/// Rectangle payload sent from the JS side (serialized form).
#[derive(Debug, Clone, PartialEq, Deserialize)]
struct ObserverReport {
    width: f64,
    height: f64,
    x: f64,
    y: f64,
}

impl From<ObserverReport> for Rect {
    fn from(report: ObserverReport) -> Self {
        Rect::new(
            Point2D::new(report.x, report.y),
            Size2D::new(report.width, report.height),
        )
    }
}

/// Generates JS code to start observation.
fn js_code_of_start_observer(target_id: &str) -> String {
    format!(
        r#"
    try {{
      const REG_KEY = Symbol.for("{REG_KEY}");
      const target_id = "{target_id}";

      if (!globalThis[REG_KEY]) {{
        globalThis[REG_KEY] = new Map();
      }}
      const reg = globalThis[REG_KEY];
      if (reg.has(target_id)) {{
        // Already observing
        // console.log("observer already started", target_id);
        return;
      }}

      const el = document.getElementById(target_id);
      if (!el) {{
        // console.log("observer not found", target_id);
        return;
      }}

      // ---- rAF throttling shared logic ----
      let rafId = null;
      const sendRect = () => {{
        const r = el.getBoundingClientRect();
        const payload = {{ 
          width: r.width,
          height: r.height,
          x: r.x,
          y: r.y 
        }};
        // console.log("sendRect", target_id, payload);
        dioxus.send(payload);
      }};
      const sendRectRaf = () => {{
        if (rafId !== null) return; // prevent multiple schedules within the same frame
        rafId = requestAnimationFrame(() => {{ 
          rafId = null;
          sendRect();
        }});
      }};

      // ---- Size change observation ----
      const ro = new ResizeObserver(() => {{
        sendRectRaf();
      }});
      ro.observe(el);

      // ---- Scroll/resize (position change) ----
      const onScroll = () => sendRectRaf();
      const onResize = () => sendRectRaf();
      window.addEventListener("scroll", onScroll, {{ passive: true, capture: true }});
      window.addEventListener("resize", onResize, {{ passive: true }});

      // console.log("start observer", target_id);

      // ---- Initial send ---- 
      sendRect();

      // Store handles so we can detach later
      reg.set(target_id, {{
        ro,
        onScroll,
        onResize,
      }});
    }} catch (e) {{
      console.error(`start observer error: ${{e}}`);
    }}
"#
    )
}

/// Generates JS code to stop observation.
fn js_code_of_stop_observer(target_id: &str) -> String {
    format!(
        r#"
    try {{
      const REG_KEY = Symbol.for("{REG_KEY}");
      const target_id = "{target_id}";

      const reg = globalThis[REG_KEY];
      if (reg && reg.has(target_id)) {{
        const rec = reg.get(target_id);
        if (rec) {{
          try {{ if (rec.ro) rec.ro.disconnect(); }} catch (_) {{}}
          try {{ if (rec.onScroll) window.removeEventListener("scroll", rec.onScroll, {{ capture: true }}); }} catch (_) {{}}
          try {{ if (rec.onResize) window.removeEventListener("resize", rec.onResize); }} catch (_) {{}}
        }}
        reg.delete(target_id);
      }}

      // console.log("stop observer", target_id);
    }} catch (e) {{
      console.error(`stop observer error: ${{e}}`);
    }}
"#
    )
}

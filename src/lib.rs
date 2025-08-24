//! Module providing portal components.
//!
//! - `PortalProvider`: Provides the render root for portals and the overlay layer
//! - `Portal`: A unit of a portal (anchor/content)
//! - `PortalAnchor`: Anchor area used as the reference for alignment. When registered, the rectangle of this component is used as the anchor
//! - `PortalContent`: Registers the content to display
//! - `PortalOverlay`: Registers the overlay element
//!
//! Placement is controlled by the combination of `Alignment`, `Spread`, and `OverflowPolicy`.

mod rect_observer;

use dioxus_core::use_drop;
use dioxus_lib::hooks::use_context_provider;
use dioxus_lib::{html::geometry::Pixels, prelude::*};
use euclid::{Point2D, Size2D};
use std::{collections::HashMap, fmt::Display, ops::Range};

use crate::rect_observer::{Rect, RectObserver};

// ------ Types for placement control --------------------------------------------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq)]
pub enum Alignment {
    Start,
    Center,
    End,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Spread {
    Inside,
    Outside,
}

#[derive(Clone, Copy, PartialEq)]
pub enum OverflowPolicy {
    Ignore,
    Shrink,
    Clamp,
    Flip,
}

// ------ Public Props -------------------------------------------------------------------------------------------------------------------

#[derive(Props, Clone, PartialEq)]
pub struct PortalProviderProps {
    #[props(default)]
    pub style: String,
    #[props(extends=GlobalAttributes)]
    pub attribute: Vec<Attribute>,
    children: Element,
}

#[derive(Props, Clone, PartialEq)]
pub struct PortalProps {
    #[props(default = false)]
    pub open: bool,
    #[props(default = 0)]
    pub layer: i32,

    // Use this when specifying the anchor rectangle directly
    // This property takes precedence over the rectangle from `PortalAnchor`
    // Note: The position is relative to the viewport
    #[props(optional)]
    pub anchor_rect : Option<Rect>, 

    #[props(default=Alignment::End)]
    pub vertical_alignment: Alignment,
    #[props(default=Spread::Outside)]
    pub vertical_spread: Spread,
    #[props(default = 0.0)]
    pub vertical_offset: f64,
    #[props(default=OverflowPolicy::Clamp)]
    pub vertical_overflow_policy: OverflowPolicy,

    #[props(default=Alignment::Center)]
    pub horizontal_alignment: Alignment,
    #[props(default=Spread::Inside)]
    pub horizontal_spread: Spread,
    #[props(default = 0.0)]
    pub horizontal_offset: f64,
    #[props(default=OverflowPolicy::Clamp)]
    pub horizontal_overflow_policy: OverflowPolicy,

    children: Element,
}

#[derive(Props, Clone, PartialEq)]
pub struct PortalAnchorProps {
    #[props(default)]
    pub style: String,
    #[props(extends=GlobalAttributes)]
    attributes: Vec<Attribute>,
    children: Element,
}

#[derive(Props, Clone, PartialEq)]
pub struct PortalContentProps {
    #[props(default)]
    pub style: String,
    #[props(extends=GlobalAttributes)]
    attributes: Vec<Attribute>,
    children: Element,
}

#[derive(Props, Clone, PartialEq)]
pub struct PortalOverlayProps {
    #[props(default)]
    pub style: String,
    #[props(extends=GlobalAttributes)]
    attributes: Vec<Attribute>,
    children: Element,
}

// ------ Public Components ---------------------------------------------------------------------------------------------------------------

#[component]
pub fn PortalAnchor(props: PortalAnchorProps) -> Element {
    let mut provider_ctx = use_context::<PortalProviderContext>();
    let portal_ctx = use_context::<PortalContext>();
    let id = portal_ctx.id;

    // When the anchor rectangle changes, update via this signal instead of
    // directly mutating entry.anchor_rect so the rectangle persists across rerenders
    let mut rect = use_signal(|| None);

    {
        let mut entries = provider_ctx.entries.write();
        let entry = entries.get_mut(&id).unwrap();
        entry.has_anchor_component = true;
        entry.measured_anchor_rect = rect();
    }

    use_drop(move || {
        // Discard rectangle info on unmount
        let mut entries = provider_ctx.entries.write();
        let entry = entries.get_mut(&id).unwrap();
        entry.has_anchor_component = false;
        entry.measured_anchor_rect = None;
    });

    let style = format!("{} width: fit-content; height: fit-content;", props.style);

    rsx! {
        RectObserver {
            on_rect_changed : move |r : Rect| { rect.set(Some(r)) },
            attributes : props.attributes,
            style : style,
            {props.children}
        }
    }
}

#[component]
pub fn PortalContent(props: PortalContentProps) -> Element {
    let mut provider_ctx = use_context::<PortalProviderContext>();
    let portal_ctx = use_context::<PortalContext>();
    let id = portal_ctx.id;

    {
        // Register content
        let mut entries = provider_ctx.entries.write();
        let entry = entries.get_mut(&id).unwrap();
        entry.content = Some(props);
    }

    use_drop(move || {
        let mut entries = provider_ctx.entries.write();
        let entry = entries.get_mut(&id).unwrap();
        entry.content = None;
    });

    rsx! {}
}

#[component]
pub fn PortalOverlay(props: PortalOverlayProps) -> Element {
    let mut provider_ctx = use_context::<PortalProviderContext>();
    let portal_ctx = use_context::<PortalContext>();
    let id = portal_ctx.id;

    {
        // Register overlay
        let mut entries = provider_ctx.entries.write();
        let entry = entries.get_mut(&id).unwrap();
        entry.overlay = Some(props);
    }

    use_drop(move || {
        let mut entries = provider_ctx.entries.write();
        let entry = entries.get_mut(&id).unwrap();
        entry.content = None;
    });

    rsx! {}
}

#[component]
pub fn Portal(props: PortalProps) -> Element {
    let mut provider_ctx = use_context::<PortalProviderContext>();
    let id = use_memo(|| alloc_id());
    let id = id(); 

    // Share the portal ID with children
    use_context_provider(|| PortalContext { id });

    let entry_data = {
        let param_v = AxisParam {
            alignment: props.vertical_alignment,
            spread: props.vertical_spread,
            offset: props.vertical_offset,
            overflow_policy: props.vertical_overflow_policy,
        };

        let param_h = AxisParam {
            alignment: props.horizontal_alignment,
            spread: props.horizontal_spread,
            offset: props.horizontal_offset,
            overflow_policy: props.horizontal_overflow_policy,
        };

        PortalEntryData {
            id: id,
            open: props.open,
            layer: props.layer,
            vertical_param: param_v,
            horizontal_param: param_h,
            has_anchor_component: false, // If an anchor exists, becomes true when `PortalAnchor` is rendered
            measured_anchor_rect: None,
            custom_anchor_rect: props.anchor_rect,
            content: None,
            overlay: None,
        }
    };

    {
        // Register portal
        let mut entries = provider_ctx.entries.write();
        entries.insert(id, entry_data);
    }

    use_drop(move || {
        let mut entries = provider_ctx.entries.write();
        entries.remove(&id);
    });

    rsx! {
        {props.children}
    }
}

#[component]
pub fn PortalProvider(props: PortalProviderProps) -> Element {
    let entries = use_signal(|| HashMap::new());

    use_context_provider(|| PortalProviderContext { entries });

    rsx! {
        div {
            style : format!("{} position: relative;", props.style),
            ..props.attribute,

            div {
                style : "position: absolute; top: 0; left: 0; width: 100%; height: 100%; z-index: 0;",
                {props.children}
            }

            PortalOutlet {}
        }
    }
}

// ------ Internal Types -------------------------------------------------------------------------------------------------------------------

// Unique identifier for a portal
#[derive(Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct PortalId(u64);

impl Display for PortalId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Portal-{}", self.0)
    }
}

static NEXT_ID: GlobalSignal<u64> = Signal::global(|| 0);
fn alloc_id() -> PortalId {
    let n = {
        let mut w = NEXT_ID.write();
        *w += 1;
        *w
    };
    PortalId(n)
}

// Context provided at the portal root and shared globally
#[derive(Clone)]
struct PortalProviderContext {
    pub entries: Signal<HashMap<PortalId, PortalEntryData>>,
}

// Context to share information for each portal
#[derive(Clone)]
struct PortalContext {
    pub id: PortalId, 
}

// Portal registration data
#[derive(Clone, PartialEq)]
struct PortalEntryData {
    pub id: PortalId,
    pub open: bool,
    pub layer: i32,
    pub has_anchor_component: bool,         // Whether a `PortalAnchor` component exists in the portal's children 
    pub measured_anchor_rect: Option<Rect>, // Rectangle of the `PortalAnchor` component
    pub custom_anchor_rect : Option<Rect>,  // Value of the `anchor_rect` property from `PortalProps`
    pub vertical_param: AxisParam,
    pub horizontal_param: AxisParam,
    pub content: Option<PortalContentProps>,
    pub overlay: Option<PortalOverlayProps>,
}

// Struct that manages placement parameters
#[derive(Clone, PartialEq)]
struct AxisParam {
    pub alignment: Alignment,
    pub spread: Spread,
    pub offset: f64,
    pub overflow_policy: OverflowPolicy,
}

// ------ Internal Components ---------------------------------------------------------------------------------------------------------------

#[derive(Props, Clone, PartialEq)]
struct PortalOutletProps {}

// Component that renders registered portal content and overlay in layer order
#[component]
fn PortalOutlet(props: PortalOutletProps) -> Element {
    let _ = &props; // Dummy to avoid warnings

    let provider_ctx = use_context::<PortalProviderContext>();
    let mut rect = use_signal(|| None);

    let sorted_ids = {
        let entries = provider_ctx.entries.read();
        let mut ids = entries
            .values()
            .filter(|data| data.open)
            .map(|data| (data.id, data.layer))
            .collect::<Vec<_>>();
        ids.sort_by_key(|(_, layer)| *layer);
        ids.into_iter().map(|(id, _)| id).collect::<Vec<_>>()
    };

    let overlay_id = {
        let entries = provider_ctx.entries.read();
        sorted_ids
            .iter()
            .rfind(|id| entries.get(id).unwrap().overlay.is_some())
            .map(|id| *id)
    };

    let outlet_measured = rect().is_some();

    rsx! {
        RectObserver {
            on_rect_changed : move |r : Rect| { rect.set(Some(r)) },
            style : "position: absolute; top: 0; left: 0; width: 100%; height: 100%; z-index: 1; pointer-events: none;",

            if outlet_measured {
                for (i, id) in sorted_ids.iter().enumerate() {
                    PortalEntry {
                        id : *id,
                        z_index : i * 2 + 1,
                        outlet_rect : rect().unwrap(),
                    }

                    if overlay_id == Some(*id) {
                        PortalOverlayEntry {
                            id : *id,
                            z_index : i * 2,
                        }
                    }
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct PortalEntryProps {
    pub id: PortalId,
    pub z_index: usize,
    pub outlet_rect: Rect,
}

// Component that renders a single registered portal content
#[component]
fn PortalEntry(props: PortalEntryProps) -> Element {
    let provider_ctx = use_context::<PortalProviderContext>();
    let mut size = use_signal(|| None);

    let on_rect_changed = move |r: Rect| {
        let current = *size.read();
        let new = Some(r.size);
        if current != new {
            size.set(new);
        }
    };

    let id = props.id;
    let z_index = props.z_index;
    let entries = provider_ctx.entries.read();
    let data = entries.get(&id).unwrap();

    let use_custom_anchor = data.custom_anchor_rect.is_some();
    let anchor_preparing = !use_custom_anchor && data.has_anchor_component && data.measured_anchor_rect.is_none();
    let has_content = data.content.is_some();

    if anchor_preparing || !has_content {
        return rsx! {};
    }

    let anchor_rect = if use_custom_anchor {
        data.custom_anchor_rect.clone()
    } else {
        data.measured_anchor_rect.clone()
    };

    let content_props = data.content.as_ref().unwrap();
    let content_style = match *size.read() {
        None => format!(
            "{} width: fit-content; height: fit-content; position: absolute; z-index: {}; opacity: 0; pointer-events: none;",
            content_props.style, z_index
        ),
        Some(size) => {
            let pos =
                calc_content_position(data, size, anchor_rect, props.outlet_rect);

            // Since `calc_content_position` uses the viewport as the reference, convert to a position relative to the outlet
            let top = pos.y - props.outlet_rect.origin.y;
            let left = pos.x - props.outlet_rect.origin.x;

            format!("pointer-events: auto; opacity: 1; {} width: fit-content; height: fit-content; position: absolute; top: {}px; left: {}px; z-index: {};", content_props.style, top, left, z_index)
        }
    };

    rsx! {
        RectObserver {
            on_rect_changed : on_rect_changed,
            attributes : content_props.attributes.clone(),
            style : content_style,
            {content_props.children.clone()}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct PortalOverlayEntryProps {
    pub id: PortalId,
    pub z_index: usize,
}

#[component]
fn PortalOverlayEntry(props: PortalOverlayEntryProps) -> Element {
    let provider_ctx = use_context::<PortalProviderContext>();
    let id = props.id;
    let z_index = props.z_index;
    let entries = provider_ctx.entries.read();
    let data = entries.get(&id).unwrap();

    match &data.overlay {
        None => rsx! {},
        Some(overlay_props) => {
            let overlay_style = format!("pointer-events: auto; {} position: absolute; top: 0; left: 0; width: 100%; height: 100%; z-index: {};", overlay_props.style, z_index);
            rsx! {
                div {
                    style : overlay_style,
                    ..overlay_props.attributes.clone(),
                    {overlay_props.children.clone()}
                }
            }
        }
    }
}

// ------ Position calculation -------------------------------------------------------------------------------------------------------------------

fn calc_content_range(
    length: f64,
    param: &AxisParam,
    base: Range<f64>,
    bounds: Range<f64>,
) -> Range<f64> {
    let desired = match (param.alignment, param.spread) {
        (Alignment::Center, _) => {
            let base_point = (base.start + base.end) * 0.5 + param.offset;
            Range {
                start: base_point - length * 0.5,
                end: base_point + length * 0.5,
            }
        }
        (Alignment::Start, Spread::Inside) => {
            let base_point = base.start + param.offset;
            Range {
                start: base_point,
                end: base_point + length,
            }
        }
        (Alignment::Start, Spread::Outside) => {
            let base_point = base.start - param.offset;
            Range {
                start: base_point - length,
                end: base_point,
            }
        }
        (Alignment::End, Spread::Inside) => {
            let base_point = base.end - param.offset;
            Range {
                start: base_point - length,
                end: base_point,
            }
        }
        (Alignment::End, Spread::Outside) => {
            let base_point = base.end + param.offset;
            Range {
                start: base_point,
                end: base_point + length,
            }
        }
    };

    match (param.overflow_policy, param.alignment) {
        (OverflowPolicy::Ignore, _) => desired,

        (OverflowPolicy::Shrink, _) => Range {
            start: desired.start.max(bounds.start),
            end: desired.end.min(bounds.end),
        },

        (OverflowPolicy::Clamp, Alignment::Center) => desired,
        (OverflowPolicy::Clamp, Alignment::Start) => {
            if bounds.end < desired.end {
                Range {
                    start: bounds.end - length,
                    end: bounds.end,
                }
            } else {
                desired
            }
        }
        (OverflowPolicy::Clamp, Alignment::End) => {
            if desired.start < bounds.start {
                Range {
                    start: bounds.start,
                    end: bounds.start + length,
                }
            } else {
                desired
            }
        }

        (OverflowPolicy::Flip, Alignment::Center) => desired,
        (OverflowPolicy::Flip, _) if bounds.start <= desired.start && desired.end <= bounds.end => {
            desired
        }
        (OverflowPolicy::Flip, _) => {
            let flip_alignment = if param.alignment == Alignment::Start {
                Alignment::End
            } else {
                Alignment::Start
            };
            let param = AxisParam {
                spread: param.spread,
                offset: param.offset,
                alignment: flip_alignment,
                overflow_policy: OverflowPolicy::Clamp,
            };
            calc_content_range(length, &param, base, bounds)
        }
    }
}

fn calc_content_position(
    data: &PortalEntryData,
    content_size: Size2D<f64, Pixels>,
    anchor: Option<Rect>,
    bounds: Rect,
) -> Point2D<f64, Pixels> {
    let bounds_v = Range {
        start: bounds.min_y(),
        end: bounds.max_y(),
    };
    let bounds_h = Range {
        start: bounds.min_x(),
        end: bounds.max_x(),
    };

    match anchor {
        Some(anchor) => {
            let anchor_v = Range {
                start: anchor.min_y(),
                end: anchor.max_y(),
            };
            let anchor_h = Range {
                start: anchor.min_x(),
                end: anchor.max_x(),
            };

            let range_v = calc_content_range(
                content_size.height,
                &data.vertical_param,
                anchor_v,
                bounds_v,
            );
            let range_h = calc_content_range(
                content_size.width,
                &data.horizontal_param,
                anchor_h,
                bounds_h,
            );

            Point2D::new(range_h.start, range_v.start)
        }
        None => {
            let param_v = AxisParam {
                spread: Spread::Inside,
                ..data.vertical_param
            };
            let param_h = AxisParam {
                spread: Spread::Inside,
                ..data.horizontal_param
            };

            let range_v =
                calc_content_range(content_size.height, &param_v, bounds_v.clone(), bounds_v);
            let range_h =
                calc_content_range(content_size.width, &param_h, bounds_h.clone(), bounds_h);

            Point2D::new(range_h.start, range_v.start)
        }
    }
}

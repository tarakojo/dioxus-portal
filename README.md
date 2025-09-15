# dioxus-portal

Portal component for Dioxus - enables overlays, tooltips, dropdowns, and modals that render outside their parent components.

## Features

- **Flexible positioning**: Control alignment, spread, and overflow behavior
- **Layer management**: z-index control for proper stacking
- **Anchor-based positioning**: Attach overlays to specific elements 
- **Viewport-centered modals**: Position content relative to the viewport
- **Overlay support**: Full-screen backgrounds for modals and dropdowns

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
dioxus-portal = "0.0.0"
```

## Basic Usage

```rust
use dioxus::prelude::*;
use dioxus_portal::*;

fn app() -> Element {
    let mut open = use_signal(|| false);
    
    rsx! {
        PortalProvider {
            // Your main app content
            button {
                onclick: move |_| open.set(true),
                "Open Dropdown"
            }
            
            Portal { open: *open.read(), layer: 10,
                PortalAnchor {
                    // Anchor element (button in this case)
                }
                PortalOverlay {
                    // Background overlay
                    div {
                        onclick: move |_| open.set(false),
                        style: "position: absolute; inset: 0; background: rgba(0,0,0,0.5);"
                    }
                }
                PortalContent {
                    // Dropdown content
                    div {
                        style: "padding: 8px; background: white; border-radius: 8px;",
                        "Dropdown content"
                    }
                }
            }
        }
    }
}
```

## Components

### PortalProvider
Root component that provides the rendering context for all portals.

### Portal
Main portal component that manages positioning and visibility.

**Props:**
- `open: bool` - Controls visibility
- `layer: i32` - Z-index for stacking order
- `anchor_rect: Option<Rect>` - Custom anchor position
- `vertical_alignment/horizontal_alignment` - Position relative to anchor
- `vertical_spread/horizontal_spread` - Inside or outside anchor bounds
- `vertical_offset/horizontal_offset` - Additional positioning offset
- `vertical_overflow_policy/horizontal_overflow_policy` - Overflow handling

### PortalAnchor
Defines the reference element for positioning. Portal content will be positioned relative to this element.

### PortalContent
The actual content to be rendered in the portal.

### PortalOverlay
Optional overlay element (typically for modal backgrounds).

## Examples

See the [demo example](examples/demo.rs) for complete implementations of:
- Dropdown menus
- Tooltips with hover interaction
- Modal dialogs

Run the demo:
```bash
cargo run --example demo
```

## License

MIT
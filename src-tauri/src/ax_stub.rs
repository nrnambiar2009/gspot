//! Stand-in for `ax.rs` on platforms without an accessibility backend yet.
//! Same names, no capabilities: GSpot runs as a panel over typed text.
//! The AT-SPI backend (gate L1) replaces this on Linux.

pub use crate::geom::Rect;

/// The focused text field of another app. Nothing can produce one here.
#[derive(Clone, Debug)]
pub struct AXElement(());

pub fn is_trusted() -> bool {
    true
}

pub fn prompt_trust() -> bool {
    true
}

pub fn mouse_location_top_left() -> Option<(f64, f64)> {
    None
}

pub fn post_mouse_move(_x: f64, _y: f64) -> bool {
    false
}

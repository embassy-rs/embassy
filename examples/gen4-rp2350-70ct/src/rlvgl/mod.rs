//! rlvgl integration for gen4-RP2350-70CT (PSRAM RGB565 + PIO scan-out).

pub mod demo;
pub mod render;

pub use demo::{DemoUi, DirtyWidgets};
pub use render::{TILE_LINES, render_node, render_tree, render_tree_tiled};

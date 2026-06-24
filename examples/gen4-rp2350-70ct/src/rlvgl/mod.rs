//! rlvgl integration for gen4-RP2350-70CT (PSRAM RGB565 + PIO scan-out).

pub mod demo;
pub mod render;

pub use demo::DemoUi;
pub use render::{render_node, render_tree};

//! LVGL integration for Riverdi RVT50 (Embassy + [lv_binding_rust](https://github.com/lvgl/lv_binding_rust)).
//!
//! Built against the vendored `lv_binding_rust` master under
//! `vendor/lv_binding_rust/` (see `[patch.crates-io]` in `Cargo.toml`); see
//! the `lvgl_buttons` binary for a from-scratch minimal LVGL demo against
//! the same API.
//!
//! - [`Rvt50Display`] wraps [`lvgl::Display::register`] for the LTDC framebuffer.
//! - [`Rvt50Touch`] wraps [`lvgl::input_device::pointer::Pointer::register`] for the I2C touch.
//! - [`HallUi`] composes them with the JSON-driven hall lighting widgets.

pub mod display;
pub mod hall_ui;
pub mod input;
pub mod theme;

pub use display::Rvt50Display;
pub use hall_ui::HallUi;
pub use input::Rvt50Touch;

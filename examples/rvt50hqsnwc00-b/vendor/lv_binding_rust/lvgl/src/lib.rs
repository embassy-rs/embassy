//! [![github]](https://github.com/rafaelcaricio/lvgl-rs)&ensp;[![crates-io]](https://crates.io/crates/lvgl)&ensp;[![docs-rs]](crate)
//!
//! [github]: https://img.shields.io/badge/github-8da0cb?style=for-the-badge&labelColor=555555&logo=github
//! [crates-io]: https://img.shields.io/badge/crates.io-fc8d62?style=for-the-badge&labelColor=555555&logo=rust
//! [docs-rs]: https://img.shields.io/badge/docs.rs-66c2a5?style=for-the-badge&labelColor=555555&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K
//!
//! <br>
//!
//! [LVGL][1] bindings for Rust. A powerful and easy-to-use embedded GUI with many widgets, advanced visual effects, and
//! low memory footprint. This crate is compatible with `#![no_std]` environments by default.
//!
//! [1]: https://docs.lvgl.io/8.3/get-started/index.html
//!

#![cfg_attr(not(test), no_std)]
#![cfg_attr(feature = "nightly", feature(cfg_accessible))]
#![cfg_attr(feature = "nightly", feature(error_in_core))]

pub use lvgl_sys as sys;

#[macro_use]
extern crate bitflags;

#[macro_use]
mod lv_core;

#[cfg(feature = "alloc")]
extern crate alloc;

// We can ONLY use `alloc::boxed::Box` if `lvgl_alloc` is enabled.
// That is because we use `Box` to send memory references to LVGL. Since the global allocator, when
// `lvgl_alloc` feature is enabled, is the LVGL memory manager then everything is in LVGL
// managed memory anyways. In that case we can use the Rust's provided Box definition.
#[cfg(feature = "lvgl_alloc")]
use ::alloc::boxed::Box;

#[cfg(feature = "lvgl_alloc")]
mod allocator;

#[cfg(not(feature = "lvgl_alloc"))]
pub(crate) mod mem;

// When LVGL allocator is not used on the Rust code, we need a way to add objects to the LVGL
// managed memory. We implement a very simple `Box` that has the minimal features to copy memory
// safely to the LVGL managed memory.
#[cfg(not(feature = "lvgl_alloc"))]
use crate::mem::Box;

#[cfg(not(feature = "embedded_graphics"))]
pub mod point;

#[cfg(not(feature = "embedded_graphics"))]
use crate::point::Point;

#[cfg(feature = "embedded_graphics")]
use embedded_graphics::geometry::Point;

pub use crate::lv_core::*;
pub use display::*;
pub use functions::*;
pub use support::*;

mod display;
mod functions;
mod support;

#[cfg(feature = "drivers")]
pub mod drivers;
pub mod font;
pub mod input_device;
pub mod misc;
pub mod widgets;

#[cfg(feature = "rust_timer")]
pub mod timer;

#[cfg(feature = "unsafe_no_autoinit")]
static mut IS_INIT: bool = false;
#[cfg(not(feature = "unsafe_no_autoinit"))]
static mut IS_INIT: bool = true;

/// Initializes LVGL. Call at the start of the program, or after safely
/// deinitializing with `deinit()`.
pub fn init() {
    unsafe {
        if !IS_INIT {
            lvgl_sys::lv_init();
            IS_INIT = true;
        }
    }
}

/// Uninitializes LVGL. Make sure to reinitialize LVGL with `init()` before
/// accessing its functionality
///
/// # Safety
///
/// After calling, ensure existing LVGL-related values are not accessed even if
/// LVGL is reinitialized.
#[cfg(not(feature = "custom_allocator"))]
pub unsafe fn deinit() {
    unsafe {
        if IS_INIT {
            lvgl_sys::lv_deinit();
            IS_INIT = false;
        }
    }
}

#[cfg(not(feature = "unsafe_no_autoinit"))]
#[ctor::ctor]
fn once_init() {
    unsafe {
        lvgl_sys::lv_init();
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use crate::display::{Display, DrawBuffer};

    pub(crate) fn initialize_test(buf: bool) {
        unsafe { crate::deinit() };
        crate::init();
        if buf {
            const REFRESH_BUFFER_SIZE: usize = 240 * 240 / 10;
            let buffer = DrawBuffer::<REFRESH_BUFFER_SIZE>::default();
            let _ = Display::register(buffer, 240, 240, |_| {}).unwrap();
        }
    }
}

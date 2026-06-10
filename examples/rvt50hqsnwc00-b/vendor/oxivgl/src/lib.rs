// SPDX-License-Identifier: MIT OR Apache-2.0
//! Safe Rust bindings for LVGL on embedded and host targets.
#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", feature(type_alias_impl_trait))]
#![cfg_attr(target_os = "none", feature(asm_experimental_arch))]

extern crate alloc;

mod fmt;

/// Built-in LVGL font handles.
pub mod fonts;
/// LVGL driver initialization (tick source, log bridge).
pub mod driver;
/// Display output: DMA-aligned render buffers and display initialisation.
pub mod display;
/// ESP32 flush pipeline: async DMA transfer between LVGL and the display driver.
#[cfg(feature = "esp-hal")]
pub mod flush_pipeline;
/// Animation descriptors, path functions, and timeline management.
pub mod anim;
/// Style system: builders, selectors, themes, gradients, and color palettes.
pub mod style;
/// Safe wrapper around LVGL events.
pub mod event;
/// Universal convenience re-exports (`use oxivgl::prelude::*`).
pub mod prelude;
/// LVGL periodic timer with polling-based trigger detection.
pub mod timer;
/// View trait and LVGL render loop.
pub mod view;
/// Navigation stack: push/pop/replace/modal view management.
pub mod navigator;
/// General LVGL enum types (event codes, object flags, states, opacity, scroll).
pub mod enums;
/// Layout types: flex flow/alignment, grid alignment/cells, layout engine.
pub mod layout;
/// Type-safe LVGL widget wrappers.
pub mod widgets;
/// Draw primitives: `Area`, `Layer`, rectangle/label descriptors, draw task wrappers.
pub mod draw;
/// Input device queries.
pub mod indev;
/// Screen capture (host-only).
#[cfg(not(target_os = "none"))]
pub mod snapshot;
/// LVGL math utility wrappers (Bezier, mapping).
pub mod math;
/// LVGL built-in icon symbols (Font Awesome subset).
pub mod symbols;
/// Owned LVGL draw buffer wrapping `lv_draw_buf_t`.
pub mod draw_buf;
/// LVGL translation/i18n support (`LV_USE_TRANSLATION`).
pub mod translation;
/// LVGL input group — focus management for keyboard/encoder navigation.
pub mod group;
/// Grid navigation (`LV_USE_GRIDNAV`) — keyboard-driven focus inside containers.
pub mod gridnav;

/// Declare an LVGL image asset compiled by `oxivgl-build`.
///
/// Equivalent to LVGL's `LV_IMAGE_DECLARE`. Generates a safe
/// `&'static lv_image_dsc_t` reference from the `extern "C"` symbol
/// produced by `LVGLImage.py`.
///
/// The generated function has the same name as the C symbol and returns
/// `&'static lv_image_dsc_t`.
///
/// # Example
///
/// ```ignore
/// oxivgl::image_declare!(img_cogwheel_argb);
/// let img = Image::new(&screen)?;
/// img.set_src(img_cogwheel_argb());
/// ```
#[macro_export]
macro_rules! image_declare {
    ($name:ident) => {
        /// Returns `&'static lv_image_dsc_t` for the compiled image asset.
        #[allow(non_snake_case)]
        fn $name() -> &'static $crate::widgets::lv_image_dsc_t {
            unsafe extern "C" {
                #[allow(non_upper_case_globals)]
                static $name: $crate::widgets::lv_image_dsc_t;
            }
            // SAFETY: the symbol is a valid lv_image_dsc_t compiled into the
            // binary by oxivgl-build. It is 'static and immutable.
            unsafe { &$name }
        }
    };
}



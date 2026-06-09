//! LVGL font handling logic and helper structures.
//!
//! Fonts can be built into LVGL or custom-generated with the [official online
//! converter].
//!
//! # Built-in fonts
//! LVGL offers several fonts for Latin (only up to ASCII), Arabic, Persian,
//! and Hebrew, along with CJK glyphs. These can be enabled in `lv_conf.h` and
//! once enabled will be usable as-is:
//! ```ignore
//! use lvgl::font::Font;
//! use lvgl::style::Style;
//!
//! fn main() {
//!     let mut my_style = Style::default();
//!     my_style.set_text_font(Font::montserrat_48());
//!     // Use the style
//! }
//! ```
//!
//! Built-in fonts are *only* available if the `nightly` feature is enabled for
//! the `lvgl` crate.
//!
//! # Custom fonts
//! Custom fonts encoded into C files can be added. At compile time, the
//! following locations will be searched in order:
//! - `LVGL_FONTS_DIR` environment variable (if set)
//! - `fonts/` in the project root directory, non-recursively
//!
//! Any detected fonts will be made available, namespaced under the `lvgl_sys`
//! crate. They can then unsafely be converted into `Font` structs, as seen
//! here with the Noto font used in the `demo` example:
//! ```
//! use lvgl::font::Font;
//! use lvgl::style::Style;
//!
//! let noto_80 = unsafe {
//!     Font::new_raw(lvgl_sys::noto_sans_numeric_80)
//! };
//! let mut my_style = Style::default();
//! my_style.set_text_font(noto_80);
//! // Use the style
//! ```
//! This operation is inherently unsafe as it instantiates and uses arbitrary
//! data structures that the Rust compiler can't check.
//!
//! [official online converter]: https://lvgl.io/tools/fontconverter

mod generic;
pub use generic::*;

#[cfg(feature = "nightly")]
mod builtin;

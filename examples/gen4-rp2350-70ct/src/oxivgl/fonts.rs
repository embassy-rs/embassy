//! Montserrat fonts with Latin-1 coverage (ä ö ü Ä Ö Ü ß, …).
//!
//! Font binaries live in [`../../fonts/`](../../fonts/) — regenerate with
//! `fonts/generate.sh` when changing glyph coverage or sizes.
//!
//! oxivgl's own `fonts` module only exposes `MONTSERRAT_14`/`16` when the
//! *standard* (ASCII-only) `LV_FONT_MONTSERRAT_14`/`16` are enabled. This board
//! instead enables the custom `LV_FONT_MONTSERRAT_*_LATIN` faces in
//! `conf/lv_conf.h` (for German umlauts), so we wrap those symbols directly via
//! [`Font::from_extern`].

use core::ptr::addr_of;

use oxivgl::fonts::Font;

/// Custom Montserrat 14 pt with Latin-1 coverage.
// SAFETY: `lv_font_montserrat_14_latin` is a valid static `lv_font_t` emitted by
// oxivgl-sys (enabled in `conf/lv_conf.h`, sources in `fonts/`).
pub static MONTSERRAT_14: Font =
    unsafe { Font::from_extern(addr_of!(oxivgl_sys::lv_font_montserrat_14_latin) as *const ()) };

/// Custom Montserrat 16 pt with Latin-1 coverage.
// SAFETY: as above for the 16 pt face.
pub static MONTSERRAT_16: Font =
    unsafe { Font::from_extern(addr_of!(oxivgl_sys::lv_font_montserrat_16_latin) as *const ()) };

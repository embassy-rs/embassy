// SPDX-License-Identifier: MIT OR Apache-2.0
//! Font types and built-in LVGL font constants.
//!
//! Embedded RVT50 builds enable only Montserrat 14/16 in `conf/lv_conf.h`.
//! Additional `MONTSERRAT_*` / CJK fonts are available when the matching
//! `LV_FONT_*` options are enabled in `lv_conf.h` and exposed by `oxivgl-sys`.

use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::ptr::addr_of;

use oxivgl_sys::{lv_font_glyph_dsc_t, lv_font_get_glyph_dsc_fmt_txt, lv_font_t};

/// Wrapper around an LVGL font pointer.
#[derive(Copy, Clone, Debug)]
pub struct Font(pub(crate) *const lv_font_t);

// SAFETY: lv_font_t is immutable C data; sharing across threads is safe.
unsafe impl Send for Font {}
unsafe impl Sync for Font {}

impl Font {
    /// # Safety
    /// `ptr` must point to a valid, static LVGL font object.
    pub const unsafe fn from_raw(ptr: *const lv_font_t) -> Self {
        Font(ptr)
    }

    /// Create a [`Font`] from an opaque extern-C symbol address.
    ///
    /// # Safety
    /// `ptr` must be the address of a valid, static `lv_font_t` object in
    /// memory.
    pub const unsafe fn from_extern(ptr: *const ()) -> Self {
        Font(ptr as *const lv_font_t)
    }

    /// Return the raw LVGL font pointer. Valid for the `'static` lifetime of
    /// the font object.
    pub fn as_ptr(self) -> *const lv_font_t {
        self.0
    }
}

/// Fixed-width font derived from an existing LVGL font.
pub struct FixedWidthFont {
    inner: UnsafeCell<MaybeUninit<lv_font_t>>,
}

// SAFETY: init() must be called from the LVGL task (single-threaded).
unsafe impl Send for FixedWidthFont {}
unsafe impl Sync for FixedWidthFont {}

impl FixedWidthFont {
    pub const fn new() -> Self {
        Self {
            inner: UnsafeCell::new(MaybeUninit::zeroed()),
        }
    }

    pub fn init(&self, source: Font, advance_w: u16) -> Font {
        // SAFETY: source.as_ptr() points to a valid static lv_font_t.
        unsafe {
            let font_ptr = self.inner.get();
            core::ptr::copy_nonoverlapping(source.as_ptr(), (*font_ptr).as_mut_ptr(), 1);
            let font = (*font_ptr).as_mut_ptr();
            (*font).get_glyph_dsc = Some(fixed_width_get_glyph_dsc);
            (*font).user_data = advance_w as usize as *mut core::ffi::c_void;
            Font((*font_ptr).as_ptr())
        }
    }
}

unsafe extern "C" fn fixed_width_get_glyph_dsc(
    font: *const lv_font_t,
    dsc: *mut lv_font_glyph_dsc_t,
    letter: u32,
    letter_next: u32,
) -> bool {
    unsafe {
        let ret = lv_font_get_glyph_dsc_fmt_txt(font, dsc, letter, letter_next);
        if !ret {
            return false;
        }
        let adv = (*font).user_data as usize as u16;
        (*dsc).adv_w = adv;
        (*dsc).ofs_x = (adv as i16 - (*dsc).box_w as i16) / 2;
        true
    }
}

/// LVGL built-in Montserrat 14 pt (enabled in `conf/lv_conf.h`).
pub static MONTSERRAT_14: Font = Font(addr_of!(oxivgl_sys::lv_font_montserrat_14));
/// LVGL built-in Montserrat 16 pt (enabled in `conf/lv_conf.h`).
pub static MONTSERRAT_16: Font = Font(addr_of!(oxivgl_sys::lv_font_montserrat_16));

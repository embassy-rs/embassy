use crate::Box;

/// An LVGL font. Can be applied to styles using `set_text_font()`.
pub struct Font {
    inner: Box<lvgl_sys::lv_font_t>,
}

impl From<Font> for *const lvgl_sys::lv_font_t {
    fn from(value: Font) -> Self {
        Box::<lvgl_sys::lv_font_t>::into_raw(value.inner)
    }
}

impl Font {
    /// Creates a `Font` from a given `lv_font_t`.
    /// # Safety
    /// The `lv_font_t` must have been well-defined in the C code which
    /// constructs it.
    pub unsafe fn new_raw(raw: lvgl_sys::lv_font_t) -> Self {
        Font {
            inner: Box::new(raw),
        }
    }
}

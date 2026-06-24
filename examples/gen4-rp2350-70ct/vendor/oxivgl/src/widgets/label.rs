// SPDX-License-Identifier: MIT OR Apache-2.0
use core::{ffi::c_void, ops::Deref, ptr::null_mut};

use oxivgl_sys::*;

use super::{
    obj::{AsLvHandle, Obj},
    subject::Subject,
    with_cstr, WidgetError,
};

/// LVGL text label widget.
///
/// # Examples
///
/// ```no_run
/// use oxivgl::widgets::{Align, Label, Screen};
///
/// let screen = Screen::active().unwrap();
/// let label = Label::new(&screen).unwrap();
/// label.text("Hello").align(Align::Center, 0, 0);
/// ```
#[derive(Debug)]
pub struct Label<'p> {
    obj: Obj<'p>,
}

impl<'p> AsLvHandle for Label<'p> {
    fn lv_handle(&self) -> *mut lv_obj_t {
        self.obj.lv_handle()
    }
}

impl<'p> Deref for Label<'p> {
    type Target = Obj<'p>;
    fn deref(&self) -> &Obj<'p> {
        &self.obj
    }
}

impl<'p> Label<'p> {
    /// Create a new label widget.
    pub fn new(parent: &impl AsLvHandle) -> Result<Self, WidgetError> {
        let parent_ptr = parent.lv_handle();
        assert_ne!(parent_ptr, null_mut(), "Parent widget cannot be null");
        // SAFETY: parent_ptr non-null (asserted above); lv_init() called via
        // LvglDriver.
        let handle = unsafe { lv_label_create(parent_ptr) };
        if handle.is_null() {
            Err(WidgetError::LvglNullPointer)
        } else {
            Ok(Label {
                obj: Obj::from_raw(handle),
            })
        }
    }

    /// Set label text. Accepts any `&str` (no length cap, no NUL terminator
    /// required); LVGL copies the string internally.
    pub fn text(&self, s: &str) -> &Self {
        assert_ne!(self.obj.handle(), null_mut(), "Label handle cannot be null");
        // SAFETY: handle non-null (asserted above); with_cstr supplies a
        // NUL-terminated buffer valid for the call. LVGL copies internally.
        with_cstr(s, |p| unsafe { lv_label_set_text(self.obj.handle(), p) });
        self
    }

    /// Set translation tag — the label text will auto-update when the
    /// language changes via [`translation::set_language`](crate::translation::set_language).
    ///
    /// Requires `LV_USE_TRANSLATION = 1` in `lv_conf.h`.
    pub fn set_translation_tag(&self, tag: &str) -> &Self {
        assert_ne!(self.obj.handle(), null_mut(), "Label handle cannot be null");
        // SAFETY: handle non-null (asserted above); with_cstr supplies a
        // NUL-terminated buffer valid for the call. LVGL copies the string via
        // lv_strdup (see lv_label_set_translation_tag in lv_label.c), so no
        // pointer is retained after this call returns.
        with_cstr(tag, |p| unsafe { lv_label_set_translation_tag(self.obj.handle(), p) });
        self
    }

    /// Set label text without a length limit.
    ///
    /// Equivalent to [`text`](Self::text), which is now itself uncapped — kept
    /// as a deprecated alias for callers written against the old 127-byte
    /// `text`.
    #[deprecated(note = "`text` is now uncapped; call `text` directly")]
    pub fn text_long(&self, s: &str) -> &Self {
        self.text(s)
    }

    /// Set the label long mode (wrap, scroll, clip, etc.).
    pub fn set_long_mode(&self, mode: LabelLongMode) -> &Self {
        assert_ne!(self.obj.handle(), null_mut(), "Label handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_label_set_long_mode(self.obj.handle(), mode as u32) };
        self
    }

    /// Get the label long mode.
    pub fn get_long_mode(&self) -> LabelLongMode {
        assert_ne!(self.obj.handle(), null_mut(), "Label handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        // lv_label_long_mode_t values 0–4 are all covered by LabelLongMode.
        unsafe { core::mem::transmute(lv_label_get_long_mode(self.obj.handle())) }
    }

    /// Get whether recoloring is enabled (inline color codes like `#ff0000 red#`).
    pub fn get_recolor(&self) -> bool {
        assert_ne!(self.obj.handle(), null_mut(), "Label handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_label_get_recolor(self.obj.handle()) }
    }

    /// Get the start index of the selected text region.
    pub fn get_text_selection_start(&self) -> u32 {
        assert_ne!(self.obj.handle(), null_mut(), "Label handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_label_get_text_selection_start(self.obj.handle()) }
    }

    /// Bind label text to a subject with a printf-style format.
    ///
    /// The format string must be `'static` because LVGL stores the pointer
    /// internally for the lifetime of the binding (spec §12.4).
    ///
    /// Use a `c"..."` literal: `label.bind_text(&subject, c"%d °C")`.
    pub fn bind_text(&self, subject: &Subject, fmt: &'static core::ffi::CStr) -> &Self {
        // SAFETY: lv_handle() non-null (checked in new()); subject is pinned;
        // fmt is 'static so the pointer remains valid while LVGL holds it.
        unsafe { lv_label_bind_text(self.lv_handle(), subject.as_ptr(), fmt.as_ptr()) };
        self
    }

    /// Reactively update this label's text based on a subject's integer value.
    ///
    /// The mapping function is called whenever the subject changes.  The
    /// observer is automatically removed when this label is deleted.
    ///
    /// ```ignore
    /// label.bind_text_map(&subject, |state| match state {
    ///     1 => "Active",
    ///     _ => "Idle",
    /// });
    /// ```
    pub fn bind_text_map(
        &self,
        subject: &Subject,
        map: fn(i32) -> &'static str,
    ) -> &Self {
        const _: () = assert!(core::mem::size_of::<fn(i32) -> &'static str>() == core::mem::size_of::<*mut core::ffi::c_void>());
        unsafe extern "C" fn trampoline(
            observer: *mut lv_observer_t,
            subject: *mut lv_subject_t,
        ) {
            // SAFETY: user_data is the fn pointer set in bind_text_map;
            // size equality verified by const assert above.
            // observer target is a valid label (registered via add_observer_obj).
            unsafe {
                let map_ptr = lv_observer_get_user_data(observer) as *const ();
                let map: fn(i32) -> &'static str = core::mem::transmute(map_ptr);
                let value = lv_subject_get_int(subject);
                let text = map(value);
                let label_ptr = lv_observer_get_target_obj(observer);
                // Pass the mapped &str to LVGL as a NUL-terminated string.
                with_cstr(text, |p| lv_label_set_text(label_ptr, p));
            }
        }
        // SAFETY: lv_handle() non-null; subject pinned; fn pointer is pointer-sized.
        unsafe {
            lv_subject_add_observer_obj(
                subject.as_ptr(),
                Some(trampoline),
                self.lv_handle(),
                map as *const () as *mut c_void,
            )
        };
        self
    }
}

/// Label long-mode behaviour.
#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum LabelLongMode {
    /// Keep the object width, wrap lines and expand height.
    Wrap = lv_label_long_mode_t_LV_LABEL_LONG_MODE_WRAP,
    /// Keep size, write dots at end if text too long.
    Dots = lv_label_long_mode_t_LV_LABEL_LONG_MODE_DOTS,
    /// Keep size, scroll text back and forth.
    Scroll = lv_label_long_mode_t_LV_LABEL_LONG_MODE_SCROLL,
    /// Keep size, scroll text circularly.
    ScrollCircular = lv_label_long_mode_t_LV_LABEL_LONG_MODE_SCROLL_CIRCULAR,
    /// Keep size, clip text.
    Clip = lv_label_long_mode_t_LV_LABEL_LONG_MODE_CLIP,
}

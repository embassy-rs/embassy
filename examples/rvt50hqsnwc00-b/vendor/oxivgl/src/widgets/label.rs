// SPDX-License-Identifier: MIT OR Apache-2.0
use alloc::vec::Vec;
use core::{ffi::{c_char, c_void}, ops::Deref, ptr::null_mut};

use oxivgl_sys::*;

use super::{
    obj::{AsLvHandle, Obj},
    subject::Subject,
    WidgetError,
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

    /// Set label text. Accepts any `&str` (no NUL terminator required).
    /// LVGL copies the string internally. Truncates at 127 bytes.
    /// For longer text use [`text_long`](Self::text_long).
    pub fn text(&self, s: &str) -> &Self {
        assert_ne!(self.obj.handle(), null_mut(), "Label handle cannot be null");
        let bytes = s.as_bytes();
        let len = bytes.len().min(127);
        let mut buf = [0u8; 128];
        buf[..len].copy_from_slice(&bytes[..len]);
        // SAFETY: handle non-null (asserted above); buf is NUL-terminated
        // (zero-initialized, len ≤ 127).
        unsafe { lv_label_set_text(self.obj.handle(), buf.as_ptr() as *const c_char) };
        self
    }

    /// Set translation tag — the label text will auto-update when the
    /// language changes via [`translation::set_language`](crate::translation::set_language).
    ///
    /// Requires `LV_USE_TRANSLATION = 1` in `lv_conf.h`.
    pub fn set_translation_tag(&self, tag: &str) -> &Self {
        assert_ne!(self.obj.handle(), null_mut(), "Label handle cannot be null");
        let len = tag.len().min(127);
        let mut buf = [0u8; 128];
        buf[..len].copy_from_slice(&tag.as_bytes()[..len]);
        // SAFETY: handle non-null; buf is NUL-terminated (zero-initialized).
        // LVGL copies the string via lv_strdup (see lv_label_set_translation_tag
        // in lv_label.c), so passing a stack buffer is safe — no pointer is
        // retained after this call returns.
        unsafe { lv_label_set_translation_tag(self.obj.handle(), buf.as_ptr() as *const c_char) };
        self
    }

    /// Set label text without the 127-byte limit. Heap-allocates a
    /// NUL-terminated copy. Use [`text`](Self::text) for short UI labels.
    pub fn text_long(&self, s: &str) -> &Self {
        assert_ne!(self.obj.handle(), null_mut(), "Label handle cannot be null");
        let mut buf = Vec::with_capacity(s.len() + 1);
        buf.extend_from_slice(s.as_bytes());
        buf.push(0);
        // SAFETY: handle non-null (asserted above); buf is NUL-terminated.
        // LVGL copies the string internally so buf can be dropped.
        unsafe { lv_label_set_text(self.obj.handle(), buf.as_ptr() as *const c_char) };
        self
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
                // Copy &str to NUL-terminated stack buffer for LVGL.
                let bytes = text.as_bytes();
                let mut buf = [0u8; 128];
                let len = bytes.len().min(127);
                buf[..len].copy_from_slice(&bytes[..len]);
                // buf[len] is already 0 from zero-init.
                lv_label_set_text(label_ptr, buf.as_ptr() as *const core::ffi::c_char);
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

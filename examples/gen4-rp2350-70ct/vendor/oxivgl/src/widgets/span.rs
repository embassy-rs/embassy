// SPDX-License-Identifier: MIT OR Apache-2.0
use core::{ops::Deref, ptr::null_mut};

use oxivgl_sys::*;

use crate::style::TextDecor;

use super::{
    WidgetError,
    obj::{AsLvHandle, Obj},
};

/// Overflow mode for a [`Spangroup`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum SpanOverflow {
    /// Clip overflowing text.
    Clip = lv_span_overflow_t_LV_SPAN_OVERFLOW_CLIP,
    /// Show ellipsis on overflow.
    Ellipsis = lv_span_overflow_t_LV_SPAN_OVERFLOW_ELLIPSIS,
}

/// Layout mode for a [`Spangroup`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum SpanMode {
    /// Fixed object size.
    Fixed = lv_span_mode_t_LV_SPAN_MODE_FIXED,
    /// Expand object to fit text.
    Expand = lv_span_mode_t_LV_SPAN_MODE_EXPAND,
    /// Keep width, break long lines.
    Break = lv_span_mode_t_LV_SPAN_MODE_BREAK,
}

/// Handle to an individual span within a [`Spangroup`].
///
/// Obtained via [`Spangroup::add_span`]. The span's lifetime is managed by
/// LVGL (freed when the parent spangroup is deleted or via
/// [`Spangroup::delete_span`]).
#[derive(Debug)]
pub struct Span {
    handle: *mut lv_span_t,
}

impl Span {
    /// Return the raw `lv_span_t` pointer.
    pub fn as_ptr(&self) -> *mut lv_span_t {
        self.handle
    }

    /// Set text on this span. LVGL copies the text internally.
    ///
    /// Call [`Spangroup::refresh`] after modifying spans.
    pub fn set_text(&self, text: &core::ffi::CStr) -> &Self {
        // SAFETY: handle non-null (constructor guarantees); LVGL copies text.
        unsafe { lv_span_set_text(self.handle, text.as_ptr()) };
        self
    }

    /// Set static text on this span. The caller must ensure `text` outlives
    /// the span — LVGL stores the pointer without copying.
    ///
    /// Call [`Spangroup::refresh`] after modifying spans.
    pub fn set_text_static(&self, text: &'static core::ffi::CStr) -> &Self {
        // SAFETY: handle non-null; text is 'static so outlives the span.
        unsafe { lv_span_set_text_static(self.handle, text.as_ptr()) };
        self
    }

    /// Get a mutable pointer to the span's built-in style.
    ///
    /// Use `lv_style_set_*` functions on the returned pointer to customise
    /// this span's appearance.
    pub fn get_style(&self) -> *mut lv_style_t {
        // SAFETY: handle non-null (constructor guarantees).
        unsafe { lv_span_get_style(self.handle) }
    }

    /// Set the text color for this span.
    pub fn set_text_color(&self, color: lv_color_t) -> &Self {
        // SAFETY: style pointer valid while span is alive.
        unsafe { lv_style_set_text_color(self.get_style(), color) };
        self
    }

    /// Set the text opacity for this span.
    pub fn set_text_opa(&self, opa: u8) -> &Self {
        // SAFETY: style pointer valid while span is alive.
        unsafe { lv_style_set_text_opa(self.get_style(), opa as lv_opa_t) };
        self
    }

    /// Set the text decoration for this span.
    pub fn set_text_decor(&self, decor: TextDecor) -> &Self {
        // SAFETY: style pointer valid while span is alive.
        unsafe { lv_style_set_text_decor(self.get_style(), decor.0 as lv_text_decor_t) };
        self
    }

    /// Set the text font for this span.
    ///
    /// # Safety
    /// `font` must point to a valid `lv_font_t` that outlives the span.
    pub unsafe fn set_text_font(&self, font: *const lv_font_t) -> &Self {
        // SAFETY: caller guarantees font validity.
        unsafe { lv_style_set_text_font(self.get_style(), font) };
        self
    }

    /// Set the text letter spacing for this span.
    pub fn set_text_letter_space(&self, space: i32) -> &Self {
        // SAFETY: style pointer valid while span is alive.
        unsafe { lv_style_set_text_letter_space(self.get_style(), space) };
        self
    }

    /// Set the text line spacing for this span.
    pub fn set_text_line_space(&self, space: i32) -> &Self {
        // SAFETY: style pointer valid while span is alive.
        unsafe { lv_style_set_text_line_space(self.get_style(), space) };
        self
    }
}

/// LVGL Spangroup widget — a container for mixed-style text spans.
///
/// Each child [`Span`] can have independent text, color, font, and
/// decoration. Wraps [`Obj`](super::obj::Obj) and `Deref`s to it for
/// style/positioning methods.
///
/// Requires `LV_USE_SPAN = 1` in `lv_conf.h`.
#[derive(Debug)]
pub struct Spangroup<'p> {
    obj: Obj<'p>,
}

impl<'p> AsLvHandle for Spangroup<'p> {
    fn lv_handle(&self) -> *mut lv_obj_t {
        self.obj.lv_handle()
    }
}

impl<'p> Deref for Spangroup<'p> {
    type Target = Obj<'p>;
    fn deref(&self) -> &Obj<'p> {
        &self.obj
    }
}

impl<'p> Spangroup<'p> {
    /// Create a spangroup as a child of `parent`. Returns
    /// [`WidgetError::LvglNullPointer`] on OOM.
    pub fn new(parent: &impl AsLvHandle) -> Result<Self, WidgetError> {
        let parent_ptr = parent.lv_handle();
        assert_ne!(parent_ptr, null_mut(), "Parent widget cannot be null");
        // SAFETY: parent_ptr non-null (asserted above); lv_init() called via
        // LvglDriver.
        let handle = unsafe { lv_spangroup_create(parent_ptr) };
        if handle.is_null() { Err(WidgetError::LvglNullPointer) } else { Ok(Spangroup { obj: Obj::from_raw(handle) }) }
    }

    /// Add a new span to this spangroup and return a handle to it.
    ///
    /// The span is owned by LVGL and freed when the spangroup is deleted.
    pub fn add_span(&self) -> Result<Span, WidgetError> {
        // SAFETY: handle non-null (constructor guarantees).
        let span = unsafe { lv_spangroup_add_span(self.obj.handle()) };
        if span.is_null() { Err(WidgetError::LvglNullPointer) } else { Ok(Span { handle: span }) }
    }

    /// Remove a span from the spangroup and free its memory.
    pub fn delete_span(&self, span: &Span) -> &Self {
        // SAFETY: both handles non-null (constructor guarantees).
        unsafe { lv_spangroup_delete_span(self.obj.handle(), span.handle) };
        self
    }

    /// Set the text alignment for the spangroup.
    pub fn set_align_text(&self, align: lv_text_align_t) -> &Self {
        // SAFETY: handle non-null (constructor guarantees).
        unsafe { lv_spangroup_set_align(self.obj.handle(), align) };
        self
    }

    /// Set the overflow mode.
    pub fn set_overflow(&self, overflow: SpanOverflow) -> &Self {
        // SAFETY: handle non-null (constructor guarantees).
        unsafe { lv_spangroup_set_overflow(self.obj.handle(), overflow as lv_span_overflow_t) };
        self
    }

    /// Set the first-line indent in pixels.
    pub fn set_indent(&self, indent: i32) -> &Self {
        // SAFETY: handle non-null (constructor guarantees).
        unsafe { lv_spangroup_set_indent(self.obj.handle(), indent) };
        self
    }

    /// Set the layout mode.
    pub fn set_mode(&self, mode: SpanMode) -> &Self {
        // SAFETY: handle non-null (constructor guarantees).
        unsafe { lv_spangroup_set_mode(self.obj.handle(), mode as lv_span_mode_t) };
        self
    }

    /// Set the maximum number of lines. Negative values mean no limit.
    pub fn set_max_lines(&self, lines: i32) -> &Self {
        // SAFETY: handle non-null (constructor guarantees).
        unsafe { lv_spangroup_set_max_lines(self.obj.handle(), lines) };
        self
    }

    /// Get the number of spans in the group.
    pub fn get_span_count(&self) -> u32 {
        // SAFETY: handle non-null (constructor guarantees).
        unsafe { lv_spangroup_get_span_count(self.obj.handle()) }
    }

    /// Get the overflow mode.
    pub fn get_overflow(&self) -> lv_span_overflow_t {
        // SAFETY: handle non-null (constructor guarantees).
        unsafe { lv_spangroup_get_overflow(self.obj.handle()) }
    }

    /// Get the indent value.
    pub fn get_indent(&self) -> i32 {
        // SAFETY: handle non-null (constructor guarantees).
        unsafe { lv_spangroup_get_indent(self.obj.handle()) }
    }

    /// Get the layout mode.
    pub fn get_mode(&self) -> lv_span_mode_t {
        // SAFETY: handle non-null (constructor guarantees).
        unsafe { lv_spangroup_get_mode(self.obj.handle()) }
    }

    /// Get the maximum number of lines.
    pub fn get_max_lines(&self) -> i32 {
        // SAFETY: handle non-null (constructor guarantees).
        unsafe { lv_spangroup_get_max_lines(self.obj.handle()) }
    }

    /// Get the maximum line height across all spans.
    pub fn get_max_line_height(&self) -> i32 {
        // SAFETY: handle non-null (constructor guarantees).
        unsafe { lv_spangroup_get_max_line_height(self.obj.handle()) }
    }

    /// Get the text content width. If `max_width` > 0 and the content width
    /// exceeds it, returns `max_width`.
    pub fn get_expand_width(&self, max_width: u32) -> u32 {
        // SAFETY: handle non-null (constructor guarantees).
        unsafe { lv_spangroup_get_expand_width(self.obj.handle(), max_width) }
    }

    /// Get the text content height for the given fixed width.
    pub fn get_expand_height(&self, width: i32) -> i32 {
        // SAFETY: handle non-null (constructor guarantees).
        unsafe { lv_spangroup_get_expand_height(self.obj.handle(), width) }
    }

    /// Refresh/invalidate the spangroup after modifying spans.
    pub fn refresh(&self) -> &Self {
        // SAFETY: handle non-null (constructor guarantees).
        unsafe { lv_spangroup_refresh(self.obj.handle()) };
        self
    }
}

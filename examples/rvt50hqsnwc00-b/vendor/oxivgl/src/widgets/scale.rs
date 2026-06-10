// SPDX-License-Identifier: MIT OR Apache-2.0
use alloc::vec::Vec;
use core::{ops::Deref, ptr::null_mut};

use oxivgl_sys::*;

use super::{
    WidgetError,
    obj::{Align, AsLvHandle, Obj},
};

/// Type-safe wrapper for `lv_scale_mode_t`.
#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum ScaleMode {
    /// Horizontal, labels on top.
    HorizontalTop = 0,
    /// Horizontal, labels on bottom.
    HorizontalBottom = 1,
    /// Vertical, labels on left.
    VerticalLeft = 2,
    /// Vertical, labels on right.
    VerticalRight = 4,
    /// Round scale, ticks point inward.
    RoundInner = 8,
    /// Round scale, ticks point outward.
    RoundOuter = 16,
}

/// Rotate labels to match tick angles on round scales.
pub const SCALE_LABEL_ROTATE_MATCH_TICKS: i32 = oxivgl_sys::LV_SCALE_LABEL_ROTATE_MATCH_TICKS as i32;

/// Keep rotated labels upright (readable).
pub const SCALE_LABEL_ROTATE_KEEP_UPRIGHT: i32 = oxivgl_sys::LV_SCALE_LABEL_ROTATE_KEEP_UPRIGHT as i32;

/// LVGL scale widget (tick marks only, no arc). Use
/// [`tick_ring`](Scale::tick_ring) for the pre-configured round gauge variant.
#[derive(Debug)]
pub struct Scale<'p> {
    obj: Obj<'p>,
    /// Styles passed to sections — kept alive here (not in ScaleSection)
    /// because sections are freed by LVGL in the scale destructor, so
    /// styles must outlive sections. Obj::drop calls lv_obj_delete which
    /// frees sections first; then Rust drops this Vec.
    section_styles: core::cell::RefCell<Vec<crate::style::Style>>,
}

impl<'p> AsLvHandle for Scale<'p> {
    fn lv_handle(&self) -> *mut lv_obj_t {
        self.obj.lv_handle()
    }
}

impl<'p> Deref for Scale<'p> {
    type Target = Obj<'p>;
    fn deref(&self) -> &Obj<'p> {
        &self.obj
    }
}

impl<'p> Scale<'p> {
    /// Create a new scale widget.
    pub fn new(parent: &impl AsLvHandle) -> Result<Self, WidgetError> {
        let parent_ptr = parent.lv_handle();
        assert_ne!(parent_ptr, null_mut(), "Parent widget cannot be null");
        // SAFETY: parent_ptr non-null (asserted above); lv_init() called via
        // LvglDriver.
        let handle = unsafe { lv_scale_create(parent_ptr) };
        if handle.is_null() {
            Err(WidgetError::LvglNullPointer)
        } else {
            Ok(Scale { obj: Obj::from_raw(handle), section_styles: core::cell::RefCell::new(Vec::new()) })
        }
    }

    /// Set the scale mode (horizontal, vertical, or round).
    pub fn set_mode(&self, mode: ScaleMode) -> &Self {
        unsafe { lv_scale_set_mode(self.lv_handle(), mode as lv_scale_mode_t) };
        self
    }

    /// Set total number of tick marks.
    pub fn set_total_tick_count(&self, count: u32) -> &Self {
        unsafe { lv_scale_set_total_tick_count(self.lv_handle(), count) };
        self
    }

    /// Set interval for major ticks (e.g. every 5th tick is major).
    pub fn set_major_tick_every(&self, interval: u32) -> &Self {
        unsafe { lv_scale_set_major_tick_every(self.lv_handle(), interval) };
        self
    }

    /// Set the value range (min, max).
    pub fn set_range(&self, min: i32, max: i32) -> &Self {
        unsafe { lv_scale_set_range(self.lv_handle(), min, max) };
        self
    }

    /// Show or hide numeric labels on major ticks.
    pub fn set_label_show(&self, show: bool) -> &Self {
        unsafe { lv_scale_set_label_show(self.lv_handle(), show) };
        self
    }

    /// Set tick length for a specific part (Items=minor ticks, Indicator=major
    /// ticks).
    pub fn set_tick_length(&self, part: super::Part, length: i32) -> &Self {
        unsafe { lv_obj_set_style_length(self.lv_handle(), length, part as u32) };
        self
    }

    /// Set the start angle rotation in degrees.
    pub fn set_rotation(&self, rotation: i32) -> &Self {
        unsafe { lv_scale_set_rotation(self.lv_handle(), rotation) };
        self
    }

    /// Set the angular extent in degrees.
    pub fn set_angle_range(&self, angle_range: u32) -> &Self {
        unsafe { lv_scale_set_angle_range(self.lv_handle(), angle_range) };
        self
    }

    /// Get the scale mode as raw `u32`.
    ///
    /// Returns the raw `lv_scale_mode_t` value. Use [`ScaleMode`] constants
    /// to compare. The raw type is returned because LVGL defines a sentinel
    /// `_LAST` value not covered by [`ScaleMode`].
    pub fn get_mode(&self) -> u32 {
        // SAFETY: handle non-null (from Scale::new/tick_ring).
        unsafe { lv_scale_get_mode(self.lv_handle()) }
    }

    /// Get the total number of tick marks.
    pub fn get_total_tick_count(&self) -> i32 {
        // SAFETY: handle non-null (from Scale::new/tick_ring).
        unsafe { lv_scale_get_total_tick_count(self.lv_handle()) }
    }

    /// Get the major tick interval.
    pub fn get_major_tick_every(&self) -> i32 {
        unsafe { lv_scale_get_major_tick_every(self.lv_handle()) }
    }

    /// Get the start angle rotation in degrees.
    pub fn get_rotation(&self) -> i32 {
        // SAFETY: handle non-null (from Scale::new/tick_ring).
        unsafe { lv_scale_get_rotation(self.lv_handle()) }
    }

    /// Get whether numeric labels are shown on major ticks.
    pub fn get_label_show(&self) -> bool {
        // SAFETY: handle non-null (from Scale::new/tick_ring).
        unsafe { lv_scale_get_label_show(self.lv_handle()) }
    }

    /// Get the angular extent in degrees.
    pub fn get_angle_range(&self) -> u32 {
        // SAFETY: handle non-null (from Scale::new/tick_ring).
        unsafe { lv_scale_get_angle_range(self.lv_handle()) }
    }

    /// Get the range minimum value.
    pub fn get_range_min_value(&self) -> i32 {
        // SAFETY: handle non-null (from Scale::new/tick_ring).
        unsafe { lv_scale_get_range_min_value(self.lv_handle()) }
    }

    /// Get the range maximum value.
    pub fn get_range_max_value(&self) -> i32 {
        // SAFETY: handle non-null (from Scale::new/tick_ring).
        unsafe { lv_scale_get_range_max_value(self.lv_handle()) }
    }

    /// Add a styled section to the scale. Returns a handle for further
    /// configuration. Add a styled section. The returned handle borrows
    /// this scale and cannot outlive it (LVGL frees sections in the scale
    /// destructor).
    pub fn add_section(&self) -> ScaleSection<'_> {
        let ptr = unsafe { lv_scale_add_section(self.lv_handle()) };
        ScaleSection { ptr, scale: self.lv_handle(), parent_styles: &self.section_styles }
    }

    /// Set custom tick labels from a null-terminated `'static` array of C
    /// strings.
    ///
    /// Use [`scale_labels!`](crate::scale_labels) to create the array safely.
    /// LVGL stores the raw pointer — the array and all strings must be
    /// `'static`.
    pub fn set_text_src(&self, labels: &'static ScaleLabels) -> &Self {
        unsafe { lv_scale_set_text_src(self.lv_handle(), labels.0.as_ptr() as *mut _) };
        self
    }

    /// Position a Line child as a needle at the given scale value.
    ///
    /// `needle_line`: a Line widget that is a child of this scale.
    /// `needle_length`: length in pixels from the scale center.
    /// `value`: the scale value to point at.
    pub fn set_line_needle_value(&self, needle_line: &super::Line, needle_length: i32, value: i32) -> &Self {
        unsafe { lv_scale_set_line_needle_value(self.lv_handle(), needle_line.lv_handle(), needle_length, value) };
        self
    }

    /// Create a tick-mark ring scale (no arc drawn, transparent background).
    ///
    /// - `size`: diameter in px; centered in parent.
    /// - `mode`: e.g. `LV_SCALE_MODE_ROUND_INNER` (ticks point inward).
    /// - `rotation` / `sweep`: same convention as
    ///   [`Arc::gauge_ring`](super::Arc::gauge_ring).
    /// - `range_max`: integer range maximum (ticks labeled 0..range_max).
    /// - `total_ticks`: total number of tick marks.
    /// - `major_every`: every N-th tick is a major (longer, labeled if
    ///   `show_labels=true`).
    /// - `major_len` / `minor_len`: tick length in px.
    /// - `major_color` / `minor_color`: RGB hex colors.
    #[allow(clippy::too_many_arguments)]
    pub fn tick_ring(
        parent: &impl AsLvHandle,
        size: i32,
        mode: ScaleMode,
        rotation: i32,
        sweep: i32,
        range_max: i32,
        total_ticks: u32,
        major_every: u32,
        show_labels: bool,
        major_len: i32,
        minor_len: i32,
        major_color: u32,
        minor_color: u32,
    ) -> Result<Self, WidgetError> {
        let scale = Scale::new(parent)?;
        let h = scale.obj.handle();
        // SAFETY: h non-null (Scale::new null-checks); all LVGL style/scale fns safe
        // with valid ptr.
        unsafe {
            lv_obj_set_size(h, size, size);
            lv_obj_align(h, Align::Center as lv_align_t, 0, 0);
            lv_scale_set_mode(h, mode as lv_scale_mode_t);
            lv_scale_set_rotation(h, rotation);
            lv_scale_set_angle_range(h, sweep as u32);
            lv_scale_set_range(h, 0, range_max);
            lv_scale_set_total_tick_count(h, total_ticks);
            lv_scale_set_major_tick_every(h, major_every);
            lv_scale_set_label_show(h, show_labels);
            // No ring; explicit line_width=1 so tick outer end = radius_edge-1 (1px inset
            // from arc outer edge)
            lv_obj_set_style_arc_width(h, 0, lv_part_t_LV_PART_MAIN as u32);
            lv_obj_set_style_line_width(h, 1, lv_part_t_LV_PART_MAIN as u32);
            lv_obj_set_style_bg_opa(h, crate::enums::Opa::TRANSP.0 as lv_opa_t, 0);
            lv_obj_set_style_border_width(h, 0, 0);
            lv_obj_set_style_pad_all(h, 0, 0);
            // Minor ticks
            lv_obj_set_style_length(h, minor_len, lv_part_t_LV_PART_ITEMS as u32);
            lv_obj_set_style_line_color(h, lv_color_hex(minor_color), lv_part_t_LV_PART_ITEMS as u32);
            lv_obj_set_style_line_width(h, 1, lv_part_t_LV_PART_ITEMS as u32);
            // Major ticks
            lv_obj_set_style_length(h, major_len, lv_part_t_LV_PART_INDICATOR as u32);
            lv_obj_set_style_line_color(h, lv_color_hex(major_color), lv_part_t_LV_PART_INDICATOR as u32);
            lv_obj_set_style_line_width(h, 2, lv_part_t_LV_PART_INDICATOR as u32);
        }
        Ok(scale)
    }
}

/// Null-terminated array of C string pointers for scale tick labels.
///
/// Use [`scale_labels!`](crate::scale_labels) to create instances.
/// `Sync` is sound because all pointers are `'static` C string literals.
#[repr(transparent)]
pub struct ScaleLabels(pub [*const core::ffi::c_char]);

impl core::fmt::Debug for ScaleLabels {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ScaleLabels").finish_non_exhaustive()
    }
}

// SAFETY: The contained pointers reference 'static C string literals
// (enforced by the `scale_labels!` macro).
unsafe impl Sync for ScaleLabels {}

/// Create a `&'static` [`ScaleLabels`] array from C string literals.
///
/// ```no_run
/// use oxivgl::scale_labels;
/// use oxivgl::widgets::ScaleLabels;
///
/// static LABELS: &ScaleLabels = scale_labels!(c"Low", c"Mid", c"High");
/// ```
#[macro_export]
macro_rules! scale_labels {
    ($($label:expr),+ $(,)?) => {
        // SAFETY: ScaleLabels is repr(transparent) over [*const c_char].
        // All pointers come from c"…" literals which are 'static.
        // The array is a const-promoted 'static temporary.
        unsafe {
            &*(&[$($label.as_ptr()),+, ::core::ptr::null()]
                as *const [*const ::core::ffi::c_char]
                as *const $crate::widgets::ScaleLabels)
        }
    };
}

/// Opaque handle to a scale section (range with custom styling).
///
/// Borrows its parent [`Scale`] — the section cannot outlive the scale
/// (LVGL frees sections in the scale destructor, `lv_scale.c:514-526`).
///
/// Styles passed to section setters are stored in the parent `Scale`
/// (not here) so they outlive the section and are freed only after
/// `lv_obj_delete` cleans up all sections (spec §5.2, §5.5).
pub struct ScaleSection<'s> {
    ptr: *mut lv_scale_section_t,
    scale: *mut lv_obj_t,
    parent_styles: &'s core::cell::RefCell<Vec<crate::style::Style>>,
}

impl<'s> core::fmt::Debug for ScaleSection<'s> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ScaleSection").finish_non_exhaustive()
    }
}

impl<'s> ScaleSection<'s> {
    /// Set the value range this section covers.
    pub fn set_range(&self, min: i32, max: i32) -> &Self {
        unsafe { lv_scale_set_section_range(self.scale, self.ptr, min, max) };
        self
    }

    /// Set style for major tick labels in this section.
    ///
    /// LVGL stores the raw `lv_style_t*` (`lv_scale.c:390`). The style
    /// is cloned internally to keep the Rc alive (spec §5.2).
    pub fn set_indicator_style(&self, style: &crate::style::Style) -> &Self {
        self.parent_styles.borrow_mut().push(style.clone());
        let style_ptr = style.lv_ptr();
        unsafe { lv_scale_set_section_style_indicator(self.scale, self.ptr, style_ptr) };
        self
    }

    /// Set style for minor ticks in this section.
    ///
    /// LVGL stores the raw `lv_style_t*` (`lv_scale.c:399`). The style
    /// is cloned internally (spec §5.2).
    pub fn set_items_style(&self, style: &crate::style::Style) -> &Self {
        self.parent_styles.borrow_mut().push(style.clone());
        let style_ptr = style.lv_ptr();
        unsafe { lv_scale_set_section_style_items(self.scale, self.ptr, style_ptr) };
        self
    }

    /// Set style for the main line in this section.
    ///
    /// LVGL stores the raw `lv_style_t*` (`lv_scale.c:381`). The style
    /// is cloned internally (spec §5.2).
    pub fn set_main_style(&self, style: &crate::style::Style) -> &Self {
        self.parent_styles.borrow_mut().push(style.clone());
        let style_ptr = style.lv_ptr();
        unsafe { lv_scale_set_section_style_main(self.scale, self.ptr, style_ptr) };
        self
    }
}

/// Builder for [`Scale::tick_ring`] — avoids 13 positional arguments.
///
/// ```ignore
/// let scale = ScaleBuilder::new(200, ScaleMode::RoundOuter)
///     .rotation(135)
///     .sweep(270)
///     .range_max(100)
///     .total_ticks(21)
///     .major_every(5)
///     .build(&screen)?;
/// ```
#[derive(Debug)]
pub struct ScaleBuilder {
    size: i32,
    mode: ScaleMode,
    rotation: i32,
    sweep: i32,
    range_max: i32,
    total_ticks: u32,
    major_every: u32,
    show_labels: bool,
    major_len: i32,
    minor_len: i32,
    major_color: u32,
    minor_color: u32,
}

impl ScaleBuilder {
    /// Start with required fields (size, mode), sensible defaults for the rest.
    pub fn new(size: i32, mode: ScaleMode) -> Self {
        Self {
            size,
            mode,
            rotation: 0,
            sweep: 360,
            range_max: 100,
            total_ticks: 11,
            major_every: 5,
            show_labels: true,
            major_len: 10,
            minor_len: 5,
            major_color: 0x000000,
            minor_color: 0x808080,
        }
    }

    /// Set start angle in degrees.
    pub fn rotation(mut self, v: i32) -> Self {
        self.rotation = v;
        self
    }
    /// Set angular extent in degrees.
    pub fn sweep(mut self, v: i32) -> Self {
        self.sweep = v;
        self
    }
    /// Set maximum range value.
    pub fn range_max(mut self, v: i32) -> Self {
        self.range_max = v;
        self
    }
    /// Set total number of tick marks.
    pub fn total_ticks(mut self, v: u32) -> Self {
        self.total_ticks = v;
        self
    }
    /// Set major tick interval.
    pub fn major_every(mut self, v: u32) -> Self {
        self.major_every = v;
        self
    }
    /// Show/hide numeric labels on major ticks.
    pub fn show_labels(mut self, v: bool) -> Self {
        self.show_labels = v;
        self
    }
    /// Set major tick length in pixels.
    pub fn major_len(mut self, v: i32) -> Self {
        self.major_len = v;
        self
    }
    /// Set minor tick length in pixels.
    pub fn minor_len(mut self, v: i32) -> Self {
        self.minor_len = v;
        self
    }
    /// Set major tick color (RGB hex).
    pub fn major_color(mut self, v: u32) -> Self {
        self.major_color = v;
        self
    }
    /// Set minor tick color (RGB hex).
    pub fn minor_color(mut self, v: u32) -> Self {
        self.minor_color = v;
        self
    }

    /// Build the scale widget.
    pub fn build(self, parent: &impl AsLvHandle) -> Result<Scale<'_>, WidgetError> {
        Scale::tick_ring(
            parent,
            self.size,
            self.mode,
            self.rotation,
            self.sweep,
            self.range_max,
            self.total_ticks,
            self.major_every,
            self.show_labels,
            self.major_len,
            self.minor_len,
            self.major_color,
            self.minor_color,
        )
    }
}

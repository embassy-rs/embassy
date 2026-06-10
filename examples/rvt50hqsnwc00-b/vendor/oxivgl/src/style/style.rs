// SPDX-License-Identifier: MIT OR Apache-2.0
use alloc::{boxed::Box, rc::Rc};

use oxivgl_sys::*;

use super::GradDir;

/// Text decoration flags. Combine with `|`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextDecor(pub(crate) u32);

impl TextDecor {
    /// No decoration.
    pub const NONE: Self = Self(0x00);
    /// Underline text.
    pub const UNDERLINE: Self = Self(0x01);
    /// Strikethrough text.
    pub const STRIKETHROUGH: Self = Self(0x02);
}

impl core::ops::BitOr for TextDecor {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

/// Wraps `lv_style_transition_dsc_t`.
///
/// The `props` slice must be `'static` — LVGL stores the raw pointer
/// (`lv_style_transition_dsc_init` stores `tr->props = props`).
pub struct TransitionDsc {
    pub(crate) inner: lv_style_transition_dsc_t,
}

impl core::fmt::Debug for TransitionDsc {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TransitionDsc").finish_non_exhaustive()
    }
}

impl TransitionDsc {
    /// Create a transition descriptor.
    ///
    /// `props`: null-terminated `'static` array of `lv_style_prop_t` (use
    /// [`props`] constants). `path_cb`: animation path function (e.g.
    /// [`crate::anim::anim_path_linear`]). `time`: transition duration in
    /// ms. `delay`: delay before transition starts in ms.
    pub fn new(
        props: &'static [lv_style_prop_t],
        path_cb: Option<unsafe extern "C" fn(*const lv_anim_t) -> i32>,
        time: u32,
        delay: u32,
    ) -> Self {
        let mut inner = unsafe { core::mem::zeroed::<lv_style_transition_dsc_t>() };
        // SAFETY: inner is zeroed; props pointer is 'static so outlives everything.
        unsafe {
            lv_style_transition_dsc_init(&mut inner, props.as_ptr(), path_cb, time, delay, core::ptr::null_mut())
        };
        Self { inner }
    }
}

/// Commonly used style property constants (cast to `lv_style_prop_t`).
pub mod props {
    pub use oxivgl_sys::lv_style_prop_t;

    /// Background color property.
    pub const BG_COLOR: lv_style_prop_t = oxivgl_sys::_lv_style_id_t_LV_STYLE_BG_COLOR as lv_style_prop_t;
    /// Border color property.
    pub const BORDER_COLOR: lv_style_prop_t = oxivgl_sys::_lv_style_id_t_LV_STYLE_BORDER_COLOR as lv_style_prop_t;
    /// Border width property.
    pub const BORDER_WIDTH: lv_style_prop_t = oxivgl_sys::_lv_style_id_t_LV_STYLE_BORDER_WIDTH as lv_style_prop_t;
    /// Background opacity property.
    pub const BG_OPA: lv_style_prop_t = oxivgl_sys::_lv_style_id_t_LV_STYLE_BG_OPA as lv_style_prop_t;
    /// Width property.
    pub const WIDTH: lv_style_prop_t = oxivgl_sys::_lv_style_id_t_LV_STYLE_WIDTH as lv_style_prop_t;
    /// Outline width property.
    pub const OUTLINE_WIDTH: lv_style_prop_t = oxivgl_sys::_lv_style_id_t_LV_STYLE_OUTLINE_WIDTH as lv_style_prop_t;
    /// Outline opacity property.
    pub const OUTLINE_OPA: lv_style_prop_t = oxivgl_sys::_lv_style_id_t_LV_STYLE_OUTLINE_OPA as lv_style_prop_t;
    /// Transform width offset property.
    pub const TRANSFORM_WIDTH: lv_style_prop_t =
        oxivgl_sys::_lv_style_id_t_LV_STYLE_TRANSFORM_WIDTH as lv_style_prop_t;
    /// Transform height offset property.
    pub const TRANSFORM_HEIGHT: lv_style_prop_t =
        oxivgl_sys::_lv_style_id_t_LV_STYLE_TRANSFORM_HEIGHT as lv_style_prop_t;
    /// Text letter spacing property.
    pub const TEXT_LETTER_SPACE: lv_style_prop_t =
        oxivgl_sys::_lv_style_id_t_LV_STYLE_TEXT_LETTER_SPACE as lv_style_prop_t;
    /// Sentinel: end of property list.
    pub const LAST: lv_style_prop_t = 0;
}

/// Bitflags for border side selection. Combine with `|` operator.
///
/// ```
/// use oxivgl::style::BorderSide;
///
/// let sides = BorderSide::BOTTOM | BorderSide::RIGHT;
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BorderSide(u32);

impl BorderSide {
    /// No border.
    pub const NONE: Self = Self(0x00);
    /// Bottom border.
    pub const BOTTOM: Self = Self(0x01);
    /// Top border.
    pub const TOP: Self = Self(0x02);
    /// Left border.
    pub const LEFT: Self = Self(0x04);
    /// Right border.
    pub const RIGHT: Self = Self(0x08);
    /// All four sides.
    pub const FULL: Self = Self(0x0F);
    /// Draw borders between buttons (internal borders only).
    pub const INTERNAL: Self = Self(0x10);
}

impl core::ops::BitOr for BorderSide {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

/// Return an LVGL percentage value. Wraps `lv_pct()`.
pub fn lv_pct(v: i32) -> i32 {
    // SAFETY: lv_pct is a pure arithmetic function.
    unsafe { oxivgl_sys::lv_pct(v) }
}

/// Special LVGL size value: object sizes itself to fit its content.
/// Equivalent to the C macro `LV_SIZE_CONTENT`.
pub const LV_SIZE_CONTENT: i32 = (oxivgl_sys::LV_COORD_MAX | oxivgl_sys::LV_COORD_TYPE_SPEC) as i32;

// ── StyleInner ──────────────────────────────────────────────────────────

/// Internal storage for a frozen style. `#[repr(C)]` with `lv` at offset 0
/// so that `*const StyleInner` can be safely cast to `*const lv_style_t`.
#[repr(C)]
pub(crate) struct StyleInner {
    lv: lv_style_t,
    _grad: Option<Box<super::GradDsc>>,
    _transition: Option<Box<TransitionDsc>>,
    _color_filter: Option<Box<ColorFilter>>,
}

// Compile-time guarantee that the cast `*const StyleInner → *const lv_style_t`
// is valid.
const _: () = assert!(core::mem::offset_of!(StyleInner, lv) == 0);

impl core::fmt::Debug for StyleInner {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("StyleInner").finish_non_exhaustive()
    }
}

impl Drop for StyleInner {
    fn drop(&mut self) {
        // SAFETY: lv was initialized by lv_style_init. lv_style_reset frees
        // the values_and_props buffer (lv_style.c:192-201) which contained all
        // stored property pointers, then zeroes the lv_style_t with lv_memzero.
        // After the buffer is freed, no LVGL data structure can reference
        // sub-descriptor addresses via it (spec §8.2).
        // Rust then drops the Option<Box<…>> fields, freeing sub-descriptors.
        unsafe { lv_style_reset(&mut self.lv) };
    }
}

// ── StyleBuilder ────────────────────────────────────────────────────────

/// Mutable build phase for an LVGL style. Not `Clone`.
///
/// Call setter methods to configure properties, then [`build()`](Self::build)
/// to produce a frozen, cheaply clonable [`Style`] handle.
///
/// ```no_run
/// use oxivgl::style::{StyleBuilder, Selector};
///
/// let mut style = StyleBuilder::new();
/// style.radius(5)
///     .bg_color_hex(0x0000FF);
/// let style = style.build();
/// // Apply with: widget.add_style(&style, Selector::DEFAULT);
/// ```
#[derive(Debug)]
pub struct StyleBuilder {
    inner: Box<StyleInner>,
}

impl StyleBuilder {
    /// Create a new empty style builder.
    pub fn new() -> Self {
        // SAFETY: lv_style_t can be zero-initialized; lv_style_init sets it up.
        let mut lv = unsafe { core::mem::zeroed::<lv_style_t>() };
        unsafe { lv_style_init(&mut lv) };
        Self { inner: Box::new(StyleInner { lv, _grad: None, _transition: None, _color_filter: None }) }
    }

    /// Consume this builder and produce a frozen [`Style`] handle.
    pub fn build(self) -> Style {
        // Box<StyleInner> → Rc<StyleInner>. Sub-descriptor Box<T> addresses
        // are stable because they are behind their own heap indirection.
        Style { inner: Rc::from(self.inner) }
    }

    /// Set corner radius.
    pub fn radius(&mut self, r: i16) -> &mut Self {
        unsafe { lv_style_set_radius(&mut self.inner.lv, r as lv_coord_t) };
        self
    }

    /// Set overall object opacity (0-255).
    pub fn opa(&mut self, opa: u8) -> &mut Self {
        unsafe { lv_style_set_opa(&mut self.inner.lv, opa as lv_opa_t) };
        self
    }

    /// Set background opacity (0-255).
    pub fn bg_opa(&mut self, opa: u8) -> &mut Self {
        unsafe { lv_style_set_bg_opa(&mut self.inner.lv, opa as lv_opa_t) };
        self
    }

    /// Set background color.
    pub fn bg_color(&mut self, color: lv_color_t) -> &mut Self {
        unsafe { lv_style_set_bg_color(&mut self.inner.lv, color) };
        self
    }

    /// Set background color from RGB hex.
    pub fn bg_color_hex(&mut self, hex: u32) -> &mut Self {
        let color = unsafe { lv_color_hex(hex) };
        self.bg_color(color)
    }

    /// Set background gradient end color.
    pub fn bg_grad_color(&mut self, color: lv_color_t) -> &mut Self {
        unsafe { lv_style_set_bg_grad_color(&mut self.inner.lv, color) };
        self
    }

    /// Set background gradient end color from RGB hex.
    pub fn bg_grad_color_hex(&mut self, hex: u32) -> &mut Self {
        let color = unsafe { lv_color_hex(hex) };
        self.bg_grad_color(color)
    }

    /// Set background gradient direction.
    pub fn bg_grad_dir(&mut self, dir: GradDir) -> &mut Self {
        unsafe { lv_style_set_bg_grad_dir(&mut self.inner.lv, dir as lv_grad_dir_t) };
        self
    }

    /// Set border color.
    pub fn border_color(&mut self, color: lv_color_t) -> &mut Self {
        unsafe { lv_style_set_border_color(&mut self.inner.lv, color) };
        self
    }

    /// Set border color from RGB hex.
    pub fn border_color_hex(&mut self, hex: u32) -> &mut Self {
        let color = unsafe { lv_color_hex(hex) };
        self.border_color(color)
    }

    /// Set border opacity (0-255).
    pub fn border_opa(&mut self, opa: u8) -> &mut Self {
        unsafe { lv_style_set_border_opa(&mut self.inner.lv, opa as lv_opa_t) };
        self
    }

    /// Set border width in pixels.
    pub fn border_width(&mut self, w: i16) -> &mut Self {
        unsafe { lv_style_set_border_width(&mut self.inner.lv, w as lv_coord_t) };
        self
    }

    /// Set text opacity (0–255).
    pub fn text_opa(&mut self, opa: u8) -> &mut Self {
        unsafe { lv_style_set_text_opa(&mut self.inner.lv, opa as lv_opa_t) };
        self
    }

    /// Set text font.
    pub fn text_font(&mut self, font: crate::fonts::Font) -> &mut Self {
        // SAFETY: Font is constructable only via unsafe from_raw/from_extern
        // which require a 'static lv_font_t. The pointer stored in the style
        // property map is released when lv_style_reset runs in StyleInner::Drop
        // (spec §4.7). The font itself is 'static so outlives the style (spec §2).
        unsafe { lv_style_set_text_font(&mut self.inner.lv, font.as_ptr()) };
        self
    }

    /// Set text color.
    pub fn text_color(&mut self, color: lv_color_t) -> &mut Self {
        unsafe { lv_style_set_text_color(&mut self.inner.lv, color) };
        self
    }

    /// Set text color from RGB hex.
    pub fn text_color_hex(&mut self, hex: u32) -> &mut Self {
        let color = unsafe { lv_color_hex(hex) };
        self.text_color(color)
    }

    /// Apply a color filter with given opacity. Takes ownership of `filter`.
    ///
    /// The filter is stored inside the style; no external lifetime management
    /// needed.
    pub fn color_filter(&mut self, filter: ColorFilter, opa: u8) -> &mut Self {
        let filter = Box::new(filter);
        // SAFETY: set new pointer first (LVGL overwrites in-place, lv_style.c:344-346),
        // then store the Box — old allocation (if any) drops after LVGL no longer
        // references it.
        unsafe {
            lv_style_set_color_filter_dsc(&mut self.inner.lv, &filter.inner);
            lv_style_set_color_filter_opa(&mut self.inner.lv, opa as lv_opa_t);
        }
        self.inner._color_filter = Some(filter);
        self
    }

    /// Apply the built-in LVGL shade color filter with the given opacity.
    ///
    /// This uses the global `lv_color_filter_shade` descriptor, so no
    /// heap allocation is needed. The filter darkens colors by the given
    /// opacity level (0 = transparent, 255 = fully dark).
    pub fn color_filter_shade(&mut self, opa: u8) -> &mut Self {
        // SAFETY: lv_color_filter_shade is a global static in LVGL,
        // valid for the entire program lifetime.
        unsafe {
            lv_style_set_color_filter_dsc(&mut self.inner.lv, &lv_color_filter_shade);
            lv_style_set_color_filter_opa(&mut self.inner.lv, opa as lv_opa_t);
        }
        // No Box ownership needed — the global static outlives everything.
        self
    }

    /// Set style width.
    pub fn width(&mut self, w: i32) -> &mut Self {
        unsafe { lv_style_set_width(&mut self.inner.lv, w) };
        self
    }

    /// Set style height.
    pub fn height(&mut self, h: i32) -> &mut Self {
        unsafe { lv_style_set_height(&mut self.inner.lv, h) };
        self
    }

    /// Set style X offset.
    pub fn x(&mut self, x: i32) -> &mut Self {
        unsafe { lv_style_set_x(&mut self.inner.lv, x) };
        self
    }

    /// Set style Y offset.
    pub fn y(&mut self, y: i32) -> &mut Self {
        unsafe { lv_style_set_y(&mut self.inner.lv, y) };
        self
    }

    /// Set gap between items (used in flex and grid layouts).
    pub fn pad_gap(&mut self, p: i32) -> &mut Self {
        // SAFETY: inner was initialized by lv_style_init.
        unsafe { lv_style_set_pad_gap(&mut self.inner.lv, p) };
        self
    }

    /// Set vertical padding (top + bottom).
    pub fn pad_ver(&mut self, p: i32) -> &mut Self {
        unsafe { lv_style_set_pad_ver(&mut self.inner.lv, p) };
        self
    }

    /// Set left padding.
    pub fn pad_left(&mut self, p: i32) -> &mut Self {
        unsafe { lv_style_set_pad_left(&mut self.inner.lv, p) };
        self
    }

    /// Set right padding.
    pub fn pad_right(&mut self, p: i32) -> &mut Self {
        unsafe { lv_style_set_pad_right(&mut self.inner.lv, p) };
        self
    }

    /// Set top padding.
    pub fn pad_top(&mut self, p: i32) -> &mut Self {
        unsafe { lv_style_set_pad_top(&mut self.inner.lv, p) };
        self
    }

    /// Set scrollbar / indicator length.
    pub fn length(&mut self, l: i32) -> &mut Self {
        unsafe { lv_style_set_length(&mut self.inner.lv, l) };
        self
    }

    /// Set background gradient from a full gradient descriptor. Takes
    /// ownership.
    ///
    /// The descriptor is stored inside the style; no external lifetime
    /// management needed. For simple two-color gradients, prefer
    /// [`bg_grad_color`](Self::bg_grad_color)
    /// + [`bg_grad_dir`](Self::bg_grad_dir).
    pub fn bg_grad(&mut self, grad: super::GradDsc) -> &mut Self {
        let grad = Box::new(grad);
        // SAFETY: set new pointer first, then store Box. Old Box drops after
        // LVGL no longer references it (lv_style_set_prop overwrites in-place).
        unsafe { lv_style_set_bg_grad(&mut self.inner.lv, &grad.inner) };
        self.inner._grad = Some(grad);
        self
    }

    /// Set which border sides to draw.
    pub fn border_side(&mut self, side: BorderSide) -> &mut Self {
        unsafe { lv_style_set_border_side(&mut self.inner.lv, side.0 as lv_border_side_t) };
        self
    }

    /// Set outline width.
    pub fn outline_width(&mut self, w: i32) -> &mut Self {
        unsafe { lv_style_set_outline_width(&mut self.inner.lv, w) };
        self
    }

    /// Set outline color.
    pub fn outline_color(&mut self, color: lv_color_t) -> &mut Self {
        unsafe { lv_style_set_outline_color(&mut self.inner.lv, color) };
        self
    }

    /// Set outline padding (gap from border).
    pub fn outline_pad(&mut self, pad: i32) -> &mut Self {
        unsafe { lv_style_set_outline_pad(&mut self.inner.lv, pad) };
        self
    }

    /// Set shadow width.
    pub fn shadow_width(&mut self, w: i32) -> &mut Self {
        unsafe { lv_style_set_shadow_width(&mut self.inner.lv, w) };
        self
    }

    /// Set shadow color.
    pub fn shadow_color(&mut self, color: lv_color_t) -> &mut Self {
        unsafe { lv_style_set_shadow_color(&mut self.inner.lv, color) };
        self
    }

    /// Set arc line color.
    pub fn arc_color(&mut self, color: lv_color_t) -> &mut Self {
        unsafe { lv_style_set_arc_color(&mut self.inner.lv, color) };
        self
    }

    /// Set arc line width.
    pub fn arc_width(&mut self, w: i32) -> &mut Self {
        unsafe { lv_style_set_arc_width(&mut self.inner.lv, w) };
        self
    }

    /// Set padding on all sides.
    pub fn pad_all(&mut self, p: i32) -> &mut Self {
        unsafe { lv_style_set_pad_all(&mut self.inner.lv, p) };
        self
    }

    /// Set letter spacing.
    pub fn text_letter_space(&mut self, s: i32) -> &mut Self {
        unsafe { lv_style_set_text_letter_space(&mut self.inner.lv, s) };
        self
    }

    /// Set line spacing.
    pub fn text_line_space(&mut self, s: i32) -> &mut Self {
        unsafe { lv_style_set_text_line_space(&mut self.inner.lv, s) };
        self
    }

    /// Set text decoration (underline, strikethrough).
    pub fn text_decor(&mut self, decor: TextDecor) -> &mut Self {
        unsafe { lv_style_set_text_decor(&mut self.inner.lv, decor.0 as lv_text_decor_t) };
        self
    }

    /// Set line color.
    pub fn line_color(&mut self, color: lv_color_t) -> &mut Self {
        unsafe { lv_style_set_line_color(&mut self.inner.lv, color) };
        self
    }

    /// Set line width.
    pub fn line_width(&mut self, w: i32) -> &mut Self {
        unsafe { lv_style_set_line_width(&mut self.inner.lv, w) };
        self
    }

    /// Enable/disable rounded line endings.
    pub fn line_rounded(&mut self, rounded: bool) -> &mut Self {
        unsafe { lv_style_set_line_rounded(&mut self.inner.lv, rounded) };
        self
    }

    /// Set transition descriptor. Takes ownership.
    ///
    /// The descriptor is stored inside the style; no external lifetime
    /// management needed.
    pub fn transition(&mut self, tr: TransitionDsc) -> &mut Self {
        let tr = Box::new(tr);
        // SAFETY: set new pointer first, then store Box.
        unsafe { lv_style_set_transition(&mut self.inner.lv, &tr.inner) };
        self.inner._transition = Some(tr);
        self
    }

    /// Set shadow X offset.
    pub fn shadow_offset_x(&mut self, x: i32) -> &mut Self {
        unsafe { lv_style_set_shadow_offset_x(&mut self.inner.lv, x) };
        self
    }

    /// Set shadow Y offset.
    pub fn shadow_offset_y(&mut self, y: i32) -> &mut Self {
        unsafe { lv_style_set_shadow_offset_y(&mut self.inner.lv, y) };
        self
    }

    /// Set shadow opacity (0-255).
    pub fn shadow_opa(&mut self, opa: u8) -> &mut Self {
        unsafe { lv_style_set_shadow_opa(&mut self.inner.lv, opa as lv_opa_t) };
        self
    }

    /// Set shadow spread (extra size).
    pub fn shadow_spread(&mut self, s: i32) -> &mut Self {
        unsafe { lv_style_set_shadow_spread(&mut self.inner.lv, s) };
        self
    }

    /// Set outline opacity (0-255).
    pub fn outline_opa(&mut self, opa: u8) -> &mut Self {
        unsafe { lv_style_set_outline_opa(&mut self.inner.lv, opa as lv_opa_t) };
        self
    }

    /// Set animation duration in ms (used for animated value changes).
    pub fn anim_duration(&mut self, ms: u32) -> &mut Self {
        unsafe { lv_style_set_anim_duration(&mut self.inner.lv, ms) };
        self
    }

    /// Set vertical translation offset.
    pub fn translate_y(&mut self, y: i32) -> &mut Self {
        unsafe { lv_style_set_translate_y(&mut self.inner.lv, y) };
        self
    }

    /// Set flex layout flow direction.
    pub fn flex_flow(&mut self, flow: crate::layout::FlexFlow) -> &mut Self {
        unsafe { lv_style_set_flex_flow(&mut self.inner.lv, flow as lv_flex_flow_t) };
        self
    }

    /// Set flex main-axis alignment.
    pub fn flex_main_place(&mut self, align: crate::layout::FlexAlign) -> &mut Self {
        unsafe { lv_style_set_flex_main_place(&mut self.inner.lv, align as lv_flex_align_t) };
        self
    }

    /// Set background image source (pointer to an `lv_image_dsc_t`).
    /// LVGL stores the raw pointer — the descriptor must be `'static`.
    ///
    /// LVGL stores the raw pointer in the style property map. The descriptor
    /// must be `'static` (spec §3.1). Use
    /// [`image_declare!`](crate::image_declare):
    /// `style.bg_image_src(my_img())`.
    pub fn bg_image_src(&mut self, src: &'static lv_image_dsc_t) -> &mut Self {
        // SAFETY: inner was initialized; src is 'static and points to a valid
        // compiled image descriptor. LVGL stores the pointer (spec §3.1).
        unsafe {
            lv_style_set_bg_image_src(&mut self.inner.lv, src as *const lv_image_dsc_t as *const core::ffi::c_void)
        };
        self
    }

    /// Set background image opacity (0 = transparent, 255 = opaque).
    pub fn bg_image_opa(&mut self, opa: u8) -> &mut Self {
        unsafe { lv_style_set_bg_image_opa(&mut self.inner.lv, opa) };
        self
    }

    /// Tile background image instead of stretching.
    pub fn bg_image_tiled(&mut self, tiled: bool) -> &mut Self {
        unsafe { lv_style_set_bg_image_tiled(&mut self.inner.lv, tiled) };
        self
    }

    /// Set image recolor tint.
    pub fn image_recolor(&mut self, color: lv_color_t) -> &mut Self {
        unsafe { lv_style_set_image_recolor(&mut self.inner.lv, color) };
        self
    }

    /// Set image recolor opacity (0 = no tint, 255 = full tint).
    pub fn image_recolor_opa(&mut self, opa: u8) -> &mut Self {
        unsafe { lv_style_set_image_recolor_opa(&mut self.inner.lv, opa) };
        self
    }

    /// Set transform width offset in pixels (expands/shrinks the widget
    /// visually).
    pub fn transform_width(&mut self, w: i32) -> &mut Self {
        unsafe { lv_style_set_transform_width(&mut self.inner.lv, w) };
        self
    }

    /// Set transform height offset in pixels (expands/shrinks the widget
    /// visually).
    pub fn transform_height(&mut self, h: i32) -> &mut Self {
        unsafe { lv_style_set_transform_height(&mut self.inner.lv, h) };
        self
    }

    /// Set transform rotation in 0.1° units (e.g. 450 = 45°).
    ///
    /// **Warning**: the LVGL SW renderer does not clip the transformed
    /// bounding box to display bounds. Center the object or ensure
    /// rotated extents stay within the display.
    pub fn transform_rotation(&mut self, angle: i32) -> &mut Self {
        unsafe { lv_style_set_transform_rotation(&mut self.inner.lv, angle) };
        self
    }

    /// Set uniform transform scale (256 = 1.0×, 320 = 1.25×, etc.).
    pub fn transform_scale(&mut self, scale: i32) -> &mut Self {
        unsafe {
            lv_style_set_transform_scale_x(&mut self.inner.lv, scale);
            lv_style_set_transform_scale_y(&mut self.inner.lv, scale);
        }
        self
    }

    /// Set transform pivot X offset in pixels.
    pub fn transform_pivot_x(&mut self, x: i32) -> &mut Self {
        unsafe { lv_style_set_transform_pivot_x(&mut self.inner.lv, x) };
        self
    }

    /// Set transform pivot Y offset in pixels.
    pub fn transform_pivot_y(&mut self, y: i32) -> &mut Self {
        unsafe { lv_style_set_transform_pivot_y(&mut self.inner.lv, y) };
        self
    }

    /// Set layout engine (flex or grid).
    pub fn layout(&mut self, layout: crate::layout::Layout) -> &mut Self {
        unsafe { lv_style_set_layout(&mut self.inner.lv, layout as u16) };
        self
    }
}

// ── Style ───────────────────────────────────────────────────────────────

/// Frozen, cheaply clonable LVGL style handle.
///
/// Produced by [`StyleBuilder::build`]. Internally reference-counted (`Rc`);
/// cloning bumps the refcount. When the last clone drops, `lv_style_reset`
/// frees the LVGL property map, then sub-descriptors are freed.
///
/// `Style` has no public constructor — obtain one via [`StyleBuilder`].
#[derive(Clone, Debug)]
pub struct Style {
    pub(crate) inner: Rc<StyleInner>,
}

impl Style {
    /// Return the raw `lv_style_t` pointer for passing to LVGL.
    ///
    /// Valid as long as at least one `Style` clone exists. The pointer is
    /// derived from `Rc::as_ptr` and the `#[repr(C)]` offset-0 guarantee.
    pub(crate) fn lv_ptr(&self) -> *const lv_style_t {
        Rc::as_ptr(&self.inner) as *const lv_style_t
    }
}

/// Wraps `lv_color_filter_dsc_t` with a C callback function pointer.
pub struct ColorFilter {
    pub(crate) inner: lv_color_filter_dsc_t,
}

impl core::fmt::Debug for ColorFilter {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ColorFilter").finish_non_exhaustive()
    }
}

impl ColorFilter {
    /// `lv_color_filter_dsc_init` is not available in bindings; set field
    /// directly.
    pub fn new(cb: unsafe extern "C" fn(*const lv_color_filter_dsc_t, lv_color_t, lv_opa_t) -> lv_color_t) -> Self {
        let mut inner = unsafe { core::mem::zeroed::<lv_color_filter_dsc_t>() };
        inner.filter_cb = Some(cb);
        Self { inner }
    }
}

/// Standard "darken" color filter callback — pass to [`ColorFilter::new`].
pub unsafe extern "C" fn darken_filter_cb(
    _dsc: *const lv_color_filter_dsc_t,
    color: lv_color_t,
    opa: lv_opa_t,
) -> lv_color_t {
    // SAFETY: lv_color_darken is a pure color computation.
    unsafe { lv_color_darken(color, opa) }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- TextDecor ---------------------------------------------------------

    #[test]
    fn text_decor_values() {
        assert_eq!(TextDecor::NONE.0, 0);
        assert_eq!(TextDecor::UNDERLINE.0, 1);
        assert_eq!(TextDecor::STRIKETHROUGH.0, 2);
    }

    #[test]
    fn text_decor_bitor() {
        let combined = TextDecor::UNDERLINE | TextDecor::STRIKETHROUGH;
        assert_eq!(combined.0, 0x03);
    }

    // -- BorderSide --------------------------------------------------------

    #[test]
    fn border_side_values() {
        assert_eq!(BorderSide::NONE.0, 0x00);
        assert_eq!(BorderSide::BOTTOM.0, 0x01);
        assert_eq!(BorderSide::TOP.0, 0x02);
        assert_eq!(BorderSide::LEFT.0, 0x04);
        assert_eq!(BorderSide::RIGHT.0, 0x08);
        assert_eq!(BorderSide::FULL.0, 0x0F);
    }

    #[test]
    fn border_side_bitor() {
        let combined = BorderSide::TOP | BorderSide::BOTTOM;
        assert_eq!(combined.0, 0x03);
    }

    #[test]
    fn border_side_full_is_all_sides() {
        let all = BorderSide::BOTTOM | BorderSide::TOP | BorderSide::LEFT | BorderSide::RIGHT;
        assert_eq!(all, BorderSide::FULL);
    }

    // -- lv_pct ------------------------------------------------------------

    #[test]
    fn lv_pct_monotonic() {
        assert!(lv_pct(50) > lv_pct(0));
        assert!(lv_pct(100) > lv_pct(50));
    }

    #[test]
    fn lv_pct_difference_matches_input() {
        // lv_pct(x) = LV_PCT_BASE + x, so difference should equal input difference.
        assert_eq!(lv_pct(100) - lv_pct(0), 100);
        assert_eq!(lv_pct(50) - lv_pct(0), 50);
    }

    // -- LV_SIZE_CONTENT ---------------------------------------------------

    #[test]
    fn size_content_uses_spec_type() {
        // LV_SIZE_CONTENT = LV_COORD_MAX | LV_COORD_TYPE_SPEC
        let expected = (oxivgl_sys::LV_COORD_MAX | oxivgl_sys::LV_COORD_TYPE_SPEC) as i32;
        assert_eq!(LV_SIZE_CONTENT, expected);
    }
}

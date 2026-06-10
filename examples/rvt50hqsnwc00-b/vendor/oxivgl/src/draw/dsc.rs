// SPDX-License-Identifier: MIT OR Apache-2.0
//! Owned draw descriptor types.

use oxivgl_sys::*;

// ── DrawRectDsc
// ───────────────────────────────────────────────────────────────

/// Owned LVGL rectangle draw descriptor.
///
/// Initialised via `lv_draw_rect_dsc_init`. Pass to [`Layer::draw_rect`](crate::draw::Layer::draw_rect).
pub struct DrawRectDsc {
    pub(crate) inner: lv_draw_rect_dsc_t,
}

impl DrawRectDsc {
    /// Create with LVGL defaults (`lv_draw_rect_dsc_init`).
    pub fn new() -> Self {
        // SAFETY: zeroed memory is a valid starting state; lv_draw_rect_dsc_init
        // fills all required fields (lv_draw.c).
        let mut inner = unsafe { core::mem::zeroed() };
        unsafe { lv_draw_rect_dsc_init(&mut inner) };
        Self { inner }
    }

    /// Set background color.
    pub fn bg_color(&mut self, color: lv_color_t) -> &mut Self {
        self.inner.bg_color = color;
        self
    }

    /// Set corner radius. Use `RADIUS_CIRCLE` (0x7FFF) for a full circle.
    pub fn radius(&mut self, r: i32) -> &mut Self {
        self.inner.radius = r;
        self
    }

    /// Set border color.
    pub fn border_color(&mut self, color: lv_color_t) -> &mut Self {
        self.inner.border_color = color;
        self
    }

    /// Set border width in pixels.
    pub fn border_width(&mut self, w: i32) -> &mut Self {
        self.inner.border_width = w;
        self
    }

    /// Set outline color.
    pub fn outline_color(&mut self, color: lv_color_t) -> &mut Self {
        self.inner.outline_color = color;
        self
    }

    /// Set outline width in pixels.
    pub fn outline_width(&mut self, w: i32) -> &mut Self {
        self.inner.outline_width = w;
        self
    }

    /// Set gap between the object border and the outline.
    pub fn outline_pad(&mut self, pad: i32) -> &mut Self {
        self.inner.outline_pad = pad;
        self
    }

    /// Set background gradient direction.
    pub fn bg_grad_dir(&mut self, dir: crate::style::GradDir) -> &mut Self {
        self.inner.bg_grad.set_dir(dir as lv_grad_dir_t);
        self
    }

    /// Configure a background gradient stop (index 0 or 1).
    ///
    /// - `frac`: position 0–255 (0 = start, 255 = end).
    /// - `opa`: opacity 0–255.
    pub fn bg_grad_stop(&mut self, idx: usize, color: lv_color_t, frac: u8, opa: u8) -> &mut Self {
        assert!(idx < 2, "gradient stop index must be 0 or 1");
        self.inner.bg_grad.stops[idx].color = color;
        self.inner.bg_grad.stops[idx].frac = frac;
        self.inner.bg_grad.stops[idx].opa = opa;
        self
    }

    /// Set background opacity (0 = transparent, 255 = opaque).
    pub fn bg_opa(&mut self, opa: u8) -> &mut Self {
        self.inner.bg_opa = opa;
        self
    }

    /// Raw pointer to the inner descriptor. Used by [`CanvasLayer::draw_rect`].
    pub(crate) fn as_ptr(&self) -> *const lv_draw_rect_dsc_t {
        &self.inner
    }
}

impl Default for DrawRectDsc {
    fn default() -> Self {
        Self::new()
    }
}

// ── DrawLabelDscOwned
// ─────────────────────────────────────────────────────────

/// Owned LVGL label draw descriptor.
///
/// Initialised via `lv_draw_label_dsc_init`. Pass to [`Layer::draw_label`](crate::draw::Layer::draw_label).
pub struct DrawLabelDscOwned {
    pub(crate) inner: lv_draw_label_dsc_t,
}

impl DrawLabelDscOwned {
    /// Create with default font (LV_FONT_DEFAULT) and LVGL defaults.
    pub fn default_font() -> Self {
        // SAFETY: zeroed is a valid starting state; init fills required fields.
        let mut inner = unsafe { core::mem::zeroed() };
        unsafe { lv_draw_label_dsc_init(&mut inner) };
        // SAFETY: lv_font_get_default() returns a pointer to the LV_FONT_DEFAULT
        // static font object, which is valid for the lifetime of the program.
        inner.font = unsafe { lv_font_get_default() };
        Self { inner }
    }

    /// Set text color.
    pub fn set_color(&mut self, color: lv_color_t) -> &mut Self {
        self.inner.color = color;
        self
    }

    /// Set the font.
    pub fn set_font(&mut self, font: crate::fonts::Font) -> &mut Self {
        self.inner.font = font.as_ptr();
        self
    }

    /// Set text alignment.
    pub fn set_align(&mut self, align: crate::widgets::TextAlign) -> &mut Self {
        self.inner.align = align as lv_text_align_t;
        self
    }

    /// Measure pixel size of `text` using this descriptor's current font and
    /// spacing.
    ///
    /// Returns `(width, height)`.
    pub fn text_size(&self, text: &str) -> (i32, i32) {
        let mut buf = [0u8; 64];
        let len = text.len().min(buf.len() - 1);
        buf[..len].copy_from_slice(&text.as_bytes()[..len]);
        buf[len] = 0;
        let mut size: lv_point_t = unsafe { core::mem::zeroed() };
        // SAFETY: font pointer valid (set in default_font); buf is null-terminated
        // stack data.
        unsafe {
            lv_text_get_size(
                &mut size,
                buf.as_ptr() as *const core::ffi::c_char,
                self.inner.font,
                self.inner.letter_space,
                self.inner.line_space,
                0x7FFF,
                lv_text_flag_t_LV_TEXT_FLAG_NONE,
            );
        }
        (size.x, size.y)
    }
}

// ── DrawArcDsc
// ────────────────────────────────────────────────────────────────

/// Descriptor for drawing an arc onto a
/// [`CanvasLayer`](crate::widgets::CanvasLayer).
pub struct DrawArcDsc {
    inner: lv_draw_arc_dsc_t,
}

impl DrawArcDsc {
    /// Create with LVGL defaults (`lv_draw_arc_dsc_init`).
    pub fn new() -> Self {
        // SAFETY: zeroed is a valid starting state; lv_draw_arc_dsc_init fills required
        // fields.
        let mut inner = unsafe { core::mem::zeroed::<lv_draw_arc_dsc_t>() };
        unsafe { lv_draw_arc_dsc_init(&mut inner) };
        Self { inner }
    }

    /// Arc center in canvas coordinates.
    pub fn center(&mut self, x: i32, y: i32) -> &mut Self {
        self.inner.center.x = x;
        self.inner.center.y = y;
        self
    }

    /// Outer radius in pixels.
    pub fn radius(&mut self, r: u16) -> &mut Self {
        self.inner.radius = r;
        self
    }

    /// Start and end angles in degrees (0–360, clockwise from 3 o'clock).
    ///
    /// Angles are `f32` (`lv_value_precise_t`) when `LV_USE_FLOAT=1`.
    pub fn angles(&mut self, start: f32, end: f32) -> &mut Self {
        self.inner.start_angle = start;
        self.inner.end_angle = end;
        self
    }

    /// Arc stroke width in pixels.
    pub fn width(&mut self, w: i32) -> &mut Self {
        self.inner.width = w;
        self
    }

    /// Stroke color.
    pub fn color(&mut self, c: lv_color_t) -> &mut Self {
        self.inner.color = c;
        self
    }

    /// Overall opacity (0 = transparent, 255 = opaque).
    pub fn opa(&mut self, o: u8) -> &mut Self {
        self.inner.opa = o;
        self
    }

    /// Draw rounded start and end caps.
    pub fn rounded(&mut self, r: bool) -> &mut Self {
        self.inner.set_rounded(r as u8);
        self
    }

    pub(crate) fn as_ptr(&self) -> *const lv_draw_arc_dsc_t {
        &self.inner
    }
}

impl Default for DrawArcDsc {
    fn default() -> Self {
        Self::new()
    }
}

// ── DrawLineDsc
// ───────────────────────────────────────────────────────────────

/// Descriptor for drawing a straight line onto a
/// [`CanvasLayer`](crate::widgets::CanvasLayer).
pub struct DrawLineDsc {
    inner: lv_draw_line_dsc_t,
}

impl DrawLineDsc {
    /// Create with LVGL defaults (`lv_draw_line_dsc_init`).
    pub fn new() -> Self {
        // SAFETY: zeroed is a valid starting state; lv_draw_line_dsc_init fills
        // required fields.
        let mut inner = unsafe { core::mem::zeroed::<lv_draw_line_dsc_t>() };
        unsafe { lv_draw_line_dsc_init(&mut inner) };
        Self { inner }
    }

    /// Start point. Coordinates are `f32` (`lv_point_precise_t`) when `LV_USE_FLOAT=1`.
    pub fn p1(&mut self, x: f32, y: f32) -> &mut Self {
        self.inner.p1.x = x;
        self.inner.p1.y = y;
        self
    }

    /// End point. Coordinates are `f32` (`lv_point_precise_t`) when `LV_USE_FLOAT=1`.
    pub fn p2(&mut self, x: f32, y: f32) -> &mut Self {
        self.inner.p2.x = x;
        self.inner.p2.y = y;
        self
    }

    /// Line stroke width in pixels.
    pub fn width(&mut self, w: i32) -> &mut Self {
        self.inner.width = w;
        self
    }

    /// Stroke color.
    pub fn color(&mut self, c: lv_color_t) -> &mut Self {
        self.inner.color = c;
        self
    }

    /// Overall opacity (0–255).
    pub fn opa(&mut self, o: u8) -> &mut Self {
        self.inner.opa = o;
        self
    }

    /// Draw a rounded cap at the start.
    pub fn round_start(&mut self, r: bool) -> &mut Self {
        self.inner.set_round_start(r as u8);
        self
    }

    /// Draw a rounded cap at the end.
    pub fn round_end(&mut self, r: bool) -> &mut Self {
        self.inner.set_round_end(r as u8);
        self
    }

    pub(crate) fn as_ptr(&self) -> *const lv_draw_line_dsc_t {
        &self.inner
    }
}

impl Default for DrawLineDsc {
    fn default() -> Self {
        Self::new()
    }
}

// ── DrawTriangleDsc
// ───────────────────────────────────────────────────────────

/// Descriptor for drawing a solid-fill or gradient-fill triangle onto a
/// [`CanvasLayer`](crate::widgets::CanvasLayer).
pub struct DrawTriangleDsc {
    inner: lv_draw_triangle_dsc_t,
}

impl DrawTriangleDsc {
    /// Create with LVGL defaults.
    pub fn new() -> Self {
        // SAFETY: zeroed is a valid starting state; init fills required fields.
        let mut inner = unsafe { core::mem::zeroed::<lv_draw_triangle_dsc_t>() };
        unsafe { lv_draw_triangle_dsc_init(&mut inner) };
        Self { inner }
    }

    /// Set the three vertex coordinates as `[(x0,y0), (x1,y1), (x2,y2)]`.
    ///
    /// Coordinates are `f32` (`lv_point_precise_t` fields).
    pub fn points(&mut self, pts: [(f32, f32); 3]) -> &mut Self {
        for (i, (x, y)) in pts.iter().enumerate() {
            self.inner.p[i].x = *x;
            self.inner.p[i].y = *y;
        }
        self
    }

    /// Overall opacity (0–255).
    pub fn opa(&mut self, o: u8) -> &mut Self {
        self.inner.opa = o;
        self
    }

    /// Solid fill color (used when no gradient is configured).
    pub fn color(&mut self, c: lv_color_t) -> &mut Self {
        self.inner.color = c;
        self
    }

    /// Number of active gradient stops (1 or 2).
    pub fn grad_stops_count(&mut self, n: u8) -> &mut Self {
        self.inner.grad.stops_count = n;
        self
    }

    /// Gradient direction.
    pub fn grad_dir(&mut self, dir: crate::style::GradDir) -> &mut Self {
        self.inner.grad.set_dir(dir as lv_grad_dir_t);
        self
    }

    /// Configure one gradient stop (index 0 or 1).
    ///
    /// - `frac`: position 0–255 (0 = start, 255 = end of gradient axis).
    /// - `opa`: stop opacity 0–255.
    pub fn grad_stop(&mut self, idx: usize, color: lv_color_t, frac: u8, opa: u8) -> &mut Self {
        assert!(idx < 2, "gradient stop index must be 0 or 1");
        self.inner.grad.stops[idx].color = color;
        self.inner.grad.stops[idx].frac = frac;
        self.inner.grad.stops[idx].opa = opa;
        self
    }

    pub(crate) fn as_ptr(&self) -> *const lv_draw_triangle_dsc_t {
        &self.inner
    }
}

impl Default for DrawTriangleDsc {
    fn default() -> Self {
        Self::new()
    }
}

// ── DrawImageDsc
// ──────────────────────────────────────────────────────────────

/// Descriptor for drawing an image onto a
/// [`CanvasLayer`](crate::widgets::CanvasLayer).
///
/// The `'i` lifetime ties this descriptor to the
/// [`ImageDsc`](crate::draw_buf::ImageDsc) source, which in turn borrows from
/// the originating [`DrawBuf`](crate::draw_buf::DrawBuf). This ensures the
/// pixel data remains valid until `lv_canvas_finish_layer` completes.
pub struct DrawImageDsc<'i> {
    inner: lv_draw_image_dsc_t,
    _img_lifetime: core::marker::PhantomData<&'i ()>,
}

impl<'i> DrawImageDsc<'i> {
    /// Create from an [`ImageDsc`](crate::draw_buf::ImageDsc) obtained via
    /// [`DrawBuf::image_dsc`](crate::draw_buf::DrawBuf::image_dsc).
    ///
    /// `'i` is bound to the `ImageDsc` borrow, which is bound to the `DrawBuf`
    /// lifetime — preventing the pixel buffer from being freed while this
    /// descriptor is in use.
    pub fn from_image_dsc(img: &'i crate::draw_buf::ImageDsc<'_>) -> Self {
        let mut inner = unsafe { core::mem::zeroed::<lv_draw_image_dsc_t>() };
        unsafe { lv_draw_image_dsc_init(&mut inner) };
        // SAFETY: img.inner contains a pointer into a DrawBuf pixel buffer.
        // 'i ties this descriptor to the ImageDsc borrow, which ties to the
        // DrawBuf lifetime — the pixel data is valid until 'i expires.
        inner.src = &img.inner as *const lv_image_dsc_t as *const core::ffi::c_void;
        Self { inner, _img_lifetime: core::marker::PhantomData }
    }

    /// Create from a static `lv_image_dsc_t` (e.g. from [`image_declare!`](crate::image_declare)).
    ///
    /// # Lifetime requirement
    /// The image descriptor must point to valid pixel data for the lifetime `'i`.
    pub fn from_static_dsc(img: &'i lv_image_dsc_t) -> Self {
        let mut inner = unsafe { core::mem::zeroed::<lv_draw_image_dsc_t>() };
        unsafe { lv_draw_image_dsc_init(&mut inner) };
        // SAFETY: img is a static image descriptor whose pixel data is
        // valid for 'i (typically 'static from image_declare!).
        inner.src = img as *const lv_image_dsc_t as *const core::ffi::c_void;
        Self { inner, _img_lifetime: core::marker::PhantomData }
    }

    /// Rotation in 0.1-degree units (e.g. `1200` = 120°).
    pub fn rotation(&mut self, r: i32) -> &mut Self {
        self.inner.rotation = r;
        self
    }

    /// Transform pivot point in canvas coordinates.
    pub fn pivot(&mut self, x: i32, y: i32) -> &mut Self {
        self.inner.pivot.x = x;
        self.inner.pivot.y = y;
        self
    }

    /// Overall opacity (0–255).
    pub fn opa(&mut self, o: u8) -> &mut Self {
        self.inner.opa = o;
        self
    }

    pub(crate) fn as_ptr(&self) -> *const lv_draw_image_dsc_t {
        &self.inner
    }
}

// ── DrawLetterDsc
// ─────────────────────────────────────────────────────────────

/// Descriptor for drawing a single Unicode glyph onto a
/// [`CanvasLayer`](crate::widgets::CanvasLayer).
///
/// Used by canvas_10 / canvas_11 for per-character animations.
pub struct DrawLetterDsc {
    inner: lv_draw_letter_dsc_t,
}

impl DrawLetterDsc {
    /// Create with LVGL defaults.
    pub fn new() -> Self {
        // SAFETY: zeroed is a valid starting state; init fills required fields.
        let mut inner = unsafe { core::mem::zeroed::<lv_draw_letter_dsc_t>() };
        unsafe { lv_draw_letter_dsc_init(&mut inner) };
        Self { inner }
    }

    /// Unicode code point to render.
    pub fn unicode(&mut self, cp: u32) -> &mut Self {
        self.inner.unicode = cp;
        self
    }

    /// Glyph color.
    pub fn color(&mut self, c: lv_color_t) -> &mut Self {
        self.inner.color = c;
        self
    }

    /// Set the font used to render the glyph.
    pub fn font(&mut self, f: crate::fonts::Font) -> &mut Self {
        self.inner.font = f.as_ptr();
        self
    }

    /// Rotation in 0.1-degree units (e.g. `900` = 90°).
    ///
    /// **Embedded note:** non-zero rotation triggers LVGL's vector-font path,
    /// which allocates an internal `ARGB8888` scratch buffer per glyph.
    /// On RAM-constrained targets use `rotation(0)` with an `RGB565` canvas.
    pub fn rotation(&mut self, r: i32) -> &mut Self {
        self.inner.rotation = r;
        self
    }

    /// Opacity (0 = transparent, 255 = opaque).
    pub fn opa(&mut self, o: u8) -> &mut Self {
        self.inner.opa = o;
        self
    }

    pub(crate) fn as_ptr(&self) -> *const lv_draw_letter_dsc_t {
        &self.inner
    }
}

impl Default for DrawLetterDsc {
    fn default() -> Self {
        Self::new()
    }
}

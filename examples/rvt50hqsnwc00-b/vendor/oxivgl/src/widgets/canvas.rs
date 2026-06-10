// SPDX-License-Identifier: GPL-3.0-only
//! Canvas widget — off-screen drawing surface.

use core::marker::PhantomData;

use oxivgl_sys::*;

use crate::{
    draw::{
        Area, DrawArcDsc, DrawImageDsc, DrawLabelDscOwned, DrawLetterDsc, DrawLineDsc, DrawRectDsc, DrawTriangleDsc,
    },
    draw_buf::DrawBuf,
    widgets::{AsLvHandle, Obj, WidgetError},
};

/// Off-screen drawing surface backed by a [`DrawBuf`].
///
/// `Canvas` takes ownership of its draw buffer. The buffer is freed
/// automatically whenever the LVGL canvas object is deleted — whether via
/// an explicit `lv_obj_delete` or parent-tree cascade.
/// No special cleanup call is required from user code.
///
/// Use [`init_layer`](Canvas::init_layer) to batch-draw onto the canvas;
/// drawing is committed when the returned [`CanvasLayer`] is dropped.
///
/// # Example
///
/// ```no_run
/// # use oxivgl::{draw_buf::{ColorFormat, DrawBuf}, widgets::{Canvas, Screen}};
/// # use oxivgl::draw::{Area, DrawRectDsc};
/// # use oxivgl::style::color_make;
/// let screen = Screen::active().unwrap();
/// let buf = DrawBuf::create(100, 100, ColorFormat::ARGB8888).unwrap();
/// let canvas = Canvas::new(&screen, buf).unwrap();
/// canvas.fill_bg(color_make(200, 200, 200), 255);
/// {
///     let mut layer = canvas.init_layer();
///     let mut rdc = DrawRectDsc::new();
///     rdc.bg_color(color_make(255, 0, 0));
///     layer.draw_rect(&rdc, Area { x1: 10, y1: 10, x2: 50, y2: 50 });
/// } // layer dropped → lv_canvas_finish_layer called
/// ```
#[derive(Debug)]
pub struct Canvas<'p> {
    obj: Obj<'p>,
    /// Raw pointer to a `Box<DrawBuf>` allocated in [`Canvas::new`].
    ///
    /// Ownership is transferred to the `LV_EVENT_DELETE` callback registered
    /// on the LVGL canvas object. The callback frees the box when the object
    /// is deleted (parent cascade or `lv_obj_delete`).
    /// Never freed by Rust's normal drop path — this raw pointer is just
    /// stored here so `draw_buf()` can return a reference.
    draw_buf: *mut DrawBuf,
}

/// LVGL `LV_EVENT_DELETE` callback registered on every `Canvas` object.
///
/// Fires when the LVGL canvas object is deleted by any path (parent cascade,
/// `lv_obj_delete`). Frees the `Box<DrawBuf>` whose raw pointer was
/// stored as `user_data` at registration time.
///
/// # Safety
/// `user_data` must be a valid `Box<DrawBuf>` raw pointer allocated in
/// `Canvas::new` and not freed elsewhere.
unsafe extern "C" fn canvas_delete_cb(e: *mut lv_event_t) {
    // SAFETY: user_data is a Box<DrawBuf> raw pointer from Canvas::new.
    let ptr = unsafe { lv_event_get_user_data(e) } as *mut DrawBuf;
    if !ptr.is_null() {
        unsafe { drop(alloc::boxed::Box::from_raw(ptr)) };
    }
}

impl<'p> Canvas<'p> {
    /// Create a canvas child of `parent`, backed by `buf`.
    ///
    /// `Canvas` takes ownership of `buf`. The buffer is freed automatically
    /// when the LVGL canvas object is deleted (parent cascade or
    /// explicit deletion); no explicit cleanup is required.
    pub fn new(parent: &impl AsLvHandle, buf: DrawBuf) -> Result<Self, WidgetError> {
        let handle = parent.lv_handle();
        if handle.is_null() {
            return Err(WidgetError::LvglNullPointer);
        }
        // SAFETY: handle is a valid non-null LVGL object pointer.
        let obj_ptr = unsafe { lv_canvas_create(handle) };
        if obj_ptr.is_null() {
            return Err(WidgetError::LvglNullPointer);
        }
        // Heap-allocate buf so we can hand its raw pointer to the DELETE callback.
        let buf_box = alloc::boxed::Box::new(buf);
        let buf_ptr = alloc::boxed::Box::into_raw(buf_box);
        // SAFETY: obj_ptr is valid; buf_ptr is valid for the canvas lifetime.
        unsafe { lv_canvas_set_draw_buf(obj_ptr, (*buf_ptr).as_ptr()) };
        // Register a DELETE callback to free buf_ptr when the LVGL object is
        // deleted by any path (parent cascade, lv_obj_delete).
        // SAFETY: obj_ptr valid; canvas_delete_cb only reads user_data on DELETE.
        unsafe {
            lv_obj_add_event_cb(
                obj_ptr,
                Some(canvas_delete_cb),
                lv_event_code_t_LV_EVENT_DELETE,
                buf_ptr as *mut core::ffi::c_void,
            )
        };
        Ok(Self { obj: Obj::from_raw(obj_ptr), draw_buf: buf_ptr })
    }

    /// Borrow the owned draw buffer (e.g. to obtain an `lv_image_dsc_t` for
    /// canvas-to-canvas drawing via [`DrawBuf::image_dsc`]).
    ///
    /// # Warning
    ///
    /// Do not call this from inside an `LV_EVENT_DELETE` callback on the same
    /// canvas — the underlying allocation may already be freed by
    /// `canvas_delete_cb`.
    pub fn draw_buf(&self) -> &DrawBuf {
        // SAFETY: draw_buf is a valid Box<DrawBuf> raw pointer for the lifetime
        // of this Canvas (freed by canvas_delete_cb on LV_EVENT_DELETE).
        unsafe { &*self.draw_buf }
    }

    /// Fill the entire canvas background with a solid color.
    ///
    /// `opa`: 0 = fully transparent, 255 = fully opaque.
    pub fn fill_bg(&self, color: lv_color_t, opa: u8) -> &Self {
        // SAFETY: handle is valid.
        unsafe { lv_canvas_fill_bg(self.lv_handle(), color, opa) };
        self
    }

    /// Set a single pixel. Effective only on ARGB/palette format buffers.
    pub fn set_px(&self, x: i32, y: i32, color: lv_color_t, opa: u8) -> &Self {
        // SAFETY: handle is valid; x/y out-of-bounds are clamped by LVGL.
        unsafe { lv_canvas_set_px(self.lv_handle(), x, y, color, opa) };
        self
    }

    /// Get a pixel's 32-bit ARGB color value.
    pub fn get_px(&self, x: i32, y: i32) -> lv_color32_t {
        // SAFETY: handle is valid.
        unsafe { lv_canvas_get_px(self.lv_handle(), x, y) }
    }

    /// Begin a batch drawing session.
    ///
    /// Returns a [`CanvasLayer`] RAII guard. Draw onto it using the `draw_*`
    /// methods. When the guard is dropped, `lv_canvas_finish_layer` is called
    /// and all queued draw commands are committed synchronously.
    pub fn init_layer(&self) -> CanvasLayer<'_> {
        CanvasLayer::new(self.lv_handle())
    }

    /// Draw into a [`DrawBuf`] using a temporary canvas, then discard the
    /// canvas.
    ///
    /// Creates a throw-away LVGL canvas object on the active screen, sets
    /// `buf` as its draw buffer, invokes the closure to perform drawing,
    /// and deletes the canvas. The draw buffer is **not** freed — it
    /// remains valid for use as a bitmap mask source or image.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use oxivgl::draw_buf::{ColorFormat, DrawBuf};
    /// # use oxivgl::draw::{Area, DrawRectDsc};
    /// # use oxivgl::style::color_make;
    /// # use oxivgl::widgets::Canvas;
    /// let buf = DrawBuf::create(100, 50, ColorFormat::L8).unwrap();
    /// Canvas::draw_to_buf(&buf, |canvas| {
    ///     canvas.fill_bg(color_make(0, 0, 0), 0);
    ///     let mut layer = canvas.init_layer();
    ///     let mut rdc = DrawRectDsc::new();
    ///     rdc.bg_color(color_make(255, 255, 255));
    ///     layer.draw_rect(&rdc, Area { x1: 0, y1: 0, x2: 99, y2: 49 });
    /// });
    /// ```
    pub fn draw_to_buf(buf: &DrawBuf, f: impl FnOnce(&TempCanvas<'_>)) {
        // SAFETY: lv_screen_active returns the current active screen.
        let screen = unsafe { lv_screen_active() };
        assert!(!screen.is_null(), "no active screen");
        // SAFETY: screen is valid; lv_canvas_create returns a valid canvas.
        let canvas_ptr = unsafe { lv_canvas_create(screen) };
        assert!(!canvas_ptr.is_null(), "canvas creation failed");
        // SAFETY: canvas_ptr valid; buf.as_ptr() valid for the DrawBuf lifetime.
        // We do NOT register a delete callback — the DrawBuf is not owned by
        // this temporary canvas.
        unsafe { lv_canvas_set_draw_buf(canvas_ptr, buf.as_ptr()) };

        // Guard ensures the LVGL canvas is deleted even if `f` panics.
        struct Guard(*mut lv_obj_t);
        impl Drop for Guard {
            fn drop(&mut self) {
                // SAFETY: canvas_ptr is valid; delete removes only the LVGL
                // object, not the draw buffer (set externally, not owned).
                unsafe { lv_obj_delete(self.0) };
            }
        }
        let _guard = Guard(canvas_ptr);

        let temp = TempCanvas { ptr: canvas_ptr, _life: PhantomData };
        f(&temp);
    }
}

/// Temporary canvas handle for [`Canvas::draw_to_buf`].
///
/// Provides `fill_bg` and `init_layer` for drawing onto an externally-owned
/// [`DrawBuf`]. The LVGL canvas object is deleted when `draw_to_buf` returns;
/// this handle must not escape the closure.
pub struct TempCanvas<'a> {
    ptr: *mut lv_obj_t,
    _life: PhantomData<&'a ()>,
}

impl<'a> TempCanvas<'a> {
    /// Fill the entire canvas background with a solid color.
    pub fn fill_bg(&self, color: lv_color_t, opa: u8) {
        // SAFETY: ptr is a valid canvas object.
        unsafe { lv_canvas_fill_bg(self.ptr, color, opa) };
    }

    /// Set a single pixel on the canvas.
    pub fn set_px(&self, x: i32, y: i32, color: lv_color_t, opa: u8) {
        // SAFETY: ptr is a valid canvas object; out-of-bounds clamped by LVGL.
        unsafe { lv_canvas_set_px(self.ptr, x, y, color, opa) };
    }

    /// Begin a batch drawing session on this temporary canvas.
    pub fn init_layer(&self) -> CanvasLayer<'_> {
        CanvasLayer::new(self.ptr)
    }
}

impl<'p> AsLvHandle for Canvas<'p> {
    fn lv_handle(&self) -> *mut lv_obj_t {
        self.obj.lv_handle()
    }
}

impl<'p> core::ops::Deref for Canvas<'p> {
    type Target = Obj<'p>;
    fn deref(&self) -> &Self::Target {
        &self.obj
    }
}

/// RAII guard for canvas batch drawing.
///
/// Obtained via [`Canvas::init_layer`]. Draw calls are queued until this guard
/// is dropped, at which point `lv_canvas_finish_layer` flushes all commands
/// synchronously.
pub struct CanvasLayer<'c> {
    canvas: *mut lv_obj_t,
    layer: lv_layer_t,
    _canvas_lifetime: PhantomData<&'c ()>,
}

impl<'c> CanvasLayer<'c> {
    pub(crate) fn new(canvas: *mut lv_obj_t) -> Self {
        // SAFETY: canvas is a valid lv_canvas object pointer. The layer is
        // stack-allocated here and initialised by lv_canvas_init_layer.
        let mut layer = unsafe { core::mem::zeroed::<lv_layer_t>() };
        unsafe { lv_canvas_init_layer(canvas, &mut layer) };
        Self { canvas, layer, _canvas_lifetime: PhantomData }
    }

    /// Draw a filled/bordered rectangle.
    pub fn draw_rect(&mut self, dsc: &DrawRectDsc, area: Area) {
        let lv_area = lv_area_t::from(area);
        // SAFETY: layer is valid until finish_layer is called in Drop.
        unsafe { lv_draw_rect(&mut self.layer, dsc.as_ptr(), &lv_area) };
    }

    /// Draw an arc.
    pub fn draw_arc(&mut self, dsc: &DrawArcDsc) {
        // SAFETY: layer valid until Drop; lv_draw_arc reads dsc synchronously before
        // returning.
        unsafe { lv_draw_arc(&mut self.layer, dsc.as_ptr()) };
    }

    /// Draw a straight line.
    pub fn draw_line(&mut self, dsc: &DrawLineDsc) {
        // SAFETY: layer valid until Drop; lv_draw_line reads dsc synchronously before
        // returning.
        unsafe { lv_draw_line(&mut self.layer, dsc.as_ptr()) };
    }

    /// Draw a filled or gradient-filled triangle.
    pub fn draw_triangle(&mut self, dsc: &DrawTriangleDsc) {
        // SAFETY: layer valid until Drop; lv_draw_triangle reads dsc synchronously.
        unsafe { lv_draw_triangle(&mut self.layer, dsc.as_ptr()) };
    }

    /// Draw an image (or rotated canvas snapshot).
    ///
    /// The [`DrawImageDsc`] lifetime ensures the image source lives long
    /// enough.
    pub fn draw_image<'i>(&mut self, dsc: &DrawImageDsc<'i>, area: Area) {
        let lv_area = lv_area_t::from(area);
        // SAFETY: layer valid; image src in dsc valid for 'i; lv_canvas_finish_layer
        // (called in Drop) processes commands before 'i expires.
        unsafe { lv_draw_image(&mut self.layer, dsc.as_ptr(), &lv_area) };
    }

    /// Draw a single Unicode glyph at canvas position `(x, y)`.
    pub fn draw_letter(&mut self, dsc: &DrawLetterDsc, x: i32, y: i32) {
        let pt = lv_point_t { x, y };
        // Copy the descriptor so lv_draw_letter can write the pivot field without
        // aliasing a Rust shared reference.
        // SAFETY: layer valid; inner is a valid initialised descriptor.
        let mut inner = unsafe { *dsc.as_ptr() };
        unsafe { lv_draw_letter(&mut self.layer, &mut inner, &pt) };
    }

    /// Draw a text label. `text` must fit in 63 bytes; longer strings are
    /// truncated.
    pub fn draw_label(&mut self, dsc: &DrawLabelDscOwned, area: Area, text: &str) {
        let mut buf = [0u8; 64];
        let len = text.len().min(buf.len() - 1);
        buf[..len].copy_from_slice(&text.as_bytes()[..len]);
        buf[len] = 0;
        // lv_draw_label_dsc_t derives Copy — copy descriptor and set text pointer.
        // text_local=1: lv_draw_label calls lv_strndup when queueing the task,
        // so buf (on the stack) need only live until lv_draw_label returns, not
        // until lv_canvas_finish_layer in Drop renders the queued commands.
        let mut local_dsc = dsc.inner;
        local_dsc.text = buf.as_ptr() as *const _;
        local_dsc.set_text_local(1);
        let lv_area = lv_area_t::from(area);
        // SAFETY: layer valid; lv_draw_label copies text via lv_strndup before
        // returning (text_local=1), so buf lifetime need only cover this call.
        unsafe { lv_draw_label(&mut self.layer, &local_dsc, &lv_area) };
    }
}

impl Drop for CanvasLayer<'_> {
    fn drop(&mut self) {
        // SAFETY: canvas and layer are valid. finish_layer flushes queued draw
        // commands synchronously and invalidates the layer. After this call the
        // layer struct must not be used — we consume it in this Drop.
        unsafe { lv_canvas_finish_layer(self.canvas, &mut self.layer) };
    }
}

// SPDX-License-Identifier: MIT OR Apache-2.0
use core::{ops::Deref, ptr::null_mut};

use oxivgl_sys::*;

use super::{
    WidgetError,
    obj::{AsLvHandle, Obj},
};

/// LVGL image widget. Wraps [`Obj`](super::obj::Obj) and `Deref`s to it for
/// style methods.
#[derive(Debug)]
pub struct Image<'p> {
    obj: Obj<'p>,
}

impl<'p> AsLvHandle for Image<'p> {
    fn lv_handle(&self) -> *mut lv_obj_t {
        self.obj.lv_handle()
    }
}

impl<'p> Deref for Image<'p> {
    type Target = Obj<'p>;
    fn deref(&self) -> &Obj<'p> {
        &self.obj
    }
}

/// Image content alignment within the widget bounds.
#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum ImageAlign {
    /// Default alignment.
    Default = 0,
    /// Top-left corner.
    TopLeft = 1,
    /// Top-middle.
    TopMid = 2,
    /// Top-right corner.
    TopRight = 3,
    /// Bottom-left corner.
    BottomLeft = 4,
    /// Bottom-middle.
    BottomMid = 5,
    /// Bottom-right corner.
    BottomRight = 6,
    /// Left-middle.
    LeftMid = 7,
    /// Right-middle.
    RightMid = 8,
    /// Center.
    Center = 9,
    /// Stretch to fill.
    Stretch = 11,
    /// Tile/repeat to fill.
    Tile = 12,
    /// Contain (fit inside, keep aspect ratio).
    Contain = 13,
    /// Cover (fill area, keep aspect ratio, crop overflow).
    Cover = 14,
}

impl<'p> Image<'p> {
    /// Create an image widget as a child of `parent`. Returns
    /// [`WidgetError::LvglNullPointer`] on OOM.
    pub fn new(parent: &impl AsLvHandle) -> Result<Self, WidgetError> {
        let parent_ptr = parent.lv_handle();
        assert_ne!(parent_ptr, null_mut(), "Parent widget cannot be null");
        // SAFETY: parent_ptr non-null (asserted above); lv_init() called via
        // LvglDriver.
        let handle = unsafe { lv_image_create(parent_ptr) };
        if handle.is_null() { Err(WidgetError::LvglNullPointer) } else { Ok(Image { obj: Obj::from_raw(handle) }) }
    }

    /// Set image rotation in 0.1 degree units (e.g. 450 = 45 degrees).
    pub fn set_rotation(&self, angle: i32) -> &Self {
        unsafe { lv_image_set_rotation(self.lv_handle(), angle) };
        self
    }

    /// Set uniform image scale (256 = 1.0x, 512 = 2.0x, 128 = 0.5x).
    pub fn set_scale(&self, zoom: u32) -> &Self {
        unsafe { lv_image_set_scale(self.lv_handle(), zoom) };
        self
    }

    /// Set the pivot point for rotation and scaling.
    pub fn set_pivot(&self, x: i32, y: i32) -> &Self {
        unsafe { lv_image_set_pivot(self.lv_handle(), x, y) };
        self
    }

    /// Set vertical image offset (scrolls the image content within the widget).
    pub fn set_offset_y(&self, y: i32) -> &Self {
        unsafe { lv_image_set_offset_y(self.lv_handle(), y) };
        self
    }

    /// Set how the image is aligned/scaled within the widget area.
    pub fn set_inner_align(&self, align: ImageAlign) -> &Self {
        unsafe { lv_image_set_inner_align(self.lv_handle(), align as lv_image_align_t) };
        self
    }

    /// Set the image source from a compiled image descriptor.
    ///
    /// LVGL stores the raw pointer (`lv_image_t.src`), so the descriptor
    /// must be `'static`. Use [`image_declare!`](crate::image_declare) to
    /// obtain a safe `&'static lv_image_dsc_t` reference (spec §3.1).
    ///
    /// # Example
    ///
    /// ```ignore
    /// oxivgl::image_declare!(my_icon);
    /// let img = Image::new(&screen)?;
    /// img.set_src(my_icon());
    /// ```
    pub fn set_src(&self, dsc: &'static lv_image_dsc_t) -> &Self {
        // SAFETY: handle non-null (from Image::new); dsc is 'static and
        // points to a valid lv_image_dsc_t compiled by oxivgl-build.
        // LVGL stores the pointer (spec §3.1); 'static satisfies this.
        unsafe { lv_image_set_src(self.obj.handle(), dsc as *const lv_image_dsc_t as *const core::ffi::c_void) };
        self
    }

    /// Get image rotation in 0.1 degree units (e.g. 450 = 45 degrees).
    pub fn get_rotation(&self) -> i32 {
        // SAFETY: handle non-null (from Image::new).
        unsafe { lv_image_get_rotation(self.lv_handle()) }
    }

    /// Get uniform image scale (256 = 1.0x, 512 = 2.0x, 128 = 0.5x).
    pub fn get_scale(&self) -> i32 {
        // SAFETY: handle non-null (from Image::new).
        unsafe { lv_image_get_scale(self.lv_handle()) }
    }

    /// Get X-axis image scale (256 = 1.0x).
    pub fn get_scale_x(&self) -> i32 {
        // SAFETY: handle non-null (from Image::new).
        unsafe { lv_image_get_scale_x(self.lv_handle()) }
    }

    /// Get horizontal image offset in pixels.
    pub fn get_offset_x(&self) -> i32 {
        // SAFETY: handle non-null (from Image::new).
        unsafe { lv_image_get_offset_x(self.lv_handle()) }
    }

    /// Get vertical image offset in pixels.
    pub fn get_offset_y(&self) -> i32 {
        // SAFETY: handle non-null (from Image::new).
        unsafe { lv_image_get_offset_y(self.lv_handle()) }
    }

    /// Get the inner alignment mode as raw `u32`.
    ///
    /// Returns the raw `lv_image_align_t` value because `lv_image_align_t`
    /// includes an internal `_AUTO_TRANSFORM = 10` variant not covered by
    /// [`ImageAlign`].
    pub fn get_inner_align(&self) -> u32 {
        // SAFETY: handle non-null (from Image::new).
        unsafe { lv_image_get_inner_align(self.lv_handle()) }
    }

    /// Get whether anti-aliasing is enabled for this image.
    pub fn get_antialias(&self) -> bool {
        // SAFETY: handle non-null (from Image::new).
        unsafe { lv_image_get_antialias(self.lv_handle()) }
    }

    /// Get the source image width in pixels (before transform).
    pub fn get_src_width(&self) -> i32 {
        // SAFETY: handle non-null (from Image::new).
        unsafe { lv_image_get_src_width(self.lv_handle()) }
    }

    /// Get the source image height in pixels (before transform).
    pub fn get_src_height(&self) -> i32 {
        // SAFETY: handle non-null (from Image::new).
        unsafe { lv_image_get_src_height(self.lv_handle()) }
    }

    /// Set image source from a snapshot draw buffer.
    ///
    /// LVGL stores the raw pointer — the [`Snapshot`](crate::snapshot::Snapshot)
    /// must outlive the image widget. Store both in the View struct with the
    /// snapshot field declared **after** the image field (Rust drops fields
    /// in declaration order — image drops first, then snapshot).
    #[cfg(not(target_os = "none"))]
    pub fn set_src_snapshot(&self, snap: &crate::snapshot::Snapshot) -> &Self {
        // SAFETY: handle non-null (from Image::new); snap.draw_buf_ptr() is
        // valid and owned by the Snapshot. LVGL stores the pointer (spec §3.1);
        // caller ensures snap outlives self via struct field ordering.
        unsafe {
            lv_image_set_src(
                self.obj.handle(),
                snap.draw_buf_ptr() as *const core::ffi::c_void,
            )
        };
        self
    }

    /// Set the image source to a built-in LVGL symbol.
    ///
    /// LVGL stores the pointer (`lv_image_t.src`); the symbol is `'static`
    /// (compiled into the binary), satisfying spec §3.1.
    ///
    /// ```ignore
    /// use oxivgl::{symbols, widgets::Image};
    /// let img = Image::new(&screen)?;
    /// img.set_src_symbol(&symbols::SETTINGS);
    /// ```
    pub fn set_src_symbol(&self, symbol: &crate::symbols::Symbol) -> &Self {
        // SAFETY: handle non-null (from Image::new); symbol.as_ptr() returns
        // a 'static NUL-terminated string. LVGL detects the string source type
        // by inspecting the first byte (lv_image.c:lv_image_src_get_type).
        unsafe { lv_image_set_src(self.obj.handle(), symbol.as_ptr() as *const core::ffi::c_void) };
        self
    }
}

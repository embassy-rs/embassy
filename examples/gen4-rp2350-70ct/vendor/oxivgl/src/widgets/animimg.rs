// SPDX-License-Identifier: MIT OR Apache-2.0
use core::{ops::Deref, ptr::null_mut};

use oxivgl_sys::*;

use super::{
    WidgetError,
    obj::{AsLvHandle, Obj},
};

/// LVGL animated image widget — cycles through a sequence of image frames.
/// Built on top of [`Image`](super::Image); wraps [`Obj`](super::obj::Obj)
/// and `Deref`s to it for style/positioning methods.
///
/// Requires `LV_USE_ANIMIMG = 1` and `LV_USE_IMAGE = 1` in `lv_conf.h`.
///
/// # Pointer safety
///
/// `lv_animimg_set_src` stores the raw pointer to the image descriptor array
/// (`lv_animimage_private.h: dsc` field). Both the array and all image
/// descriptors it references must be `'static` (spec §3.1).
#[derive(Debug)]
pub struct AnimImg<'p> {
    obj: Obj<'p>,
}

impl<'p> AsLvHandle for AnimImg<'p> {
    fn lv_handle(&self) -> *mut lv_obj_t {
        self.obj.lv_handle()
    }
}

impl<'p> Deref for AnimImg<'p> {
    type Target = Obj<'p>;
    fn deref(&self) -> &Obj<'p> {
        &self.obj
    }
}

impl<'p> AnimImg<'p> {
    /// Create an animated image widget as a child of `parent`. Returns
    /// [`WidgetError::LvglNullPointer`] on OOM.
    pub fn new(parent: &impl AsLvHandle) -> Result<Self, WidgetError> {
        let parent_ptr = parent.lv_handle();
        assert_ne!(parent_ptr, null_mut(), "Parent widget cannot be null");
        // SAFETY: parent_ptr non-null (asserted above); lv_init() called via
        // LvglDriver.
        let handle = unsafe { lv_animimg_create(parent_ptr) };
        if handle.is_null() {
            Err(WidgetError::LvglNullPointer)
        } else {
            Ok(AnimImg { obj: Obj::from_raw(handle) })
        }
    }

    /// Set the image animation source frames.
    ///
    /// LVGL stores the raw pointer to `dsc` (`lv_animimage_private.h:
    /// _lv_animimg_t.dsc`). Both the slice and all image descriptors it
    /// references must be `'static` (spec §3.1).
    ///
    /// # Example
    ///
    /// ```ignore
    /// oxivgl::image_declare!(frame1);
    /// oxivgl::image_declare!(frame2);
    ///
    /// static FRAMES: [&lv_image_dsc_t; 2] = [frame1(), frame2()];
    /// // Use a static array of raw pointers for lv_animimg_set_src:
    /// static FRAME_PTRS: [*const core::ffi::c_void; 2] = [ ... ];
    /// animimg.set_src(&FRAME_PTRS);
    /// ```
    pub fn set_src(&self, dsc: &'static [*const core::ffi::c_void]) -> &Self {
        // SAFETY: handle non-null (from AnimImg::new); dsc is 'static and
        // points to a valid array of lv_image_dsc_t pointers. LVGL stores the
        // raw pointer to the array (spec §3.1); 'static satisfies this.
        unsafe {
            lv_animimg_set_src(
                self.obj.handle(),
                dsc.as_ptr() as *mut *const core::ffi::c_void,
                dsc.len(),
            )
        };
        self
    }

    /// Set the animation cycle duration in milliseconds.
    pub fn set_duration(&self, duration: u32) -> &Self {
        // SAFETY: handle non-null (constructor guarantees).
        unsafe { lv_animimg_set_duration(self.obj.handle(), duration) };
        self
    }

    /// Set how many times the animation repeats.
    ///
    /// Use [`oxivgl::anim::ANIM_REPEAT_INFINITE`](crate::anim::ANIM_REPEAT_INFINITE)
    /// for infinite looping.
    pub fn set_repeat_count(&self, count: u32) -> &Self {
        // SAFETY: handle non-null (constructor guarantees).
        unsafe { lv_animimg_set_repeat_count(self.obj.handle(), count) };
        self
    }

    /// Start the frame animation.
    pub fn start(&self) -> &Self {
        // SAFETY: handle non-null (constructor guarantees).
        unsafe { lv_animimg_start(self.obj.handle()) };
        self
    }

    /// Get the number of source frames.
    pub fn get_src_count(&self) -> u8 {
        // SAFETY: handle non-null (constructor guarantees).
        unsafe { lv_animimg_get_src_count(self.obj.handle()) }
    }

    /// Get the animation duration in milliseconds.
    pub fn get_duration(&self) -> u32 {
        // SAFETY: handle non-null (constructor guarantees).
        unsafe { lv_animimg_get_duration(self.obj.handle()) }
    }

    /// Get the repeat count.
    pub fn get_repeat_count(&self) -> u32 {
        // SAFETY: handle non-null (constructor guarantees).
        unsafe { lv_animimg_get_repeat_count(self.obj.handle()) }
    }
}

#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! AnimImg 1 — Animated image cycling through frames
//!
//! Demonstrates the `AnimImg` widget with a two-frame animation using the
//! cogwheel image asset. Both frames use the same image; in a real
//! application each frame would be a distinct image. The animation loops
//! infinitely with a 1-second cycle.

extern crate alloc;

use oxivgl::{
    anim::ANIM_REPEAT_INFINITE,
    view::{NavAction, View},
    widgets::{Obj, AnimImg, WidgetError},
};

oxivgl::image_declare!(img_cogwheel_argb);

#[derive(Default)]
struct AnimImg1 {
    _animimg: Option<AnimImg<'static>>,
}

impl View for AnimImg1 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let animimg = AnimImg::new(container)?;
        animimg.center();

        // Both frames use the same image; in a real application each frame
        // would be a distinct image descriptor.
        let dsc = img_cogwheel_argb();
        let ptr = dsc as *const _ as *const core::ffi::c_void;
        // Leak a small heap array so the frame pointer slice is 'static
        // (LVGL stores the raw pointer — spec §3.1).
        let frames: &'static [*const core::ffi::c_void] =
            alloc::boxed::Box::leak(alloc::boxed::Box::new([ptr, ptr]));
        animimg
            .set_src(frames)
            .set_duration(1000)
            .set_repeat_count(ANIM_REPEAT_INFINITE)
            .start();

        self._animimg = Some(animimg);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(AnimImg1::default());

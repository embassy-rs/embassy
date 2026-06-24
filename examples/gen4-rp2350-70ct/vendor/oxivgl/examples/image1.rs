#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Image 1 — Basic image display
//!
//! Centered cogwheel image.

use oxivgl::{
    view::{NavAction, View},
    widgets::{Obj, Image, WidgetError},
};

oxivgl::image_declare!(img_cogwheel_argb);

#[derive(Default)]
struct WidgetImage1 {
    _img: Option<Image<'static>>,
}

impl View for WidgetImage1 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let img = Image::new(container)?;
        img.set_src(img_cogwheel_argb());
        img.center();

                self._img = Some(img);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(WidgetImage1::default());

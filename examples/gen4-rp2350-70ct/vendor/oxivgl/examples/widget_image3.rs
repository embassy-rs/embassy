#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Image 3 — Image rotation
//!
//! Cogwheel image rotating continuously via `update()`.

use oxivgl::{
    view::{NavAction, View},
    widgets::{Obj, Image, WidgetError},
};

oxivgl::image_declare!(img_cogwheel_argb);

#[derive(Default)]
struct WidgetImage3 {
    img: Option<Image<'static>>,
    angle: i32,
}

impl View for WidgetImage3 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let img = Image::new(container)?;
        img.set_src(img_cogwheel_argb());
        img.center();
        img.set_pivot(50, 50);

                self.img = Some(img);
        self.angle = 0;
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        self.angle = (self.angle + 30) % 3600;
        if let Some(ref img) = self.img { img.set_rotation(self.angle); }
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(WidgetImage3::default());

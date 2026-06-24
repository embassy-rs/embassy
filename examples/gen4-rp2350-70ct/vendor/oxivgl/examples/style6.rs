#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Style 6 — Image style properties
//!
//! Cogwheel image rotated 30°, blue recolor tint (50% opacity), grey
//! background with blue border.

extern crate alloc;

use oxivgl::{
    style::{palette_lighten, palette_main, Palette, Selector, Style, StyleBuilder},
    view::{NavAction, View},
    widgets::{Obj, Image, WidgetError},
};

oxivgl::image_declare!(img_cogwheel_argb);

#[derive(Default)]
struct Style6 {
    _style: Option<Style>,
    _img: Option<Image<'static>>,
}

impl View for Style6 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let mut builder = StyleBuilder::new();
        builder
            .radius(5)
            .bg_opa(255)
            .bg_color(palette_lighten(Palette::Grey, 3))
            .border_width(2)
            .border_color(palette_main(Palette::Blue))
            .image_recolor(palette_main(Palette::Blue))
            .image_recolor_opa(128)
            .transform_rotation(300);
        let style = builder.build();

        let img = Image::new(container)?;
        img.add_style(&style, Selector::DEFAULT);
        img.set_src(img_cogwheel_argb());
        img.center();

                self._style = Some(style);
        self._img = Some(img);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(Style6::default());

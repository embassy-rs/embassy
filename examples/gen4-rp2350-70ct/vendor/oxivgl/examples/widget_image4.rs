#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Image 4 — Image offset animation
//!
//! Stripe image with recolor tint and animated vertical offset scroll.

extern crate alloc;

use oxivgl::{
    style::{color_black, palette_main, Palette, Selector, Style, StyleBuilder},
    view::{NavAction, View},
    widgets::{Obj, Image, WidgetError},
};

oxivgl::image_declare!(img_skew_strip);

#[derive(Default)]
struct WidgetImage4 {
    _style: Option<Style>,
    img: Option<Image<'static>>,
    offset_y: i32,
    direction: i32,
}

impl View for WidgetImage4 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let mut sb = StyleBuilder::new();
        sb.bg_color(palette_main(Palette::Yellow))
            .bg_opa(255)
            .image_recolor_opa(255)
            .image_recolor(color_black());
        let style = sb.build();

        let img = Image::new(container)?;
        img.add_style(&style, Selector::DEFAULT);
        img.set_src(img_skew_strip());
        img.size(150, 100);
        img.center();

                self._style = Some(style);
        self.img = Some(img);
        self.offset_y = 0;
        self.direction = 1;
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        self.offset_y += self.direction;
        if self.offset_y >= 100 || self.offset_y <= 0 {
            self.direction = -self.direction;
        }
        if let Some(ref img) = self.img {
            img.set_offset_y(self.offset_y);
        }
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(WidgetImage4::default());

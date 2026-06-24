#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Image 5 — Image inner alignment
//!
//! Three images showing different inner alignment modes: default, stretch, tile.

extern crate alloc;

use oxivgl::{
    style::{palette_main, Palette, Selector, Style, StyleBuilder},
    view::{NavAction, View},
    widgets::{Obj, Align, Image, ImageAlign, WidgetError},
};

oxivgl::image_declare!(img_cogwheel_argb);

#[derive(Default)]
struct WidgetImage5 {
    _style: Option<Style>,
}

impl View for WidgetImage5 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let mut sb = StyleBuilder::new();
        sb.outline_width(1)
            .outline_color(palette_main(Palette::Grey));
        let style = sb.build();

        // Image 1: Default alignment
        let img1 = Image::new(container)?;
        img1.set_src(img_cogwheel_argb());
        img1.size(100, 100);
        img1.set_inner_align(ImageAlign::Default);
        img1.add_style(&style, Selector::DEFAULT);
        img1.align(Align::LeftMid, 20, 0);

        // Image 2: Stretch mode
        let img2 = Image::new(container)?;
        img2.set_src(img_cogwheel_argb());
        img2.size(100, 100);
        img2.set_inner_align(ImageAlign::Stretch);
        img2.add_style(&style, Selector::DEFAULT);
        img2.center();

        // Image 3: Tile mode
        let img3 = Image::new(container)?;
        img3.set_src(img_cogwheel_argb());
        img3.size(100, 100);
        img3.set_inner_align(ImageAlign::Tile);
        img3.add_style(&style, Selector::DEFAULT);
        img3.align(Align::RightMid, -20, 0);

                self._style = Some(style);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(WidgetImage5::default());

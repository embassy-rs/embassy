#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Style 3 — Border

extern crate alloc;

use oxivgl::{
    style::{
        palette_lighten, palette_main, BorderSide, Palette, Selector, Style, StyleBuilder,
    },
    view::{NavAction, View},
    widgets::{Obj, WidgetError},
};

#[derive(Default)]
struct Style3 {
    _obj: Option<Obj<'static>>,
    _style: Option<Style>,
}

impl View for Style3 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let mut builder = StyleBuilder::new();
        builder
            .radius(10)
            .bg_opa(255)
            .bg_color(palette_lighten(Palette::Grey, 1))
            .border_color(palette_main(Palette::Blue))
            .border_width(5)
            .border_opa(127)
            .border_side(BorderSide::BOTTOM | BorderSide::RIGHT);
        let style = builder.build();

        let obj = Obj::new(container)?;
        obj.add_style(&style, Selector::DEFAULT);
        obj.center();

                self._obj = Some(obj);
        self._style = Some(style);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(Style3::default());

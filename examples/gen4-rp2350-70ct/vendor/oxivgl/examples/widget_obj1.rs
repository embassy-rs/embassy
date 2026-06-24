#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Obj 1 — Base objects with custom styles
//!
//! Two base objects: a plain one and one with a blue shadow style.

use oxivgl::{
    style::{palette_main, Palette, Selector, Style, StyleBuilder},
    view::{NavAction, View},
    widgets::{Align, Obj, WidgetError},
};

#[derive(Default)]
struct WidgetObj1 {
    _obj1: Option<Obj<'static>>,
    _obj2: Option<Obj<'static>>,
    _style_shadow: Option<Style>,
}

impl View for WidgetObj1 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let obj1 = Obj::new(container)?;
        obj1.size(100, 50);
        obj1.align(Align::Center, -60, -30);

        let mut style_shadow = StyleBuilder::new();
        style_shadow
            .shadow_width(10)
            .shadow_spread(2)
            .shadow_color(palette_main(Palette::Blue));
        let style_shadow = style_shadow.build();

        let obj2 = Obj::new(container)?;
        obj2.add_style(&style_shadow, Selector::DEFAULT);
        obj2.align(Align::Center, 60, 30);

                self._obj1 = Some(obj1);
        self._obj2 = Some(obj2);
        self._style_shadow = Some(style_shadow);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(WidgetObj1::default());

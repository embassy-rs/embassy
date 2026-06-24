#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Style 2 — Background gradient

extern crate alloc;

use oxivgl::{
    style::{
        palette_lighten, palette_main, GradDir, GradDsc, Palette, Selector, Style, StyleBuilder,
    },
    view::{NavAction, View},
    widgets::{Obj, WidgetError},
};

struct Style2 {
    _obj: Option<Obj<'static>>,
    _style: Style,
}

impl Style2 {
    fn new() -> Self {
        Self {
            _obj: None,
            _style: StyleBuilder::new().build(),
        }
    }
}

impl View for Style2 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let mut grad = GradDsc::new();
        grad.set_dir(GradDir::Ver)
            .set_stops_count(2)
            .set_stop(0, palette_lighten(Palette::Grey, 1), 255, 128)
            .set_stop(1, palette_main(Palette::Blue), 255, 192);

        let mut builder = StyleBuilder::new();
        builder.radius(5).bg_opa(255).bg_grad(grad);
        let style = builder.build();

        let obj = Obj::new(container)?;
        obj.add_style(&style, Selector::DEFAULT);
        obj.center();

                self._obj = Some(obj);
        self._style = style;
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(Style2::new());

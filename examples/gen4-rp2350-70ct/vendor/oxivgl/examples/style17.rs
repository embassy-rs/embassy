#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Style 17 — Radial gradient

use oxivgl::{
    style::{color_make, lv_pct, GradDsc, GradExtend, Selector, Style, StyleBuilder},
    view::{NavAction, View},
    widgets::{Obj, WidgetError},
};

#[derive(Default)]
struct Style17 {
    _obj: Option<Obj<'static>>,
    _style: Option<Style>,
}

impl View for Style17 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let colors = [color_make(0x9b, 0x18, 0x42), color_make(0x00, 0x00, 0x00)];

        let mut grad = GradDsc::new();
        grad.init_stops(&colors, &[], &[]).radial(
            lv_pct(50),
            lv_pct(50),
            lv_pct(100),
            lv_pct(100),
            GradExtend::Pad,
        );

        let mut style = StyleBuilder::new();
        style.bg_grad(grad);
        let style = style.build();

        let obj = Obj::new(container)?;
        obj.add_style(&style, Selector::DEFAULT);
        obj.size(320, 240);
        obj.center();

                self._obj = Some(obj);
        self._style = Some(style);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(Style17::default());

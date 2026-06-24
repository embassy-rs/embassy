#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Getting Started 8 — Conical Gradient

use oxivgl::{
    style::{color_make, lv_pct, GradDsc, GradExtend, Selector, Style, StyleBuilder},
    view::{NavAction, View},
    widgets::{Obj, WidgetError},
};

#[derive(Default)]
struct GettingStarted8 {
    _obj: Option<Obj<'static>>,
    _style: Option<Style>,
}

impl View for GettingStarted8 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {
        let colors = [color_make(0xff, 0, 0), color_make(0, 0xff, 0)];
        let opas = [255u8, 0];

        let mut grad = GradDsc::new();
        grad.init_stops(&colors, &opas, &[]).conical(
            lv_pct(50),
            lv_pct(50),
            0,
            180,
            GradExtend::Pad,
        );

        let mut style = StyleBuilder::new();
        style
            .bg_opa(255)
            .bg_grad(grad)
            .border_width(2)
            .radius(12)
            .pad_all(0);
        let style = style.build();
        let obj = Obj::new(container)?;
        obj.size(lv_pct(80), lv_pct(80)).center();
        obj.add_style(&style, Selector::DEFAULT);

                self._obj = Some(obj);
        self._style = Some(style);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(GettingStarted8::default());

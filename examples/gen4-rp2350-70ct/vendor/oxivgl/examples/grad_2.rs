#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Grad 2 — Linear gradient
//!
//! Simplified: interactive drag omitted — `lv_indev_get_point` and direct
//! gradient field mutation not yet wrapped. Bullets shown at initial positions.

use oxivgl::{
    style::{GradDsc, GradExtend, Selector, Style, StyleBuilder, color_make, lv_pct},
    view::{NavAction, View},
    widgets::{Button, Obj, WidgetError},
};

#[derive(Default)]
struct Grad2 {
    _obj: Option<Obj<'static>>,
    _bullet1: Option<Button<'static>>,
    _bullet2: Option<Button<'static>>,
    _bullet1_style: Option<Style>,
    _bullet2_style: Option<Style>,
    _style: Option<Style>, // last — drop after widgets that reference it
}

impl View for Grad2 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {
        let colors = [color_make(0xff, 0, 0), color_make(0, 0xff, 0)];
        let opas = [255u8, 0];

        let mut grad = GradDsc::new();
        grad.init_stops(&colors, &opas, &[])
            .linear(100, 100, 200, 150, GradExtend::Pad);

        let mut style = StyleBuilder::new();
        style
            .bg_opa(255)
            .bg_grad(grad)
            .border_width(2)
            .pad_all(0)
            .radius(12);
        let style = style.build();
        let obj = Obj::new(container)?;
        obj.size(lv_pct(80), lv_pct(80)).center();
        obj.add_style(&style, Selector::DEFAULT);

        // Bullet 1: blue at (100, 100)
        let bullet1_style = Style::new(|s| {
            s.bg_color_hex(0x0000ff).bg_opa(255);
        });
        let bullet1 = Button::new(&obj)?;
        bullet1.size(15, 15).pos(100, 100);
        bullet1.add_style(&bullet1_style, Selector::DEFAULT);

        // Bullet 2: cyan at (200, 150)
        let bullet2_style = Style::new(|s| {
            s.bg_color_hex(0x00ffff).bg_opa(255);
        });
        let bullet2 = Button::new(&obj)?;
        bullet2.size(15, 15).pos(200, 150);
        bullet2.add_style(&bullet2_style, Selector::DEFAULT);

                self._style = Some(style);
        self._bullet1_style = Some(bullet1_style);
        self._bullet2_style = Some(bullet2_style);
        self._obj = Some(obj);
        self._bullet1 = Some(bullet1);
        self._bullet2 = Some(bullet2);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(Grad2::default());

#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: GPL-3.0-only
//! Canvas 5 — Arc

use oxivgl::{
    draw::DrawArcDsc,
    draw_buf::{ColorFormat, DrawBuf},
    style::color_make,
    view::{NavAction, View},
    widgets::{Obj, Align, Canvas, WidgetError},
};

#[derive(Default)]
struct Canvas5 {
    _canvas: Option<Canvas<'static>>,
}

impl View for Canvas5 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {
        let canvas = Canvas::new(
            container,
            DrawBuf::create(50, 50, ColorFormat::ARGB8888).ok_or(WidgetError::LvglNullPointer)?,
        )?;
        canvas.fill_bg(color_make(0xcc, 0xcc, 0xcc), 255);
        canvas.align(Align::Center, 0, 0);
        {
            let mut layer = canvas.init_layer();
            let mut dsc = DrawArcDsc::new();
            dsc.center(25, 25)
                .radius(15)
                .angles(0.0, 220.0)
                .width(10)
                .color(color_make(0xff, 0, 0))
                .opa(255);
            layer.draw_arc(&dsc);
        }
                self._canvas = Some(canvas);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(Canvas5::default());

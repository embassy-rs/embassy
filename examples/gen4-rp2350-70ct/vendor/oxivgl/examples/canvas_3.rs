#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: GPL-3.0-only
//! Canvas 3 — Rectangle with border and outline

use oxivgl::{
    draw::{Area, DrawRectDsc},
    draw_buf::{ColorFormat, DrawBuf},
    style::color_make,
    view::{NavAction, View},
    widgets::{Obj, Align, Canvas, WidgetError},
};

#[derive(Default)]
struct Canvas3 {
    _canvas: Option<Canvas<'static>>,
}

impl View for Canvas3 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {
        let canvas = Canvas::new(
            container,
            DrawBuf::create(70, 70, ColorFormat::ARGB8888).ok_or(WidgetError::LvglNullPointer)?,
        )?;
        canvas.fill_bg(color_make(0xcc, 0xcc, 0xcc), 255);
        canvas.align(Align::Center, 0, 0);
        {
            let mut layer = canvas.init_layer();
            let mut dsc = DrawRectDsc::new();
            dsc.bg_color(color_make(0xff, 0, 0))
                .border_color(color_make(0, 0, 0xff))
                .border_width(4)
                .outline_color(color_make(0, 0xff, 0))
                .outline_width(2)
                .outline_pad(4)
                .radius(5);
            layer.draw_rect(&dsc, Area { x1: 10, y1: 10, x2: 60, y2: 60 });
        }
                self._canvas = Some(canvas);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(Canvas3::default());

#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: GPL-3.0-only
//! Canvas 4 — Text label

use oxivgl::{
    draw::{Area, DrawLabelDscOwned},
    draw_buf::{ColorFormat, DrawBuf},
    style::color_make,
    view::{NavAction, View},
    widgets::{Obj, Align, Canvas, WidgetError},
};

#[derive(Default)]
struct Canvas4 {
    _canvas: Option<Canvas<'static>>,
}

impl View for Canvas4 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {
        let canvas = Canvas::new(
            container,
            DrawBuf::create(80, 30, ColorFormat::ARGB8888).ok_or(WidgetError::LvglNullPointer)?,
        )?;
        canvas.fill_bg(color_make(0xcc, 0xcc, 0xcc), 255);
        canvas.align(Align::Center, 0, 0);
        {
            let mut layer = canvas.init_layer();
            let mut dsc = DrawLabelDscOwned::default_font();
            dsc.set_color(color_make(0xff, 0, 0));
            layer.draw_label(&dsc, Area { x1: 5, y1: 5, x2: 75, y2: 25 }, "Hello");
        }
                self._canvas = Some(canvas);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(Canvas4::default());

#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: GPL-3.0-only
//! Canvas 2 — Transparent pixel bands

use oxivgl::{
    draw_buf::{ColorFormat, DrawBuf},
    style::color_make,
    view::{NavAction, View},
    widgets::{Obj, Align, Canvas, WidgetError},
};

#[derive(Default)]
struct Canvas2 {
    _canvas: Option<Canvas<'static>>,
}

impl View for Canvas2 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {
        let buf = DrawBuf::create(80, 60, ColorFormat::RGB565)
            .ok_or(WidgetError::LvglNullPointer)?;
        let canvas = Canvas::new(container, buf)?;
        canvas.fill_bg(color_make(0, 0, 196), 255);
        for y in 10..20_i32 {
            for x in 0..80_i32 {
                canvas.set_px(x, y, color_make(255, 0, 0), 128);
            }
        }
        for y in 20..30_i32 {
            for x in 0..80_i32 {
                canvas.set_px(x, y, color_make(255, 0, 0), 51);
            }
        }
        for y in 30..40_i32 {
            for x in 0..80_i32 {
                canvas.set_px(x, y, color_make(255, 0, 0), 0);
            }
        }
        canvas.align(Align::Center, 0, 0);
                self._canvas = Some(canvas);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(Canvas2::default());

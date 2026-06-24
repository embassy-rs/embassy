#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: GPL-3.0-only
//! Canvas 1 — Dual canvas + image rotation

use oxivgl::{
    draw::{Area, DrawImageDsc, DrawLabelDscOwned, DrawRectDsc},
    draw_buf::{ColorFormat, DrawBuf},
    style::color_make,
    view::{NavAction, View},
    widgets::{Obj, Align, Canvas, WidgetError},
};

#[derive(Default)]
struct Canvas1 {
    _canvas1: Option<Canvas<'static>>,
    _canvas2: Option<Canvas<'static>>,
}

impl View for Canvas1 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        // Canvas 1: RGB565, gradient rect + label
        let buf1 = DrawBuf::create(100, 70, ColorFormat::RGB565)
            .ok_or(WidgetError::LvglNullPointer)?;
        let canvas1 = Canvas::new(container, buf1)?;
        canvas1.fill_bg(color_make(0xcc, 0xcc, 0xcc), 255);
        canvas1.align(Align::TopMid, 0, 5);
        {
            let mut layer = canvas1.init_layer();
            let mut rdc = DrawRectDsc::new();
            rdc.bg_color(color_make(0xff, 0x00, 0x00))
                .border_width(2)
                .border_color(color_make(0, 0, 0))
                .radius(5);
            layer.draw_rect(&rdc, Area { x1: 5, y1: 5, x2: 94, y2: 64 });
            let mut ldc = DrawLabelDscOwned::default_font();
            ldc.set_color(color_make(0xff, 0xa5, 0x00));
            layer.draw_label(&ldc, Area { x1: 30, y1: 27, x2: 90, y2: 43 }, "Canvas 1");
        }

        // Canvas 2: RGB565, rotated snapshot of canvas1
        let buf2 = DrawBuf::create(100, 70, ColorFormat::RGB565)
            .ok_or(WidgetError::LvglNullPointer)?;
        let canvas2 = Canvas::new(container, buf2)?;
        canvas2.fill_bg(color_make(0x80, 0x80, 0x80), 255);
        canvas2.align(Align::BottomMid, 0, -5);
        {
            let img = canvas1.draw_buf().image_dsc();
            let mut layer = canvas2.init_layer();
            let mut dsc = DrawImageDsc::from_image_dsc(&img);
            dsc.rotation(120).pivot(50, 35).opa(255);
            layer.draw_image(&dsc, Area { x1: 0, y1: 0, x2: 99, y2: 69 });
        }

        self._canvas1 = Some(canvas1);
        self._canvas2 = Some(canvas2);
        Ok(())
    }

    fn register_events_on(&mut self, _container: &Obj<'static>) {}
    fn on_event(&mut self, _: &oxivgl::event::Event) -> NavAction { NavAction::None }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(Canvas1::default());

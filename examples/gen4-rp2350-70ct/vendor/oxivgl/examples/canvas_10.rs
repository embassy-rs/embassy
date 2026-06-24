#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: GPL-3.0-only
//! Canvas 10 — Wavy text animation

use oxivgl::{
    draw_buf::{ColorFormat, DrawBuf},
    style::color_make,
    view::{NavAction, View},
    widgets::{Obj, Align, Canvas, WidgetError},
};

#[derive(Default)]
struct Canvas10 {
    canvas: Option<Canvas<'static>>,
    counter: i32,
}

impl View for Canvas10 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {
        let canvas = Canvas::new(
            container,
            DrawBuf::create(160, 100, ColorFormat::RGB565).ok_or(WidgetError::LvglNullPointer)?,
        )?;
        canvas.fill_bg(color_make(0xff, 0xff, 0xff), 255);
        canvas.align(Align::Center, 0, 0);
        self.canvas = Some(canvas);
        self.counter = 0;
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        use oxivgl::draw::DrawLetterDsc;
        use oxivgl::math::trigo_sin;
        use oxivgl::style::color_hsv;
        const TXT: &[u8] = b"Hello wavy world!";
        if let Some(ref canvas) = self.canvas {
            canvas.fill_bg(color_make(0xff, 0xff, 0xff), 255);
            {
                let mut layer = canvas.init_layer();
                for (i, &ch) in TXT.iter().enumerate() {
                    let angle = i as i32 * 10;
                    let x = i as i32 * 8 + 5;
                    let y = trigo_sin(((angle + self.counter / 2) * 5) as i32) * 20 / 32767 + 50;
                    let mut dsc = DrawLetterDsc::new();
                    dsc.unicode(ch as u32)
                        .color(color_hsv(((i as u16 * 15) % 360) as u16, 100, 100))
                        .rotation(0);
                    layer.draw_letter(&dsc, x, y);
                }
            }
        }
        self.counter += 1;
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(Canvas10::default());

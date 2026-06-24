#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Canvas 12 — Curved text along a circular path
//!
//! "HELLO LVGL 9.5" rendered character-by-character along a circular arc,
//! each letter rotated tangent to the circle. The text orbits continuously.

use oxivgl::{
    draw::DrawLetterDsc,
    draw_buf::{ColorFormat, DrawBuf},
    fonts,
    math::{trigo_cos, trigo_sin},
    style::{color_hsv, color_make},
    view::{NavAction, View},
    widgets::{Obj, Align, Canvas, WidgetError},
};

/// Canvas size — 160×160 fits ESP32's 50KB heap (160×160×2 = 51200 bytes).
/// Use 240 on host for higher quality screenshots.
#[cfg(target_arch = "xtensa")]
const SIZE: i32 = 140;
#[cfg(not(target_arch = "xtensa"))]
const SIZE: i32 = 240;
const TXT: &[u8] = b"HELLO LVGL 9.5";
const RADIUS: i32 = 80;
const SPACING_DEG: i32 = 22; // degrees between characters

#[derive(Default)]
struct Canvas12 {
    canvas: Option<Canvas<'static>>,
    tick: i32,
}

impl View for Canvas12 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {
        let canvas = Canvas::new(
            container,
            DrawBuf::create(SIZE as u32, SIZE as u32, ColorFormat::RGB565)
                .ok_or(WidgetError::LvglNullPointer)?,
        )?;
        canvas.fill_bg(color_make(20, 20, 40), 255);
        canvas.align(Align::Center, 0, 0);
                self.canvas = Some(canvas);
        self.tick = 0;
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        let cx = SIZE / 2;
        let cy = SIZE / 2;
        let n = TXT.len() as i32;

        let canvas = self.canvas.as_ref().unwrap();
        canvas.fill_bg(color_make(20, 20, 40), 255);
        {
            let mut layer = canvas.init_layer();
            for i in 0..n {
                let ch = TXT[i as usize];
                // angle in degrees for this character
                let angle_deg = self.tick + i * SPACING_DEG;
                // Position on circle using LVGL trig (returns [-32767..32767])
                let cos_val = trigo_cos(angle_deg);
                let sin_val = trigo_sin(angle_deg);
                let x = cx + (RADIUS * cos_val) / 32767 - 5;
                let y = cy + (RADIUS * sin_val) / 32767 - 8;
                // Rainbow color per character
                let hue = ((i * 25 + self.tick as i32) % 360) as u16;
                let mut dsc = DrawLetterDsc::new();
                dsc.unicode(ch as u32)
                    .font(fonts::MONTSERRAT_20)
                    .color(color_hsv(hue, 100, 100))
                    .rotation(0);
                layer.draw_letter(&dsc, x, y);
            }
        }
        self.tick = (self.tick + 1) % 360;
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(Canvas12::default());

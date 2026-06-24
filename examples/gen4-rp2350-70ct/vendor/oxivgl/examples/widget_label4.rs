#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Label 4 — Gradient text via canvas bitmap mask
//!
//! Renders text into an L8 canvas mask, then applies it to a gradient
//! rectangle so the text appears with a horizontal color gradient.
//!
//! Uses an ARGB8888 intermediate canvas because LVGL's SW renderer does
//! not support text rendering directly onto L8 surfaces. The luminance
//! is copied pixel-by-pixel into the L8 mask.

extern crate alloc;

use oxivgl::{
    draw::{Area, DrawLabelDscOwned},
    draw_buf::{ColorFormat, DrawBuf},
    fonts,
    style::{GradDir, Selector, Style, color_black, color_make, color_white},
    view::{NavAction, View},
    widgets::{Canvas, Obj, TextAlign, WidgetError},
};

const MASK_WIDTH: u32 = 150;
const MASK_HEIGHT: u32 = 60;

#[derive(Default)]
struct WidgetLabel4 {
    _mask: Option<DrawBuf>,
    _grad: Option<Obj<'static>>,
}

impl View for WidgetLabel4 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        // Step 1: Render text on an ARGB8888 scratch canvas.
        let scratch_buf = DrawBuf::create(MASK_WIDTH, MASK_HEIGHT, ColorFormat::ARGB8888)
            .ok_or(WidgetError::LvglNullPointer)?;
        let scratch = Canvas::new(container, scratch_buf)?;
        scratch.fill_bg(color_black(), 255);
        {
            let mut layer = scratch.init_layer();
            let mut ldc = DrawLabelDscOwned::default_font();
            ldc.set_color(color_white())
                .set_align(TextAlign::Center)
                .set_font(fonts::MONTSERRAT_24);
            layer.draw_label(
                &ldc,
                Area { x1: 0, y1: 0, x2: MASK_WIDTH as i32 - 1, y2: MASK_HEIGHT as i32 - 1 },
                "Text with gradient",
            );
        }

        // Step 2: Create L8 mask and copy luminance from the ARGB8888 scratch.
        let mask = DrawBuf::create(MASK_WIDTH, MASK_HEIGHT, ColorFormat::L8)
            .ok_or(WidgetError::LvglNullPointer)?;
        Canvas::draw_to_buf(&mask, |mask_canvas| {
            mask_canvas.fill_bg(color_black(), 255);
            for y in 0..MASK_HEIGHT as i32 {
                for x in 0..MASK_WIDTH as i32 {
                    let px = scratch.get_px(x, y);
                    // Extract max channel as luminance from the ARGB8888 pixel.
                    let luma = {
                        let r = px.red;
                        let g = px.green;
                        let b = px.blue;
                        r.max(g).max(b)
                    };
                    if luma > 0 {
                        mask_canvas.set_px(x, y, color_make(luma, luma, luma), 255);
                    }
                }
            }
        });

        // Clean up the scratch canvas (drop it).
        drop(scratch);

        // Step 3: Create a gradient object and apply the mask.
        let grad = Obj::new(container)?;
        grad.size(MASK_WIDTH as i32, MASK_HEIGHT as i32);
        grad.center();
        let grad_style = Style::new(|s| {
            s.border_width(0)
                .bg_color(color_make(0xff, 0, 0))
                .bg_grad_color(color_make(0, 0, 0xff))
                .bg_grad_dir(GradDir::Hor);
        });
        grad.add_style(&grad_style, Selector::DEFAULT);
        // SAFETY: mask is stored in Self._mask (Box) and outlives the grad widget.
        unsafe { grad.style_bitmap_mask_src(&mask, Selector::DEFAULT) };

        self._mask = Some(mask);
        self._grad = Some(grad);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(WidgetLabel4::default());

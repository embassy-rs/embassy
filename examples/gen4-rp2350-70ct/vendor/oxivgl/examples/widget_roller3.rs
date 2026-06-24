#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Roller 3 — Roller with fade mask
//!
//! Applies an L8 gradient mask to a roller so that the top and bottom
//! rows fade to black, producing a 3D cylinder effect.

extern crate alloc;

use oxivgl::{
    draw::{Area, DrawRectDsc},
    draw_buf::{ColorFormat, DrawBuf},
    style::{GradDir, Selector, StyleBuilder, color_black, color_white},
    view::{NavAction, View},
    widgets::{Obj, Canvas, Part, Roller, RollerMode, WidgetError},
};

const MASK_W: u32 = 130;
const MASK_H: u32 = 150;

/// Generate a vertical fade mask: top half fades black→white,
/// bottom half fades white→black, with a 20-pixel opaque band in the middle.
fn generate_mask(buf: &DrawBuf) {
    let h = buf.height();
    Canvas::draw_to_buf(buf, |canvas| {
        canvas.fill_bg(color_white(), 0); // LV_OPA_TRANSP

        let mut layer = canvas.init_layer();

        // Top gradient: black → white
        let mut rdc = DrawRectDsc::new();
        rdc.bg_grad_dir(GradDir::Ver)
            .bg_grad_stop(0, color_black(), 0, 255)
            .bg_grad_stop(1, color_white(), 255, 255);
        layer.draw_rect(
            &rdc,
            Area { x1: 0, y1: 0, x2: buf.width() - 1, y2: h / 2 - 10 },
        );

        // Bottom gradient: white → black
        rdc.bg_grad_stop(0, color_white(), 0, 255)
            .bg_grad_stop(1, color_black(), 255, 255);
        layer.draw_rect(
            &rdc,
            Area { x1: 0, y1: h / 2 + 10, x2: buf.width() - 1, y2: h - 1 },
        );
    });
}

#[derive(Default)]
struct WidgetRoller3 {
    _mask: Option<alloc::boxed::Box<DrawBuf>>,
    _roller: Option<Roller<'static>>,
}

impl View for WidgetRoller3 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {
        let mut cont_sb = StyleBuilder::new();
        cont_sb.bg_color_hex(0x607D8B); // LV_PALETTE_BLUE_GREY
        let cont_style = cont_sb.build();
        container.add_style(&cont_style, Selector::DEFAULT);

        let mut sb = StyleBuilder::new();
        sb.bg_color(color_black())
            .text_color(color_white())
            .border_width(0)
            .radius(0);
        let style = sb.build();

        // Selected-row background opacity (shared style for Part::Selected).
        let mut sel_sb = StyleBuilder::new();
        sel_sb.bg_opa(50);
        let sel_style = sel_sb.build();

        let roller = Roller::new(container)?;
        roller.add_style(&style, Selector::DEFAULT);
        roller.add_style(&sel_style, Part::Selected);
        roller.set_options(
            "January\n\
             February\n\
             March\n\
             April\n\
             May\n\
             June\n\
             July\n\
             August\n\
             September\n\
             October\n\
             November\n\
             December",
            RollerMode::Normal,
        );
        roller.center();
        roller.set_visible_row_count(4);

        let mask = DrawBuf::create(MASK_W, MASK_H, ColorFormat::L8)
            .ok_or(WidgetError::LvglNullPointer)?;
        generate_mask(&mask);
        let mask = alloc::boxed::Box::new(mask);

        // SAFETY: mask is stored in Self._mask (Box) and outlives the roller.
        unsafe { roller.style_bitmap_mask_src(&mask, Selector::DEFAULT) };

                self._mask = Some(mask);
        self._roller = Some(roller);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(WidgetRoller3::default());

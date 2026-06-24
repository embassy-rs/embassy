#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Roller 2 — Styled rollers with alignments
//!
//! Three rollers: left-aligned on green gradient, center-aligned (default),
//! right-aligned. All share a custom selected-row style with larger font.

extern crate alloc;

use oxivgl::{
    fonts::MONTSERRAT_20,
    style::{GradDir, Selector, Style, StyleBuilder},
    view::{NavAction, View},
    widgets::{Obj, Align, Part, Roller, RollerMode, TextAlign, WidgetError},
};

#[derive(Default)]
struct WidgetRoller2 {
    _r1: Option<Roller<'static>>,
    _r2: Option<Roller<'static>>,
    _r3: Option<Roller<'static>>,
    _style_sel: Option<Style>,
    _style_green: Option<Style>,
}

const OPTS: &str = "1\n2\n3\n4\n5\n6\n7\n8\n9\n10";

impl View for WidgetRoller2 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        // Selected-row style: larger font, pink bg, red border
        let mut sb = StyleBuilder::new();
        sb.text_font(MONTSERRAT_20)
            .bg_color_hex(0xFF8888)
            .border_width(2)
            .border_color_hex(0xFF0000);
        let style_sel = sb.build();

        // Green vertical gradient style for left roller
        let mut sb_green = StyleBuilder::new();
        sb_green
            .bg_color_hex(0x00FF00)
            .bg_grad_color_hex(0xAAFFAA)
            .bg_grad_dir(GradDir::Ver)
            .text_align(TextAlign::Left);
        let style_green = sb_green.build();

        // Left roller: left-aligned text, green gradient, 2 visible rows
        let r1 = Roller::new(container)?;
        r1.set_options(OPTS, RollerMode::Normal);
        r1.set_visible_row_count(2);
        r1.width(100);
        r1.add_style(&style_sel, Selector::from(Part::Selected));
        r1.add_style(&style_green, Selector::DEFAULT);
        r1.align(Align::LeftMid, 10, 0);
        r1.set_selected(2, false);

        // Center roller: default alignment, 3 visible rows
        let r2 = Roller::new(container)?;
        r2.set_options(OPTS, RollerMode::Normal);
        r2.set_visible_row_count(3);
        r2.add_style(&style_sel, Selector::from(Part::Selected));
        r2.align(Align::Center, 0, 0);
        r2.set_selected(5, false);

        // Right roller: right-aligned text, 4 visible rows
        let r3 = Roller::new(container)?;
        r3.set_options(OPTS, RollerMode::Normal);
        r3.set_visible_row_count(4);
        r3.width(80);
        r3.add_style(&style_sel, Selector::from(Part::Selected));
        let style_align_right = Style::new(|s| {
            s.text_align(TextAlign::Right);
        });
        r3.add_style(&style_align_right, Selector::DEFAULT);
        r3.align(Align::RightMid, -10, 0);
        r3.set_selected(8, false);

                self._r1 = Some(r1);
        self._r2 = Some(r2);
        self._r3 = Some(r3);
        self._style_sel = Some(style_sel);
        self._style_green = Some(style_green);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(WidgetRoller2::default());

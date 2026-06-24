#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Bar 2 — Styled progress bar
//!
//! Custom blue-themed bar with rounded corners, padding, and animated fill.

use oxivgl::{
    style::{palette_main, Palette, Selector, Style, StyleBuilder},
    view::{NavAction, View},
    widgets::{Obj, Bar, Part, WidgetError},
};

#[derive(Default)]
struct WidgetBar2 {
    _bar: Option<Bar<'static>>,
    _style_bg: Option<Style>,
    _style_indic: Option<Style>,
}

impl View for WidgetBar2 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let mut style_bg = StyleBuilder::new();
        style_bg
            .border_color(palette_main(Palette::Blue))
            .border_width(2)
            .pad_all(6)
            .radius(6)
            .anim_duration(1000);
        let style_bg = style_bg.build();

        let mut style_indic = StyleBuilder::new();
        style_indic
            .bg_opa(255)
            .bg_color(palette_main(Palette::Blue))
            .radius(3);
        let style_indic = style_indic.build();

        let bar = Bar::new(container)?;
        bar.remove_style_all();
        bar.add_style(&style_bg, Selector::DEFAULT);
        bar.add_style(&style_indic, Part::Indicator);
        bar.size(200, 20).center();
        bar.set_range_raw(0, 100);
        bar.set_value_raw(100, true);

                self._bar = Some(bar);
        self._style_bg = Some(style_bg);
        self._style_indic = Some(style_indic);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(WidgetBar2::default());

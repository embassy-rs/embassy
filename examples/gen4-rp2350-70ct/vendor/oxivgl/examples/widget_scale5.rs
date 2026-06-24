#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Scale 5 — Horizontal scale with sections
//!
//! Horizontal scale with colored sections for low (blue) and high (red) ranges.

use oxivgl::{
    style::{lv_pct, palette_darken, palette_lighten, Palette, Style, StyleBuilder},
    view::{NavAction, View},
    widgets::{Obj, Part, Scale, ScaleMode, WidgetError},
};

struct WidgetScale5 {
    _scale: Option<Scale<'static>>,
    _styles: [Style; 6],
}

impl WidgetScale5 {
    fn new() -> Self {
        Self {
            _scale: None,
            _styles: core::array::from_fn(|_| StyleBuilder::new().build()),
        }
    }
}

impl View for WidgetScale5 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {
        let scale = Scale::new(container)?;
        scale.size(lv_pct(80), 100).center();
        scale
            .set_mode(ScaleMode::HorizontalBottom)
            .set_label_show(true)
            .set_total_tick_count(31)
            .set_major_tick_every(5)
            .set_tick_length(Part::Items, 5)
            .set_tick_length(Part::Indicator, 10)
            .set_range(0, 100);

        // Base major tick style (dark gray)
        let mut sb = StyleBuilder::new();
        sb.line_color(palette_darken(Palette::Grey, 3)).line_width(2);
        let indicator_style = sb.build();
        scale.add_style(&indicator_style, Part::Indicator);

        // Base minor tick style (gray)
        let mut sb = StyleBuilder::new();
        sb.line_color(palette_lighten(Palette::Grey, 1)).line_width(1);
        let items_style = sb.build();
        scale.add_style(&items_style, Part::Items);

        // Red section: 75–100
        let mut sb = StyleBuilder::new();
        sb.text_color(palette_darken(Palette::Red, 3))
            .line_color(palette_darken(Palette::Red, 3))
            .line_width(5);
        let red_label = sb.build();

        let mut sb = StyleBuilder::new();
        sb.line_color(palette_lighten(Palette::Red, 2)).line_width(4);
        let red_ticks = sb.build();

        let section = scale.add_section();
        section
            .set_range(75, 100)
            .set_indicator_style(&red_label)
            .set_items_style(&red_ticks);

        // Blue section: 0–25
        let mut sb = StyleBuilder::new();
        sb.text_color(palette_darken(Palette::Blue, 3))
            .line_color(palette_darken(Palette::Blue, 3))
            .line_width(5);
        let blue_label = sb.build();

        let mut sb = StyleBuilder::new();
        sb.line_color(palette_lighten(Palette::Blue, 2))
            .line_width(4);
        let blue_ticks = sb.build();

        let section = scale.add_section();
        section
            .set_range(0, 25)
            .set_indicator_style(&blue_label)
            .set_items_style(&blue_ticks);

        self._scale = Some(scale);
        self._styles = [
            indicator_style,
            items_style,
            red_label,
            red_ticks,
            blue_label,
            blue_ticks,
        ];
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(WidgetScale5::new());

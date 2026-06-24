#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Scale 4 — Round scale with sections
//!
//! Round outer scale with custom labels and colored sections for low/high ranges.

use oxivgl::{
    scale_labels,
    style::{palette_darken, palette_lighten, Palette, Selector, Style, StyleBuilder},
    view::{NavAction, View},
    widgets::{Obj, Part, Scale, ScaleLabels, ScaleMode, WidgetError},
};

/// Custom labels for ticks 1–10 (null-terminated).
static CUSTOM_LABELS: &ScaleLabels =
    scale_labels!(c"1", c"2", c"3", c"4", c"5", c"6", c"7", c"8", c"9", c"10");

struct WidgetScale4 {
    _scale: Option<Scale<'static>>,
    _styles: [Style; 9],
}

impl WidgetScale4 {
    fn new() -> Self {
        Self {
            _scale: None,
            _styles: core::array::from_fn(|_| StyleBuilder::new().build()),
        }
    }
}

impl View for WidgetScale4 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {
        let scale = Scale::new(container)?;
        scale.size(200, 200).center();
        scale
            .set_mode(ScaleMode::RoundOuter)
            .set_total_tick_count(11)
            .set_major_tick_every(1)
            .set_label_show(true)
            .set_tick_length(Part::Items, 5)
            .set_tick_length(Part::Indicator, 10)
            .set_range(1, 10)
            .set_text_src(CUSTOM_LABELS);

        // Base major tick style (blue)
        let mut sb = StyleBuilder::new();
        sb.text_color(palette_darken(Palette::Blue, 3))
            .line_color(palette_darken(Palette::Blue, 3))
            .line_width(2);
        let indicator_style = sb.build();
        scale.add_style(&indicator_style, Part::Indicator);

        // Base minor tick style (light blue)
        let mut sb = StyleBuilder::new();
        sb.line_color(palette_lighten(Palette::Blue, 2)).line_width(2);
        let items_style = sb.build();
        scale.add_style(&items_style, Part::Items);

        // Base main arc style (blue)
        let mut sb = StyleBuilder::new();
        sb.arc_color(palette_darken(Palette::Blue, 3)).arc_width(2);
        let main_style = sb.build();
        scale.add_style(&main_style, Selector::DEFAULT);

        // Red section: 8–10
        let mut sb = StyleBuilder::new();
        sb.text_color(palette_darken(Palette::Red, 3))
            .line_color(palette_darken(Palette::Red, 3))
            .line_width(5);
        let red_label = sb.build();

        let mut sb = StyleBuilder::new();
        sb.line_color(palette_lighten(Palette::Red, 2)).line_width(4);
        let red_ticks = sb.build();

        let mut sb = StyleBuilder::new();
        sb.arc_color(palette_darken(Palette::Red, 3)).arc_width(4);
        let red_main = sb.build();

        let section = scale.add_section();
        section
            .set_range(8, 10)
            .set_indicator_style(&red_label)
            .set_items_style(&red_ticks)
            .set_main_style(&red_main);

        // Green section: 1–3
        let mut sb = StyleBuilder::new();
        sb.text_color(palette_darken(Palette::Green, 3))
            .line_color(palette_darken(Palette::Green, 3))
            .line_width(5);
        let green_label = sb.build();

        let mut sb = StyleBuilder::new();
        sb.line_color(palette_lighten(Palette::Green, 2))
            .line_width(4);
        let green_ticks = sb.build();

        let mut sb = StyleBuilder::new();
        sb.arc_color(palette_darken(Palette::Green, 3)).arc_width(4);
        let green_main = sb.build();

        let section = scale.add_section();
        section
            .set_range(1, 3)
            .set_indicator_style(&green_label)
            .set_items_style(&green_ticks)
            .set_main_style(&green_main);

        self._scale = Some(scale);
        self._styles = [
            indicator_style,
            items_style,
            main_style,
            red_label,
            red_ticks,
            red_main,
            green_label,
            green_ticks,
            green_main,
        ];
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(WidgetScale4::new());

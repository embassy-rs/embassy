#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Scale 8 — Round scale with rotated labels
//!
//! Round inner scale with label rotation matching tick angles and a needle.

use oxivgl::{
    style::{lv_pct, palette_lighten, Palette, Selector, StyleBuilder},
    view::{NavAction, View},
    widgets::{Obj, Align, Line, Part, Scale, ScaleMode, WidgetError, RADIUS_MAX,
        SCALE_LABEL_ROTATE_KEEP_UPRIGHT, SCALE_LABEL_ROTATE_MATCH_TICKS,
    },
};

#[derive(Default)]
struct WidgetScale8 {
    _scale: Option<Scale<'static>>,
    _needle: Option<Line<'static>>,
}

impl View for WidgetScale8 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let scale = Scale::new(container)?;
        scale.size(150, 150);
        scale.set_mode(ScaleMode::RoundInner);

        // Red-tinted background, fully round
        let mut bg_sb = StyleBuilder::new();
        bg_sb
            .bg_opa(255)
            .bg_color(palette_lighten(Palette::Red, 5))
            .radius(RADIUS_MAX as i16);
        let bg_style = bg_sb.build();
        scale.add_style(&bg_style, Selector::DEFAULT);

        scale.align(Align::LeftMid, lv_pct(2), 0);

        // Major ticks + labels: rotate to match tick angles, keep upright
        scale.style_transform_rotation(
            SCALE_LABEL_ROTATE_MATCH_TICKS | SCALE_LABEL_ROTATE_KEEP_UPRIGHT,
            Part::Indicator,
        );
        scale.style_translate_x(10, Part::Indicator);
        scale.set_tick_length(Part::Indicator, 15);
        let mut indicator_sb = StyleBuilder::new();
        indicator_sb.radial_offset(10);
        let indicator_style = indicator_sb.build();
        scale.add_style(&indicator_style, Part::Indicator);

        // Minor ticks
        scale.set_tick_length(Part::Items, 10);
        let mut items_sb = StyleBuilder::new();
        items_sb.radial_offset(5).line_opa(128);
        let items_style = items_sb.build();
        scale.add_style(&items_style, Part::Items);

        scale
            .set_label_show(true)
            .set_total_tick_count(31)
            .set_major_tick_every(5)
            .set_range(10, 40)
            .set_angle_range(270)
            .set_rotation(135);

        // Needle line
        let needle = Line::new(&scale)?;
        let mut sb = StyleBuilder::new();
        sb.line_width(3).line_rounded(true);
        let needle_style = sb.build();
        needle.add_style(&needle_style, Selector::DEFAULT);

        scale.set_line_needle_value(&needle, 60, 33);

                self._scale = Some(scale);
        self._needle = Some(needle);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(WidgetScale8::default());

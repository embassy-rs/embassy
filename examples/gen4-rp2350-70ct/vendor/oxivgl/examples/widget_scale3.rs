#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Scale 3 — Round scale with needle
//!
//! Round gauge with animated line needle sweeping 0–100.

use oxivgl::{
    style::{color_make, Selector, StyleBuilder},
    view::{NavAction, View},
    enums::Opa,
    widgets::{Obj, Line, Part, Scale, ScaleMode, WidgetError, RADIUS_MAX},
};

#[derive(Default)]
struct WidgetScale3 {
    scale: Option<Scale<'static>>,
    needle: Option<Line<'static>>,
    value: i32,
    ascending: bool,
}

impl View for WidgetScale3 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let scale = Scale::new(container)?;
        scale.size(200, 200).center();
        scale
            .set_mode(ScaleMode::RoundInner)
            .set_rotation(135)
            .set_angle_range(270)
            .set_range(0, 100)
            .set_total_tick_count(21)
            .set_major_tick_every(5)
            .set_label_show(true)
            .set_tick_length(Part::Items, 5)
            .set_tick_length(Part::Indicator, 10);

        // Transparent bg, no border, round corners
        let mut sb = StyleBuilder::new();
        sb.bg_opa(Opa::TRANSP.0).border_width(0).radius(RADIUS_MAX as i16);
        let scale_style = sb.build();
        scale.add_style(&scale_style, Selector::DEFAULT);

        // Create needle line as child of scale
        let needle = Line::new(&scale)?;
        let mut sb = StyleBuilder::new();
        sb.line_width(3)
            .line_color(color_make(0x33, 0x33, 0x33))
            .line_rounded(true);
        let needle_style = sb.build();
        needle.add_style(&needle_style, Selector::DEFAULT);

        // Initial needle position
        scale.set_line_needle_value(&needle, 80, 50);

        self.scale = Some(scale);
        self.needle = Some(needle);
        self.value = 50;
        self.ascending = true;
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        if self.ascending {
            self.value += 1;
            if self.value >= 100 {
                self.ascending = false;
            }
        } else {
            self.value -= 1;
            if self.value <= 0 {
                self.ascending = true;
            }
        }
        if let (Some(scale), Some(needle)) = (&self.scale, &self.needle) {
            scale.set_line_needle_value(needle, 80, self.value);
        }
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(WidgetScale3::default());

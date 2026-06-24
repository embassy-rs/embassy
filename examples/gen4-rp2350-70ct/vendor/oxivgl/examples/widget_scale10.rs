#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Scale 10 — Heart rate gauge
//!
//! Round gauge with timer-driven needle oscillating between 80–180 BPM.

use core::fmt::Write;

use oxivgl::{
    style::{color_make, Selector, StyleBuilder},
    view::{NavAction, View},
    enums::Opa,
    timer::Timer,
    widgets::{Obj, Align, Label, Line, Part, Scale, ScaleMode, WidgetError, RADIUS_MAX},
};

#[derive(Default)]
struct WidgetScale10 {
    scale: Option<Scale<'static>>,
    needle: Option<Line<'static>>,
    label: Option<Label<'static>>,
    timer: Option<Timer>,
    hr: i32,
    ascending: bool,
}

impl View for WidgetScale10 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let scale = Scale::new(container)?;
        scale.size(200, 200).center();
        scale
            .set_mode(ScaleMode::RoundInner)
            .set_rotation(135)
            .set_angle_range(270)
            .set_range(60, 200)
            .set_total_tick_count(15)
            .set_major_tick_every(3)
            .set_label_show(true)
            .set_tick_length(Part::Items, 5)
            .set_tick_length(Part::Indicator, 10);

        // Transparent bg, no border, round corners
        let mut sb = StyleBuilder::new();
        sb.bg_opa(Opa::TRANSP.0).border_width(0).radius(RADIUS_MAX as i16);
        let scale_style = sb.build();
        scale.add_style(&scale_style, Selector::DEFAULT);

        // Needle line
        let needle = Line::new(&scale)?;
        let mut sb = StyleBuilder::new();
        sb.line_width(3)
            .line_color(color_make(0xE0, 0x30, 0x30))
            .line_rounded(true);
        let needle_style = sb.build();
        needle.add_style(&needle_style, Selector::DEFAULT);

        scale.set_line_needle_value(&needle, 80, 98);

        // Center label showing HR value
        let label = Label::new(&scale)?;
        label.text("98 BPM");
        label.align(Align::Center, 0, 40);

        let timer = Timer::new(100)?;

                self.scale = Some(scale);
        self.needle = Some(needle);
        self.label = Some(label);
        self.timer = Some(timer);
        self.hr = 98;
        self.ascending = true;
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        let triggered = self.timer.as_ref().map_or(false, |t| t.triggered());
        if triggered {
            if self.ascending {
                self.hr += 1;
                if self.hr >= 180 {
                    self.ascending = false;
                }
            } else {
                self.hr -= 1;
                if self.hr <= 80 {
                    self.ascending = true;
                }
            }

            if let (Some(scale), Some(needle)) = (&self.scale, &self.needle) {
                scale.set_line_needle_value(needle, 80, self.hr);
            }

            let mut buf = heapless::String::<16>::new();
            let _ = write!(buf, "{} BPM", self.hr);
            if let Some(ref label) = self.label {
                label.text(&buf);
            }
        }
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(WidgetScale10::default());

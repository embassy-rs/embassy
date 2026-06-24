#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Scale 6 — Clock with timer-driven needles
//!
//! Round scale clock face with minute and hour hands updated by a 250 ms Timer.

use oxivgl::{
    scale_labels,
    style::{color_black, color_white, palette_main, Palette, Selector, StyleBuilder},
    timer::Timer,
    view::{NavAction, View},
    widgets::{Obj, Line, Part, Scale, ScaleLabels, ScaleMode, WidgetError, RADIUS_MAX,
    },
};

static HOUR_LABELS: &ScaleLabels =
    scale_labels!(c"12", c"1", c"2", c"3", c"4", c"5", c"6", c"7", c"8", c"9", c"10", c"11");

#[derive(Default)]
struct WidgetScale6 {
    scale: Option<Scale<'static>>,
    minute_hand: Option<Line<'static>>,
    hour_hand: Option<Line<'static>>,
    timer: Option<Timer>,
    minute: i32,
    hour: i32,
}

impl View for WidgetScale6 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let scale = Scale::new(container)?;
        scale.size(150, 150).center();
        scale
            .set_mode(ScaleMode::RoundInner)
            .set_total_tick_count(61)
            .set_major_tick_every(5)
            .set_label_show(true)
            .set_text_src(HOUR_LABELS)
            .set_range(0, 60)
            .set_angle_range(360)
            .set_rotation(270);

        // Dark background with clipped corners
        let mut sb = StyleBuilder::new();
        sb.bg_opa(153).bg_color(color_black()).radius(RADIUS_MAX as i16).clip_corner(true);
        let bg_style = sb.build();
        scale.add_style(&bg_style, Selector::DEFAULT);

        // Indicator style: yellow labels + major tick lines
        let mut sb = StyleBuilder::new();
        sb.text_color(palette_main(Palette::Yellow))
            .line_color(palette_main(Palette::Yellow))
            .length(8)
            .line_width(2);
        let indicator_style = sb.build();
        scale.add_style(&indicator_style, Selector::from(Part::Indicator));

        // Minor tick style: yellow
        let mut sb = StyleBuilder::new();
        sb.line_color(palette_main(Palette::Yellow))
            .length(6)
            .line_width(2);
        let items_style = sb.build();
        scale.add_style(&items_style, Selector::from(Part::Items));

        // Main arc: dark
        let mut sb = StyleBuilder::new();
        sb.arc_color(color_black()).arc_width(5);
        let main_style = sb.build();
        scale.add_style(&main_style, Selector::from(Part::Main));

        // Minute hand — white, width 3
        let minute_hand = Line::new(&scale)?;
        let mut sb = StyleBuilder::new();
        sb.line_width(3).line_color(color_white()).line_rounded(true);
        let minute_style = sb.build();
        minute_hand.add_style(&minute_style, Selector::DEFAULT);

        // Hour hand — red, width 5
        let hour_hand = Line::new(&scale)?;
        let mut sb = StyleBuilder::new();
        sb.line_width(5)
            .line_color(palette_main(Palette::Red))
            .line_rounded(true);
        let hour_style = sb.build();
        hour_hand.add_style(&hour_style, Selector::DEFAULT);

        let hour = 11;
        let minute = 5;

        // Set initial needle positions
        scale.set_line_needle_value(&minute_hand, 60, minute);
        scale.set_line_needle_value(&hour_hand, 40, hour * 5 + minute / 12);

        let timer = Timer::new(250)?;
        timer.ready();

                self.scale = Some(scale);
        self.minute_hand = Some(minute_hand);
        self.hour_hand = Some(hour_hand);
        self.timer = Some(timer);
        self.minute = minute;
        self.hour = hour;
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        if let Some(ref timer) = self.timer {
            if timer.triggered() {
                self.minute += 1;
                if self.minute > 59 {
                    self.minute = 0;
                    self.hour += 1;
                    if self.hour > 11 {
                        self.hour = 0;
                    }
                }
                if let (Some(scale), Some(minute_hand), Some(hour_hand)) =
                    (&self.scale, &self.minute_hand, &self.hour_hand)
                {
                    scale.set_line_needle_value(minute_hand, 60, self.minute);
                    scale.set_line_needle_value(hour_hand, 40, self.hour * 5 + self.minute / 12);
                }
            }
        }
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(WidgetScale6::default());

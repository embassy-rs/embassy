#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget ArcLabel 1 — Text curved along circular arcs
//!
//! Three ArcLabel widgets with different radius, angle, and direction settings.

use oxivgl::{
    style::{Selector, Style},
    view::{NavAction, View},
    widgets::{Obj, Align, ArcLabel, ArcLabelDir, WidgetError},
};

#[derive(Default)]
struct WidgetArclabel1 {
    _al1: Option<ArcLabel<'static>>,
    _al2: Option<ArcLabel<'static>>,
    _al3: Option<ArcLabel<'static>>,
}

impl View for WidgetArclabel1 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {
        let bg_style = Style::new(|s| {
            s.bg_color_hex(0xffffff).bg_opa(255);
        });
        container.add_style(&bg_style, Selector::DEFAULT);

        // Large clockwise arc across the top
        let al1_style = Style::new(|s| {
            s.text_color_hex(0x2196F3);
        });
        let al1 = ArcLabel::new(container)?;
        al1.set_text_static(c"Hello curved world!");
        al1.set_radius(90);
        al1.set_angle_start(40.0);
        al1.set_angle_size(100.0);
        al1.set_dir(ArcLabelDir::Clockwise);
        al1.size(200, 200);
        al1.align(Align::Center, 0, -30);
        al1.add_style(&al1_style, Selector::DEFAULT);

        // Counter-clockwise arc below
        let al2_style = Style::new(|s| {
            s.text_color_hex(0xE91E63);
        });
        let al2 = ArcLabel::new(container)?;
        al2.set_text_static(c"ArcLabel CCW");
        al2.set_radius(70);
        al2.set_angle_start(320.0);
        al2.set_angle_size(140.0);
        al2.set_dir(ArcLabelDir::CounterClockwise);
        al2.size(160, 160);
        al2.align(Align::Center, -50, 30);
        al2.add_style(&al2_style, Selector::DEFAULT);

        // Small clockwise arc, right side
        let al3_style = Style::new(|s| {
            s.text_color_hex(0x4CAF50).text_letter_space(4);
        });
        let al3 = ArcLabel::new(container)?;
        al3.set_text_static(c"OXIVGL");
        al3.set_radius(45);
        al3.set_angle_start(60.0);
        al3.set_angle_size(120.0);
        al3.set_dir(ArcLabelDir::Clockwise);
        al3.size(110, 110);
        al3.align(Align::Center, 70, 40);
        al3.add_style(&al3_style, Selector::DEFAULT);

                self._al1 = Some(al1);
        self._al2 = Some(al2);
        self._al3 = Some(al3);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(WidgetArclabel1::default());

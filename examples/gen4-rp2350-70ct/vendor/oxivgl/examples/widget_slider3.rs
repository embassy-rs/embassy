#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Slider 3 — Range slider
//!
//! Range-mode slider with two handles and a label showing min–max values.

use oxivgl::{
    view::{NavAction, View},
    widgets::{Obj, Align, Label, Slider, SliderMode, WidgetError},
};

#[derive(Default)]
struct WidgetSlider3 {
    slider: Option<Slider<'static>>,
    label: Option<Label<'static>>,
}

impl View for WidgetSlider3 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let slider = Slider::new(container)?;
        slider
            .set_range(0, 100)
            .set_mode(SliderMode::Range)
            .set_value(80)
            .set_start_value(20);
        slider.center();

        let label = Label::new(container)?;
        label.text("20 \u{2013} 80");
        label.align_to(&slider, Align::OutBottomMid, 0, 10);

                self.slider = Some(slider);
        self.label = Some(label);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        use core::fmt::Write;
        if let (Some(slider), Some(label)) = (&self.slider, &self.label) {
            let left = slider.get_left_value();
            let right = slider.get_value();
            let mut buf = heapless::String::<16>::new();
            let _ = write!(buf, "{} \u{2013} {}", left, right);
            label.text(&buf);
            label.align_to(slider, Align::OutBottomMid, 0, 10);
        }
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(WidgetSlider3::default());

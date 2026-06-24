#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Slider 1 — Slider with value label
//!
//! A centered slider with a label below that shows the current value.

use oxivgl::{
    view::{NavAction, View},
    widgets::{Obj, Align, Label, Slider, WidgetError},
};

#[derive(Default)]
struct WidgetSlider1 {
    slider: Option<Slider<'static>>,
    label: Option<Label<'static>>,
}

impl View for WidgetSlider1 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let slider = Slider::new(container)?;
        slider.center();

        let label = Label::new(container)?;
        label.text("0");
        label.align_to(&slider, Align::OutBottomMid, 0, 10);

                self.slider = Some(slider);
        self.label = Some(label);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        use core::fmt::Write;
        if let (Some(slider), Some(label)) = (&self.slider, &self.label) {
            let val = slider.get_value();
            let mut buf = heapless::String::<8>::new();
            let _ = write!(buf, "{}", val);
            label.text(&buf);
            label.align_to(slider, Align::OutBottomMid, 0, 10);
        }
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(WidgetSlider1::default());

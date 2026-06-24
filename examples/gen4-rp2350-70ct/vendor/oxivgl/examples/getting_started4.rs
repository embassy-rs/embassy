#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Getting Started 4 — Slider with live value label
//!
//! A centered slider with a label above showing the current value, updated
//! via VALUE_CHANGED event.

use oxivgl::view::NavAction;
use oxivgl::{
    enums::EventCode,
    event::Event,
    view::{register_event_on, View},
    widgets::{Obj, Align, Label, Slider, WidgetError},
};

#[derive(Default)]
struct GettingStarted4 {
    slider: Option<Slider<'static>>,
    label: Option<Label<'static>>,
}

impl View for GettingStarted4 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let slider = Slider::new(container)?;
        slider.width(200).center();
        slider.bubble_events();

        let label = Label::new(container)?;
        label.text("0");
        label.align_to(&slider, Align::OutTopMid, 0, -15);

                self.slider = Some(slider);
        self.label = Some(label);
        Ok(())
    }

    fn register_events_on(&mut self, _container: &Obj<'static>) {
        if let Some(ref slider) = self.slider { register_event_on(self, slider.handle()); }
    }

    fn on_event(&mut self, event: &Event) -> NavAction {
        if event.code() == EventCode::VALUE_CHANGED {
            if let (Some(slider), Some(label)) = (&self.slider, &self.label) {
                let val = slider.get_value();
                let mut buf = heapless::String::<8>::new();
                let _ = core::fmt::Write::write_fmt(&mut buf, format_args!("{}", val));
                label.text(&buf);
                label.align_to(slider, Align::OutTopMid, 0, -15);
            }
        }
        NavAction::None
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(GettingStarted4::default());

#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Observer 1 — Slider bound to a temperature label via a Subject

use oxivgl::{
    view::{NavAction, View},
    widgets::{Obj, Align, Label, Slider, Subject, WidgetError},
};

struct Observer1 {
    _slider: Option<Slider<'static>>,
    _label: Option<Label<'static>>,
    _subject: Option<Subject>, // last — drop after widgets
}

impl Observer1 {
    fn new() -> Self {
        Self {
            _slider: None,
            _label: None,
            _subject: None,
        }
    }
}

impl View for Observer1 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let subject = Subject::new_int(28);

        let slider = Slider::new(container)?;
        slider.set_range(0, 100).align(Align::Center, 0, 0);
        slider.bind_value(&subject);

        let label = Label::new(container)?;
        label.align(Align::Center, 0, 30);
        label.bind_text(&subject, c"%d \u{00b0}C");

                self._slider = Some(slider);
        self._label = Some(label);
        self._subject = Some(subject);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(Observer1::new());

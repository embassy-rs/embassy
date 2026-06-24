#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Label 5 — Circular scroll
//!
//! Label with scroll-circular long mode — text scrolls in a continuous loop.

use oxivgl::{
    view::{NavAction, View},
    widgets::{Obj, Align, Label, LabelLongMode, WidgetError},
};

#[derive(Default)]
struct WidgetLabel5 {
    _label: Option<Label<'static>>,
}

impl View for WidgetLabel5 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let label = Label::new(container)?;
        label.set_long_mode(LabelLongMode::ScrollCircular);
        label.width(150);
        label.text("It is a circularly scrolling text. ");
        label.align(Align::Center, 0, 0);

                self._label = Some(label);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(WidgetLabel5::default());

#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Label 1 — Line wrap and scrolling
//!
//! Two labels: one with word-wrap (centered), one with circular scrolling.

use oxivgl::{
    style::{Selector, Style},
    view::{NavAction, View},
    widgets::{Obj, Align, Label, LabelLongMode, TextAlign, WidgetError},
};

#[derive(Default)]
struct WidgetLabel1 {
    _label1: Option<Label<'static>>,
    _label2: Option<Label<'static>>,
}

impl View for WidgetLabel1 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let label1 = Label::new(container)?;
        label1.set_long_mode(LabelLongMode::Wrap);
        label1.text(
            "Words of a label, align the lines to the center \
             and wrap long text automatically.",
        );
        label1.width(150);
        let label1_style = Style::new(|s| {
            s.text_align(TextAlign::Center);
        });
        label1.add_style(&label1_style, Selector::DEFAULT);
        label1.align(Align::Center, 0, -40);

        let label2 = Label::new(container)?;
        label2.set_long_mode(LabelLongMode::ScrollCircular);
        label2.width(150);
        label2.text("It is a circularly scrolling text. ");
        label2.align(Align::Center, 0, 40);

                self._label1 = Some(label1);
        self._label2 = Some(label2);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(WidgetLabel1::default());

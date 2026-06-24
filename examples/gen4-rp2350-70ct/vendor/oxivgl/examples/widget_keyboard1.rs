#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Keyboard 1 — Keyboard with two textareas
//!
//! Two textareas at top (left and right) with placeholder text. A keyboard
//! at the bottom is linked to the first textarea by default. Clicking a
//! textarea switches the keyboard to it.

use oxivgl::{
    enums::EventCode,
    event::Event,
    view::{NavAction, View},
    widgets::{Obj, Align, Keyboard, Textarea, WidgetError},
};

#[derive(Default)]
struct WidgetKeyboard1 {
    ta1: Option<Textarea<'static>>,
    ta2: Option<Textarea<'static>>,
    kb: Option<Keyboard<'static>>,
}

impl View for WidgetKeyboard1 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let kb = Keyboard::new(container)?;
        kb.align(Align::BottomMid, 0, 0);

        let ta1 = Textarea::new(container)?;
        ta1.size(140, 80);
        ta1.align(Align::TopLeft, 10, 10);
        ta1.set_placeholder_text("Hello");
        ta1.bubble_events();

        let ta2 = Textarea::new(container)?;
        ta2.size(140, 80);
        ta2.align(Align::TopRight, -10, 10);
        ta2.set_placeholder_text("Hello");
        ta2.bubble_events();

        kb.set_textarea(&ta1);

        self.ta1 = Some(ta1);
        self.ta2 = Some(ta2);
        self.kb = Some(kb);
        Ok(())
    }

    fn on_event(&mut self, event: &Event) -> NavAction {
        if let (Some(kb), Some(ta1), Some(ta2)) =
            (&self.kb, &self.ta1, &self.ta2)
        {
            if event.matches(ta1, EventCode::FOCUSED) {
                kb.set_textarea(ta1);
            } else if event.matches(ta2, EventCode::FOCUSED) {
                kb.set_textarea(ta2);
            }
        }
        NavAction::None
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(WidgetKeyboard1::default());

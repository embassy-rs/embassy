#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Event Click — Add click event to a button
//!
//! A button that counts clicks and displays the count as the label text.

use oxivgl::{
    view::{NavAction, View},
    enums::EventCode,
    event::Event,
    widgets::{Obj, Button, Label, WidgetError},
};

#[derive(Default)]
struct EventClick {
    btn: Option<Button<'static>>,
    label: Option<Label<'static>>,
    cnt: u32,
}

impl View for EventClick {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let btn = Button::new(container)?;
        btn.size(100, 50).center();
        btn.bubble_events();

        let label = Label::new(&btn)?;
        label.text("Click me!").center();

                self.btn = Some(btn);
        self.label = Some(label);
        self.cnt = 0;
        Ok(())
    }

    fn on_event(&mut self, event: &Event) -> NavAction {
        if let Some(ref btn) = self.btn {
            if event.matches(btn, EventCode::CLICKED) {
                self.cnt += 1;
                let mut buf = heapless::String::<12>::new();
                let _ = core::fmt::Write::write_fmt(&mut buf, format_args!("{}", self.cnt));
                if let Some(ref label) = self.label {
                    label.text(&buf);
                }
            }
        }
        NavAction::None
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(EventClick::default());

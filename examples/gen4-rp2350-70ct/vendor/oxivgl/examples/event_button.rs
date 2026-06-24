#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Event Button — Handle multiple events
//!
//! A button that displays which event was last triggered (PRESSED, CLICKED,
//! LONG_PRESSED, LONG_PRESSED_REPEAT) in an info label.

use oxivgl::view::NavAction;
use oxivgl::{
    view::{register_event_on, View},
    enums::EventCode,
    event::Event,
    widgets::{Obj, Button, Label, WidgetError},
};

#[derive(Default)]
struct EventButton {
    btn: Option<Button<'static>>,
    _btn_label: Option<Label<'static>>,
    info_label: Option<Label<'static>>,
}

impl View for EventButton {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let btn = Button::new(container)?;
        btn.size(100, 50).center();
        btn.bubble_events();

        let btn_label = Label::new(&btn)?;
        btn_label.text("Click me!").center();

        let info_label = Label::new(container)?;
        info_label.text("The last button event:\nNone");

                self.btn = Some(btn);
        self._btn_label = Some(btn_label);
        self.info_label = Some(info_label);
        Ok(())
    }

    fn register_events_on(&mut self, _container: &Obj<'static>) {
        if let Some(ref btn) = self.btn { register_event_on(self, btn.handle()); }
    }

    fn on_event(&mut self, event: &Event) -> NavAction {
        let text = match event.code() {
            EventCode::PRESSED => "The last button event:\nLV_EVENT_PRESSED",
            EventCode::CLICKED => "The last button event:\nLV_EVENT_CLICKED",
            EventCode::LONG_PRESSED => "The last button event:\nLV_EVENT_LONG_PRESSED",
            EventCode::LONG_PRESSED_REPEAT => {
                "The last button event:\nLV_EVENT_LONG_PRESSED_REPEAT"
            }
            _ => return NavAction::None,
        };
        if let Some(ref info_label) = self.info_label {
            info_label.text(text);
        }
        NavAction::None
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(EventButton::default());

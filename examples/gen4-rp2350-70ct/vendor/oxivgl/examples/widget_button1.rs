#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Button 1 — Click and toggle buttons
//!
//! Two buttons: a standard button with a click counter and a checkable toggle
//! button whose label reflects ON/OFF state.

use oxivgl::view::NavAction;
use oxivgl::{
    enums::{EventCode, ObjFlag, ObjState},
    event::Event,
    view::{register_event_on, View},
    widgets::{Obj, Align, Button, Label, WidgetError},
};

#[derive(Default)]
struct WidgetButton1 {
    btn1: Option<Button<'static>>,
    btn2: Option<Button<'static>>,
    label1: Option<Label<'static>>,
    label2: Option<Label<'static>>,
    cnt: u32,
}

impl View for WidgetButton1 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let btn1 = Button::new(container)?;
        btn1.align(Align::Center, 0, -40);
        btn1.remove_flag(ObjFlag::PRESS_LOCK);
        btn1.bubble_events();

        let label1 = Label::new(&btn1)?;
        label1.text("Button").center();

        let btn2 = Button::new(container)?;
        btn2.align(Align::Center, 0, 40);
        btn2.add_flag(ObjFlag::CHECKABLE);
        btn2.bubble_events();

        let label2 = Label::new(&btn2)?;
        label2.text("Toggle").center();

                self.btn1 = Some(btn1);
        self.btn2 = Some(btn2);
        self.label1 = Some(label1);
        self.label2 = Some(label2);
        self.cnt = 0;
        Ok(())
    }

    fn register_events_on(&mut self, _container: &Obj<'static>) {
        if let Some(ref btn1) = self.btn1 { register_event_on(self, btn1.handle()); }
        if let Some(ref btn2) = self.btn2 { register_event_on(self, btn2.handle()); }
    }

    fn on_event(&mut self, event: &Event) -> NavAction {
        if let Some(ref btn1) = self.btn1 {
            if event.matches(btn1, EventCode::CLICKED) {
                self.cnt += 1;
                let mut buf = heapless::String::<16>::new();
                let _ = core::fmt::Write::write_fmt(&mut buf, format_args!("Button: {}", self.cnt));
                if let Some(ref label1) = self.label1 { label1.text(&buf); }
            }
        }
        if let Some(ref btn2) = self.btn2 {
            if event.matches(btn2, EventCode::VALUE_CHANGED) {
                if let Some(ref label2) = self.label2 {
                    if btn2.has_state(ObjState::CHECKED) {
                        label2.text("ON");
                    } else {
                        label2.text("OFF");
                    }
                }
            }
        }
        NavAction::None
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(WidgetButton1::default());

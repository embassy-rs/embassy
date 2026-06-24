#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Textarea 2 — Password and text fields with keyboard
//!
//! A password textarea (left) and a plain text textarea (right) with an
//! on-screen keyboard at the bottom. Clicking either textarea switches
//! the keyboard focus.

use oxivgl::view::NavAction;
use oxivgl::{
    enums::EventCode,
    event::Event,
    style::lv_pct,
    view::{register_event_on, View},
    widgets::{Obj, Align, Keyboard, Label, Textarea, WidgetError},
};

#[derive(Default)]
struct WidgetTextarea2 {
    pwd_ta: Option<Textarea<'static>>,
    text_ta: Option<Textarea<'static>>,
    kb: Option<Keyboard<'static>>,
    _pwd_label: Option<Label<'static>>,
    _text_label: Option<Label<'static>>,
}

impl View for WidgetTextarea2 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        // Password textarea
        let pwd_ta = Textarea::new(container)?;
        pwd_ta.set_text("");
        pwd_ta.set_password_mode(true);
        pwd_ta.set_one_line(true);
        pwd_ta.width(lv_pct(40));
        pwd_ta.pos(5, 20);
        pwd_ta.bubble_events();

        let pwd_label = Label::new(container)?;
        pwd_label.text("Password:");
        pwd_label.align_to(&pwd_ta, Align::OutTopLeft, 0, 0);

        // Plain text textarea
        let text_ta = Textarea::new(container)?;
        text_ta.set_one_line(true);
        text_ta.set_password_mode(false);
        text_ta.width(lv_pct(40));
        text_ta.align(Align::TopRight, -5, 20);
        text_ta.bubble_events();

        let text_label = Label::new(container)?;
        text_label.text("Text:");
        text_label.align_to(&text_ta, Align::OutTopLeft, 0, 0);

        // Keyboard
        let kb = Keyboard::new(container)?;
        kb.size(320, 120);
        kb.set_textarea(&pwd_ta);

                self.pwd_ta = Some(pwd_ta);
        self.text_ta = Some(text_ta);
        self.kb = Some(kb);
        self._pwd_label = Some(pwd_label);
        self._text_label = Some(text_label);
        Ok(())
    }

    fn register_events_on(&mut self, _container: &Obj<'static>) {
        if let Some(ref pwd_ta) = self.pwd_ta {
            register_event_on(self, pwd_ta.handle());
        }
        if let Some(ref text_ta) = self.text_ta {
            register_event_on(self, text_ta.handle());
        }
    }

    fn on_event(&mut self, event: &Event) -> NavAction {
        let code = event.code();
        if code == EventCode::CLICKED || code == EventCode::FOCUSED {
            if let (Some(pwd_ta), Some(kb)) = (&self.pwd_ta, &self.kb) {
                if event.target_handle() == pwd_ta.handle() {
                    kb.set_textarea(pwd_ta);
                    return NavAction::None;
                }
            }
            if let (Some(text_ta), Some(kb)) = (&self.text_ta, &self.kb) {
                if event.target_handle() == text_ta.handle() {
                    kb.set_textarea(text_ta);
                }
            }
        }
        NavAction::None
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(WidgetTextarea2::default());

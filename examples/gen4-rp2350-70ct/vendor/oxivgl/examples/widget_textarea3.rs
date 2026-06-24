#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Textarea 3 — Clock format auto-insert
//!
//! A textarea restricted to digits and ':', max 5 characters.
//! After typing two digits, ':' is auto-inserted. A numeric keyboard
//! is shown below.

use oxivgl::view::NavAction;
use oxivgl::{
    enums::EventCode,
    event::Event,
    view::{register_event_on, View},
    widgets::{Obj, Keyboard, KeyboardMode, Textarea, WidgetError},
};

#[derive(Default)]
struct WidgetTextarea3 {
    ta: Option<Textarea<'static>>,
    _kb: Option<Keyboard<'static>>,
}

impl View for WidgetTextarea3 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let ta = Textarea::new(container)?;
        ta.set_accepted_chars(c"0123456789:");
        ta.set_max_length(5);
        ta.set_one_line(true);
        ta.set_text("");
        ta.bubble_events();

        let kb = Keyboard::new(container)?;
        kb.size(320, 120);
        kb.set_mode(KeyboardMode::Number);
        kb.set_textarea(&ta);

                self.ta = Some(ta);
        self._kb = Some(kb);
        Ok(())
    }

    fn register_events_on(&mut self, _container: &Obj<'static>) {
        if let Some(ref ta) = self.ta {
            register_event_on(self, ta.handle());
        }
    }

    fn on_event(&mut self, event: &Event) -> NavAction {
        if let Some(ref ta) = self.ta {
            if event.matches(ta, EventCode::VALUE_CHANGED) {
                if let Some(txt) = ta.get_text() {
                    let bytes = txt.as_bytes();
                    if bytes.len() >= 2
                        && bytes[0].is_ascii_digit()
                        && bytes[1].is_ascii_digit()
                        && bytes.get(2).copied() != Some(b':')
                    {
                        ta.set_cursor_pos(2);
                        ta.add_char(':');
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

oxivgl_examples_common::example_main!(WidgetTextarea3::default());

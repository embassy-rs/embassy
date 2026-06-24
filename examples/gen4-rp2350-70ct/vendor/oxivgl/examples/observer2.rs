#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Observer 2 — PIN login screen with state bindings
//!
//! Demonstrates `bind_state_if_eq`, `bind_state_if_not_eq`, `bind_checked`,
//! `bind_text_map`, and `on_change`.
//!
//! - A textarea in password mode accepts a PIN.
//! - Pressing Enter checks whether the text equals `"hello"`.
//! - An info label is reactively updated via `bind_text_map`.
//! - A "LOG OUT" button clears the auth state.
//! - A "START ENGINE" checkable button is two-way bound to `engine_subject`.

use oxivgl::view::NavAction;
use oxivgl::{
    enums::{EventCode, ObjFlag, ObjState},
    event::Event,
    view::{View, register_event_on},
    widgets::{
        Obj, Align, Button, Child, Keyboard, KeyboardMode, Label, Subject, Textarea,
        WidgetError,
    },
};

const LOGGED_OUT: i32 = 0;
const LOGGED_IN: i32 = 1;
const AUTH_FAILED: i32 = 2;

#[derive(Default)]
struct Observer2 {
    ta: Option<Textarea<'static>>,
    btn_logout: Option<Button<'static>>,
    _info_label: Option<Label<'static>>,
    _kb: Option<Keyboard<'static>>,
    _btn_engine: Option<Button<'static>>,
    // Subjects last — drop after widgets so observers are removed before deinit.
    _engine_subject: Option<Subject>,
    auth_state_subject: Option<Subject>,
}

impl View for Observer2 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let auth_state_subject = Subject::new_int(LOGGED_OUT);
        let engine_subject = Subject::new_int(0);

        // Textarea in password mode, disabled when logged in.
        let ta = Textarea::new(container)?;
        ta.set_password_mode(true)
            .set_one_line(true)
            .set_placeholder_text("Password")
            .set_max_length(20)
            .size(200, 40)
            .align(Align::TopMid, 0, 20);
        ta.bind_state_if_eq(&auth_state_subject, ObjState::DISABLED, LOGGED_IN);
        ta.bubble_events();

        // Keyboard attached to textarea.
        let kb = Keyboard::new(container)?;
        kb.set_mode(KeyboardMode::TextLower);
        kb.set_textarea(&ta);

        // LOG OUT button — disabled when not logged in.
        let btn_logout = Button::new(container)?;
        btn_logout.size(120, 40).align(Align::TopLeft, 10, 70);
        let lbl_logout = Child::new(Label::new(&btn_logout)?);
        lbl_logout.text("LOG OUT").center();
        btn_logout.bind_state_if_not_eq(&auth_state_subject, ObjState::DISABLED, LOGGED_IN);
        btn_logout.bubble_events();

        // Info label — reactively updated via bind_text_map.
        let info_label = Label::new(container)?;
        info_label.text("Logged out").align(Align::TopLeft, 10, 120);
        info_label.bind_text_map(&auth_state_subject, |state| match state {
            LOGGED_IN => "Login successful",
            AUTH_FAILED => "Login failed",
            _ => "Logged out",
        });

        // START ENGINE checkable button — two-way bound to engine_subject.
        let btn_engine = Button::new(container)?;
        btn_engine
            .add_flag(ObjFlag::CHECKABLE)
            .size(160, 40)
            .align(Align::TopLeft, 10, 160);
        let lbl_engine = Child::new(Label::new(&btn_engine)?);
        lbl_engine.text("START ENGINE").center();
        btn_engine.bind_checked(&engine_subject);

        // Engine observer — fires when engine state changes.
        engine_subject.on_change(|_value| {
            // In a real app, would set/clear a GPIO pin.
        });

                self.ta = Some(ta);
        self._kb = Some(kb);
        self.btn_logout = Some(btn_logout);
        self._info_label = Some(info_label);
        self._btn_engine = Some(btn_engine);
        self._engine_subject = Some(engine_subject);
        self.auth_state_subject = Some(auth_state_subject);
        Ok(())
    }

    fn register_events_on(&mut self, _container: &Obj<'static>) {
        if let Some(ref ta) = self.ta { register_event_on(self, ta.handle()); }
        if let Some(ref btn_logout) = self.btn_logout { register_event_on(self, btn_logout.handle()); }
    }

    fn on_event(&mut self, event: &Event) -> NavAction {
        if let Some(ref ta) = self.ta {
            if event.matches(ta, EventCode::READY) {
                if ta.get_text() == Some("hello") {
                    if let Some(ref auth) = self.auth_state_subject { auth.set_int(LOGGED_IN); }
                } else {
                    if let Some(ref auth) = self.auth_state_subject { auth.set_int(AUTH_FAILED); }
                }
            }
        }
        if let Some(ref btn_logout) = self.btn_logout {
            if event.matches(btn_logout, EventCode::CLICKED) {
                if let Some(ref auth) = self.auth_state_subject { auth.set_int(LOGGED_OUT); }
            }
        }
        NavAction::None
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(Observer2::default());

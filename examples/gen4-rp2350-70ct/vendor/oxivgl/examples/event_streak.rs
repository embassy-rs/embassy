#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Event Streak — Short-click streak counting
//!
//! Button reports short-clicked (with streak count), single-clicked,
//! double-clicked, and triple-clicked events to labels.

use oxivgl::{
    enums::EventCode,
    event::Event,
    indev::Indev,
    view::{NavAction, View},
    widgets::{Obj, Button, Label, WidgetError},
};

#[derive(Default)]
struct EventStreak {
    btn: Option<Button<'static>>,
    btn_label: Option<Label<'static>>,
    info_label: Option<Label<'static>>,
}

impl View for EventStreak {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let info_label = Label::new(container)?;
        info_label.text("No events yet");

        let btn = Button::new(container)?;
        btn.size(100, 50).center();
        btn.bubble_events();

        let btn_label = Label::new(&btn)?;
        btn_label.text("Click me!").center();

                self.btn = Some(btn);
        self.btn_label = Some(btn_label);
        self.info_label = Some(info_label);
        Ok(())
    }

    fn on_event(&mut self, event: &Event) -> NavAction {
        let Some(ref btn) = self.btn else { return NavAction::None };
        if event.matches(btn, EventCode::SHORT_CLICKED) {
            if let Some(indev) = Indev::active() {
                let cnt = indev.short_click_streak();
                let mut buf = heapless::String::<32>::new();
                let _ = core::fmt::Write::write_fmt(
                    &mut buf,
                    format_args!("Short click streak: {}", cnt),
                );
                if let Some(ref info_label) = self.info_label {
                    info_label.text(&buf);
                }
            }
        } else if event.matches(btn, EventCode::SINGLE_CLICKED) {
            if let Some(ref btn_label) = self.btn_label {
                btn_label.text("Single clicked");
            }
        } else if event.matches(btn, EventCode::DOUBLE_CLICKED) {
            if let Some(ref btn_label) = self.btn_label {
                btn_label.text("Double clicked");
            }
        } else if event.matches(btn, EventCode::TRIPLE_CLICKED) {
            if let Some(ref btn_label) = self.btn_label {
                btn_label.text("Triple clicked");
            }
        }
        NavAction::None
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(EventStreak::default());

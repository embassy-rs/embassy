#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Anim 4 — Animation with timed pause
//!
//! Switch toggles label X animation (overshoot / ease-in).
//! A one-shot 200 ms timer pauses the running animation for 1 s.

use oxivgl::{
    anim::{anim_path_ease_in, anim_path_overshoot, anim_set_x, Anim, AnimHandle},
    enums::{EventCode, ObjState},
    event::Event,
    timer::Timer,
    view::{NavAction, View},
    widgets::{Obj, Label, Switch, WidgetError},
};

#[derive(Default)]
struct Anim4 {
    label: Option<Label<'static>>,
    sw: Option<Switch<'static>>,
    pause_timer: Option<Timer>,
    anim_handle: Option<AnimHandle>,
}

impl View for Anim4 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let label = Label::new(container)?;
        label.text("Hello animations!").pos(0, 10);

        let sw = Switch::new(container)?;
        sw.center();
        sw.add_state(ObjState::CHECKED);
        sw.bubble_events();

        // One-shot timer, starts paused — armed in on_event.
        let pause_timer = Timer::new(200)?;
        pause_timer.set_repeat_count(1).pause();

        // Intro: slide label from x=0 to x=100 with overshoot.
        let mut a = Anim::new();
        a.set_var(&label)
            .set_values(0, 100)
            .set_duration(500)
            .set_exec_cb(Some(anim_set_x))
            .set_path_cb(Some(anim_path_overshoot));
        let anim_handle = a.start();

        self.label = Some(label);
        self.sw = Some(sw);
        self.pause_timer = Some(pause_timer);
        self.anim_handle = Some(anim_handle);
        Ok(())
    }

    fn on_event(&mut self, event: &Event) -> NavAction {
        if let Some(ref sw) = self.sw {
            if event.matches(sw, EventCode::VALUE_CHANGED) {
                let checked = sw.has_state(ObjState::CHECKED);

                if let Some(ref label) = self.label {
                    let mut a = Anim::new();
                    a.set_var(label)
                        .set_duration(500)
                        .set_exec_cb(Some(anim_set_x));

                    if checked {
                        a.set_values(label.get_x(), 100)
                            .set_path_cb(Some(anim_path_overshoot));
                    } else {
                        a.set_values(label.get_x(), -label.get_width())
                            .set_path_cb(Some(anim_path_ease_in));
                    }

                    self.anim_handle = Some(a.start());
                }

                // Arm the one-shot 200 ms timer.
                if let Some(ref pause_timer) = self.pause_timer {
                    pause_timer.resume().ready();
                }
            }
        }
        NavAction::None
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        if let Some(ref pause_timer) = self.pause_timer {
            if pause_timer.triggered() {
                if let Some(ref handle) = self.anim_handle {
                    handle.pause_for(1000);
                }
            }
        }
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(Anim4::default());

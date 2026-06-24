#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Anim 1 — Start animation on an event

use oxivgl::{
    anim::{anim_path_ease_in, anim_path_overshoot, anim_set_x, Anim},
    view::{NavAction, View},
    enums::{EventCode, ObjState},
    event::Event,
    widgets::{Obj, Label, Switch, WidgetError},
};

#[derive(Default)]
struct Anim1 {
    label: Option<Label<'static>>,
    sw: Option<Switch<'static>>,
}

impl View for Anim1 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let label = Label::new(container)?;
        label.text("Hello animations!").pos(100, 10);

        let sw = Switch::new(container)?;
        sw.center();
        sw.add_state(ObjState::CHECKED);
        sw.bubble_events();

                self.label = Some(label);
        self.sw = Some(sw);
        Ok(())
    }

    fn on_event(&mut self, event: &Event) -> NavAction {
        if let (Some(sw), Some(label)) = (&self.sw, &self.label) {
            if event.matches(sw, EventCode::VALUE_CHANGED) {
                let checked = sw.has_state(ObjState::CHECKED);

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
                a.start();
            }
        }
        NavAction::None
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(Anim1::default());

#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Checkbox 2 — Radio button groups
//!
//! Two independent groups of checkboxes acting as radio buttons via event
//! bubbling. Clicking one unchecks the rest in its group.

extern crate alloc;

use alloc::vec::Vec;
use oxivgl::view::NavAction;
use oxivgl::{
    view::{register_event_on, View},
    enums::{EventCode, ObjState},
    event::Event,
    layout::{FlexAlign, FlexFlow},
    widgets::{Checkbox, Label, Obj, WidgetError},
};

#[derive(Default)]
struct WidgetCheckbox2 {
    group1: Option<Obj<'static>>,
    group2: Option<Obj<'static>>,
    _checkboxes: Option<Vec<Checkbox<'static>>>,
    _label: Option<Label<'static>>,
}

impl View for WidgetCheckbox2 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {
        container.set_flex_flow(FlexFlow::Column);
        container.set_flex_align(FlexAlign::Center, FlexAlign::Start, FlexAlign::Center);

        let label = Label::new(container)?;
        label.text("Selected: none");

        let mut checkboxes = Vec::new();

        // Group 1
        let group1 = Obj::new(container)?;
        group1.set_flex_flow(FlexFlow::Column);

        let titles1 = ["A1", "A2", "A3"];
        for (i, title) in titles1.iter().enumerate() {
            let cb = Checkbox::new(&group1)?;
            cb.text(title);
            cb.bubble_events();
            if i == 0 {
                cb.add_state(ObjState::CHECKED);
            }
            checkboxes.push(cb);
        }

        // Group 2
        let group2 = Obj::new(container)?;
        group2.set_flex_flow(FlexFlow::Column);

        let titles2 = ["B1", "B2", "B3"];
        for title in titles2.iter() {
            let cb = Checkbox::new(&group2)?;
            cb.text(title);
            cb.bubble_events();
            checkboxes.push(cb);
        }

                self.group1 = Some(group1);
        self.group2 = Some(group2);
        self._checkboxes = Some(checkboxes);
        self._label = Some(label);
        Ok(())
    }

    fn register_events_on(&mut self, _container: &Obj<'static>) {
        if let Some(ref g) = self.group1 {
            register_event_on(self, g.handle());
        }
        if let Some(ref g) = self.group2 {
            register_event_on(self, g.handle());
        }
    }

    fn on_event(&mut self, event: &Event) -> NavAction {
        if event.code() != EventCode::CLICKED {
            return NavAction::None;
        }
        let target = event.target_handle();
        let container = event.current_target_handle();

        // Ignore clicks on the container itself
        if target == container {
            return NavAction::None;
        }

        // Uncheck all children, check only the clicked one
        let g1_handle = self.group1.as_ref().map(|g| g.handle());
        let cont_obj = if Some(container) == g1_handle {
            &self.group1
        } else {
            &self.group2
        };

        if let Some(obj) = cont_obj {
            let count = obj.get_child_count();
            for i in 0..count as i32 {
                if let Some(child) = obj.get_child(i) {
                    child.remove_state(ObjState::CHECKED);
                }
            }
        }

        // Check the clicked target
        let target_obj = event.target();
        target_obj.add_state(ObjState::CHECKED);
        NavAction::None
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(WidgetCheckbox2::default());

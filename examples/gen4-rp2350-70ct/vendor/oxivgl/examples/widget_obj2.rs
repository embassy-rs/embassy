#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Obj 2 — Draggable object
//!
//! A base object that follows the pointer when pressed, using
//! `Indev::active().get_vect()` to read the movement delta.

use oxivgl::view::NavAction;
use oxivgl::{
    enums::EventCode,
    event::Event,
    indev::Indev,
    view::{View, register_event_on},
    widgets::{Label, Obj, WidgetError},
};

#[derive(Default)]
struct WidgetObj2 {
    obj: Option<Obj<'static>>,
    _label: Option<Label<'static>>,
}

impl View for WidgetObj2 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let obj = Obj::new(container)?;
        obj.size(150, 100);

        let label = Label::new(&obj)?;
        label.text("Drag me").center();

        self.obj = Some(obj);
        self._label = Some(label);
        Ok(())
    }

    fn register_events_on(&mut self, _container: &Obj<'static>) {
        if let Some(ref obj) = self.obj {
            register_event_on(self, obj.handle());
        }
    }

    fn on_event(&mut self, event: &Event) -> NavAction {
        if let Some(ref obj) = self.obj {
            if event.matches(obj, EventCode::PRESSING) {
                if let Some(indev) = Indev::active() {
                    let vect = indev.get_vect();
                    let x = obj.get_x_aligned() + vect.x;
                    let y = obj.get_y_aligned() + vect.y;
                    obj.pos(x, y);
                }
            }
        }
        NavAction::None
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(WidgetObj2::default());

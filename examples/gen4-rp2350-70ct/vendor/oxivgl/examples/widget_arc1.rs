#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Arc 1 — Arc with value-changed event
//!
//! Draggable arc (150×150, 270° sweep) with a label that shows the current
//! percentage and follows the arc's edge.

use oxivgl::{
    view::{NavAction, View},
    enums::EventCode,
    event::Event,
    widgets::{Obj, Arc, Label, WidgetError},
};

#[derive(Default)]
struct WidgetArc1 {
    arc: Option<Arc<'static>>,
    label: Option<Label<'static>>,
}

impl View for WidgetArc1 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let label = Label::new(container)?;

        let arc = Arc::new(container)?;
        arc.size(150, 150);
        arc.set_rotation(135);
        arc.set_bg_angles(0, 270);
        arc.set_value_raw(10);
        arc.center();

        // Initial label update
        let v = arc.get_value_raw();
        let mut buf = heapless::String::<8>::new();
        let _ = core::fmt::Write::write_fmt(&mut buf, format_args!("{}%", v));
        label.text(&buf);
        arc.rotate_obj_to_angle(&label, 25);

                self.arc = Some(arc);
        self.label = Some(label);
        Ok(())
    }

    fn on_event(&mut self, event: &Event) -> NavAction {
        if let Some(ref arc) = self.arc {
            if event.matches(arc, EventCode::VALUE_CHANGED) {
                let v = arc.get_value_raw();
                let mut buf = heapless::String::<8>::new();
                let _ = core::fmt::Write::write_fmt(&mut buf, format_args!("{}%", v));
                if let Some(ref label) = self.label {
                    label.text(&buf);
                    arc.rotate_obj_to_angle(label, 25);
                }
            }
        }
        NavAction::None
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(WidgetArc1::default());

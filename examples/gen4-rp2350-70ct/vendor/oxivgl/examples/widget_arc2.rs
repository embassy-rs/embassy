#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Arc 2 — Animated arc loader
//!
//! Full-circle arc that animates from 0 to 100 in a 1-second loop
//! (infinite repeat with 500 ms delay). Knob hidden, not clickable.

use oxivgl::{
    anim::{anim_set_arc_value, Anim, ANIM_REPEAT_INFINITE},
    style::Style,
    view::{NavAction, View},
    enums::{ObjFlag, Opa},
    widgets::{Obj, Arc, Part, WidgetError},
};

#[derive(Default)]
struct WidgetArc2 {
    _arc: Option<Arc<'static>>,
}

impl View for WidgetArc2 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let arc = Arc::new(container)?;
        arc.set_rotation(270);
        arc.set_bg_angles(0, 360);
        arc.set_range_raw(0, 100);
        // Hide knob
        let knob_style = Style::new(|s| {
            s.radius(0).opa(Opa::TRANSP.0).pad_all(0);
        });
        arc.add_style(&knob_style, Part::Knob);
        // Not interactive
        arc.remove_flag(ObjFlag::CLICKABLE);
        arc.center();

        let mut anim = Anim::new();
        anim.set_var(&arc)
            .set_exec_cb(Some(anim_set_arc_value))
            .set_duration(1000)
            .set_values(0, 100)
            .set_repeat_count(ANIM_REPEAT_INFINITE)
            .set_repeat_delay(500)
            .start();

                self._arc = Some(arc);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(WidgetArc2::default());

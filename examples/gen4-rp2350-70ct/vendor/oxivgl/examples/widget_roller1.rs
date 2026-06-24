#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Roller 1 — Simple month roller
//!
//! Infinite roller displaying month names with 4 visible rows.

extern crate alloc;

use oxivgl::{
    view::{NavAction, View},
    widgets::{Obj, Roller, RollerMode, WidgetError},
};

#[derive(Default)]
struct WidgetRoller1 {
    _roller: Option<Roller<'static>>,
}

impl View for WidgetRoller1 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let roller = Roller::new(container)?;
        roller.set_options(
            "January\n\
             February\n\
             March\n\
             April\n\
             May\n\
             June\n\
             July\n\
             August\n\
             September\n\
             October\n\
             November\n\
             December",
            RollerMode::Infinite,
        );
        roller.set_visible_row_count(4);
        roller.center();

                self._roller = Some(roller);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(WidgetRoller1::default());

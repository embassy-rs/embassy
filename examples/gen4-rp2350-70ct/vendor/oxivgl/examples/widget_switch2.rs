#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Switch 2 — Horizontal and vertical switches
//!
//! Two switches: horizontal (default) and vertical, with the vertical one
//! pre-checked.

use oxivgl::{
    view::{NavAction, View},
    enums::ObjState,
    layout::{FlexAlign, FlexFlow},
    widgets::{Obj, Switch, SwitchOrientation, WidgetError},
};

#[derive(Default)]
struct WidgetSwitch2 {
    _sw1: Option<Switch<'static>>,
    _sw2: Option<Switch<'static>>,
}

impl View for WidgetSwitch2 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {
        container.set_flex_flow(FlexFlow::Column);
        container.set_flex_align(FlexAlign::Center, FlexAlign::Center, FlexAlign::Center);

        let sw1 = Switch::new(container)?;

        let sw2 = Switch::new(container)?;
        sw2.set_orientation(SwitchOrientation::Vertical);
        sw2.size(50, 120);
        sw2.add_state(ObjState::CHECKED);

                self._sw1 = Some(sw1);
        self._sw2 = Some(sw2);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(WidgetSwitch2::default());

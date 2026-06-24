#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Switch 1 — Toggle switches with states
//!
//! Four switches in a column: default, checked, disabled, checked+disabled.

use oxivgl::{
    view::{NavAction, View},
    enums::ObjState,
    layout::{FlexAlign, FlexFlow},
    widgets::{Obj, Switch, WidgetError},
};

#[derive(Default)]
struct WidgetSwitch1 {
    _sw1: Option<Switch<'static>>,
    _sw2: Option<Switch<'static>>,
    _sw3: Option<Switch<'static>>,
    _sw4: Option<Switch<'static>>,
}

impl View for WidgetSwitch1 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {
        container.set_flex_flow(FlexFlow::Column);
        container.set_flex_align(FlexAlign::Center, FlexAlign::Center, FlexAlign::Center);

        let sw1 = Switch::new(container)?;

        let sw2 = Switch::new(container)?;
        sw2.add_state(ObjState::CHECKED);

        let sw3 = Switch::new(container)?;
        sw3.add_state(ObjState::DISABLED);

        let sw4 = Switch::new(container)?;
        sw4.add_state(ObjState::CHECKED | ObjState::DISABLED);

                self._sw1 = Some(sw1);
        self._sw2 = Some(sw2);
        self._sw3 = Some(sw3);
        self._sw4 = Some(sw4);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(WidgetSwitch1::default());

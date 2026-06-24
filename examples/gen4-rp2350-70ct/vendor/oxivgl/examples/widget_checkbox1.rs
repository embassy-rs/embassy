#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Checkbox 1 — Simple checkboxes
//!
//! Four checkboxes in a column: unchecked, checked, disabled, checked+disabled.

use oxivgl::{
    view::{NavAction, View},
    enums::ObjState,
    layout::{FlexAlign, FlexFlow},
    widgets::{Obj, Checkbox, WidgetError},
};

#[derive(Default)]
struct WidgetCheckbox1 {
    _cb1: Option<Checkbox<'static>>,
    _cb2: Option<Checkbox<'static>>,
    _cb3: Option<Checkbox<'static>>,
    _cb4: Option<Checkbox<'static>>,
}

impl View for WidgetCheckbox1 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {
        container.set_flex_flow(FlexFlow::Column);
        container.set_flex_align(FlexAlign::Center, FlexAlign::Start, FlexAlign::Center);

        let cb1 = Checkbox::new(container)?;
        cb1.text("Apple");

        let cb2 = Checkbox::new(container)?;
        cb2.text("Banana");
        cb2.add_state(ObjState::CHECKED);

        let cb3 = Checkbox::new(container)?;
        cb3.text("Lemon");
        cb3.add_state(ObjState::DISABLED);

        let cb4 = Checkbox::new(container)?;
        cb4.text("Melon");
        cb4.add_state(ObjState::CHECKED | ObjState::DISABLED);

                self._cb1 = Some(cb1);
        self._cb2 = Some(cb2);
        self._cb3 = Some(cb3);
        self._cb4 = Some(cb4);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(WidgetCheckbox1::default());

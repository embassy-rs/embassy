#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Flex 3 — Demonstrate flex grow

use oxivgl::{
    layout::FlexFlow,
    view::{NavAction, View},
    widgets::{Obj, WidgetError},
};

#[derive(Default)]
struct Flex3 {
    _cont: Option<Obj<'static>>,
    _obj0: Option<Obj<'static>>,
    _obj1: Option<Obj<'static>>,
    _obj2: Option<Obj<'static>>,
    _obj3: Option<Obj<'static>>,
}

impl View for Flex3 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let cont = Obj::new(container)?;
        cont.size(300, 220).center();
        cont.set_flex_flow(FlexFlow::Row);

        let obj0 = Obj::new(&cont)?;
        obj0.size(40, 40);

        let obj1 = Obj::new(&cont)?;
        obj1.height(40);
        obj1.set_flex_grow(1);

        let obj2 = Obj::new(&cont)?;
        obj2.height(40);
        obj2.set_flex_grow(2);

        let obj3 = Obj::new(&cont)?;
        obj3.size(40, 40);

        self._cont = Some(cont);
        self._obj0 = Some(obj0);
        self._obj1 = Some(obj1);
        self._obj2 = Some(obj2);
        self._obj3 = Some(obj3);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(Flex3::default());

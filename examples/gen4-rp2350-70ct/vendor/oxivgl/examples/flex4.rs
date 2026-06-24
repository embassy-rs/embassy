#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Flex 4 — Reverse the order of flex items

use oxivgl::{
    layout::FlexFlow,
    view::{NavAction, View},
    widgets::{Label, Obj, WidgetError},
};

#[derive(Default)]
struct Flex4 {
    _cont: Option<Obj<'static>>,
    _items: heapless::Vec<Obj<'static>, 6>,
    _labels: heapless::Vec<Label<'static>, 6>,
}

impl View for Flex4 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let cont = Obj::new(container)?;
        cont.size(300, 220).center();
        cont.set_flex_flow(FlexFlow::ColumnReverse);

        let mut items = heapless::Vec::<Obj<'static>, 6>::new();
        let mut labels = heapless::Vec::<Label<'static>, 6>::new();

        for i in 0..6u32 {
            let obj = Obj::new(&cont)?;
            obj.size(100, 50);

            let label = Label::new(&obj)?;
            let mut buf = heapless::String::<12>::new();
            let _ = core::fmt::Write::write_fmt(&mut buf, format_args!("Item: {}", i));
            label.text(&buf).center();

            let _ = items.push(obj);
            let _ = labels.push(label);
        }

                self._cont = Some(cont);
        self._items = items;
        self._labels = labels;
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(Flex4::default());

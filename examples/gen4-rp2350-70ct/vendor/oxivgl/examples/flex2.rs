#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Flex 2 — Arrange items in rows with wrap and even spacing (via style)

use oxivgl::{
    style::{Selector, Style, StyleBuilder, LV_SIZE_CONTENT},
    view::{NavAction, View},
    enums::ObjFlag,
    layout::{FlexAlign, FlexFlow, Layout},
    widgets::{Label, Obj, WidgetError},
};

#[derive(Default)]
struct Flex2 {
    _style: Option<Style>,
    _cont: Option<Obj<'static>>,
    _items: heapless::Vec<Obj<'static>, 8>,
    _labels: heapless::Vec<Label<'static>, 8>,
}

impl View for Flex2 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let mut style = StyleBuilder::new();
        style
            .flex_flow(FlexFlow::RowWrap)
            .flex_main_place(FlexAlign::SpaceEvenly)
            .layout(Layout::Flex);
        let style = style.build();

        let cont = Obj::new(container)?;
        cont.size(300, 220).center();
        cont.add_style(&style, Selector::DEFAULT);

        let mut items = heapless::Vec::<Obj<'static>, 8>::new();
        let mut labels = heapless::Vec::<Label<'static>, 8>::new();

        for i in 0..8u32 {
            let obj = Obj::new(&cont)?;
            obj.size(70, LV_SIZE_CONTENT);
            obj.add_flag(ObjFlag::CHECKABLE);

            let label = Label::new(&obj)?;
            let mut buf = heapless::String::<4>::new();
            let _ = core::fmt::Write::write_fmt(&mut buf, format_args!("{}", i));
            label.text(&buf).center();

            let _ = items.push(obj);
            let _ = labels.push(label);
        }

                self._style = Some(style);
        self._cont = Some(cont);
        self._items = items;
        self._labels = labels;
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(Flex2::default());

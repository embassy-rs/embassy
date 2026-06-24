#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Style 1 — Size, Position and Padding

extern crate alloc;

use oxivgl::{
    style::{lv_pct, Selector, Style, StyleBuilder, LV_SIZE_CONTENT},
    view::{NavAction, View},
    widgets::{Label, Obj, WidgetError},
};

#[derive(Default)]
struct Style1 {
    _label: Option<Label<'static>>,
    _obj: Option<Obj<'static>>,
    _style: Option<Style>,
}

impl View for Style1 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let mut builder = StyleBuilder::new();
        builder
            .radius(5)
            .width(150)
            .height(LV_SIZE_CONTENT)
            .pad_ver(20)
            .pad_left(5)
            .x(lv_pct(50))
            .y(80);
        let style = builder.build();

        let obj = Obj::new(container)?;
        obj.add_style(&style, Selector::DEFAULT);

        let label = Label::new(&obj)?;
        label.text("Hello");

                self._label = Some(label);
        self._obj = Some(obj);
        self._style = Some(style);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(Style1::default());

#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Style 7 — Arc color and width

extern crate alloc;

use oxivgl::{
    style::{palette_main, Palette, Selector, Style, StyleBuilder},
    view::{NavAction, View},
    widgets::{Obj, Arc, WidgetError},
};

#[derive(Default)]
struct Style7 {
    _arc: Option<Arc<'static>>,
    _style: Option<Style>,
}

impl View for Style7 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let mut builder = StyleBuilder::new();
        builder.arc_color(palette_main(Palette::Red)).arc_width(4);
        let style = builder.build();

        let arc = Arc::new(container)?;
        arc.add_style(&style, Selector::DEFAULT);
        arc.center();

                self._arc = Some(arc);
        self._style = Some(style);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(Style7::default());

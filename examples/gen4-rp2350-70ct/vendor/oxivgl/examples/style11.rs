#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Style 11 — Multiple styles

extern crate alloc;

use oxivgl::{
    style::{
        color_white, palette_darken, palette_main, Palette, Selector, Style, StyleBuilder,
        LV_SIZE_CONTENT,
    },
    view::{NavAction, View},
    widgets::{Align, Label, Obj, WidgetError},
};

#[derive(Default)]
struct Style11 {
    _label_warn: Option<Label<'static>>,
    _obj_warn: Option<Obj<'static>>,
    _label_base: Option<Label<'static>>,
    _obj_base: Option<Obj<'static>>,
    _style_warning: Option<Style>,
    _style_base: Option<Style>,
}

impl View for Style11 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let mut builder_base = StyleBuilder::new();
        builder_base
            .bg_color(palette_main(Palette::LightBlue))
            .border_color(palette_darken(Palette::LightBlue, 3))
            .border_width(2)
            .radius(10)
            .shadow_width(10)
            .shadow_offset_y(5)
            .shadow_opa(127)
            .text_color(color_white())
            .width(100)
            .height(LV_SIZE_CONTENT);
        let style_base = builder_base.build();

        let mut builder_warning = StyleBuilder::new();
        builder_warning
            .bg_color(palette_main(Palette::Yellow))
            .border_color(palette_darken(Palette::Yellow, 3))
            .text_color(palette_darken(Palette::Yellow, 4));
        let style_warning = builder_warning.build();

        let obj_base = Obj::new(container)?;
        obj_base.add_style(&style_base, Selector::DEFAULT);
        obj_base.align(Align::LeftMid, 20, 0);

        let label_base = Label::new(&obj_base)?;
        label_base.text("Base").center();

        let obj_warn = Obj::new(container)?;
        obj_warn.add_style(&style_base, Selector::DEFAULT);
        obj_warn.add_style(&style_warning, Selector::DEFAULT);
        obj_warn.align(Align::RightMid, -20, 0);

        let label_warn = Label::new(&obj_warn)?;
        label_warn.text("Warning").center();

                self._label_warn = Some(label_warn);
        self._obj_warn = Some(obj_warn);
        self._label_base = Some(label_base);
        self._obj_base = Some(obj_base);
        self._style_warning = Some(style_warning);
        self._style_base = Some(style_base);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(Style11::default());

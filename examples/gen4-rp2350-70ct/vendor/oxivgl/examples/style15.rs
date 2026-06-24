#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Style 15 — Opacity and Transformations

use oxivgl::{
    style::{Selector, Style},
    view::{NavAction, View},
    widgets::{Obj, Align, Button, Label, WidgetError},
};

#[derive(Default)]
struct Style15 {
    _label3: Option<Label<'static>>,
    _btn3: Option<Button<'static>>,
    _label2: Option<Label<'static>>,
    _btn2: Option<Button<'static>>,
    _label1: Option<Label<'static>>,
    _btn1: Option<Button<'static>>,
}

impl View for Style15 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        // Shared opacity style (opa 50%) applied to the buttons that need it.
        let opa_style = Style::new(|s| {
            s.opa(128);
        });

        let btn1 = Button::new(container)?;
        btn1.size(100, 40).align(Align::Center, 0, -70);
        let label1 = Label::new(&btn1)?;
        label1.text("Normal").center();

        let btn2 = Button::new(container)?;
        btn2.size(100, 40).align(Align::Center, 0, 0);
        btn2.add_style(&opa_style, Selector::DEFAULT);
        let label2 = Label::new(&btn2)?;
        label2.text("Opa:50%").center();

        let btn3 = Button::new(container)?;
        btn3.size(100, 40).align(Align::Center, 0, 70);
        btn3.add_style(&opa_style, Selector::DEFAULT);
        btn3.style_transform_rotation(150, Selector::DEFAULT)
            .style_transform_scale(256 + 64, Selector::DEFAULT)
            .style_transform_pivot_x(50, Selector::DEFAULT)
            .style_transform_pivot_y(20, Selector::DEFAULT);
        let label3 = Label::new(&btn3)?;
        label3.text("Transf.").center();

                self._label3 = Some(label3);
        self._btn3 = Some(btn3);
        self._label2 = Some(label2);
        self._btn2 = Some(btn2);
        self._label1 = Some(label1);
        self._btn1 = Some(btn1);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(Style15::default());

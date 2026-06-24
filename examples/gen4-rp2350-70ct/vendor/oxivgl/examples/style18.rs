#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Style 18 — Various gradient buttons

use oxivgl::{
    style::{color_make, lv_pct, GradDir, GradDsc, GradExtend, Selector, Style, StyleBuilder},
    view::{NavAction, View},
    widgets::{Obj, Align, Button, Label, WidgetError},
};

#[derive(Default)]
struct Style18 {
    _label4: Option<Label<'static>>,
    _btn4: Option<Button<'static>>,
    _label3: Option<Label<'static>>,
    _btn3: Option<Button<'static>>,
    _label2: Option<Label<'static>>,
    _btn2: Option<Button<'static>>,
    _label1: Option<Label<'static>>,
    _btn1: Option<Button<'static>>,
    _style_radial: Option<Style>,
    _style_linear: Option<Style>,
    _style_hor: Option<Style>,
    _style_ver: Option<Style>,
}

impl View for Style18 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let c0 = color_make(0x26, 0xa0, 0xda);
        let c1 = color_make(0x31, 0x47, 0x55);
        let colors = [c0, c1];

        let mut grad_linear = GradDsc::new();
        grad_linear.init_stops(&colors, &[], &[]).linear(
            lv_pct(0),
            lv_pct(0),
            lv_pct(20),
            lv_pct(100),
            GradExtend::Reflect,
        );
        let mut style_linear = StyleBuilder::new();
        style_linear.bg_grad(grad_linear).bg_opa(255);
        let style_linear = style_linear.build();

        let mut grad_radial = GradDsc::new();
        grad_radial.init_stops(&colors, &[], &[]).radial(
            lv_pct(30),
            lv_pct(30),
            lv_pct(100),
            lv_pct(100),
            GradExtend::Reflect,
        );
        let mut style_radial = StyleBuilder::new();
        style_radial.bg_grad(grad_radial).bg_opa(255);
        let style_radial = style_radial.build();

        let style_hor = Style::new(|s| {
            s.bg_color(c0).bg_grad_color(c1).bg_grad_dir(GradDir::Hor);
        });
        let style_ver = Style::new(|s| {
            s.bg_color(c0).bg_grad_color(c1).bg_grad_dir(GradDir::Ver);
        });

        let btn1 = Button::new(container)?;
        btn1.size(150, 50).align(Align::Center, 0, -100);
        btn1.add_style(&style_hor, Selector::DEFAULT);
        let label1 = Label::new(&btn1)?;
        label1.text("Horizontal").center();

        let btn2 = Button::new(container)?;
        btn2.size(150, 50).align(Align::Center, 0, -40);
        btn2.add_style(&style_ver, Selector::DEFAULT);
        let label2 = Label::new(&btn2)?;
        label2.text("Vertical").center();

        let btn3 = Button::new(container)?;
        btn3.size(150, 50).align(Align::Center, 0, 20);
        btn3.add_style(&style_linear, Selector::DEFAULT);
        let label3 = Label::new(&btn3)?;
        label3.text("Linear").center();

        let btn4 = Button::new(container)?;
        btn4.size(150, 50).align(Align::Center, 0, 80);
        btn4.add_style(&style_radial, Selector::DEFAULT);
        let label4 = Label::new(&btn4)?;
        label4.text("Radial").center();

                self._label4 = Some(label4);
        self._btn4 = Some(btn4);
        self._label3 = Some(label3);
        self._btn3 = Some(btn3);
        self._label2 = Some(label2);
        self._btn2 = Some(btn2);
        self._label1 = Some(label1);
        self._btn1 = Some(btn1);
        self._style_radial = Some(style_radial);
        self._style_linear = Some(style_linear);
        self._style_hor = Some(style_hor);
        self._style_ver = Some(style_ver);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(Style18::default());

#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Button 2 — Styled button from scratch
//!
//! A button with custom default and pressed styles, including gradient,
//! shadow, outline, and a transition that expands the outline on press.

use oxivgl::{
    anim::anim_path_linear,
    style::{
        color_white, palette_darken, palette_main, props, GradDir, Palette, Selector, Style,
        StyleBuilder, TransitionDsc, LV_SIZE_CONTENT,
    },
    view::{NavAction, View},
    enums::ObjState,
    widgets::{Obj, Button, Label, WidgetError},
};

#[derive(Default)]
struct WidgetButton2 {
    _btn: Option<Button<'static>>,
    _label: Option<Label<'static>>,
    _style: Option<Style>,
    _style_pr: Option<Style>,
}

/// Transition property list: outline width + outline opacity + sentinel.
static TRANS_PROPS: [props::lv_style_prop_t; 3] =
    [props::OUTLINE_WIDTH, props::OUTLINE_OPA, props::LAST];

impl View for WidgetButton2 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        // Default state style
        let mut style = StyleBuilder::new();
        style
            .radius(3)
            .bg_opa(255)
            .bg_color(palette_main(Palette::Blue))
            .bg_grad_color(palette_darken(Palette::Blue, 2))
            .bg_grad_dir(GradDir::Ver)
            .border_opa(102) // LV_OPA_40
            .border_width(2)
            .border_color(palette_main(Palette::Grey))
            .shadow_width(8)
            .shadow_color(palette_main(Palette::Grey))
            .shadow_offset_y(8)
            .outline_opa(255) // LV_OPA_COVER
            .outline_color(palette_main(Palette::Blue))
            .text_color(color_white())
            .pad_all(10);
        let style = style.build();

        // Pressed state style
        let trans = TransitionDsc::new(&TRANS_PROPS, Some(anim_path_linear), 300, 0);

        let mut style_pr = StyleBuilder::new();
        style_pr
            .outline_width(30)
            .outline_opa(0) // LV_OPA_TRANSP
            .translate_y(5)
            .shadow_offset_y(3)
            .bg_color(palette_darken(Palette::Blue, 2))
            .bg_grad_color(palette_darken(Palette::Blue, 4))
            .transition(trans);
        let style_pr = style_pr.build();

        let btn = Button::new(container)?;
        btn.remove_style_all();
        btn.add_style(&style, Selector::DEFAULT);
        btn.add_style(&style_pr, ObjState::PRESSED);
        btn.size(LV_SIZE_CONTENT, LV_SIZE_CONTENT);
        btn.center();

        let label = Label::new(&btn)?;
        label.text("Button").center();

                self._btn = Some(btn);
        self._label = Some(label);
        self._style = Some(style);
        self._style_pr = Some(style_pr);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(WidgetButton2::default());

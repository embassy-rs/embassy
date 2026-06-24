#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Button 3 — Gum squeeze animation
//!
//! Button with style transitions that stretch width and shrink height on press,
//! using overshoot easing on release for a "gum" effect.

use oxivgl::{
    anim::{anim_path_ease_in_out, anim_path_overshoot},
    style::{props, Selector, Style, StyleBuilder, TransitionDsc},
    view::{NavAction, View},
    enums::ObjState,
    widgets::{Obj, Align, Button, Label, WidgetError},
};

/// Transition property list: transform width + height + letter spacing + sentinel.
static TRANS_PROPS: [props::lv_style_prop_t; 4] = [
    props::TRANSFORM_WIDTH,
    props::TRANSFORM_HEIGHT,
    props::TEXT_LETTER_SPACE,
    props::LAST,
];

#[derive(Default)]
struct WidgetButton3 {
    _btn: Option<Button<'static>>,
    _label: Option<Label<'static>>,
    _style_def: Option<Style>,
    _style_pr: Option<Style>,
}

impl View for WidgetButton3 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        // Default state: overshoot transition back from press (100ms delay)
        let trans_def =
            TransitionDsc::new(&TRANS_PROPS, Some(anim_path_overshoot), 250, 100);
        let mut sb_def = StyleBuilder::new();
        sb_def.transition(trans_def);
        let style_def = sb_def.build();

        // Pressed state: stretch width, shrink height, spread letters
        let trans_pr =
            TransitionDsc::new(&TRANS_PROPS, Some(anim_path_ease_in_out), 250, 0);
        let mut sb_pr = StyleBuilder::new();
        sb_pr
            .transform_width(10)
            .transform_height(-10)
            .text_letter_space(10)
            .transition(trans_pr);
        let style_pr = sb_pr.build();

        let btn = Button::new(container)?;
        btn.add_style(&style_def, Selector::DEFAULT);
        btn.add_style(&style_pr, ObjState::PRESSED);
        btn.align(Align::Center, 0, 0);

        let label = Label::new(&btn)?;
        label.text("Gum");

                self._btn = Some(btn);
        self._label = Some(label);
        self._style_def = Some(style_def);
        self._style_pr = Some(style_pr);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(WidgetButton3::default());

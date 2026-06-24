#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Style 10 — Transition

extern crate alloc;

use oxivgl::{
    anim::anim_path_linear,
    style::{
        palette_darken, palette_main, props, Palette, Selector, Style, StyleBuilder, TransitionDsc,
    },
    view::{NavAction, View},
    enums::ObjState,
    widgets::{Obj, WidgetError},
};

#[derive(Default)]
struct Style10 {
    _obj: Option<Obj<'static>>,
    _style_def: Option<Style>,
    _style_pr: Option<Style>,
}

static TRANS_PROPS: [props::lv_style_prop_t; 4] =
    [props::BG_COLOR, props::BORDER_COLOR, props::BORDER_WIDTH, 0];

impl View for Style10 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let trans_def = TransitionDsc::new(&TRANS_PROPS, Some(anim_path_linear), 100, 200);

        let trans_pr = TransitionDsc::new(&TRANS_PROPS, Some(anim_path_linear), 500, 0);

        let mut builder_def = StyleBuilder::new();
        builder_def.transition(trans_def);
        let style_def = builder_def.build();

        let mut builder_pr = StyleBuilder::new();
        builder_pr
            .bg_color(palette_main(Palette::Red))
            .border_width(6)
            .border_color(palette_darken(Palette::Red, 3))
            .transition(trans_pr);
        let style_pr = builder_pr.build();

        let obj = Obj::new(container)?;
        obj.add_style(&style_def, Selector::DEFAULT);
        obj.add_style(&style_pr, ObjState::PRESSED);
        obj.center();

                self._obj = Some(obj);
        self._style_def = Some(style_def);
        self._style_pr = Some(style_pr);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(Style10::default());

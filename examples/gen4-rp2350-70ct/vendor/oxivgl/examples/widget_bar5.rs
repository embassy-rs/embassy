#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Bar 5 — LTR vs RTL bars
//!
//! Two bars side by side: one left-to-right (default), one right-to-left.

use oxivgl::{
    style::{Selector, Style},
    view::{NavAction, View},
    widgets::{Obj, Align, Bar, BaseDir, Label, WidgetError},
};

#[derive(Default)]
struct WidgetBar5 {
    _bar_ltr: Option<Bar<'static>>,
    _bar_rtl: Option<Bar<'static>>,
    _label_ltr: Option<Label<'static>>,
    _label_rtl: Option<Label<'static>>,
    _style_rtl: Option<Style>,
}

impl View for WidgetBar5 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        // LTR bar
        let bar_ltr = Bar::new(container)?;
        bar_ltr.size(200, 20);
        bar_ltr.set_range_raw(0, 100);
        bar_ltr.set_value_raw(70, false);
        bar_ltr.align(Align::Center, 0, -30);

        let label_ltr = Label::new(container)?;
        label_ltr.text("Left to Right base direction");
        label_ltr.align_to(&bar_ltr, Align::OutTopMid, 0, -5);

        // RTL bar
        let bar_rtl = Bar::new(container)?;
        bar_rtl.size(200, 20);
        bar_rtl.set_range_raw(0, 100);
        bar_rtl.set_value_raw(70, false);
        let style_rtl = Style::new(|s| {
            s.base_dir(BaseDir::Rtl);
        });
        bar_rtl.add_style(&style_rtl, Selector::DEFAULT);
        bar_rtl.align(Align::Center, 0, 30);

        let label_rtl = Label::new(container)?;
        label_rtl.text("Right to Left base direction");
        label_rtl.align_to(&bar_rtl, Align::OutTopMid, 0, -5);

                self._bar_ltr = Some(bar_ltr);
        self._bar_rtl = Some(bar_rtl);
        self._label_ltr = Some(label_ltr);
        self._label_rtl = Some(label_rtl);
        self._style_rtl = Some(style_rtl);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(WidgetBar5::default());

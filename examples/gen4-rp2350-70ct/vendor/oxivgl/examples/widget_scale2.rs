#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Scale 2 — Horizontal scale
//!
//! Horizontal bottom-aligned scale with labeled major ticks from 10 to 40.

use oxivgl::{
    style::lv_pct,
    view::{NavAction, View},
    widgets::{Obj, Part, Scale, ScaleMode, WidgetError},
};

#[derive(Default)]
struct WidgetScale2 {
    _scale: Option<Scale<'static>>,
}

impl View for WidgetScale2 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let scale = Scale::new(container)?;
        scale.size(lv_pct(80), 100).center();
        scale
            .set_mode(ScaleMode::HorizontalBottom)
            .set_label_show(true)
            .set_total_tick_count(31)
            .set_major_tick_every(5)
            .set_tick_length(Part::Items, 5)
            .set_tick_length(Part::Indicator, 10)
            .set_range(10, 40);

                self._scale = Some(scale);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(WidgetScale2::default());

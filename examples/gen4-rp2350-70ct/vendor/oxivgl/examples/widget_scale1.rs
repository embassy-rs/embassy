#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Scale 1 — Round gauge with tick marks
//!
//! A 270° round scale with labeled major ticks, built via `ScaleBuilder`.

use oxivgl::{
    view::{NavAction, View},
    widgets::{Obj, Scale, ScaleMode, WidgetError},
};

#[derive(Default)]
struct WidgetScale1 {
    _scale: Option<Scale<'static>>,
}

impl View for WidgetScale1 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let scale = Scale::tick_ring(
            container,
            200,
            ScaleMode::RoundInner,
            135,
            270,
            100,
            21,
            5,
            true,
            15,
            8,
            0x333333,
            0x999999,
        )?;

        self._scale = Some(scale);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(WidgetScale1::default());

#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Calendar 2 — Chinese calendar with dropdown header
//!
//! Displays March 2024 with Chinese lunar day names. A dropdown header
//! allows selecting month and year. Requires `LV_USE_CALENDAR_CHINESE`
//! and `LV_FONT_SOURCE_HAN_SANS_SC_14_CJK` enabled in `lv_conf.h`.
//!
//! Requires the btnmatrix text_length patch applied in `oxivgl-sys/build.rs`
//! for correct rendering on ESP32 (LVGL 9.5 bug).

use oxivgl::{
    fonts,
    view::{NavAction, View},
    widgets::{Obj, Align, Calendar, WidgetError},
};

#[derive(Default)]
struct Calendar2 {
    _cal: Option<Calendar<'static>>,
}

impl View for Calendar2 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let cal = Calendar::new(container)?;
        cal.size(320, 240).align(Align::TopLeft, 0, 0);
        cal.set_today_date(2024, 3, 22)
            .set_month_shown(2024, 3);
        cal.set_chinese_mode(true, fonts::SOURCE_HAN_SANS_SC_14_CJK);
        cal.add_header_dropdown();

                self._cal = Some(cal);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(Calendar2::default());

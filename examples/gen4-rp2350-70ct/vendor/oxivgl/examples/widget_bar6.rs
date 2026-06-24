#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Bar 6 — Bar with custom draw value label
//!
//! A bar animates 0→100→0 (4 s each). A `DRAW_MAIN_END` handler draws the
//! current value as text: white inside the indicator when wide, black outside
//! when the indicator is short.

use oxivgl::view::NavAction;
use oxivgl::{
    anim::{anim_set_bar_value, Anim, AnimHandle, ANIM_REPEAT_INFINITE},
    draw::{Area, DrawLabelDscOwned},
    enums::EventCode,
    event::Event,
    style::color_make,
    view::{register_event_on, View},
    widgets::{Obj, Align, Bar, WidgetError},
};

struct WidgetBar6 {
    bar: Option<Bar<'static>>,
    _anim: Option<AnimHandle>,
}

impl WidgetBar6 {
    fn new() -> Self {
        Self {
            bar: None,
            _anim: None,
        }
    }
}

impl View for WidgetBar6 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let bar = Bar::new(container)?;
        bar.set_range_raw(0, 100);
        bar.size(200, 20).center();

        let mut a = Anim::new();
        a.set_var(&bar)
            .set_values(0, 100)
            .set_exec_cb(Some(anim_set_bar_value))
            .set_duration(4000)
            .set_reverse_duration(4000)
            .set_repeat_count(ANIM_REPEAT_INFINITE);
        let handle = a.start();

                self.bar = Some(bar);
        self._anim = Some(handle);
        Ok(())
    }

    fn register_events_on(&mut self, _container: &Obj<'static>) {
        if let Some(ref bar) = self.bar {
            register_event_on(self, bar.handle());
        }
    }

    fn on_event(&mut self, event: &Event) -> NavAction {
        let Some(ref bar) = self.bar else { return NavAction::None };
        if !event.matches(bar, EventCode::DRAW_MAIN_END) {
            return NavAction::None;
        }
        let Some(layer) = event.layer() else { return NavAction::None };
        let value = bar.get_value_raw();
        if value == 0 {
            return NavAction::None;
        }
        let mut buf = heapless::String::<8>::new();
        let _ = core::fmt::Write::write_fmt(&mut buf, format_args!("{}", value));
        let mut dsc = DrawLabelDscOwned::default_font();
        let (txt_w, txt_h) = dsc.text_size(&buf);
        let mut txt_area = Area { x1: 0, y1: 0, x2: txt_w - 1, y2: txt_h - 1 };
        // Simplified: lv_bar_get_indicator_area() not yet wrapped, so approximate
        // the indicator rect from get_coords() + proportional width.
        let mut indic_area = bar.get_coords();
        indic_area.set_width(indic_area.width() * value / 100);
        if indic_area.width() > txt_w + 20 {
            txt_area.align_to_area(indic_area, Align::RightMid, -10, 0);
            dsc.set_color(color_make(255, 255, 255));
        } else {
            txt_area.align_to_area(indic_area, Align::OutRightMid, 10, 0);
            dsc.set_color(color_make(0, 0, 0));
        }
        layer.draw_label(&dsc, txt_area, &buf);
        NavAction::None
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(WidgetBar6::new());

#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Calendar 1 — Month view with highlighted dates and arrow header
//!
//! Displays February 2021 with three highlighted dates. An arrow header
//! allows navigating between months. Clicking a day fires a VALUE_CHANGED
//! event; the label below shows the selected date.

use oxivgl::{
    enums::EventCode,
    event::Event,
    view::{NavAction, View},
    widgets::{Obj, Align, Calendar, CalendarDate, Label, WidgetError},
};

#[derive(Default)]
struct Calendar1 {
    cal: Option<Calendar<'static>>,
    label: Option<Label<'static>>,
}

impl View for Calendar1 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let cal = Calendar::new(container)?;
        cal.size(185, 230).align(Align::Center, 0, 27);
        cal.set_today_date(2021, 2, 23)
            .set_month_shown(2021, 2)
            .set_highlighted_dates(&[
                CalendarDate::new(2021, 2, 6),
                CalendarDate::new(2021, 2, 11),
                CalendarDate::new(2022, 2, 22),
            ]);
        cal.add_header_arrow();
        cal.bubble_events();

        let label = Label::new(container)?;
        label.text("Click a day").align(Align::Center, 0, -90);

                self.cal = Some(cal);
        self.label = Some(label);
        Ok(())
    }

    fn on_event(&mut self, event: &Event) -> NavAction {
        if let Some(ref cal) = self.cal {
            if event.matches(cal, EventCode::VALUE_CHANGED) {
                if let Some(date) = cal.get_pressed_date() {
                    let mut buf = heapless::String::<24>::new();
                    let _ = core::fmt::Write::write_fmt(
                        &mut buf,
                        format_args!("{:02}/{:02}/{}", date.day, date.month, date.year),
                    );
                    if let Some(ref label) = self.label {
                        label.text(&buf);
                    }
                }
            }
        }
        NavAction::None
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(Calendar1::default());

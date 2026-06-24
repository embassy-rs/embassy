#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Chart 6 — Cursor on clicked point
//!
//! A line chart with a cursor crosshair. Clicking a data point moves the
//! cursor to that location. A label above says "Click on a point".

use oxivgl::{
    enums::EventCode,
    event::Event,
    view::{NavAction, View},
    widgets::{Obj, Align, Chart, ChartAxis, ChartCursor, ChartSeries, ChartType, Label, WidgetError},
};

#[derive(Default)]
struct WidgetChart6 {
    chart: Option<Chart<'static>>,
    cursor: Option<ChartCursor>,
    _ser: Option<ChartSeries>,
    _label: Option<Label<'static>>,
}

impl View for WidgetChart6 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let chart = Chart::new(container)?;
        chart.size(200, 150);
        chart.align(Align::Center, 0, 10);
        chart.set_type(ChartType::Line);
        chart.bubble_events();

        let color_blue = oxivgl::style::palette_main(oxivgl::style::Palette::Blue);
        let cursor = chart.add_cursor(color_blue, 0x01 | 0x08); // LEFT | BOTTOM

        let color_red = oxivgl::style::palette_main(oxivgl::style::Palette::Red);
        let ser = chart.add_series(color_red, ChartAxis::PrimaryY);
        for &v in &[32, 58, 17, 73, 45, 82, 29, 65, 41, 70] {
            chart.set_next_value(&ser, v);
        }

        let label = Label::new(container)?;
        label.text("Click on a point");
        label.align(Align::TopMid, 0, 5);

                self.chart = Some(chart);
        self.cursor = Some(cursor);
        self._ser = Some(ser);
        self._label = Some(label);
        Ok(())
    }

    fn on_event(&mut self, event: &Event) -> NavAction {
        if let (Some(chart), Some(cursor)) = (&self.chart, &self.cursor) {
            if event.matches(chart, EventCode::VALUE_CHANGED) {
                if let Some(id) = chart.get_pressed_point() {
                    chart.set_cursor_point(cursor, None, id);
                }
            }
        }
        NavAction::None
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(WidgetChart6::default());

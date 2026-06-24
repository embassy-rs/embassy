#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Table 1 — Scrollable fruit/price table with custom cell drawing.
//!
//! 2-column, 8-row table (Name / Price). A `DRAW_TASK_ADDED` handler:
//! - Header row (row 0): blue-tinted background, center-aligned text.
//! - First column (col 0, non-header): right-aligned text.
//! - Even non-header rows: light grey tint.

use oxivgl::view::NavAction;
use oxivgl::{
    draw::DrawTask,
    enums::EventCode,
    event::Event,
    style::{color_mix, palette_main, Palette},
    view::{register_event_on, View},
    widgets::{Obj, Align, Part, Table, TextAlign, WidgetError},
};

const NAMES: [&str; 8] = ["Name", "Apple", "Banana", "Lemon", "Grape", "Melon", "Peach", "Nuts"];
const PRICES: [&str; 8] = ["Price", "$7", "$4", "$6", "$2", "$5", "$1", "$9"];

const OPA_COVER: u8 = 255;

#[derive(Default)]
struct Table1 {
    table: Option<Table<'static>>,
}

impl Table1 {
    fn handle_draw_task(&self, draw_task: &DrawTask) {
        let base = draw_task.base();
        if base.part != Part::Items {
            return;
        }
        let row = base.id1;
        let col = base.id2;

        if row == 0 {
            if let Some(fill) = draw_task.fill_dsc() {
                // Blue tint at LV_OPA_20 (≈51/255)
                fill.set_color(color_mix(palette_main(Palette::Blue), fill.color(), 51));
                fill.set_opa(OPA_COVER);
            }
            if let Some(label) = draw_task.label_dsc() {
                label.set_align(TextAlign::Center);
            }
        } else if col == 0 {
            if let Some(label) = draw_task.label_dsc() {
                label.set_align(TextAlign::Right);
            }
        }

        if row != 0 && row % 2 == 0 {
            if let Some(fill) = draw_task.fill_dsc() {
                // Grey tint at LV_OPA_10 (≈26/255)
                fill.set_color(color_mix(palette_main(Palette::Grey), fill.color(), 26));
                fill.set_opa(OPA_COVER);
            }
        }
    }
}

impl View for Table1 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {
        let table = Table::new(container)?;

        for (row, (name, price)) in NAMES.iter().zip(PRICES.iter()).enumerate() {
            table.set_cell_value(row as u32, 0, name);
            table.set_cell_value(row as u32, 1, price);
        }

        table.height(200).align(Align::Center, 0, 0);
        table.send_draw_task_events();

                self.table = Some(table);
        Ok(())
    }

    fn register_events_on(&mut self, _container: &Obj<'static>) {
        if let Some(ref table) = self.table {
            register_event_on(self, table.handle());
        }
    }

    fn on_event(&mut self, event: &Event) -> NavAction {
        if event.code() == EventCode::DRAW_TASK_ADDED {
            if let Some(draw_task) = event.draw_task() {
                self.handle_draw_task(&draw_task);
            }
        }
        NavAction::None
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(Table1::default());

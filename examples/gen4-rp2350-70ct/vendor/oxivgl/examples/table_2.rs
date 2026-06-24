#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(target_arch = "xtensa", feature(impl_trait_in_assoc_type, type_alias_impl_trait))]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Table 2 — Lightweight scrollable list of 200 items with toggle state.
//!
//! A single-column 200-row table used as a memory-efficient scrollable list.
//! Each row uses the `CUSTOM_1` cell-ctrl bit as a boolean checked state.
//! - `DRAW_TASK_ADDED`: highlights checked rows with a blue tint.
//! - `VALUE_CHANGED`: toggles the `CUSTOM_1` bit on the selected row.

use oxivgl::view::NavAction;
use oxivgl::{
    draw::DrawTask,
    enums::EventCode,
    event::Event,
    style::{Palette, palette_lighten},
    view::{View, register_event_on},
    widgets::{Obj, Align, Part, Table, TableCellCtrl, WidgetError},
};

const ITEM_CNT: u32 = 200;

#[derive(Default)]
struct Table2 {
    table: Option<Table<'static>>,
}

impl Table2 {
    fn handle_draw_task(&self, draw_task: &DrawTask) {
        let base = draw_task.base();
        if base.part != Part::Items {
            return;
        }
        if let Some(ref table) = self.table {
            if table.has_cell_ctrl(base.id1, 0, TableCellCtrl::CUSTOM_1) {
                if let Some(fill) = draw_task.fill_dsc() {
                    fill.set_color(palette_lighten(Palette::Blue, 3));
                }
            }
        }
    }
}

impl View for Table2 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {
        let table = Table::new(container)?;

        table.set_column_width(0, 200);
        table.set_row_count(ITEM_CNT);
        table.set_column_count(1);

        for i in 0..ITEM_CNT {
            let n = i + 1;
            let mut buf = [0u8; 12];
            let prefix = b"Item ";
            buf[..prefix.len()].copy_from_slice(prefix);
            let num_len = u32_to_str(n, &mut buf[prefix.len()..]);
            let total = prefix.len() + num_len;
            let text = core::str::from_utf8(&buf[..total]).unwrap_or("Item");
            table.set_cell_value(i, 0, text);
        }

        table.align(Align::Center, 0, 0);
        table.height(220);
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
        } else if event.code() == EventCode::VALUE_CHANGED {
            if let Some(ref table) = self.table {
                if let Some((row, _col)) = table.get_selected_cell() {
                    if table.has_cell_ctrl(row, 0, TableCellCtrl::CUSTOM_1) {
                        table.clear_cell_ctrl(row, 0, TableCellCtrl::CUSTOM_1);
                    } else {
                        table.set_cell_ctrl(row, 0, TableCellCtrl::CUSTOM_1);
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

/// Write decimal representation of `n` into `buf`. Returns bytes written.
fn u32_to_str(mut n: u32, buf: &mut [u8]) -> usize {
    if n == 0 {
        if !buf.is_empty() {
            buf[0] = b'0';
        }
        return 1;
    }
    let mut tmp = [0u8; 10];
    let mut len = 0;
    while n > 0 {
        tmp[len] = b'0' + (n % 10) as u8;
        n /= 10;
        len += 1;
    }
    let out = len.min(buf.len());
    for i in 0..out {
        buf[i] = tmp[len - 1 - i];
    }
    out
}

oxivgl_examples_common::example_main!(Table2::default());

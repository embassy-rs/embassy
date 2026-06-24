#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Event Draw — Custom draw task with growing circle
//!
//! A 200×200 container handles `DRAW_TASK_ADDED` events. `update()` cycles a
//! size counter 0→50→0. The draw handler draws a filled circle (RADIUS_CIRCLE)
//! that grows and shrinks, centered in the container.

use oxivgl::view::NavAction;
use oxivgl::{
    draw::{Area, DrawRectDsc, RADIUS_CIRCLE},
    enums::EventCode,
    event::Event,
    style::color_make,
    view::{register_event_on, View},
    widgets::{Align, Obj, Part, WidgetError},
};

#[derive(Default)]
struct EventDraw {
    cont: Option<Obj<'static>>,
    size: i32,
    size_dec: bool,
}

impl View for EventDraw {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let cont = Obj::new(container)?;
        cont.size(200, 200).center();
        cont.send_draw_task_events();

                self.cont = Some(cont);
        self.size = 0;
        self.size_dec = false;
        Ok(())
    }

    fn register_events_on(&mut self, _container: &Obj<'static>) {
        if let Some(ref cont) = self.cont { register_event_on(self, cont.handle()); }
    }

    fn on_event(&mut self, event: &Event) -> NavAction {
        let Some(ref cont) = self.cont else { return NavAction::None };
        if !event.matches(cont, EventCode::DRAW_TASK_ADDED) {
            return NavAction::None;
        }
        let Some(task) = event.draw_task() else { return NavAction::None };
        let base = task.base();
        if base.part != Part::Main {
            return NavAction::None;
        }
        let Some(layer) = task.layer() else { return NavAction::None };
        let obj_coords = cont.get_coords();
        let mut a = Area { x1: 0, y1: 0, x2: self.size, y2: self.size };
        a.align_to_area(obj_coords, Align::Center, 0, 0);
        let mut dsc = DrawRectDsc::new();
        dsc.bg_color(color_make(255, 170, 170))
            .radius(RADIUS_CIRCLE)
            .border_color(color_make(255, 85, 85))
            .border_width(2)
            .outline_color(color_make(255, 0, 0))
            .outline_width(2)
            .outline_pad(3);
        layer.draw_rect(&dsc, a);
        NavAction::None
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        if self.size_dec {
            self.size -= 1;
            if self.size <= 0 {
                self.size_dec = false;
            }
        } else {
            self.size += 1;
            if self.size >= 50 {
                self.size_dec = true;
            }
        }
        if let Some(ref cont) = self.cont {
            cont.invalidate();
        }
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(EventDraw::default());

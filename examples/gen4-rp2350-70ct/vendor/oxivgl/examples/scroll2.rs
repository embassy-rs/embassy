#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Scroll 2 — Scroll snap with center alignment
//!
//! A horizontal row of buttons snaps to center. Panel 3 is marked non-snappable.
//! A switch toggles "scroll one" mode.

use oxivgl::{
    enums::{EventCode, ObjFlag, ObjState, ScrollSnap},
    event::Event,
    layout::FlexFlow,
    view::{NavAction, View},
    widgets::{Align, Button, Label, Obj, Switch, WidgetError},
};

#[derive(Default)]
struct Scroll2 {
    panel: Option<Obj<'static>>,
    sw: Option<Switch<'static>>,
    _btns: heapless::Vec<Button<'static>, 10>,
    _labels: heapless::Vec<Label<'static>, 12>,
}

impl View for Scroll2 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let panel = Obj::new(container)?;
        panel
            .size(280, 120)
            .set_scroll_snap_x(ScrollSnap::Center)
            .set_flex_flow(FlexFlow::Row)
            .align(Align::Center, 0, 20);

        let mut btns = heapless::Vec::<Button<'static>, 10>::new();
        let mut labels = heapless::Vec::<Label<'static>, 12>::new();

        for i in 0u32..10 {
            let btn = Button::new(&panel)?;
            btn.size(150, oxivgl::style::lv_pct(100));

            let label = Label::new(&btn)?;
            if i == 3 {
                let mut buf = heapless::String::<20>::new();
                let _ = core::fmt::Write::write_fmt(&mut buf, format_args!("Panel {}\nno snap", i));
                label.text(&buf).center();
                btn.remove_flag(ObjFlag::SNAPPABLE);
            } else {
                let mut buf = heapless::String::<10>::new();
                let _ = core::fmt::Write::write_fmt(&mut buf, format_args!("Panel {}", i));
                label.text(&buf).center();
            }

            let _ = btns.push(btn);
            let _ = labels.push(label);
        }

        panel.update_snap(true);

        // Switch to toggle "one scroll" mode
        let sw = Switch::new(container)?;
        sw.align(Align::TopRight, -20, 10);
        sw.bubble_events();

        let sw_label = Label::new(container)?;
        sw_label
            .text("One scroll")
            .align_to(&sw, Align::OutBottomMid, 0, 5);
        let _ = labels.push(sw_label);

                self.panel = Some(panel);
        self.sw = Some(sw);
        self._btns = btns;
        self._labels = labels;
        Ok(())
    }

    fn on_event(&mut self, event: &Event) -> NavAction {
        if let (Some(sw), Some(panel)) = (&self.sw, &self.panel) {
            if event.matches(sw, EventCode::VALUE_CHANGED) {
                if sw.has_state(ObjState::CHECKED) {
                    panel.add_flag(ObjFlag::SCROLL_ONE);
                } else {
                    panel.remove_flag(ObjFlag::SCROLL_ONE);
                }
            }
        }
        NavAction::None
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(Scroll2::default());

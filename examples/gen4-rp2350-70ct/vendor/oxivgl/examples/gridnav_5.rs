#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Gridnav 5 — Single-axis grid navigation with slider↔roller sync
//!
//! Top container (flex column): 3 sliders with VERTICAL_MOVE_ONLY gridnav.
//! Bottom container (flex row): 3 rollers with HORIZONTAL_MOVE_ONLY gridnav.
//!
//! On every KEY event, slider values are synced to the matching roller and
//! vice-versa, so both controls always reflect the same selection.

use oxivgl::view::NavAction;
use oxivgl::{
    enums::EventCode,
    event::Event,
    gridnav::{GridnavCtrl, gridnav_add},
    group::{Group, group_remove_obj},
    layout::{FlexAlign, FlexFlow},
    style::lv_pct,
    view::{View, register_event_on},
    widgets::{Align, Obj, Roller, RollerMode, Slider, WidgetError},
};

#[derive(Default)]
struct Gridnav5 {
    _group: Option<Group>,
    sliders: heapless::Vec<Slider<'static>, 3>,
    rollers: heapless::Vec<Roller<'static>, 3>,
    _cont_top: Option<Obj<'static>>,
    _cont_bot: Option<Obj<'static>>,
}

/// Roller options and matching slider ranges for each of the three columns.
const OPTS: [&str; 3] = [
    "0\n1\n2\n3\n4\n5",
    "0\n1\n2\n3\n4\n5\n6\n7\n8\n9",
    "s\nm\nh",
];
const COUNTS: [i32; 3] = [6, 10, 3];

impl View for Gridnav5 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        // ── Top container: sliders, vertical-move-only ────────────────────
        let cont_top = Obj::new(container)?;
        cont_top
            .set_flex_flow(FlexFlow::Column)
            .set_flex_align(FlexAlign::Center, FlexAlign::Center, FlexAlign::Center)
            .size(lv_pct(100), lv_pct(50))
            .align(Align::TopMid, 0, 0);
        gridnav_add(&cont_top, GridnavCtrl::VERTICAL_MOVE_ONLY);

        let mut sliders: heapless::Vec<Slider<'static>, 3> = heapless::Vec::new();
        for i in 0usize..3 {
            let sl = Slider::new(&cont_top)?;
            sl.set_range(0, COUNTS[i] - 1).width(lv_pct(85));
            group_remove_obj(&sl);
            let _ = sliders.push(sl);
        }

        // ── Bottom container: rollers, horizontal-move-only ───────────────
        let cont_bot = Obj::new(container)?;
        cont_bot
            .set_flex_flow(FlexFlow::Row)
            .set_flex_align(FlexAlign::Center, FlexAlign::Center, FlexAlign::Center)
            .size(lv_pct(100), lv_pct(50))
            .align(Align::BottomMid, 0, 0);
        gridnav_add(&cont_bot, GridnavCtrl::HORIZONTAL_MOVE_ONLY);

        let mut rollers: heapless::Vec<Roller<'static>, 3> = heapless::Vec::new();
        for i in 0usize..3 {
            let ro = Roller::new(&cont_bot)?;
            ro.set_options(OPTS[i], RollerMode::Infinite)
                .size(lv_pct(30), lv_pct(100));
            group_remove_obj(&ro);
            let _ = rollers.push(ro);
        }

        // ── Group: only containers ────────────────────────────────────────
        let group = Group::new()?;
        group.set_default();
        group.add_obj(&cont_top);
        group.add_obj(&cont_bot);
        group.assign_to_keyboard_indevs();

        self._group = Some(group);
        self.sliders = sliders;
        self.rollers = rollers;
        self._cont_top = Some(cont_top);
        self._cont_bot = Some(cont_bot);
        Ok(())
    }

    fn register_events_on(&mut self, _container: &Obj<'static>) {
        if let Some(ref ct) = self._cont_top { register_event_on(self, ct.handle()); }
        if let Some(ref cb) = self._cont_bot { register_event_on(self, cb.handle()); }
    }

    fn on_event(&mut self, event: &Event) -> NavAction {
        if event.code() != EventCode::KEY {
            return NavAction::None;
        }
        let target = event.target();
        let target_handle = target.handle();
        let ct_handle = self._cont_top.as_ref().map(|c| c.handle());
        let cb_handle = self._cont_bot.as_ref().map(|c| c.handle());
        if Some(target_handle) == ct_handle {
            for i in 0..3 {
                let val = self.sliders[i].get_value() as u32;
                self.rollers[i].set_selected(val, true);
            }
        } else if Some(target_handle) == cb_handle {
            for i in 0..3 {
                let sel = self.rollers[i].get_selected() as i32;
                self.sliders[i].set_value_animated(sel, true);
            }
        }
        NavAction::None
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(Gridnav5::default());

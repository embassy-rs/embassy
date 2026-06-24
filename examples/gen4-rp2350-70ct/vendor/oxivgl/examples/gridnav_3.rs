#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Gridnav 3 — Nested grid navigations
//!
//! A main container (ROLLOVER | SCROLL_FIRST) holds two buttons and two
//! sub-containers. `cont_sub1` contains a long scrollable text. `cont_sub2`
//! has its own ROLLOVER gridnav with two buttons; pressing ENTER focuses it,
//! ESC moves group focus to the next member.
//!
//! Only the containers are in the group; buttons are removed so gridnav
//! controls focus inside each container.

use oxivgl::view::NavAction;
use oxivgl::{
    enums::{EventCode, Key, ObjState},
    event::Event,
    gridnav::{GridnavCtrl, gridnav_add},
    group::{Group, group_remove_obj},
    layout::FlexFlow,
    style::{LV_SIZE_CONTENT, Palette, Style, lv_pct, palette_lighten},
    view::{View, register_event_on},
    widgets::{Button, Label, Obj, WidgetError},
};

#[derive(Default)]
struct Gridnav3 {
    _group: Option<Group>,
    // containers kept alive
    _cont_main: Option<Obj<'static>>,
    _cont_sub1: Option<Obj<'static>>,
    cont_sub2: Option<Obj<'static>>,
    // child widgets kept alive
    _btn1: Option<Button<'static>>,
    _btn1_lbl: Option<Label<'static>>,
    _btn2: Option<Button<'static>>,
    _btn2_lbl: Option<Label<'static>>,
    _sub1_lbl: Option<Label<'static>>,
    _sub2_hint_lbl: Option<Label<'static>>,
    _btn3: Option<Button<'static>>,
    _btn3_lbl: Option<Label<'static>>,
    _btn4: Option<Button<'static>>,
    _btn4_lbl: Option<Label<'static>>,
}

impl View for Gridnav3 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        // Shared focused-background styles (one per color, reused across
        // containers for memory efficiency).
        let focus_blue = Style::new(|s| {
            s.bg_color(palette_lighten(Palette::Blue, 5));
        });
        let focus_red = Style::new(|s| {
            s.bg_color(palette_lighten(Palette::Red, 5));
        });

        // ── Main container ────────────────────────────────────────────────
        let cont_main = Obj::new(container)?;
        cont_main
            .set_flex_flow(FlexFlow::RowWrap)
            .size(lv_pct(80), LV_SIZE_CONTENT);
        cont_main.add_style(&focus_blue, ObjState::FOCUSED);

        gridnav_add(&cont_main, GridnavCtrl::ROLLOVER | GridnavCtrl::SCROLL_FIRST);

        let group = Group::new()?;
        group.set_default();
        group.add_obj(&cont_main);
        group.assign_to_keyboard_indevs();

        // ── Two plain buttons in main (not in group) ──────────────────────
        let btn1 = Button::new(&cont_main)?;
        group_remove_obj(&btn1);
        let btn1_lbl = Label::new(&btn1)?;
        btn1_lbl.text("Button 1");

        let btn2 = Button::new(&cont_main)?;
        group_remove_obj(&btn2);
        let btn2_lbl = Label::new(&btn2)?;
        btn2_lbl.text("Button 2");

        // ── Sub-container 1: long scrollable text ────────────────────────
        let cont_sub1 = Obj::new(&cont_main)?;
        cont_sub1.size(lv_pct(100), 100);
        cont_sub1.add_style(&focus_red, ObjState::FOCUSED);

        let sub1_lbl = Label::new(&cont_sub1)?;
        sub1_lbl.width(lv_pct(100));
        sub1_lbl.text(
            "I'm a very long text which is makes my container scrollable. \
             As LV_GRIDNAV_FLAG_SCROLL_FIRST is enabled arrow will scroll me first \
             and a new objects will be focused only when an edge is reached with the scrolling.\n\n\
             This is only some placeholder text to be sure the parent will be scrollable. \n\n\
             Hello world!\n\
             Hello world!\n\
             Hello world!\n\
             Hello world!\n\
             Hello world!\n\
             Hello world!",
        );

        // ── Sub-container 2: nested gridnav, ENTER/ESC focus control ─────
        let cont_sub2 = Obj::new(&cont_main)?;
        cont_sub2
            .set_flex_flow(FlexFlow::RowWrap)
            .size(lv_pct(100), LV_SIZE_CONTENT);
        cont_sub2.add_style(&focus_red, ObjState::FOCUSED);

        gridnav_add(&cont_sub2, GridnavCtrl::ROLLOVER);
        group.add_obj(&cont_sub2);

        let sub2_hint_lbl = Label::new(&cont_sub2)?;
        sub2_hint_lbl.text("Use ENTER/ESC to focus/defocus this container");
        sub2_hint_lbl.width(lv_pct(100));

        let btn3 = Button::new(&cont_sub2)?;
        group_remove_obj(&btn3);
        let btn3_lbl = Label::new(&btn3)?;
        btn3_lbl.text("Button 3");

        let btn4 = Button::new(&cont_sub2)?;
        group_remove_obj(&btn4);
        let btn4_lbl = Label::new(&btn4)?;
        btn4_lbl.text("Button 4");

        self._group = Some(group);
        self._cont_main = Some(cont_main);
        self._cont_sub1 = Some(cont_sub1);
        self.cont_sub2 = Some(cont_sub2);
        self._btn1 = Some(btn1);
        self._btn1_lbl = Some(btn1_lbl);
        self._btn2 = Some(btn2);
        self._btn2_lbl = Some(btn2_lbl);
        self._sub1_lbl = Some(sub1_lbl);
        self._sub2_hint_lbl = Some(sub2_hint_lbl);
        self._btn3 = Some(btn3);
        self._btn3_lbl = Some(btn3_lbl);
        self._btn4 = Some(btn4);
        self._btn4_lbl = Some(btn4_lbl);
        Ok(())
    }

    fn register_events_on(&mut self, _container: &Obj<'static>) {
        // Receive KEY events from cont_sub2 (no bubble needed — direct registration).
        if let Some(ref cont_sub2) = self.cont_sub2 {
            register_event_on(self, cont_sub2.handle());
        }
    }

    fn on_event(&mut self, event: &Event) -> NavAction {
        if event.code() != EventCode::KEY {
            return NavAction::None;
        }
        // Only handle events from cont_sub2.
        if let Some(ref cont_sub2) = self.cont_sub2 {
            if event.target_handle() != cont_sub2.handle() {
                return NavAction::None;
            }
            match event.key() {
                Some(Key::ENTER) => {
                    if let Some(ref group) = self._group {
                        group.focus_obj(cont_sub2);
                    }
                }
                Some(Key::ESC) => {
                    if let Some(ref group) = self._group {
                        group.focus_next();
                    }
                }
                _ => {}
            }
        }
        NavAction::None
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(Gridnav3::default());

#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Gridnav 1 — Grid navigation with keyboard focus
//!
//! Two containers side-by-side. Left container has 10 checkable buttons in a
//! row-wrap flex layout (no rollover). Right container has a textarea,
//! checkbox, and two switches arranged manually (rollover enabled).
//! Arrow keys move focus between children; Tab switches between containers.

use oxivgl::{
    enums::{ObjFlag, ObjState},
    gridnav::{GridnavCtrl, gridnav_add},
    group::{Group, group_remove_obj},
    layout::FlexFlow,
    style::{LV_SIZE_CONTENT, Palette, Style, lv_pct, palette_lighten},
    view::{NavAction, View},
    widgets::{Align, Button, Checkbox, Label, Obj, Switch, Textarea, WidgetError},
};

#[derive(Default)]
struct Gridnav1 {
    _group: Option<Group>,
    _label1: Option<Label<'static>>,
    _buttons: heapless::Vec<Button<'static>, 10>,
    _btn_labels: heapless::Vec<Label<'static>, 10>,
    _label2: Option<Label<'static>>,
    _ta: Option<Textarea<'static>>,
    _cb: Option<Checkbox<'static>>,
    _sw1: Option<Switch<'static>>,
    _sw2: Option<Switch<'static>>,
    // containers kept alive
    _cont1: Option<oxivgl::widgets::Obj<'static>>,
    _cont2: Option<oxivgl::widgets::Obj<'static>>,
}

impl View for Gridnav1 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        // Shared focused-background style for both containers.
        let focused_bg = Style::new(|s| {
            s.bg_color(palette_lighten(Palette::Blue, 5));
        });

        // ── Container 1: buttons, no rollover ──────────────────────────────
        let cont1 = oxivgl::widgets::Obj::new(container)?;
        cont1
            .set_flex_flow(FlexFlow::RowWrap)
            .size(lv_pct(50), lv_pct(100));
        cont1.add_style(&focused_bg, ObjState::FOCUSED);

        gridnav_add(&cont1, GridnavCtrl::NONE);

        let label1 = Label::new(&cont1)?;
        label1.text("No rollover");

        let mut buttons: heapless::Vec<Button<'static>, 10> = heapless::Vec::new();
        let mut btn_labels: heapless::Vec<Label<'static>, 10> = heapless::Vec::new();
        for i in 0u32..10 {
            let btn = Button::new(&cont1)?;
            btn.size(70, LV_SIZE_CONTENT);
            btn.add_flag(ObjFlag::CHECKABLE);
            group_remove_obj(&btn);

            let lbl = Label::new(&btn)?;
            // Format button index as text; heapless label buffer fits small numbers.
            lbl.text(match i {
                0 => "0", 1 => "1", 2 => "2", 3 => "3", 4 => "4",
                5 => "5", 6 => "6", 7 => "7", 8 => "8", _ => "9",
            });
            lbl.center();

            // heapless::Vec::push returns Err if full; capacity is 10 == loop count.
            let _ = buttons.push(btn);
            let _ = btn_labels.push(lbl);
        }

        // ── Container 2: textarea/checkbox/switches, rollover ──────────────
        let cont2 = oxivgl::widgets::Obj::new(container)?;
        cont2
            .size(lv_pct(50), lv_pct(100))
            .align(Align::RightMid, 0, 0);
        cont2.add_style(&focused_bg, ObjState::FOCUSED);

        gridnav_add(&cont2, GridnavCtrl::ROLLOVER);

        let label2 = Label::new(&cont2)?;
        label2.width(lv_pct(100));
        label2.text("Rollover\nUse tab to focus the other container");

        let ta = Textarea::new(&cont2)?;
        ta.size(lv_pct(100), 80).pos(0, 80);
        group_remove_obj(&ta);

        let cb = Checkbox::new(&cont2)?;
        cb.pos(0, 170);
        group_remove_obj(&cb);

        let sw1 = Switch::new(&cont2)?;
        sw1.pos(0, 200);
        group_remove_obj(&sw1);

        let sw2 = Switch::new(&cont2)?;
        sw2.pos(lv_pct(50), 200);
        group_remove_obj(&sw2);

        // ── Group: only containers ─────────────────────────────────────────
        let group = Group::new()?;
        group.set_default();
        group.add_obj(&cont1);
        group.add_obj(&cont2);
        group.assign_to_keyboard_indevs();

        self._group = Some(group);
        self._label1 = Some(label1);
        self._buttons = buttons;
        self._btn_labels = btn_labels;
        self._label2 = Some(label2);
        self._ta = Some(ta);
        self._cb = Some(cb);
        self._sw1 = Some(sw1);
        self._sw2 = Some(sw2);
        self._cont1 = Some(cont1);
        self._cont2 = Some(cont2);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(Gridnav1::default());

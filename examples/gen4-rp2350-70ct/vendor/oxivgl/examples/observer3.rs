#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Observer 3 — Time picker with subject groups
//!
//! Four integer subjects (hour, minute, format, AM/PM) grouped into one.
//! Clicking "Set" opens a settings panel with rollers and dropdowns bound
//! to the subjects. Clicking close destroys the panel — subjects persist
//! across create/delete cycles, preserving widget values.
//!
//! **Minor simplification**: The time label is formatted by polling in
//! `update()` rather than via a group observer callback (would need a
//! safe group-aware callback API not yet implemented).

use core::ffi::c_void;
use core::fmt::Write as _;
use core::ptr::null_mut;

use heapless::String as HString;
use oxivgl::{
    enums::{EventCode, ObjFlag, ObjState},
    event::Event,
    fonts::MONTSERRAT_30,
    style::{LV_SIZE_CONTENT, Selector, Style, lv_pct},
    view::{NavAction, View},
    widgets::{Align, AsLvHandle, Button, Child, Dropdown, Label, Obj, Roller, RollerMode, Screen, Subject, WidgetError},
};

const TIME_FORMAT_12: i32 = 0;
const TIME_FORMAT_24: i32 = 1;
const TIME_AM: i32 = 0;

const HOUR12_OPTIONS: &str =
    "01\n02\n03\n04\n05\n06\n07\n08\n09\n10\n11\n12";
const HOUR24_OPTIONS: &str =
    "00\n01\n02\n03\n04\n05\n06\n07\n08\n09\n10\n11\n12\
     \n13\n14\n15\n16\n17\n18\n19\n20\n21\n22\n23";
const MINUTE_OPTIONS: &str =
    "00\n01\n02\n03\n04\n05\n06\n07\n08\n09\n10\n11\n12\n13\n14\n15\
     \n16\n17\n18\n19\n20\n21\n22\n23\n24\n25\n26\n27\n28\n29\n30\
     \n31\n32\n33\n34\n35\n36\n37\n38\n39\n40\n41\n42\n43\n44\n45\
     \n46\n47\n48\n49\n50\n51\n52\n53\n54\n55\n56\n57\n58\n59";

#[derive(Default)]
struct Observer3 {
    time_label: Option<Label<'static>>,
    set_btn: Option<Button<'static>>,
    panel: Option<Obj<'static>>,
    hour_roller: Option<Child<Roller<'static>>>,
    close_btn_handle: *mut c_void,
    last_format: i32,
    _time_label_style: Option<Style>,
    // Subjects last — drop after widgets so observers removed before deinit.
    hour_subject: Option<Subject>,
    minute_subject: Option<Subject>,
    format_subject: Option<Subject>,
    am_pm_subject: Option<Subject>,
    _time_subject: Option<Subject>,
}

impl View for Observer3 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        // Subjects.
        let hour_subject = Subject::new_int(7);
        let minute_subject = Subject::new_int(45);
        let format_subject = Subject::new_int(TIME_FORMAT_12);
        let am_pm_subject = Subject::new_int(TIME_AM);
        let time_subject = Subject::new_group(&[
            &hour_subject,
            &minute_subject,
            &format_subject,
            &am_pm_subject,
        ]);

        // Time display label.
        let time_label = Label::new(container)?;
        let time_label_style = Style::new(|s| {
            s.text_font(MONTSERRAT_30);
        });
        time_label.add_style(&time_label_style, Selector::DEFAULT);
        time_label.pos(24, 24);

        // Set button — opens the settings panel.
        let set_btn = Button::new(container)?;
        set_btn.pos(180, 24);
        let set_lbl = Child::new(Label::new(&set_btn)?);
        set_lbl.text("Set").center();

        // Update subjects to show 9:30 PM (matches C original).
        hour_subject.set_int(9);
        minute_subject.set_int(30);
        am_pm_subject.set_int(1); // PM

        self.time_label = Some(time_label);
        self.set_btn = Some(set_btn);
        self.panel = None;
        self.hour_roller = None;
        self.close_btn_handle = null_mut();
        self.last_format = TIME_FORMAT_12;
        self._time_label_style = Some(time_label_style);
        self.hour_subject = Some(hour_subject);
        self.minute_subject = Some(minute_subject);
        self.format_subject = Some(format_subject);
        self.am_pm_subject = Some(am_pm_subject);
        self._time_subject = Some(time_subject);
        Ok(())
    }

    fn on_event(&mut self, event: &Event) -> NavAction {
        // Set button clicked — open settings panel.
        let set_btn_match = if let Some(ref btn) = self.set_btn {
            event.matches(btn, EventCode::CLICKED)
        } else {
            false
        };
        if set_btn_match && self.panel.is_none() {
            if let Some(ref btn) = self.set_btn {
                btn.add_state(ObjState::DISABLED);
            }

            let screen = match Screen::active() {
                Some(s) => s,
                None => return NavAction::None,
            };

            let cont = match Obj::new(&screen) {
                Ok(o) => o,
                Err(_) => return NavAction::None,
            };
            cont.size(lv_pct(100), LV_SIZE_CONTENT)
                .align(Align::BottomMid, 0, 0);

            // Hour roller — options updated on format change.
            let hour_roller = match Roller::new(&cont) {
                Ok(r) => Child::new(r),
                Err(_) => return NavAction::None,
            };
            hour_roller
                .add_flag(ObjFlag::FLEX_IN_NEW_TRACK)
                .pos(0, 0);
            hour_roller
                .set_options(HOUR12_OPTIONS, RollerMode::Normal)
                .set_visible_row_count(3);
            if let Some(ref subj) = self.hour_subject { hour_roller.bind_value(subj); }

            // Minute roller.
            let min_roller = match Roller::new(&cont) {
                Ok(r) => Child::new(r),
                Err(_) => return NavAction::None,
            };
            min_roller
                .set_options(MINUTE_OPTIONS, RollerMode::Normal)
                .set_visible_row_count(3)
                .pos(64, 0);
            if let Some(ref subj) = self.minute_subject { min_roller.bind_value(subj); }

            // Format dropdown (12/24).
            let format_dd = match Dropdown::new(&cont) {
                Ok(d) => Child::new(d),
                Err(_) => return NavAction::None,
            };
            format_dd.set_options("12\n24").pos(128, 0).size(80, 40);
            if let Some(ref subj) = self.format_subject { format_dd.bind_value(subj); }

            // AM/PM dropdown — disabled in 24-hour mode.
            let ampm_dd = match Dropdown::new(&cont) {
                Ok(d) => Child::new(d),
                Err(_) => return NavAction::None,
            };
            ampm_dd
                .set_options("am\npm")
                .pos(128, 48)
                .size(80, 40);
            if let Some(ref subj) = self.am_pm_subject { ampm_dd.bind_value(subj); }
            if let Some(ref subj) = self.format_subject { ampm_dd.bind_state_if_eq(subj, ObjState::DISABLED, TIME_FORMAT_24); }

            // Close button — bubbles CLICKED to screen for on_event matching.
            let close_btn = match Button::new(&cont) {
                Ok(b) => Child::new(b),
                Err(_) => return NavAction::None,
            };
            close_btn.align(Align::TopRight, 0, 0).bubble_events();
            let close_lbl = match Label::new(&*close_btn) {
                Ok(l) => Child::new(l),
                Err(_) => return NavAction::None,
            };
            close_lbl.text("X");

            self.close_btn_handle = close_btn.lv_handle() as *mut c_void;
            self.hour_roller = Some(hour_roller);
            self.panel = Some(cont);
        }

        // Close button clicked — destroy panel.
        if !self.close_btn_handle.is_null()
            && event.target_handle() as *mut c_void == self.close_btn_handle
            && event.code() == EventCode::CLICKED
        {
            self.close_btn_handle = null_mut();
            // Clear hour_roller before dropping panel — Child::drop is a no-op
            // but the pointer becomes dangling once LVGL cascade-deletes.
            self.hour_roller = None;
            self.panel = None;
            if let Some(ref btn) = self.set_btn { btn.remove_state(ObjState::DISABLED); }
        }
        NavAction::None
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        // Format time label by polling subjects.
        let hour = self.hour_subject.as_ref().map(|s| s.get_int()).unwrap_or(0);
        let minute = self.minute_subject.as_ref().map(|s| s.get_int()).unwrap_or(0);
        let format = self.format_subject.as_ref().map(|s| s.get_int()).unwrap_or(0);
        let am_pm = self.am_pm_subject.as_ref().map(|s| s.get_int()).unwrap_or(0);

        let mut buf: HString<32> = HString::new();
        if format == TIME_FORMAT_24 {
            let _ = write!(buf, "{}:{:02}", hour, minute);
        } else {
            let suffix = if am_pm == TIME_AM { "am" } else { "pm" };
            let _ = write!(buf, "{:02}:{:02} {}", hour + 1, minute, suffix);
        }
        if let Some(ref label) = self.time_label { label.text(&buf); }

        // Swap hour roller options when 12/24-hour format changes.
        if format != self.last_format {
            self.last_format = format;
            if let Some(ref roller) = self.hour_roller {
                let prev = roller.get_selected();
                if format == TIME_FORMAT_12 {
                    // 24→12: shift selected index, clamp to 0–11.
                    let new_sel = if prev == 0 { 11 } else { (prev - 1) % 12 };
                    roller.set_options(HOUR12_OPTIONS, RollerMode::Normal);
                    roller.set_selected(new_sel, false);
                } else {
                    // 12→24: shift selected index.
                    let new_sel = (prev + 1) % 24;
                    roller.set_options(HOUR24_OPTIONS, RollerMode::Normal);
                    roller.set_selected(new_sel, false);
                }
            }
        }

        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(Observer3::default());

#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Observer 4 — Tabbed interface with indicator animation
//!
//! Three-tab footer with an animated indicator bar that slides to the active
//! tab. Each tab shows different content: sliders (tab 0), dropdowns (tab 1),
//! or rollers (tab 2), each bound to their own subject.
//!
//! **Simplification**: Content transitions are instant (clean + recreate)
//! instead of animated, because the safe API does not yet expose
//! `set_completed_cb` for auto-deleting animated widgets. The indicator slide
//! animation IS preserved.

use core::ffi::c_void;
use core::ptr::null_mut;

use oxivgl::{
    anim::{Anim, anim_path_ease_in_out, anim_set_x},
    enums::{EventCode, ObjFlag, ObjState, ScrollDir},
    event::Event,
    layout::{FlexAlign, FlexFlow},
    style::{Selector, Style, lv_pct},
    view::{NavAction, View},
    widgets::{Align, AsLvHandle, Button, Child, Dropdown, Label, Obj, Roller, RollerMode, Slider, Subject, WidgetError},
};

const DROPDOWN_OPTIONS: &str = "Red\nGreen\nBlue";
const ROLLER_OPTIONS: &str = "Alpha\nBeta\nGamma\nDelta\nEpsilon";

#[derive(Default)]
struct Observer4 {
    // Widgets stored before subjects so subjects drop last.
    _main_cont: Option<Obj<'static>>,
    cont: Option<Obj<'static>>,
    footer: Option<Obj<'static>>,
    indicator: Option<Obj<'static>>,
    btn_handles: [*mut c_void; 3],
    last_tab: i32,

    // Subjects — drop after widgets so observers are removed before deinit.
    tab_subject: Option<Subject>,
    slider_subjects: Option<[Subject; 4]>,
    dropdown_subjects: Option<[Subject; 3]>,
    roller_subjects: Option<[Subject; 2]>,
}

impl View for Observer4 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        // Subjects.
        let tab_subject = Subject::new_int(0);
        let slider_subjects = [
            Subject::new_int(25),
            Subject::new_int(50),
            Subject::new_int(75),
            Subject::new_int(40),
        ];
        let dropdown_subjects = [
            Subject::new_int(0),
            Subject::new_int(1),
            Subject::new_int(2),
        ];
        let roller_subjects = [
            Subject::new_int(0),
            Subject::new_int(2),
        ];

        // Shared styles.
        let pad0_style = Style::new(|s| {
            s.pad_all(0);
        });
        // pad_all(8) is shared by the content area and footer.
        let pad8_style = Style::new(|s| {
            s.pad_all(8);
        });
        let btn_radius_style = Style::new(|s| {
            s.radius(0);
        });
        let indicator_style = Style::new(|s| {
            s.bg_opa(102); // 40% of 255 ≈ 102
        });

        // Main container — full screen, flex column.
        let main_cont = Obj::new(container)?;
        main_cont
            .remove_style_all()
            .size(lv_pct(100), lv_pct(100))
            .set_flex_flow(FlexFlow::Column);
        main_cont.add_style(&pad0_style, Selector::DEFAULT);

        // Content area — flex-grow 1, scrollable vertically.
        let cont = Obj::new(&main_cont)?;
        cont.remove_style_all()
            .set_flex_grow(1)
            .width(lv_pct(100))
            .set_scroll_dir(ScrollDir::VER);
        cont.add_style(&pad8_style, Selector::DEFAULT);

        // Footer — 60px tall, row layout with buttons.
        let footer = Obj::new(&main_cont)?;
        footer
            .remove_style_all()
            .size(lv_pct(100), 60)
            .set_flex_flow(FlexFlow::Row)
            .set_flex_align(FlexAlign::Center, FlexAlign::Center, FlexAlign::Center);
        let footer_pad_column_style = Style::new(|s| {
            s.pad_column(8);
        });
        footer.add_style(&footer_pad_column_style, Selector::DEFAULT);
        footer.add_style(&pad8_style, Selector::DEFAULT);

        // Three tab buttons.
        let btn_labels = ["First", "Second", "Third"];
        let mut btn_handles: [*mut c_void; 3] = [null_mut(); 3];

        for (i, lbl_text) in btn_labels.iter().enumerate() {
            let btn = Child::new(Button::new(&footer)?);
            btn.set_flex_grow(1)
                .height(lv_pct(100))
                .bubble_events();
            btn.add_style(&btn_radius_style, Selector::DEFAULT);
            btn.bind_state_if_eq(&tab_subject, ObjState::CHECKED, i as i32);

            let lbl = Child::new(Label::new(&*btn)?);
            lbl.text(lbl_text).center();

            btn_handles[i] = btn.lv_handle() as *mut c_void;
        }

        // Indicator bar — 10px tall, 40% bg opacity, outside flex layout.
        let indicator = Obj::new(&footer)?;
        indicator
            .remove_style(None, Selector::DEFAULT)
            .height(10)
            .align(Align::BottomLeft, 0, 0)
            .add_flag(ObjFlag::IGNORE_LAYOUT);
        indicator.add_style(&indicator_style, Selector::DEFAULT);

        // Force layout so we can read button positions for the initial indicator.
        indicator.update_layout();

        // Trigger initial state so buttons get the CHECKED binding applied.
        tab_subject.notify();

        self._main_cont = Some(main_cont);
        self.cont = Some(cont);
        self.footer = Some(footer);
        self.indicator = Some(indicator);
        self.btn_handles = btn_handles;
        self.last_tab = -1; // force initial content build
        self.tab_subject = Some(tab_subject);
        self.slider_subjects = Some(slider_subjects);
        self.dropdown_subjects = Some(dropdown_subjects);
        self.roller_subjects = Some(roller_subjects);
        Ok(())
    }

    fn on_event(&mut self, event: &Event) -> NavAction {
        if event.code() != EventCode::CLICKED {
            return NavAction::None;
        }
        let target = event.target_handle() as *mut c_void;
        for (i, &handle) in self.btn_handles.iter().enumerate() {
            if !handle.is_null() && target == handle {
                if let Some(ref tab_subject) = self.tab_subject {
                    tab_subject.set_int(i as i32);
                }
                return NavAction::None;
            }
        }
        NavAction::None
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        let tab = if let Some(ref ts) = self.tab_subject { ts.get_int() } else { return Ok(NavAction::None); };
        if tab == self.last_tab {
            return Ok(NavAction::None);
        }
        self.last_tab = tab;

        let Some(ref cont) = self.cont else { return Ok(NavAction::None); };
        // Remove old content and rebuild for the active tab.
        cont.clean();

        match tab {
            0 => {
                // Tab 0: four sliders.
                if let Some(ref slider_subjects) = self.slider_subjects {
                    for i in 0..4_i32 {
                        let slider = Child::new(Slider::new(cont)?);
                        slider
                            .set_range(0, 100)
                            .align(Align::TopMid, 0, 10 + i * 30);
                        slider.bind_value(&slider_subjects[i as usize]);
                    }
                }
            }
            1 => {
                // Tab 1: three dropdowns.
                if let Some(ref dropdown_subjects) = self.dropdown_subjects {
                    for i in 0..3_i32 {
                        let dd = Child::new(Dropdown::new(cont)?);
                        dd.set_options(DROPDOWN_OPTIONS)
                            .align(Align::TopMid, 0, i * 50);
                        dd.bind_value(&dropdown_subjects[i as usize]);
                    }
                }
            }
            2 => {
                // Tab 2: two rollers side by side.
                if let Some(ref roller_subjects) = self.roller_subjects {
                    for i in 0..2_i32 {
                        let roller = Child::new(Roller::new(cont)?);
                        roller
                            .set_options(ROLLER_OPTIONS, RollerMode::Normal)
                            .align(Align::Center, -80 + i * 160, 0);
                        roller.bind_value(&roller_subjects[i as usize]);
                    }
                }
            }
            _ => {}
        }

        // Animate indicator to slide under the active button.
        if let Some(ref footer) = self.footer {
            if let Some(btn_child) = footer.get_child(tab) {
                let btn_x = btn_child.get_x();
                let btn_w = btn_child.get_width();
                if let Some(ref indicator) = self.indicator {
                    let ind_x = indicator.get_x();
                    indicator.width(btn_w);

                    let mut anim = Anim::new();
                    anim.set_var(indicator)
                        .set_exec_cb(Some(anim_set_x))
                        .set_values(ind_x, btn_x)
                        .set_duration(300)
                        .set_path_cb(Some(anim_path_ease_in_out));
                    anim.start();
                }
            }
        }

        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(Observer4::default());

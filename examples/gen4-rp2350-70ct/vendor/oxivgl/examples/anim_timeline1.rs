#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Anim Timeline 1 — Animation timeline

extern crate alloc;

use alloc::boxed::Box;
use oxivgl::{
    anim::{
        anim_path_ease_out, anim_path_linear, anim_path_overshoot, anim_set_height,
        anim_set_slider_value, anim_set_width, Anim, AnimTimeline, ANIM_TIMELINE_PROGRESS_MAX,
    },
    view::{NavAction, View},
    enums::{EventCode, ObjFlag, ObjState, ScrollbarMode},
    event::Event,
    layout::{FlexAlign, FlexFlow},
    widgets::{Align, Button, Label, Obj, Slider, WidgetError},
};

const OBJ_WIDTH: i32 = 90;
const OBJ_HEIGHT: i32 = 70;

#[derive(Default)]
struct AnimTimeline1 {
    timeline: Option<Box<AnimTimeline>>,
    btn_start: Option<Button<'static>>,
    btn_pause: Option<Button<'static>>,
    slider: Option<Slider<'static>>,
    _obj1: Option<Obj<'static>>,
    _obj2: Option<Obj<'static>>,
    _obj3: Option<Obj<'static>>,
    _label_start: Option<Label<'static>>,
    _label_pause: Option<Label<'static>>,
}

impl View for AnimTimeline1 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let mut timeline = Box::new(AnimTimeline::new());

        container.set_flex_flow(FlexFlow::Row);
        container.set_flex_align(FlexAlign::SpaceAround, FlexAlign::Center, FlexAlign::Center);

        // Start button (checkable)
        let btn_start = Button::new(container)?;
        btn_start.add_flag(ObjFlag::IGNORE_LAYOUT);
        btn_start.add_flag(ObjFlag::CHECKABLE);
        btn_start.bubble_events();
        btn_start.align(Align::TopMid, -100, 20);
        let label_start = Label::new(&btn_start)?;
        label_start.text("Start").center();

        // Pause button
        let btn_pause = Button::new(container)?;
        btn_pause.add_flag(ObjFlag::IGNORE_LAYOUT);
        btn_pause.bubble_events();
        btn_pause.align(Align::TopMid, 100, 20);
        let label_pause = Label::new(&btn_pause)?;
        label_pause.text("Pause").center();

        // Progress slider
        let slider = Slider::new(container)?;
        slider.add_flag(ObjFlag::IGNORE_LAYOUT);
        slider.bubble_events();
        slider.align(Align::BottomMid, 0, -20);
        slider.set_range(0, ANIM_TIMELINE_PROGRESS_MAX as i32);

        // 3 objects
        let obj1 = Obj::new(container)?;
        obj1.size(OBJ_WIDTH, OBJ_HEIGHT)
            .set_scrollbar_mode(ScrollbarMode::Off);

        let obj2 = Obj::new(container)?;
        obj2.size(OBJ_WIDTH, OBJ_HEIGHT)
            .set_scrollbar_mode(ScrollbarMode::Off);

        let obj3 = Obj::new(container)?;
        obj3.size(OBJ_WIDTH, OBJ_HEIGHT)
            .set_scrollbar_mode(ScrollbarMode::Off);

        // Animations — slider progress
        let mut a_slider = Anim::new();
        a_slider
            .set_var(&slider)
            .set_values(0, ANIM_TIMELINE_PROGRESS_MAX as i32)
            .set_custom_exec_cb(Some(anim_set_slider_value))
            .set_path_cb(Some(anim_path_linear))
            .set_duration(700);

        // obj1 width + height
        let mut a1 = Anim::new();
        a1.set_var(&obj1)
            .set_values(0, OBJ_WIDTH)
            .set_custom_exec_cb(Some(anim_set_width))
            .set_path_cb(Some(anim_path_overshoot))
            .set_duration(300);

        let mut a2 = Anim::new();
        a2.set_var(&obj1)
            .set_values(0, OBJ_HEIGHT)
            .set_custom_exec_cb(Some(anim_set_height))
            .set_path_cb(Some(anim_path_ease_out))
            .set_duration(300);

        // obj2
        let mut a3 = Anim::new();
        a3.set_var(&obj2)
            .set_values(0, OBJ_WIDTH)
            .set_custom_exec_cb(Some(anim_set_width))
            .set_path_cb(Some(anim_path_overshoot))
            .set_duration(300);

        let mut a4 = Anim::new();
        a4.set_var(&obj2)
            .set_values(0, OBJ_HEIGHT)
            .set_custom_exec_cb(Some(anim_set_height))
            .set_path_cb(Some(anim_path_ease_out))
            .set_duration(300);

        // obj3
        let mut a5 = Anim::new();
        a5.set_var(&obj3)
            .set_values(0, OBJ_WIDTH)
            .set_custom_exec_cb(Some(anim_set_width))
            .set_path_cb(Some(anim_path_overshoot))
            .set_duration(300);

        let mut a6 = Anim::new();
        a6.set_var(&obj3)
            .set_values(0, OBJ_HEIGHT)
            .set_custom_exec_cb(Some(anim_set_height))
            .set_path_cb(Some(anim_path_ease_out))
            .set_duration(300);

        // Add to timeline
        timeline.add(0, &a_slider);
        timeline.add(0, &a1);
        timeline.add(0, &a2);
        timeline.add(200, &a3);
        timeline.add(200, &a4);
        timeline.add(400, &a5);
        timeline.add(400, &a6);

        timeline.set_progress(ANIM_TIMELINE_PROGRESS_MAX);

                self.timeline = Some(timeline);
        self.btn_start = Some(btn_start);
        self.btn_pause = Some(btn_pause);
        self.slider = Some(slider);
        self._obj1 = Some(obj1);
        self._obj2 = Some(obj2);
        self._obj3 = Some(obj3);
        self._label_start = Some(label_start);
        self._label_pause = Some(label_pause);
        Ok(())
    }

    fn on_event(&mut self, event: &Event) -> NavAction {
        if let Some(ref btn_start) = self.btn_start {
            if event.matches(btn_start, EventCode::VALUE_CHANGED) {
                let reverse = btn_start.has_state(ObjState::CHECKED);
                if let Some(ref mut timeline) = self.timeline {
                    timeline.set_reverse(reverse);
                    timeline.start();
                }
                return NavAction::None;
            }
        }
        if let Some(ref btn_pause) = self.btn_pause {
            if event.matches(btn_pause, EventCode::CLICKED) {
                if let Some(ref mut timeline) = self.timeline {
                    timeline.pause();
                }
                return NavAction::None;
            }
        }
        if let Some(ref slider) = self.slider {
            if event.matches(slider, EventCode::VALUE_CHANGED) {
                let progress = slider.get_value();
                if let Some(ref mut timeline) = self.timeline {
                    timeline.set_progress(progress as u16);
                }
            }
        }
        NavAction::None
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(AnimTimeline1::default());

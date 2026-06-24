#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Anim 2 — Playback animation

use oxivgl::{
    anim::{anim_path_ease_in_out, anim_set_size, anim_set_x, Anim, ANIM_REPEAT_INFINITE},
    style::{palette_main, Palette, Selector, Style},
    view::{NavAction, View},
    widgets::{Align, Obj, WidgetError, RADIUS_MAX},
};

#[derive(Default)]
struct Anim2 {
    _obj: Option<Obj<'static>>,
}

impl View for Anim2 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let obj = Obj::new(container)?;
        obj.remove_scrollable();
        let obj_style = Style::new(|s| {
            s.bg_color(palette_main(Palette::Red)).radius(RADIUS_MAX as i16);
        });
        obj.add_style(&obj_style, Selector::DEFAULT);
        obj.align(Align::LeftMid, 10, 0);

        let mut a = Anim::new();
        a.set_var(&obj)
            .set_values(10, 50)
            .set_duration(1000)
            .set_reverse_delay(100)
            .set_reverse_duration(300)
            .set_repeat_delay(500)
            .set_repeat_count(ANIM_REPEAT_INFINITE)
            .set_path_cb(Some(anim_path_ease_in_out));

        a.set_exec_cb(Some(anim_set_size));
        a.start();

        a.set_exec_cb(Some(anim_set_x));
        a.set_values(10, 240);
        a.start();

                self._obj = Some(obj);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(Anim2::default());

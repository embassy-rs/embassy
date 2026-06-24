#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Scroll 6 — Curved Scroll
//!
//! A circular clipped flex column where items are displaced horizontally based
//! on their distance from the container centre, creating a curved/perspective
//! effect. Items far from centre are also made more transparent.
//!
//! Note: the C original uses plain `lv_obj_t` children. This port uses
//! `Button` widgets; they are visual-only here — scroll events are caught on
//! the container via `SCROLL`, not on the buttons themselves.

use oxivgl::view::NavAction;
use oxivgl::{
    enums::{EventCode, ScrollDir, ScrollSnap, ScrollbarMode},
    event::Event,
    layout::FlexFlow,
    math::map,
    style::{lv_pct, Selector, Style},
    view::{register_event_on, View},
    widgets::{Button, Label, Obj, WidgetError, RADIUS_MAX},
};

#[derive(Default)]
struct Scroll6 {
    cont: Option<Obj<'static>>,
    _cont_style: Option<Style>,
}

impl View for Scroll6 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let cont = Obj::new(container)?;
        cont.size(200, 200).center();
        cont.set_flex_flow(FlexFlow::Column);
        let cont_style = Style::new(|s| {
            s.radius(RADIUS_MAX as i16).clip_corner(true);
        });
        cont.add_style(&cont_style, Selector::DEFAULT);
        cont.set_scroll_dir(ScrollDir::VER);
        cont.set_scroll_snap_y(ScrollSnap::Center);
        cont.set_scrollbar_mode(ScrollbarMode::Off);

        for i in 0u32..20 {
            let btn = Button::new(&cont)?;
            btn.width(lv_pct(100));
            let lbl = Label::new(&btn)?;
            let mut s = heapless::String::<16>::new();
            let _ = core::fmt::Write::write_fmt(&mut s, format_args!("Button {}", i));
            lbl.text(&s);
        }

        // Scroll first child to center (layout must be done first)
        cont.update_layout();
        if let Some(first) = cont.get_child(0) {
            first.scroll_to_view(false);
        }

                self.cont = Some(cont);
        self._cont_style = Some(cont_style);
        Ok(())
    }

    fn register_events_on(&mut self, _container: &Obj<'static>) {
        if let Some(ref cont) = self.cont {
            register_event_on(self, cont.handle());
        }
    }

    fn on_event(&mut self, event: &Event) -> NavAction {
        if event.code() != EventCode::SCROLL {
            return NavAction::None;
        }
        let cont = match self.cont.as_ref() {
            Some(c) => c,
            None => return NavAction::None,
        };
        if event.current_target_handle() != cont.handle() {
            return NavAction::None;
        }
        let coords = cont.get_coords();
        let cont_y_center = coords.y1 + coords.height() / 2;
        let r = cont.get_height() * 7 / 10;

        for i in 0..cont.get_child_count() as i32 {
            let Some(child) = cont.get_child(i) else {
                continue;
            };
            let child_coords = child.get_coords();
            let child_y_center = child_coords.y1 + child_coords.height() / 2;
            let diff_y = (child_y_center - cont_y_center).abs();
            let x = if diff_y >= r {
                r
            } else {
                let x_sqr = (r * r - diff_y * diff_y) as u32;
                r - x_sqr.isqrt() as i32
            };
            child.style_translate_x(x, Selector::DEFAULT);
            let opa = map(x, 0, r, 0, 255) as u8;
            #[allow(deprecated)]
            // runtime-varying style; must stay inline
            child.style_opa(255 - opa, Selector::DEFAULT);
        }
        NavAction::None
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(Scroll6::default());

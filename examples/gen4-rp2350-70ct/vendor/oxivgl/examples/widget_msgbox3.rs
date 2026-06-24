#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Msgbox 3 — Message box with backdrop blur
//!
//! Background content (labels) with a modal message box on the top layer.
//! The layer_top backdrop uses `style_blur_radius` and `style_blur_backdrop`
//! to blur the content behind the dialog, plus `bg_opa` for dimming.
//!
//! Note: blur requires a GPU-accelerated backend to produce visible results.
//! On the SDL host the blur may be a no-op; the backdrop dimming (bg_opa)
//! is always visible.

use oxivgl::{
    style::{Selector, Style},
    view::{NavAction, View},
    widgets::{Align, Label, Msgbox, Obj, Screen, WidgetError},
};

#[derive(Default)]
struct WidgetMsgbox3 {
    _label1: Option<Label<'static>>,
    _label2: Option<Label<'static>>,
    _label3: Option<Label<'static>>,
}

impl View for WidgetMsgbox3 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        // Background content — visible through the blurred/dimmed backdrop
        let label1 = Label::new(container)?;
        label1.text("Background text line 1");
        label1.align(Align::TopMid, 0, 30);

        let label2 = Label::new(container)?;
        label2.text("Background text line 2");
        label2.align(Align::Center, 0, 0);

        let label3 = Label::new(container)?;
        label3.text("Background text line 3");
        label3.align(Align::BottomMid, 0, -30);

        // Style layer_top for blur + dimming (applied to the msgbox backdrop)
        let layer = Screen::layer_top();
        let blur_style = Style::new(|s| {
            s.blur_radius(20).blur_backdrop(true);
        });
        layer.add_style(&blur_style, Selector::DEFAULT);
        // Prevent layer_top from being deleted on drop — LVGL owns it
        core::mem::forget(layer);

        // Modal message box (parent = None → LVGL creates a full-screen
        // backdrop child on layer_top and centers the msgbox on it)
        let mbox = Msgbox::new(None::<&oxivgl::widgets::Obj<'_>>)?;
        mbox.add_title("Notice");
        mbox.add_text("This dialog has a blurred backdrop.\nClose to dismiss.");
        mbox.add_close_button();
        mbox.add_footer_button("OK");
        // LVGL owns the msgbox — forget the Rust handle
        core::mem::forget(mbox);

                self._label1 = Some(label1);
        self._label2 = Some(label2);
        self._label3 = Some(label3);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(WidgetMsgbox3::default());

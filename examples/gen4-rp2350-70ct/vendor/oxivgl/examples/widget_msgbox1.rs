#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Msgbox 1 — Standalone message box
//!
//! A modal message box with a title, body text, and a close button.
//! Clicking the button dismisses the message box.
//!
//! Ownership note: when `parent = None`, LVGL creates a full-screen backdrop
//! and becomes the owner of the msgbox object. The Rust handle is forgotten
//! so that `Obj::drop` does not call `lv_obj_delete` while LVGL still holds
//! the object (it remains valid until the close button fires).

use oxivgl::{
    view::{NavAction, View},
    widgets::{Msgbox, Obj, WidgetError},
};

#[derive(Default)]
struct WidgetMsgbox1 {
}

impl View for WidgetMsgbox1 {
    fn create(&mut self, _container: &Obj<'static>) -> Result<(), WidgetError> {

        let mbox = Msgbox::new(None::<&Obj<'_>>)?;
        mbox.add_title("Hello");
        mbox.add_text("This is a message box.\nClick Close to dismiss.");
        mbox.add_close_button();
        // LVGL owns the msgbox (its modal backdrop is the parent). Forget the
        // Rust handle so Obj::drop does not call lv_obj_delete while LVGL
        // still holds the object.
        core::mem::forget(mbox);

                Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(WidgetMsgbox1::default());

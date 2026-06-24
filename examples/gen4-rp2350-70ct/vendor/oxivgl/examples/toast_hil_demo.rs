#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Toast HIL demo — validates the toast visibility fix on real hardware.
//!
//! In PARTIAL render mode (ESP32) a toast raised on a static screen with no
//! navigation could be invisible — worst on the first cold boot — because
//! `lv_layer_sys()` is not composited reliably. The fix renders the toast on
//! the **active screen** instead (re-parented across navigation/modals),
//! which the panel composites as reliably as any other widget. This demo
//! shows a **persistent** toast on the very first `update()` — before any
//! push/replace/pop — so if the green screen shows a "NO SD CARD" card at
//! boot, the toast surface is composited on the target.
//!
//! Note: widgets created in `create()` MUST be stored in the view struct,
//! otherwise the local `Obj` drops at end of scope and `lv_obj_delete`s
//! the widget (the parent keeps it alive, so `lv_obj_is_valid` is true).
//! That is why `label` fields exist below.
//!
//! The committed host screenshot (`./run_host.sh -s`) shows only the green
//! "TOAST TEST" root: the screenshot harness discards the `NavAction` that
//! `update()` returns, so the toast is never raised in that path. The toast
//! itself is verified on real PARTIAL-mode hardware (its whole point —
//! host/SDL is FULL/DIRECT and never exhibited the bug).
//!
//! (The rapid-toast sequencing fix is covered by host integration tests.)

use oxivgl::{
    style::{Selector, Style},
    view::{NavAction, View},
    widgets::{Align, Label, Obj, WidgetError},
};

/// Passive toast content: a colored card with a single line of text.
struct ToastView {
    text: &'static str,
    bg: u32,
    /// Retains the label so it is not deleted when `create` returns.
    label: Option<Label<'static>>,
}

impl ToastView {
    fn new(text: &'static str, bg: u32) -> Self {
        Self { text, bg, label: None }
    }
}

impl View for ToastView {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {
        let bg = self.bg;
        let card_style = Style::new(|s| {
            s.bg_color_hex(bg).bg_opa(255).text_color_hex(0x000000);
        });
        container.add_style(&card_style, Selector::DEFAULT);
        let label = Label::new(container)?;
        label.text(self.text).align(Align::Center, 0, 0);
        self.label = Some(label);
        Ok(())
    }
}

/// Root view. Renders a caption and, on its first update tick (before any
/// navigation), raises a persistent global toast.
#[derive(Default)]
struct ToastDemo {
    boot_toast_shown: bool,
    /// Retains the caption label (see module note).
    label: Option<Label<'static>>,
}

impl View for ToastDemo {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {
        // Solid bright green — a color no other app on the rig uses, so the
        // board running this example is unmistakable in the camera frame.
        let root_style = Style::new(|s| {
            s.bg_color_hex(0x18a018).bg_opa(255).text_color_hex(0xffffff);
        });
        container.add_style(&root_style, Selector::DEFAULT);
        let label = Label::new(container)?;
        label.text("TOAST TEST").align(Align::Center, 0, 0);
        self.label = Some(label);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        // Raise the toast on the very first tick — before any navigation.
        if !self.boot_toast_shown {
            self.boot_toast_shown = true;
            return Ok(NavAction::show_toast(
                // High-contrast orange card, black text.
                ToastView::new("NO SD CARD", 0xff6000),
                None, // persistent: stays on screen for the photo
            ));
        }
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main_nav!(ToastDemo::default());

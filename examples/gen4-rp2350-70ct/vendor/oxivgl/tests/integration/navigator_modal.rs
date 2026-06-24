// SPDX-License-Identifier: MIT OR Apache-2.0
//! Tests for the OSD-enablement work on `Navigator::modal`:
//! - backdrop is created on `lv_layer_top` and is full-size + clickable
//! - `View::register_events_on` default attaches to the backdrop (not the
//!   background screen)
//! - `View::input_group` causes the navigator to push focus into the
//!   modal's group on open and restore the previous focus on dismiss

use crate::common::ensure_init;

use oxivgl::enums::ObjFlag;
use oxivgl::group::{Group, group_get_default};
use oxivgl::navigator::Navigator;
use oxivgl::view::View;
use oxivgl::widgets::{AsLvHandle, Button, Obj, WidgetError};

// ── Fixtures ─────────────────────────────────────────────────────────────────

#[derive(Default)]
struct EmptyRoot;
impl View for EmptyRoot {
    fn create(&mut self, _container: &Obj<'static>) -> Result<(), WidgetError> {
        Ok(())
    }
}

/// A modal that records, in a static atomic, the LVGL handle of the
/// container it was registered against.
mod register_probe {
    use super::*;
    use core::sync::atomic::{AtomicUsize, Ordering};
    pub static REGISTER_TARGET: AtomicUsize = AtomicUsize::new(0);

    #[derive(Default)]
    pub struct ProbeModal;
    impl View for ProbeModal {
        fn create(&mut self, _container: &Obj<'static>) -> Result<(), WidgetError> {
            Ok(())
        }
        fn register_events_on(&mut self, container: &Obj<'static>) {
            REGISTER_TARGET.store(container.lv_handle() as usize, Ordering::SeqCst);
            // Don't actually register handlers — we only want the target.
        }
    }
}

/// A modal that owns its own Group and exposes it to the navigator.
struct GroupModal {
    group: Group,
}
impl GroupModal {
    fn new() -> Self {
        Self { group: Group::new().expect("group create") }
    }
}
impl View for GroupModal {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {
        let btn = Button::new(container)?;
        self.group.add_obj(&btn);
        Ok(())
    }
    fn input_group(&self) -> Option<oxivgl::group::GroupRef> {
        Some(self.group.as_ref())
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn fresh_navigator() -> Navigator {
    ensure_init();
    // Establish an active screen + clean overlay layers so prior tests
    // don't leak state into this one.
    unsafe {
        let new_screen = oxivgl_sys::lv_obj_create(core::ptr::null_mut());
        oxivgl_sys::lv_screen_load(new_screen);
        oxivgl_sys::lv_obj_clean(oxivgl_sys::lv_layer_top());
        oxivgl_sys::lv_obj_clean(oxivgl_sys::lv_layer_sys());
    }
    let mut nav = Navigator::new();
    nav.push_root(EmptyRoot);
    nav
}

fn layer_top_child_count() -> u32 {
    unsafe { oxivgl_sys::lv_obj_get_child_count(oxivgl_sys::lv_layer_top()) }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[test]
fn modal_creates_clickable_full_size_backdrop_on_layer_top() {
    let mut nav = fresh_navigator();
    assert_eq!(layer_top_child_count(), 0);

    nav.modal(EmptyRoot);
    assert!(nav.has_modal());
    assert_eq!(
        layer_top_child_count(),
        1,
        "modal should create exactly one direct child (the backdrop)",
    );

    // SAFETY: layer_top is a valid LVGL global.
    let backdrop = unsafe {
        oxivgl_sys::lv_obj_get_child(oxivgl_sys::lv_layer_top(), 0)
    };
    assert!(!backdrop.is_null());
    // Backdrop must be clickable so it absorbs touches; must NOT be scrollable.
    // SAFETY: backdrop is the live child we just fetched.
    unsafe {
        assert!(
            oxivgl_sys::lv_obj_has_flag(backdrop, ObjFlag::CLICKABLE.0),
            "backdrop must be CLICKABLE so it absorbs touches",
        );
        assert!(
            !oxivgl_sys::lv_obj_has_flag(backdrop, ObjFlag::SCROLLABLE.0),
            "backdrop should not be SCROLLABLE",
        );
    }

    nav.dismiss_modal().expect("dismiss");
    assert_eq!(
        layer_top_child_count(),
        0,
        "dismiss must delete the backdrop and any descendants",
    );
}

#[test]
fn modal_default_register_events_targets_the_backdrop_not_active_screen() {
    use core::sync::atomic::Ordering;
    register_probe::REGISTER_TARGET.store(0, Ordering::SeqCst);

    let mut nav = fresh_navigator();
    let background_screen = unsafe { oxivgl_sys::lv_screen_active() as usize };

    nav.modal(register_probe::ProbeModal);

    let recorded = register_probe::REGISTER_TARGET.load(Ordering::SeqCst);
    assert_ne!(recorded, 0, "register_events_on was not called");
    assert_ne!(
        recorded, background_screen,
        "register_events_on must NOT receive the background screen for a modal — \
         this was the dangling-handler bug the OSD work is meant to fix",
    );
    // The container handed to register_events_on must be the backdrop:
    // a direct child of lv_layer_top.
    let backdrop = unsafe {
        oxivgl_sys::lv_obj_get_child(oxivgl_sys::lv_layer_top(), 0) as usize
    };
    assert_eq!(
        recorded, backdrop,
        "register_events_on container must be the backdrop, not lv_screen_active()",
    );

    nav.dismiss_modal().expect("dismiss");
}

#[test]
fn modal_with_input_group_swaps_default_group_and_restores_on_dismiss() {
    let mut nav = fresh_navigator();

    // Set up a "background" default group representing the app's normal
    // focus group before the modal opens.
    let app_group = Group::new().expect("app group create");
    app_group.set_default();
    let app_group_ptr =
        group_get_default().expect("default after set_default").add_obj_ptr_for_test();

    nav.modal(GroupModal::new());

    // While the modal is open, the default group must NOT be the app's.
    let mid_default = group_get_default()
        .expect("default still set while modal is up")
        .add_obj_ptr_for_test();
    assert_ne!(
        mid_default, app_group_ptr,
        "navigator must swap the default group to the modal's group on open",
    );

    nav.dismiss_modal().expect("dismiss");

    // After dismiss, the previous default group must be restored.
    let restored = group_get_default()
        .expect("default still set after dismiss")
        .add_obj_ptr_for_test();
    assert_eq!(
        restored, app_group_ptr,
        "navigator must restore the previous default group on dismiss",
    );
}

#[test]
fn modal_without_input_group_does_not_touch_focus() {
    let mut nav = fresh_navigator();

    let app_group = Group::new().expect("app group create");
    app_group.set_default();
    let before = group_get_default().expect("default").add_obj_ptr_for_test();

    nav.modal(EmptyRoot);

    let during = group_get_default().expect("default").add_obj_ptr_for_test();
    assert_eq!(
        during, before,
        "modal without input_group must not change default group",
    );

    nav.dismiss_modal().expect("dismiss");

    let after = group_get_default().expect("default").add_obj_ptr_for_test();
    assert_eq!(after, before, "still unchanged after dismiss");
}

#[test]
fn full_screen_view_with_input_group_becomes_default_group() {
    // Regression guard: full-screen `input_group` routing (push/pop/replace),
    // not just modals. The navigator must bind a pushed view's group as the
    // default + to the keyboard indevs via `activate_view_group`.
    let mut nav = fresh_navigator();

    // A baseline app group, distinct from the pushed view's group.
    let app_group = Group::new().expect("app group create");
    app_group.set_default();
    let app_group_ptr = group_get_default().expect("default").add_obj_ptr_for_test();

    // Push a full-screen root view that exposes an input_group.
    nav.push_root(GroupModal::new());

    let after = group_get_default()
        .expect("default set after push_root")
        .add_obj_ptr_for_test();
    assert_ne!(
        after, app_group_ptr,
        "navigator must route a full-screen view's input_group to the default group",
    );
}

// ── GroupRef test extension trait ───────────────────────────────────────────
// GroupRef doesn't expose its raw pointer; for tests we need an identity to
// compare. Add a sentinel widget and use its address as a stand-in.

trait GroupRefTestExt {
    /// Return a stable address derived from the group identity, by
    /// adding a temporary widget and reading its handle back. Two calls
    /// on the same group return the same address only if both find the
    /// group still tracking that widget — for test identity comparison
    /// we use the raw lv_group_get_default pointer via a different route.
    fn add_obj_ptr_for_test(&self) -> usize;
}

impl GroupRefTestExt for oxivgl::group::GroupRef {
    fn add_obj_ptr_for_test(&self) -> usize {
        // Use the raw lv_group_get_default pointer as identity — both
        // sides of the comparison query the same global, so as long as
        // we call this right after observing the default group it is a
        // sound identity proxy.
        unsafe { oxivgl_sys::lv_group_get_default() as usize }
    }
}

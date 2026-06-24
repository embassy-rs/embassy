// SPDX-License-Identifier: MIT OR Apache-2.0
//! Tests for the navigator-level global toast overlay.

use crate::common::{ensure_init, pump};

use std::sync::atomic::{AtomicUsize, Ordering};

use std::time::Duration;
use oxivgl::enums::ObjFlag;
use oxivgl::navigator::Navigator;
use oxivgl::view::{NavigationError, View};
use oxivgl::widgets::{Button, Label, Obj, WidgetError};

// ── Test fixtures ────────────────────────────────────────────────────────────

/// A trivial full-screen view used as the root in these tests.
#[derive(Default)]
struct EmptyRoot;
impl View for EmptyRoot {
    fn create(&mut self, _container: &Obj<'static>) -> Result<(), WidgetError> {
        Ok(())
    }
}

/// A passive toast view: one Label.
#[derive(Default)]
struct LabelToast;
impl View for LabelToast {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {
        let lbl = Label::new(container)?;
        lbl.text("status");
        Ok(())
    }
}

/// A toast view that builds a Button — used to verify Navigator strips
/// CLICKABLE regardless of what the view did.
#[derive(Default)]
struct ButtonToast;
impl View for ButtonToast {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {
        let _btn = Button::new(container)?;
        Ok(())
    }
}

/// Counts how many times any `CountingToast` has had its widgets built.
/// Lets the sequencing tests prove a *queued* toast is not created until
/// it is promoted (and is dropped uncreated if the queue is cleared).
/// Safe to use as a global because integration tests run single-threaded.
static TOAST_CREATE_COUNT: AtomicUsize = AtomicUsize::new(0);

fn reset_toast_create_count() {
    TOAST_CREATE_COUNT.store(0, Ordering::SeqCst);
}
fn toast_create_count() -> usize {
    TOAST_CREATE_COUNT.load(Ordering::SeqCst)
}

/// A passive toast that bumps [`TOAST_CREATE_COUNT`] each time its widgets
/// are actually built.
#[derive(Default)]
struct CountingToast;
impl View for CountingToast {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {
        TOAST_CREATE_COUNT.fetch_add(1, Ordering::SeqCst);
        let lbl = Label::new(container)?;
        lbl.text("status");
        Ok(())
    }
}

/// A toast view that *stores* its widget wrapper — the leak-free pattern,
/// where the held `Label`'s `Drop` releases its styles on teardown. Holding
/// an `Obj`-based wrapper (a raw `lv_obj_t` pointer) makes the view `!Send`,
/// so it cannot be posted by value: only a builder closure can carry it to a
/// background task. Used by `post_toast_with_shows_a_non_send_view`.
#[derive(Default)]
struct StoringToast {
    label: Option<Label<'static>>,
}
impl View for StoringToast {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {
        let lbl = Label::new(container)?;
        lbl.text("status");
        self.label = Some(lbl);
        Ok(())
    }
}

/// A long duration that will not elapse during a test, so a timed toast
/// stays put until it is explicitly dismissed.
fn long() -> Option<Duration> {
    Some(Duration::from_secs(3600))
}

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Count children of the surface the active toast rides. Toasts render on
/// the active screen now (not `lv_layer_sys()`), and the root/pushed views in
/// these tests (`EmptyRoot`) create no widgets of their own, so the active
/// screen's child count equals the number of live toast containers on it.
/// None of the tests here open a modal, so the surface is always the screen.
fn toast_surface_child_count() -> u32 {
    // SAFETY: lv_screen_active() returns the active screen, valid after init.
    unsafe { oxivgl_sys::lv_obj_get_child_count(oxivgl_sys::lv_screen_active()) }
}

/// True iff the navigator's toast container is alive and parented to the
/// current active screen — the core invariant of the sys-layer→screen fix.
fn toast_on_active_screen(nav: &Navigator) -> bool {
    match nav.toast_container_handle() {
        // SAFETY: handle came from the live navigator; lv_obj_is_valid guards
        // any pointer, and lv_obj_get_parent/lv_screen_active are always safe.
        Some(h) => unsafe {
            oxivgl_sys::lv_obj_is_valid(h)
                && oxivgl_sys::lv_obj_get_parent(h) == oxivgl_sys::lv_screen_active()
        },
        None => false,
    }
}

/// Children on `lv_layer_sys()` — must stay 0 now that toasts no longer use it.
fn sys_layer_child_count() -> u32 {
    // SAFETY: lv_layer_sys() returns a valid LVGL global after init.
    unsafe { oxivgl_sys::lv_obj_get_child_count(oxivgl_sys::lv_layer_sys()) }
}

fn fresh_navigator() -> Navigator {
    ensure_init();
    // Establish a fresh active screen (other tests may have left LVGL
    // without one) and clear any residue from the system layer.
    // SAFETY: LVGL initialised; lv_obj_create(NULL) creates a screen.
    unsafe {
        let new_screen = oxivgl_sys::lv_obj_create(core::ptr::null_mut());
        oxivgl_sys::lv_screen_load(new_screen);
        oxivgl_sys::lv_obj_clean(oxivgl_sys::lv_layer_sys());
    }
    let mut nav = Navigator::new();
    nav.push_root(EmptyRoot);
    nav
}

/// Walk `obj` and every descendant; assert `CLICKABLE` is cleared.
fn assert_subtree_not_clickable(obj: *mut oxivgl_sys::lv_obj_t) {
    // SAFETY: caller passes a valid handle; we don't mutate the tree here.
    unsafe {
        let flags = ObjFlag::CLICKABLE.0;
        assert_eq!(
            oxivgl_sys::lv_obj_has_flag(obj, flags),
            false,
            "subtree node still has CLICKABLE set",
        );
        let n = oxivgl_sys::lv_obj_get_child_count(obj);
        for i in 0..n {
            let c = oxivgl_sys::lv_obj_get_child(obj, i as i32);
            assert_subtree_not_clickable(c);
        }
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[test]
fn push_root_loads_its_own_screen() {
    // Regression: push_root must create and LOAD its own screen rather than
    // reusing the default active screen, so the root owns a screen like every
    // pushed view (uniform pop / toast re-parenting). If someone reverts to
    // reusing lv_screen_active(), the active screen would be unchanged and
    // this fails.
    ensure_init();
    // SAFETY: LVGL initialised; create + load a known "default" screen.
    let before = unsafe {
        let s = oxivgl_sys::lv_obj_create(core::ptr::null_mut());
        oxivgl_sys::lv_screen_load(s);
        s
    };

    let mut nav = Navigator::new();
    nav.push_root(EmptyRoot);

    // SAFETY: returns the currently-active screen (always valid post-init).
    let after = unsafe { oxivgl_sys::lv_screen_active() };
    assert!(!after.is_null());
    assert_ne!(
        after, before,
        "push_root must load its own screen, not reuse the active one",
    );
}

#[test]
fn pop_to_root_does_not_panic() {
    // Regression: with the root owning a real screen, popping back to it
    // must load that screen via the normal path — not fall through to the
    // (now unreachable) default-screen fallback, which is a debug_assert.
    let mut nav = fresh_navigator(); // root pushed
    nav.push(EmptyRoot, None); // depth 2
    assert_eq!(nav.depth(), 2);

    nav.pop(None).expect("pop back to root");
    pump();
    assert_eq!(nav.depth(), 1, "should be back at the root view");
}

#[test]
fn show_then_dismiss_clears_surface() {
    let mut nav = fresh_navigator();
    assert!(!nav.has_toast());
    assert_eq!(toast_surface_child_count(), 0);

    nav.show_toast(LabelToast, None);
    pump();
    assert!(nav.has_toast());
    assert_eq!(toast_surface_child_count(), 1);
    // The fix: the toast rides the active screen, NOT lv_layer_sys().
    assert!(toast_on_active_screen(&nav), "toast must be a child of the active screen");
    assert_eq!(sys_layer_child_count(), 0, "toast must NOT be on the system layer");

    nav.dismiss_toast().expect("dismiss_toast");
    pump();
    assert!(!nav.has_toast());
    assert_eq!(toast_surface_child_count(), 0);
}

#[test]
fn dismiss_without_toast_is_error() {
    let mut nav = fresh_navigator();
    match nav.dismiss_toast() {
        Err(NavigationError::NoActiveToast) => {}
        other => panic!("expected NoActiveToast, got {:?}", other),
    }
}

#[test]
fn auto_dismiss_after_duration() {
    let mut nav = fresh_navigator();
    nav.show_toast(LabelToast, Some(Duration::from_millis(40)));
    assert!(nav.has_toast());

    // Before the deadline, tick must NOT dismiss.
    std::thread::sleep(std::time::Duration::from_millis(10));
    nav.tick_toast();
    assert!(nav.has_toast(), "toast dismissed too early");

    // After the deadline, tick must dismiss.
    std::thread::sleep(std::time::Duration::from_millis(60));
    nav.tick_toast();
    assert!(!nav.has_toast(), "toast not auto-dismissed");
    pump();
    assert_eq!(toast_surface_child_count(), 0);
}

#[test]
fn persists_across_push_replace_pop() {
    // Push twice up front so we can `pop` back to a non-root entry,
    // sidestepping a pre-existing pop-to-root edge case in Navigator
    // unrelated to the toast feature.
    let mut nav = fresh_navigator();
    nav.push(EmptyRoot, None);
    nav.show_toast(LabelToast, None);
    let baseline = toast_surface_child_count();
    assert_eq!(baseline, 1);
    assert!(toast_on_active_screen(&nav));

    nav.push(EmptyRoot, None);
    assert!(nav.has_toast(), "toast slot cleared by push");
    assert_eq!(toast_surface_child_count(), baseline, "toast widgets removed by push");
    // The toast must have been re-parented onto the NEW active screen, and
    // never linger on the system layer.
    assert!(toast_on_active_screen(&nav), "toast not re-parented onto pushed screen");
    assert_eq!(sys_layer_child_count(), 0);

    nav.replace(EmptyRoot, None);
    assert!(nav.has_toast(), "toast slot cleared by replace");
    assert_eq!(toast_surface_child_count(), baseline, "toast widgets removed by replace");
    assert!(toast_on_active_screen(&nav), "toast not re-parented onto replacement screen");

    nav.pop(None).expect("pop");
    assert!(nav.has_toast(), "toast slot cleared by pop");
    assert_eq!(toast_surface_child_count(), baseline, "toast widgets removed by pop");
    assert!(toast_on_active_screen(&nav), "toast not re-parented onto restored screen");
    assert_eq!(sys_layer_child_count(), 0);

    nav.dismiss_toast().expect("dismiss_toast");
    assert_eq!(toast_surface_child_count(), 0);
}

#[test]
fn show_toast_replaces_existing() {
    let mut nav = fresh_navigator();
    nav.show_toast(LabelToast, None);
    assert_eq!(toast_surface_child_count(), 1);

    nav.show_toast(LabelToast, None);
    // Old toast container deleted, new one created — net count still 1.
    assert_eq!(toast_surface_child_count(), 1);

    nav.dismiss_toast().expect("dismiss");
    assert_eq!(toast_surface_child_count(), 0);
}

#[test]
fn input_transparency_strips_clickable() {
    let mut nav = fresh_navigator();
    nav.show_toast(ButtonToast, None);
    pump();

    // The toast container is the navigator's, parented to the active screen.
    assert!(toast_on_active_screen(&nav), "toast must ride the active screen");
    let container = nav.toast_container_handle().expect("toast container");
    assert_subtree_not_clickable(container);

    nav.dismiss_toast().expect("dismiss");
}

// ── post_toast / post_dismiss_toast (cross-task queue) ──────────────────────

#[test]
fn post_toast_then_drain_shows_toast() {
    let mut nav = fresh_navigator();
    assert!(!nav.has_toast());

    // Post from "elsewhere" — same task here, but the channel path is
    // identical to a real cross-task post.
    oxivgl::navigator::post_toast(LabelToast, None);
    assert!(!nav.has_toast(), "post must not bypass the drain step");

    nav.drain_toast_requests();
    assert!(nav.has_toast(), "drain_toast_requests should pick up the queued show");
    assert_eq!(toast_surface_child_count(), 1);

    nav.dismiss_toast().expect("dismiss");
}

#[test]
fn post_toast_with_shows_a_non_send_view() {
    let mut nav = fresh_navigator();
    assert!(!nav.has_toast());

    // `StoringToast` retains a `Label<'static>` (raw pointer) and is `!Send`,
    // so `post_toast` would reject it. A builder closure carries only its
    // (Send) capture across the channel; the view is constructed render-side
    // on drain. This test won't compile if the channel ever requires the
    // view itself to be Send again.
    oxivgl::navigator::post_toast_with(StoringToast::default, None);
    assert!(!nav.has_toast(), "post must not bypass the drain step");

    nav.drain_toast_requests();
    assert!(nav.has_toast(), "drain should build the !Send view and show it");
    assert_eq!(toast_surface_child_count(), 1);

    nav.dismiss_toast().expect("dismiss");
}

#[test]
fn post_dismiss_toast_then_drain_dismisses() {
    let mut nav = fresh_navigator();
    nav.show_toast(LabelToast, None);
    assert!(nav.has_toast());

    oxivgl::navigator::post_dismiss_toast();
    nav.drain_toast_requests();
    assert!(!nav.has_toast());
    assert_eq!(toast_surface_child_count(), 0);
}

#[test]
fn multiple_persistent_posts_in_one_drain_collapse_to_latest() {
    let mut nav = fresh_navigator();
    // Persistent (None) toasts are sticky-status: each supersedes the
    // previous immediately. Three persistent shows + one dismiss: each
    // show replaces the last, then dismiss clears it. End state: no toast.
    // (Timed toasts instead play back in order — see the sequencing tests.)
    oxivgl::navigator::post_toast(LabelToast, None);
    oxivgl::navigator::post_toast(LabelToast, None);
    oxivgl::navigator::post_toast(LabelToast, None);
    oxivgl::navigator::post_dismiss_toast();

    nav.drain_toast_requests();
    assert!(!nav.has_toast());
    assert_eq!(toast_surface_child_count(), 0);
}

// ── Sequencing of timed toasts (anti-collapse) ──────────────────────────────

#[test]
fn timed_toasts_queue_and_play_sequentially() {
    let mut nav = fresh_navigator();
    reset_toast_create_count();

    // First timed toast displays immediately.
    nav.show_toast(CountingToast, long());
    pump();
    assert!(nav.has_toast());
    assert_eq!(toast_surface_child_count(), 1);
    assert_eq!(toast_create_count(), 1, "first toast should be built");

    // A second timed toast requested while the first is up must QUEUE,
    // not replace — its widgets are not built yet and the sys layer still
    // holds exactly one container.
    nav.show_toast(CountingToast, long());
    pump();
    assert!(nav.has_toast());
    assert_eq!(toast_surface_child_count(), 1, "queued toast must not be shown yet");
    assert_eq!(toast_create_count(), 1, "queued toast must not be built yet");

    // Dismissing the first promotes the queued one into the slot.
    nav.dismiss_toast().expect("dismiss first");
    pump();
    assert!(nav.has_toast(), "queued toast should be promoted");
    assert_eq!(toast_surface_child_count(), 1);
    assert_eq!(toast_create_count(), 2, "second toast built on promotion");

    // Dismissing the second empties the slot — nothing left to promote.
    nav.dismiss_toast().expect("dismiss second");
    pump();
    assert!(!nav.has_toast());
    assert_eq!(toast_surface_child_count(), 0);
}

#[test]
fn multiple_timed_posts_play_back_in_order() {
    let mut nav = fresh_navigator();
    nav.drain_toast_requests(); // clear any residue
    reset_toast_create_count();

    // Three timed toasts posted then drained in a single iteration must
    // NOT collapse to the last one: one shows, two queue.
    oxivgl::navigator::post_toast(CountingToast, long());
    oxivgl::navigator::post_toast(CountingToast, long());
    oxivgl::navigator::post_toast(CountingToast, long());
    nav.drain_toast_requests();
    pump();
    assert!(nav.has_toast());
    assert_eq!(toast_surface_child_count(), 1);
    assert_eq!(toast_create_count(), 1, "only the first should be built after drain");

    // Each dismiss reveals the next queued toast in order.
    nav.dismiss_toast().expect("dismiss 1");
    assert_eq!(toast_create_count(), 2);
    assert_eq!(toast_surface_child_count(), 1);

    nav.dismiss_toast().expect("dismiss 2");
    assert_eq!(toast_create_count(), 3);
    assert_eq!(toast_surface_child_count(), 1);

    nav.dismiss_toast().expect("dismiss 3");
    assert!(!nav.has_toast());
    assert_eq!(toast_surface_child_count(), 0);
}

#[test]
fn persistent_toast_supersedes_pending_queue() {
    let mut nav = fresh_navigator();
    reset_toast_create_count();

    // A timed toast shown, plus a second timed toast queued behind it.
    nav.show_toast(CountingToast, long());
    nav.show_toast(CountingToast, long()); // queued, not built
    assert_eq!(toast_create_count(), 1);
    assert_eq!(toast_surface_child_count(), 1);

    // A persistent toast supersedes the active one AND clears the queue,
    // so the queued toast is dropped without ever being built.
    nav.show_toast(CountingToast, None);
    pump();
    assert!(nav.has_toast());
    assert_eq!(toast_surface_child_count(), 1);
    assert_eq!(
        toast_create_count(),
        2,
        "active replaced + persistent built; queued one dropped uncreated",
    );

    // Dismissing the persistent toast leaves nothing — the queue was cleared.
    nav.dismiss_toast().expect("dismiss persistent");
    pump();
    assert!(!nav.has_toast());
    assert_eq!(toast_surface_child_count(), 0);
}

#[test]
fn drain_with_empty_queue_is_a_noop() {
    let mut nav = fresh_navigator();
    nav.drain_toast_requests();
    nav.drain_toast_requests();
    assert!(!nav.has_toast());
}

#[test]
fn post_toast_queue_full_drops_silently() {
    // Drain anything left over from prior tests in the queue.
    let mut nav = fresh_navigator();
    nav.drain_toast_requests();

    // Fill the queue (capacity 4 today). Posting more must not panic
    // or block — excess requests are dropped with a logged warning.
    for _ in 0..16 {
        oxivgl::navigator::post_toast(LabelToast, None);
    }
    nav.drain_toast_requests();
    // We can't assert exactly how many were dropped (it depends on the
    // queue capacity constant), but draining must leave us in a sane
    // state and the next dismiss must succeed.
    assert!(nav.has_toast());
    nav.dismiss_toast().expect("dismiss after queue-overflow recovery");
}

// ── Surface re-parenting (sys-layer → screen/backdrop visibility fix) ────────

#[test]
fn toast_rides_modal_backdrop_then_returns_to_screen() {
    // The toast lives on the active screen, but must stay ABOVE a modal.
    // Opening a modal lifts the toast onto the modal backdrop (on layer_top);
    // dismissing it returns the toast to the active screen. Throughout, the
    // toast never touches lv_layer_sys().
    let mut nav = fresh_navigator();
    nav.show_toast(LabelToast, None);
    assert!(toast_on_active_screen(&nav));

    nav.modal(EmptyRoot);
    let container = nav.toast_container_handle().expect("toast container");
    // SAFETY: container is the live toast handle; parent / layer_top queries
    // are always safe on a valid object.
    unsafe {
        let parent = oxivgl_sys::lv_obj_get_parent(container);
        assert_ne!(
            parent,
            oxivgl_sys::lv_screen_active(),
            "toast should move off the screen onto the backdrop",
        );
        assert_eq!(
            oxivgl_sys::lv_obj_get_parent(parent),
            oxivgl_sys::lv_layer_top(),
            "toast must ride the modal backdrop (a layer_top child)",
        );
    }
    assert_eq!(sys_layer_child_count(), 0, "toast must never use the system layer");

    nav.dismiss_modal().expect("dismiss modal");
    assert!(
        toast_on_active_screen(&nav),
        "toast must return to the active screen after the modal closes",
    );
    assert_eq!(sys_layer_child_count(), 0);

    nav.dismiss_toast().expect("dismiss toast");
}

#[test]
fn toast_shown_during_modal_lands_on_backdrop() {
    // A toast raised WHILE a modal is open must be created directly on the
    // backdrop (above the modal), not on the screen beneath it.
    let mut nav = fresh_navigator();
    nav.modal(EmptyRoot);
    nav.show_toast(LabelToast, None);

    let container = nav.toast_container_handle().expect("toast container");
    // SAFETY: valid handle; parent / layer_top queries are safe.
    unsafe {
        let parent = oxivgl_sys::lv_obj_get_parent(container);
        assert_eq!(
            oxivgl_sys::lv_obj_get_parent(parent),
            oxivgl_sys::lv_layer_top(),
            "toast shown during a modal must be created on the backdrop",
        );
    }
    assert_eq!(sys_layer_child_count(), 0);

    nav.dismiss_modal().expect("dismiss modal");
    assert!(toast_on_active_screen(&nav));
    nav.dismiss_toast().expect("dismiss toast");
}

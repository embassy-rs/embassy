// SPDX-License-Identifier: MIT OR Apache-2.0
//! View navigation stack with modal overlay support.
//!
//! `Navigator` manages a stack of [`View`](crate::view::View) instances,
//! supporting push/pop/replace transitions and modal overlays. Only the
//! topmost view (and any active modal) have live LVGL widgets.
//!
//! Views cannot call Navigator methods directly (the navigator owns
//! the view). Instead, [`View::update`](crate::view::View::update) and
//! [`View::on_event`](crate::view::View::on_event) return
//! [`NavAction`](crate::view::NavAction), which the render loop dispatches.

extern crate alloc;

use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::vec::Vec;

use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use core::time::Duration;
use oxivgl_sys::*;

use crate::driver::get_tick_ms;

use crate::view::{
    AnyView, NavAction, NavigationError, View,
    take_pending_event_action,
};
use crate::widgets::{AsLvHandle, Obj, Screen, ScreenAnim};

/// Symmetric inset (in pixels) applied to the default toast container
/// on left, right, and bottom — gives the toast a "floating card" look
/// by leaving a uniform sliver of the underlying view visible on all
/// three free sides. Override by re-setting size / alignment / margin
/// on the container inside [`View::create`].
pub const TOAST_MARGIN_PX: i32 = 2;

/// Default shadow blur radius (px) for the toast container. Together
/// with [`TOAST_SHADOW_OPA`], gives the toast a soft elevated halo.
pub const TOAST_SHADOW_WIDTH_PX: i32 = 12;

/// Default shadow opacity (0..=255) for the toast container.
pub const TOAST_SHADOW_OPA: u8 = 80;

/// Entry on the navigation stack, pairing a type-erased view with its
/// owning screen object (if any).
struct ViewEntry {
    view: Box<dyn AnyView>,
    /// The LVGL screen created for this view. Every view — including the
    /// root (see [`Navigator::push_root`]) — now owns its own loaded
    /// screen, so current code paths always store `Some`. The type stays
    /// `Option` only for the defensive fallback in [`Navigator::pop`].
    screen: Option<Obj<'static>>,
}

/// View navigation stack with modal overlay support.
///
/// The navigator owns all view instances. Views lower in the stack
/// have their widget trees destroyed but their struct state preserved.
/// Only the topmost view (and any active modal) have live widgets.
///
/// # Usage
///
/// For single-screen applications, use
/// [`run_app`](crate::view::run_app) directly. `Navigator` is for
/// multi-screen applications that need push/pop/replace/modal.
pub struct Navigator {
    /// Full-screen navigation stack. Index 0 is the root view.
    stack: Vec<ViewEntry>,
    /// Currently active modal, if any. Rendered as a child of the
    /// modal backdrop on `lv_layer_top()`.
    modal: Option<Box<dyn AnyView>>,
    /// Full-size click-absorbing backdrop the modal is built inside.
    /// Dropping this `Obj` deletes the backdrop and (cascade) the modal
    /// widget tree in one shot.
    modal_backdrop: Option<Obj<'static>>,
    /// Focus state captured when the active modal was opened — restored
    /// on dismiss. `None` when no modal is active or when the modal did
    /// not provide an [`input_group`](crate::view::View::input_group).
    saved_focus: Option<SavedFocus>,
    /// Currently active global toast, if any. Rendered on the current
    /// topmost real surface (active screen, or the modal backdrop while a
    /// modal is open) rather than `lv_layer_sys()`, because the system layer
    /// is not composited reliably in PARTIAL render mode. It is re-parented
    /// across navigation and modal changes by `reattach_toast`
    /// so it persists across page switches and stays above any modal. See
    /// [`Navigator::show_toast`].
    toast: Option<Box<dyn AnyView>>,
    /// The container the active toast was created into. Owned here
    /// (rather than by the toast view) so dismissal deletes exactly the
    /// toast's widgets and nothing else on the surface it currently rides.
    toast_container: Option<Obj<'static>>,
    /// Auto-dismiss deadline for the active toast, in `get_tick_ms` units
    /// (wrap-aware u32 milliseconds). Compared via `wrapping_sub`.
    /// `None` while a toast is showing means it is persistent (no
    /// auto-dismiss); `None` with no active toast means the slot is empty.
    toast_deadline_ms: Option<u32>,
    /// Toasts waiting to be shown after the active one is dismissed.
    /// Populated when a timed toast is requested while another is already
    /// on screen; drained one-at-a-time by [`Navigator::promote_next_toast`]
    /// so rapidly-posted toasts are displayed sequentially instead of
    /// collapsing to only the last one. Bounded by [`TOAST_PENDING_CAPACITY`].
    toast_queue: VecDeque<PendingToast>,
}

/// A toast deferred behind the currently-displayed one. Its widgets are
/// not created until it is promoted into the active slot, so `did_show` /
/// `will_hide` are never called while it waits here.
struct PendingToast {
    view: Box<dyn AnyView>,
    duration: Option<Duration>,
}

/// Maximum number of timed toasts that may wait behind the active one.
/// Toasts are notifications, not data; once the queue is full further
/// requests are dropped (with a warning) rather than displacing the ones
/// already waiting.
const TOAST_PENDING_CAPACITY: usize = 4;

impl Navigator {
    /// Create a new empty navigator.
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            modal: None,
            modal_backdrop: None,
            saved_focus: None,
            toast: None,
            toast_container: None,
            toast_deadline_ms: None,
            toast_queue: VecDeque::new(),
        }
    }

    /// Push the initial root view. Called once during setup.
    ///
    /// The root view is built on its own freshly-created LVGL screen, which
    /// is made active via `lv_screen_load`, rather than reusing the default
    /// active screen LVGL creates at `lv_init`. This gives the root the same
    /// owned-screen treatment as every pushed view, so `pop`-to-root and the
    /// toast surface re-parenting (`reattach_toast`)
    /// work uniformly. Its widgets are created immediately.
    pub fn push_root(&mut self, view: impl View) {
        let mut boxed: Box<dyn AnyView> = Box::new(view);

        // Create and load a real screen for the root (NOT the default active
        // screen) so the root owns a screen like every other view.
        let new_screen = Screen::create();
        Screen::load_instant(&new_screen);

        boxed
            .create(&new_screen)
            .expect("root view create failed");

        // Default impl of register_events_on attaches the trampoline on the
        // root screen we just created and loaded.
        boxed.register_events_on(&new_screen);
        boxed.did_show();
        activate_view_group(boxed.input_group());

        self.stack.push(ViewEntry {
            view: boxed,
            screen: Some(new_screen),
        });
    }

    /// Push a new view onto the stack.
    ///
    /// 1. Calls `will_hide()` on the current top view.
    /// 2. Creates a new LVGL screen for the new view.
    /// 3. Loads the new screen (makes it active).
    /// 4. Calls `create(container)` on the new view.
    /// 5. Registers event handlers.
    /// 6. Cleans the old screen's widget tree (preserving view state).
    /// 7. Calls `did_show()` on the new view.
    pub fn push(&mut self, view: impl View, anim: Option<ScreenAnim>) {
        self.push_boxed(Box::new(view), anim);
    }

    /// Push a boxed (type-erased) view.
    fn push_boxed(&mut self, mut boxed: Box<dyn AnyView>, anim: Option<ScreenAnim>) {
        // Notify current top view.
        if let Some(top) = self.stack.last_mut() {
            top.view.will_hide();
        }

        // Capture the old screen handle BEFORE loading the new screen,
        // because lv_screen_active() will change after Screen::load.
        let old_screen_h = self.stack.last().map(|top| {
            top.screen
                .as_ref()
                .map(|s| s.lv_handle())
                .unwrap_or_else(|| {
                    // Root view uses the LVGL default screen.
                    unsafe { lv_screen_active() }
                })
        });

        // Create a new screen for the incoming view.
        let new_screen = Screen::create();

        // Load the new screen BEFORE create/register_events so that
        // lv_screen_active() returns the new screen during those calls.
        if let Some(ref a) = anim {
            Screen::load(&new_screen, a, false);
        } else {
            Screen::load_instant(&new_screen);
        }

        boxed
            .create(&new_screen)
            .expect("pushed view create failed");

        // Default impl of register_events_on attaches on the new screen
        // we just created and loaded.
        boxed.register_events_on(&new_screen);

        // Move the active toast (if any) onto the new screen before cleaning
        // the old one — otherwise lv_obj_clean below would delete it. The new
        // view is already built, so the toast lands on top of it.
        self.reattach_toast();

        // Clean the old screen's children (widget tree) to free memory,
        // but keep the screen object alive for potential pop animation.
        // SAFETY: old_screen_h was captured above while still valid. The old
        // screen object is still alive (just no longer active). lv_obj_clean
        // deletes all children but keeps the screen itself. Note: any Obj
        // wrappers held by the old view now contain stale pointers — their
        // Drop uses lv_obj_is_valid() as a guard (see spec-memory-lifetime §8.1).
        if let Some(h) = old_screen_h {
            unsafe { lv_obj_clean(h) };
        }

        boxed.did_show();
        activate_view_group(boxed.input_group());

        self.stack.push(ViewEntry {
            view: boxed,
            screen: Some(new_screen),
        });
    }

    /// Pop the current view and return to the previous one.
    ///
    /// Returns `Err(NavigationError::StackEmpty)` if only the root view
    /// remains (the root cannot be popped).
    pub fn pop(&mut self, anim: Option<ScreenAnim>) -> Result<(), NavigationError> {
        if self.stack.len() <= 1 {
            return Err(NavigationError::StackEmpty);
        }

        // Remove the top view — will_hide + drop.
        let mut popped = self.stack.pop().unwrap();
        popped.view.will_hide();

        // Load the restored screen BEFORE dropping the popped screen.
        // This ensures lv_screen_active() returns the correct screen
        // during create/register_events, and avoids the undefined state
        // of having no active screen. Scope the `top` borrow so we can call
        // `&self` helpers (reattach_toast) once it is released.
        let container_handle = {
            let top = self.stack.last_mut().unwrap();
            if let Some(ref top_screen) = top.screen {
                if let Some(ref a) = anim {
                    Screen::load(top_screen, a, false);
                } else {
                    Screen::load_instant(top_screen);
                }
                top_screen.lv_handle()
            } else {
                // Defensive fallback — NOT expected to run. Every `ViewEntry`
                // (root included, since `push_root` now creates and loads its
                // own screen) stores `Some(screen)`, so `top.screen` is always
                // `Some` here. Kept only to avoid leaving LVGL with no active
                // screen if that invariant is ever broken; loads the LVGL
                // default screen as a last resort.
                debug_assert!(false, "pop: top ViewEntry has no screen — invariant broken");
                // SAFETY: lv_display_get_default/lv_display_get_screen returns
                // the LVGL default screen (index 0), which is always valid. We
                // get the handle BEFORE dropping popped (which deletes its screen).
                let default_screen = unsafe {
                    let disp = lv_display_get_default();
                    lv_display_get_screen_active(disp)
                };
                Screen::load_instant(&Obj::from_raw_non_owning(default_screen));
                default_screen
            }
        };

        // Move the active toast onto the restored (now active) screen before
        // dropping the popped screen, so it is not deleted with it.
        self.reattach_toast();

        // Now safe to drop the popped view and its screen.
        drop(popped);

        // Non-owning handle — the screen is owned by the ViewEntry, not
        // this temporary. Child suppresses Drop so no screen deletion.
        let container = Obj::from_raw_non_owning(container_handle);
        {
            let top = self.stack.last_mut().unwrap();
            top.view
                .create(&container)
                .map_err(NavigationError::CreateFailed)?;

            top.view.register_events_on(&container);
            top.view.did_show();
            activate_view_group(top.view.input_group());
        }

        // Raise the toast above the just-rebuilt view.
        self.reattach_toast();
        Ok(())
    }

    /// Replace the current view without preserving it on the stack.
    ///
    /// The current view is dropped. The new view takes its place at the
    /// same stack depth.
    pub fn replace(&mut self, view: impl View, anim: Option<ScreenAnim>) {
        self.replace_boxed(Box::new(view), anim);
    }

    /// Replace with a boxed view.
    fn replace_boxed(&mut self, mut boxed: Box<dyn AnyView>, anim: Option<ScreenAnim>) {
        // Notify the view being replaced so it can save state if needed.
        if let Some(top) = self.stack.last_mut() {
            top.view.will_hide();
        }

        // Create a new screen and load it BEFORE dropping the old view,
        // ensuring there is always a valid active screen.
        let new_screen = Screen::create();
        if let Some(ref a) = anim {
            Screen::load(&new_screen, a, false);
        } else {
            Screen::load_instant(&new_screen);
        }

        // Move the active toast onto the new (now active) screen before
        // dropping the old one, so it survives the screen deletion.
        self.reattach_toast();

        // Now safe to drop the old view and its screen.
        self.stack.pop();

        boxed
            .create(&new_screen)
            .expect("replaced view create failed");
        boxed.register_events_on(&new_screen);
        boxed.did_show();
        activate_view_group(boxed.input_group());

        // Raise the toast above the just-built replacement view.
        self.reattach_toast();

        self.stack.push(ViewEntry {
            view: boxed,
            screen: Some(new_screen),
        });
    }

    /// Show a modal overlay on top of the current view.
    ///
    /// The current view's widget tree stays alive and visible underneath.
    /// The modal's widgets are created on `lv_layer_top()`.
    ///
    /// Only one modal can be active at a time. Calling `modal()` while
    /// a modal is already open replaces it.
    pub fn modal(&mut self, view: impl View) {
        self.modal_boxed(Box::new(view));
    }

    /// Show a boxed modal.
    fn modal_boxed(&mut self, mut boxed: Box<dyn AnyView>) {
        // Dismiss any existing modal first.
        if self.modal.is_some() {
            let _ = self.dismiss_modal();
        }

        // Create a full-size click-absorbing backdrop on lv_layer_top,
        // then build the modal's widgets as children of the backdrop.
        // This gives every modal automatic input absorption (touches on
        // the backdrop never reach the view beneath) and a single root
        // to delete on dismiss (deleting the backdrop cascades).
        // SAFETY: lv_layer_top is a valid LVGL global after lv_init.
        let layer_top_h = unsafe { lv_layer_top() };
        assert!(!layer_top_h.is_null(), "lv_layer_top returned NULL");
        // SAFETY: layer_top_h is non-null.
        let backdrop_h = unsafe { lv_obj_create(layer_top_h) };
        assert!(!backdrop_h.is_null(), "modal backdrop creation failed");
        let backdrop = Obj::from_raw(backdrop_h);

        // Full-size, click-absorbing, no scroll. Transparent fill so the
        // background view stays visible unless the modal itself draws a
        // dim layer.
        // SAFETY: backdrop_h is the freshly-created object above.
        unsafe {
            let pct100 = lv_pct(100);
            lv_obj_set_size(backdrop_h, pct100, pct100);
            lv_obj_set_style_bg_opa(backdrop_h, 0, 0);
            lv_obj_set_style_border_width(backdrop_h, 0, 0);
            lv_obj_set_style_pad_all(backdrop_h, 0, 0);
            lv_obj_add_flag(
                backdrop_h,
                crate::enums::ObjFlag::CLICKABLE.0,
            );
            lv_obj_remove_flag(
                backdrop_h,
                crate::enums::ObjFlag::SCROLLABLE.0,
            );
        }

        boxed
            .create(&backdrop)
            .expect("modal view create failed");
        // register_events_on defaults to attaching the trampoline on the
        // backdrop, so bubbled events from any modal widget reach the
        // view's on_event. Modal views that catch events on intermediate
        // widgets can still override register_events_on.
        boxed.register_events_on(&backdrop);
        boxed.did_show();

        // If the modal exposes a focus group, snapshot the current focus
        // state and route input to the modal's group. Restored on dismiss.
        if let Some(modal_group) = boxed.input_group() {
            self.saved_focus = Some(SavedFocus::capture());
            modal_group.set_default();
            modal_group.assign_to_keyboard_indevs();
        }

        self.modal = Some(boxed);
        self.modal_backdrop = Some(backdrop);

        // Raise the active toast (if any) onto the modal backdrop so it stays
        // above the modal — preserving the "toast is always on top" contract.
        self.reattach_toast();
    }

    /// Dismiss the current modal overlay.
    ///
    /// Deletes the backdrop (which cascades to the modal's widget tree)
    /// and restores any focus state captured on open. Returns `Err` if
    /// no modal is active.
    pub fn dismiss_modal(&mut self) -> Result<(), NavigationError> {
        let mut modal = match self.modal.take() {
            Some(m) => m,
            None => return Err(NavigationError::NoActiveModal),
        };
        modal.will_hide();
        // Move the active toast off the backdrop onto the active screen
        // before the backdrop is deleted. Take the backdrop out first so
        // `current_toast_surface` resolves to the screen (modal_backdrop is
        // now None), then drop it.
        let backdrop = self.modal_backdrop.take();
        self.reattach_toast();
        // Drop the backdrop Obj — lv_obj_delete cascades to all descendants
        // (the modal's widget tree). Any Obj wrappers held inside the
        // modal view now hold stale pointers; their Drop uses
        // lv_obj_is_valid as a guard (spec-memory-lifetime §8.1).
        drop(backdrop);
        drop(modal);

        // Restore the pre-modal focus state, if we captured it.
        if let Some(saved) = self.saved_focus.take() {
            saved.restore();
        }
        Ok(())
    }

    /// Whether a modal is currently showing.
    pub fn has_modal(&self) -> bool {
        self.modal.is_some()
    }

    /// Show a global passive status overlay on top of the current view.
    ///
    /// Unlike [`modal`](Self::modal), the toast:
    /// - rides the current topmost real surface (active screen, or the modal
    ///   backdrop while a modal is open) and is automatically re-parented
    ///   across `push` / `replace` / `pop` and modal open/dismiss, so it
    ///   persists across page switches and stays above any modal. It does
    ///   **not** live on `lv_layer_sys()` — that layer is not composited
    ///   reliably in PARTIAL render mode (see
    ///   `docs/spec-navigation.md §4.3`);
    /// - is **passive** — `register_events` is never called, and every
    ///   widget the view creates has the `CLICKABLE` flag cleared so
    ///   touches pass through to the view beneath;
    /// - has its auto-dismiss timer owned by the navigator. If
    ///   `duration` is `Some`, the toast is dismissed automatically the
    ///   next time [`tick_toast`](Self::tick_toast) runs after the
    ///   deadline. `None` means the caller must dismiss explicitly.
    ///
    /// # Default geometry
    ///
    /// Before [`View::create`] runs, the container is sized and
    /// positioned for a bottom-anchored "floating card": full sys-layer
    /// width with a symmetric [`TOAST_MARGIN_PX`]-pixel inset on left
    /// and right, height hugging its content, anchored at
    /// `Align::BottomMid` lifted by the same margin. The view can
    /// override any of this by re-setting size, alignment, or styles
    /// on the container inside `create`.
    ///
    /// # Sequencing
    ///
    /// Toasts are displayed one at a time and **never overwritten before
    /// they have been seen**:
    ///
    /// - A **timed** toast requested while another is on screen is
    ///   *queued* (FIFO, a few deep) and shown after
    ///   the current one is dismissed — so a burst of `show_toast` /
    ///   [`post_toast`] calls plays back in order instead of collapsing to
    ///   only the last one. Each timed toast therefore stays for its full
    ///   requested `duration` before the next appears; because
    ///   [`tick_toast`](Self::tick_toast) (which auto-dismisses) runs after
    ///   the render in each loop iteration, even a very short duration
    ///   still gets at least one render cycle on screen.
    /// - A **persistent** toast (`duration == None`) is a sticky status:
    ///   it supersedes whatever is showing, clears the pending queue, and
    ///   is displayed immediately. It stays until dismissed explicitly.
    pub fn show_toast(&mut self, view: impl View, duration: Option<Duration>) {
        self.show_toast_boxed(Box::new(view), duration);
    }

    fn show_toast_boxed(&mut self, boxed: Box<dyn AnyView>, duration: Option<Duration>) {
        match duration {
            // Persistent (sticky) status: supersede everything and show now.
            None => {
                self.toast_queue.clear();
                if self.toast.is_some() {
                    let _ = self.teardown_active_toast();
                }
                self.display_toast_now(boxed, None);
            }
            // Timed toast: show now if the slot is free, otherwise queue it
            // behind the active toast for sequential playback.
            Some(_) => {
                if self.toast.is_some() {
                    self.enqueue_toast(boxed, duration);
                } else {
                    self.display_toast_now(boxed, duration);
                }
            }
        }
    }

    /// Queue a toast to be shown after the active one is dismissed. Drops
    /// (with a warning) if the pending queue is already full — toasts are
    /// notifications, not data.
    fn enqueue_toast(&mut self, boxed: Box<dyn AnyView>, duration: Option<Duration>) {
        if self.toast_queue.len() >= TOAST_PENDING_CAPACITY {
            warn!(
                "nav show_toast: pending queue full ({}), toast dropped",
                TOAST_PENDING_CAPACITY,
            );
            return;
        }
        self.toast_queue.push_back(PendingToast { view: boxed, duration });
    }

    /// The LVGL object the active toast should be parented to so it renders
    /// on the current topmost surface: the modal backdrop while a modal is
    /// open (so the toast stays above the modal), otherwise the active screen.
    ///
    /// Toasts ride a real screen/backdrop object rather than `lv_layer_sys()`
    /// because the system layer is not composited reliably in PARTIAL render
    /// mode — see the comment in [`display_toast_now`](Self::display_toast_now)
    /// and `docs/spec-navigation.md §4.3`.
    fn current_toast_surface(&self) -> *mut lv_obj_t {
        if let Some(backdrop) = self.modal_backdrop.as_ref() {
            backdrop.lv_handle()
        } else {
            // SAFETY: lv_screen_active returns the active screen, always valid
            // after init (the navigator always has a loaded screen).
            unsafe { lv_screen_active() }
        }
    }

    /// Re-parent the active toast (if any) onto the current topmost surface
    /// and raise it to the front. `lv_obj_set_parent` re-appends the toast as
    /// the last child of the surface, so it lands on top of whatever was just
    /// built there.
    ///
    /// Called after every change of the topmost surface — full-screen
    /// navigation (`push`/`pop`/`replace`) and modal open/dismiss — so the
    /// toast persists across page switches and is never destroyed together
    /// with the surface it was on. Must run **before** the previous surface's
    /// widget tree is cleaned/deleted, and **after** the new surface's view is
    /// created (so the toast stays on top).
    fn reattach_toast(&self) {
        if let Some(container) = self.toast_container.as_ref() {
            let handle = container.lv_handle();
            let surface = self.current_toast_surface();
            // SAFETY: handle is the live toast container (checked valid);
            // surface is a valid screen/backdrop object.
            unsafe {
                if lv_obj_is_valid(handle) && !surface.is_null() {
                    lv_obj_set_parent(handle, surface);
                }
            }
        }
    }

    /// Build and display `boxed` in the (assumed empty) active toast slot.
    /// Returns `true` on success; logs and returns `false` if the view's
    /// `create` fails (leaving the slot empty and the surface clean).
    fn display_toast_now(&mut self, mut boxed: Box<dyn AnyView>, duration: Option<Duration>) -> bool {
        debug_assert!(self.toast.is_none(), "display_toast_now: slot not empty");

        // Create a dedicated container on the current topmost *real* surface
        // (active screen, or the modal backdrop while a modal is open) — NOT
        // on `lv_layer_sys()`. In PARTIAL render mode (ESP32) the system layer
        // is not composited reliably onto passive redraws, so a toast shown on
        // a static screen could silently fail to appear (worst on the first
        // cold boot). Ordinary screen content, by contrast, is always
        // composited — the background view renders every frame. Parenting the
        // toast into the normal screen tree therefore makes it as reliable as
        // any other widget. Dismissal deletes only this container's subtree;
        // `reattach_toast` keeps it on the topmost surface across navigation
        // and modal changes. See `docs/spec-navigation.md §4.3`.
        let surface = self.current_toast_surface();
        assert!(!surface.is_null(), "toast surface is NULL");
        // SAFETY: surface is a valid LVGL screen/backdrop object.
        let container_handle = unsafe { lv_obj_create(surface) };
        assert!(!container_handle.is_null(), "toast container creation failed");
        let container = Obj::from_raw(container_handle);

        // Default geometry: bottom-anchored floating card with a
        // symmetric margin on left / right / bottom. Applied BEFORE
        // create() so a custom view can override on the same container.
        // SAFETY: container_handle is the freshly-created object above.
        unsafe {
            lv_obj_set_width(container_handle, lv_pct(100));
            lv_obj_set_style_margin_left(container_handle, TOAST_MARGIN_PX, 0);
            lv_obj_set_style_margin_right(container_handle, TOAST_MARGIN_PX, 0);
            lv_obj_set_height(container_handle, crate::style::LV_SIZE_CONTENT);
            lv_obj_align(
                container_handle,
                lv_align_t_LV_ALIGN_BOTTOM_MID as lv_align_t,
                0,
                -TOAST_MARGIN_PX,
            );
            // Soft symmetric halo — reinforces the "elevated card" look.
            lv_obj_set_style_shadow_width(container_handle, TOAST_SHADOW_WIDTH_PX, 0);
            lv_obj_set_style_shadow_opa(container_handle, TOAST_SHADOW_OPA, 0);
        }

        if let Err(e) = boxed.create(&container) {
            warn!("nav show_toast: create failed: {:?}", e);
            // Drop the container so we leave the surface clean.
            drop(container);
            return false;
        }

        // Strip CLICKABLE from the container and all its descendants so
        // touches pass through to whatever is beneath the toast. This
        // enforces the passivity contract regardless of what the view did.
        // SAFETY: container_handle is the freshly-created object above;
        // its tree is what the view just populated.
        unsafe { remove_clickable_recursive(container_handle) };

        // Settle layout synchronously, then invalidate the toast's final area.
        // The container is `LV_SIZE_CONTENT`-high and bottom-aligned, so its
        // real coordinates are only known after a layout pass; forcing it here
        // makes the single post-create invalidation target the correct stripe
        // deterministically (no reliance on the redraw racing layout settling).
        // In PARTIAL render mode (ESP32) only invalidated regions are
        // recomposited; this guarantees the toast's region is dirty. Harmless
        // in FULL/DIRECT mode (host).
        // SAFETY: container_handle is the live toast container.
        unsafe { lv_obj_update_layout(container_handle) };
        container.invalidate();

        // Intentionally do NOT call boxed.register_events_on(): the default
        // impl registers on lv_screen_active() (the background view's
        // screen) and would dangle across page switches. The toast manages
        // its own surface re-parenting via `reattach_toast`.

        boxed.did_show();

        self.toast = Some(boxed);
        self.toast_container = Some(container);
        self.toast_deadline_ms = duration.map(|d| {
            // Saturate the Duration into u32 ms (≈49.7 days max), then
            // wrap-add to the current tick. The compare in tick_toast uses
            // `wrapping_sub` so wrap-around is correct as long as the
            // duration is < ~25 days.
            let ms = d.as_millis().min(u32::MAX as u128) as u32;
            get_tick_ms().wrapping_add(ms)
        });
        true
    }

    /// Promote the next queued toast into the active slot, if the slot is
    /// free and the queue is non-empty. Displays at most one toast per
    /// call, so each gets its own render cycles. Skips past any whose
    /// `create` fails so a single bad toast doesn't stall the queue.
    fn promote_next_toast(&mut self) {
        if self.toast.is_some() {
            return;
        }
        while let Some(next) = self.toast_queue.pop_front() {
            if self.display_toast_now(next.view, next.duration) {
                break;
            }
        }
    }

    /// Tear down the currently-displayed toast (delete its widgets, clear
    /// the slot). Does **not** touch the pending queue. Returns
    /// `Err(NoActiveToast)` if none is showing.
    fn teardown_active_toast(&mut self) -> Result<(), NavigationError> {
        let mut toast = match self.toast.take() {
            Some(t) => t,
            None => return Err(NavigationError::NoActiveToast),
        };
        self.toast_deadline_ms = None;
        toast.will_hide();

        // Delete the toast's container (and thus its widget subtree).
        // The toast view's internal Obj wrappers now hold stale pointers;
        // Obj::Drop uses lv_obj_is_valid as a guard (spec §8.1).
        if let Some(container) = self.toast_container.take() {
            let handle = container.lv_handle();
            // Suppress container's own Drop — we delete it explicitly.
            core::mem::forget(container);
            // SAFETY: handle was returned by lv_obj_create above; checked
            // valid here in case lv_obj_clean on the sys layer destroyed
            // it externally between show and dismiss.
            unsafe {
                if lv_obj_is_valid(handle) {
                    lv_obj_delete(handle);
                }
            }
        }
        drop(toast);
        Ok(())
    }

    /// Dismiss the active toast overlay and advance to the next queued
    /// toast, if any.
    ///
    /// Returns `Err(NoActiveToast)` if none is showing. Note that on
    /// success a *different* toast may immediately take the slot if one
    /// was queued behind the dismissed one (see [`show_toast`](Self::show_toast)
    /// sequencing).
    pub fn dismiss_toast(&mut self) -> Result<(), NavigationError> {
        self.teardown_active_toast()?;
        self.promote_next_toast();
        Ok(())
    }

    /// Whether a toast is currently showing.
    pub fn has_toast(&self) -> bool {
        self.toast.is_some()
    }

    /// Test-only introspection: the raw LVGL handle of the active toast's
    /// container, if any. Exposed so the integration test crate can verify
    /// which surface the toast is parented to (the visibility fix moved it
    /// off `lv_layer_sys()` onto the active screen / modal backdrop). Not
    /// part of the stable API.
    #[doc(hidden)]
    pub fn toast_container_handle(&self) -> Option<*mut lv_obj_t> {
        self.toast_container.as_ref().map(|c| c.lv_handle())
    }

    /// Get a mutable reference to the active toast view, if any.
    pub fn active_toast_mut(&mut self) -> Option<&mut dyn AnyView> {
        self.toast.as_mut().map(|t| &mut **t as &mut dyn AnyView)
    }

    /// Maintenance tick for the toast slot — call once per render-loop
    /// iteration.
    ///
    /// - Dismisses the toast if its auto-dismiss deadline has passed, then
    ///   promotes the next queued toast (if any) into the slot.
    /// - Self-heals the slot if the toast container was destroyed
    ///   externally (e.g. some other code cleared the system layer):
    ///   drops the orphaned view and promotes the next queued toast so the
    ///   slot doesn't get stuck.
    pub fn tick_toast(&mut self) {
        if self.toast.is_none() {
            return;
        }

        // External-destruction guard: if the container handle is no
        // longer valid, drop the view + clear the slot.
        if let Some(container) = self.toast_container.as_ref() {
            let handle = container.lv_handle();
            // SAFETY: lv_obj_is_valid handles any pointer (returns false
            // for freed objects).
            if !unsafe { lv_obj_is_valid(handle) } {
                if let Some(mut t) = self.toast.take() {
                    t.will_hide();
                }
                // Suppress the container's Drop — its target is already gone.
                if let Some(orphan) = self.toast_container.take() {
                    core::mem::forget(orphan);
                }
                self.toast_deadline_ms = None;
                self.promote_next_toast();
                return;
            }
        }

        // Wrap-aware compare: `now - deadline >= 0` (as i32) means we've
        // reached the deadline, robust to u32 wrap. `dismiss_toast` also
        // promotes the next queued toast.
        if let Some(deadline) = self.toast_deadline_ms
            && get_tick_ms().wrapping_sub(deadline) as i32 >= 0
        {
            let _ = self.dismiss_toast();
        }
    }

    /// Number of views on the stack.
    pub fn depth(&self) -> usize {
        self.stack.len()
    }

    /// Get a mutable reference to the active (topmost) view.
    pub fn active_view_mut(&mut self) -> Option<&mut dyn AnyView> {
        self.stack
            .last_mut()
            .map(|e| &mut *e.view as &mut dyn AnyView)
    }

    /// Get a mutable reference to the active modal, if any.
    pub fn active_modal_mut(&mut self) -> Option<&mut dyn AnyView> {
        self.modal.as_mut().map(|m| &mut **m as &mut dyn AnyView)
    }

    /// Process a [`NavAction`] returned by a view.
    pub fn process_action(&mut self, action: NavAction) {
        match action {
            NavAction::None => {}
            NavAction::Push(view, anim) => self.push_boxed(view, anim),
            NavAction::Pop(anim) => {
                if let Err(e) = self.pop(anim) {
                    warn!("nav pop failed: {}", e);
                }
            }
            NavAction::Replace(view, anim) => self.replace_boxed(view, anim),
            NavAction::Modal(view) => self.modal_boxed(view),
            NavAction::DismissModal => {
                if let Err(e) = self.dismiss_modal() {
                    warn!("nav dismiss_modal failed: {}", e);
                }
            }
            NavAction::ShowToast(view, duration) => self.show_toast_boxed(view, duration),
            NavAction::DismissToast => {
                if let Err(e) = self.dismiss_toast() {
                    warn!("nav dismiss_toast failed: {}", e);
                }
            }
        }
    }

    /// Process any pending event action stashed by the on_event trampoline.
    /// Returns `true` if an event action was processed.
    pub fn process_pending_event_action(&mut self) -> bool {
        if let Some(action) = take_pending_event_action() {
            self.process_action(action);
            true
        } else {
            false
        }
    }

    /// Drain any toast requests posted from background tasks via
    /// [`post_toast`] / [`post_dismiss_toast`].
    ///
    /// Called once per render-loop iteration by `run_app_nav`. Each
    /// queued request becomes a `show_toast` or `dismiss_toast` call.
    /// Draining several `Show` requests in one iteration is safe: at most
    /// one is displayed and the rest are queued for sequential playback
    /// (see [`show_toast`](Self::show_toast) sequencing), so they no longer
    /// collapse to only the last one.
    pub fn drain_toast_requests(&mut self) {
        while let Ok(req) = TOAST_CHANNEL.try_receive() {
            match req {
                ToastRequest::Show(make, duration) => {
                    // Construct the view here, on the render task — so it need
                    // not be `Send`. Only the builder closure crossed the
                    // channel; `create()` then runs on the LVGL task as usual.
                    self.show_toast_boxed(make(), duration);
                }
                ToastRequest::Dismiss => {
                    let _ = self.dismiss_toast();
                }
            }
        }
    }
}

impl Default for Navigator {
    fn default() -> Self {
        Self::new()
    }
}

/// Route keypad/encoder input to the active full-screen view's focus group,
/// if it provides one via [`View::input_group`](crate::view::View::input_group).
///
/// Called after every full-screen activation (`push_root` / `push` / `pop` /
/// `replace`). Unlike the modal path, there is no save/restore: a full-screen
/// transition *replaces* the active view, so each activation simply rebinds the
/// default group and the keyboard/encoder devices to the new top view's group.
/// A view returning `None` leaves the previous routing untouched (its widgets
/// are gone, so the stale group has no focusable members).
fn activate_view_group(group: Option<crate::group::GroupRef>) {
    if let Some(g) = group {
        g.set_default();
        g.assign_to_keyboard_indevs();
    }
}

// ---------------------------------------------------------------------------
// Cross-task toast posting (TOAST_CHANNEL + post_toast / post_dismiss_toast)
// ---------------------------------------------------------------------------

/// Capacity of the global toast request queue. Toasts are infrequent;
/// 4 outstanding requests tolerate brief contention without backpressure.
const TOAST_QUEUE_CAPACITY: usize = 4;

/// A boxed `Send` closure that constructs a toast view on the render task.
///
/// Only this closure crosses the [`TOAST_CHANNEL`]; the `View` it produces
/// is built render-side, so the view itself need not be `Send` (widget
/// wrappers hold raw `lv_obj_t` pointers and are `!Send`).
type ToastBuilder = Box<dyn FnOnce() -> Box<dyn AnyView> + Send>;

/// A request enqueued by [`post_toast`] / [`post_toast_with`] /
/// [`post_dismiss_toast`] and drained by [`Navigator::drain_toast_requests`].
enum ToastRequest {
    Show(ToastBuilder, Option<Duration>),
    Dismiss,
}

/// Global queue of toast requests. Posted to from any async task,
/// drained by the render loop via [`Navigator::drain_toast_requests`].
static TOAST_CHANNEL: Channel<CriticalSectionRawMutex, ToastRequest, TOAST_QUEUE_CAPACITY> =
    Channel::new();

/// Queue a global passive toast from **any** async task by handing the
/// render loop a **builder closure** — including background workers that
/// hold no `Navigator` handle.
///
/// The closure is invoked on the next render-loop iteration (`run_app_nav`
/// calls [`Navigator::drain_toast_requests`] every tick), which constructs
/// the view and feeds it through the same path as
/// [`Navigator::show_toast`], so all the passivity / persistence /
/// auto-dismiss guarantees apply identically — the only difference is who
/// initiated it.
///
/// Use this for **truly global status messages** (e.g. "No SD card",
/// "BLE disconnected") raised before any particular view is on screen
/// or from a task that has no reason to know about the active view.
///
/// # Why a closure, not a `View`
///
/// Only the closure crosses the `Channel`, so only `make` must be `Send` —
/// and it genuinely is, since it captures just the toast's config
/// (`String`, colors, icon source). The `View` it returns is built
/// render-side, where `create()` already runs, so it **need not be
/// `Send`**. This is what lets a toast store its widget wrappers (whose
/// `Drop` frees the style `Rc`s, the leak-free pattern) even though those
/// wrappers hold raw `lv_obj_t` pointers and are `!Send`.
///
/// For a toast whose view *is* `Send` (a trivial config struct that builds
/// and forgets its widgets), [`post_toast`] is shorter sugar over this.
///
/// # Backpressure
///
/// The queue holds 4 outstanding requests (`TOAST_QUEUE_CAPACITY`).
/// If full, this call logs a warning and drops the request rather than
/// blocking — toasts are notifications, not data; a dropped duplicate
/// is preferable to async deadlock.
pub fn post_toast_with<V: View>(
    make: impl FnOnce() -> V + Send + 'static,
    duration: Option<Duration>,
) {
    let req = ToastRequest::Show(Box::new(move || Box::new(make()) as Box<dyn AnyView>), duration);
    if TOAST_CHANNEL.try_send(req).is_err() {
        warn!("post_toast: queue full ({}), request dropped", TOAST_QUEUE_CAPACITY);
    }
}

/// Queue a global passive toast from **any** async task by value.
///
/// Thin convenience over [`post_toast_with`] for toasts whose view is
/// `Send` — i.e. a config struct that does not retain its widget wrappers.
/// A view that *stores* its widgets (the leak-free pattern) is `!Send`;
/// post it with [`post_toast_with`] instead, handing over a builder closure.
///
/// Same delivery, sequencing, and backpressure semantics as
/// [`post_toast_with`].
pub fn post_toast<V: View + Send>(view: V, duration: Option<Duration>) {
    post_toast_with(move || view, duration);
}

/// Queue a request to dismiss the active toast from any async task.
///
/// Same delivery semantics as [`post_toast`]. No-op on the render-loop
/// side if no toast is active when the request is processed.
pub fn post_dismiss_toast() {
    if TOAST_CHANNEL.try_send(ToastRequest::Dismiss).is_err() {
        warn!(
            "post_dismiss_toast: queue full ({}), request dropped",
            TOAST_QUEUE_CAPACITY,
        );
    }
}

/// Snapshot of the input-focus state captured when a modal with an
/// [`input_group`](crate::view::View::input_group) opens. Restored on
/// dismiss so the background view regains its key/encoder input routing.
///
/// Stores raw pointers; we do not own any of these — the groups and
/// indevs are managed by app or LVGL.
struct SavedFocus {
    /// Previous default group (may be NULL).
    prev_default: *mut lv_group_t,
    /// Per-indev (KEYPAD/ENCODER) group bindings to restore on dismiss.
    /// Heap-allocated rather than fixed-size since LVGL allows any
    /// number of indevs in principle; in practice this is 1–2 entries.
    prev_indev_groups: Vec<(*mut lv_indev_t, *mut lv_group_t)>,
}

impl SavedFocus {
    /// Capture the current default group and per-indev group bindings.
    fn capture() -> Self {
        // SAFETY: lv_group_get_default reads a global; safe after lv_init.
        let prev_default = unsafe { lv_group_get_default() };

        let mut prev_indev_groups = Vec::new();
        // SAFETY: lv_indev_get_next(NULL) returns the first indev or
        // NULL. lv_indev_get_type and lv_indev_get_group are safe on
        // any non-null lv_indev_t.
        unsafe {
            let mut indev = lv_indev_get_next(core::ptr::null_mut());
            while !indev.is_null() {
                let kind = lv_indev_get_type(indev);
                if kind == lv_indev_type_t_LV_INDEV_TYPE_KEYPAD
                    || kind == lv_indev_type_t_LV_INDEV_TYPE_ENCODER
                {
                    prev_indev_groups.push((indev, lv_indev_get_group(indev)));
                }
                indev = lv_indev_get_next(indev);
            }
        }

        Self { prev_default, prev_indev_groups }
    }

    /// Restore the captured state.
    fn restore(self) {
        // SAFETY: pointers were valid when captured. If the previous
        // default group or any indev has been deleted since (rare,
        // would imply app-side teardown during a modal), LVGL handles a
        // NULL/dangling group pointer by routing to no group; we do not
        // attempt to verify aliveness because LVGL does not expose a
        // per-group is_valid check.
        unsafe {
            lv_group_set_default(self.prev_default);
            for (indev, group) in self.prev_indev_groups {
                lv_indev_set_group(indev, group);
            }
        }
    }
}

/// Strip `CLICKABLE` and `CLICK_FOCUSABLE` from `obj` and every descendant.
///
/// Used to make a toast subtree input-transparent regardless of what the
/// toast view did when it built its widgets.
///
/// # Safety
/// `obj` must be a valid `lv_obj_t*` whose subtree is fully constructed
/// (no concurrent mutation from another task).
unsafe fn remove_clickable_recursive(obj: *mut lv_obj_t) {
    if obj.is_null() {
        return;
    }
    let flags = crate::enums::ObjFlag::CLICKABLE.0 | crate::enums::ObjFlag::CLICK_FOCUSABLE.0;
    unsafe {
        lv_obj_remove_flag(obj, flags);
        let n = lv_obj_get_child_count(obj);
        for i in 0..n {
            let child = lv_obj_get_child(obj, i as i32);
            remove_clickable_recursive(child);
        }
    }
}

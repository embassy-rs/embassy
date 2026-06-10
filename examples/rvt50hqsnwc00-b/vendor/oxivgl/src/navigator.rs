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
use alloc::vec::Vec;

use oxivgl_sys::*;

use crate::view::{
    AnyView, NavAction, NavigationError, View,
    take_pending_event_action,
};
use crate::widgets::{AsLvHandle, Obj, Screen, ScreenAnim};

/// Entry on the navigation stack, pairing a type-erased view with its
/// owning screen object (if any).
struct ViewEntry {
    view: Box<dyn AnyView>,
    /// The LVGL screen created for this view. `None` for the root view
    /// which uses LVGL's default screen.
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
    /// Currently active modal, if any. Rendered on `lv_layer_top()`.
    modal: Option<Box<dyn AnyView>>,
}

impl Navigator {
    /// Create a new empty navigator.
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            modal: None,
        }
    }

    /// Push the initial root view. Called once during setup.
    ///
    /// The root view uses the default LVGL screen. Its widgets are
    /// created immediately.
    pub fn push_root(&mut self, view: impl View) {
        let mut boxed: Box<dyn AnyView> = Box::new(view);

        // Use the default active screen as the container. Child suppresses
        // Drop so the LVGL screen is never deleted by Rust.
        let screen_handle = unsafe { lv_screen_active() };
        assert!(!screen_handle.is_null(), "no active screen");
        let container = Obj::from_raw_non_owning(screen_handle);

        boxed
            .create(&container)
            .expect("root view create failed");

        // register_events() default calls register_event_on(self, lv_screen_active()).
        // lv_screen_active() is the default screen — correct at this point.
        boxed.register_events();
        boxed.did_show();

        self.stack.push(ViewEntry {
            view: boxed,
            screen: None,
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

        // register_events() calls register_event_on(self, lv_screen_active()).
        // Since we loaded new_screen above, lv_screen_active() == new_screen.
        boxed.register_events();

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

        // Rebuild the now-top view's widgets.
        let top = self.stack.last_mut().unwrap();

        // Load the restored screen BEFORE dropping the popped screen.
        // This ensures lv_screen_active() returns the correct screen
        // during create/register_events, and avoids the undefined state
        // of having no active screen.
        let container_handle = if let Some(ref top_screen) = top.screen {
            if let Some(ref a) = anim {
                Screen::load(top_screen, a, false);
            } else {
                Screen::load_instant(top_screen);
            }
            top_screen.lv_handle()
        } else {
            // Root view: load the default LVGL screen. We must get its
            // handle BEFORE dropping popped (which deletes popped.screen).
            // SAFETY: lv_display_get_default/lv_display_get_screen returns
            // the LVGL default screen (index 0), which is always valid.
            let default_screen = unsafe {
                let disp = lv_display_get_default();
                lv_display_get_screen_active(disp)
            };
            // The default screen may be behind the popped screen. Load it
            // before dropping so it becomes active.
            Screen::load_instant(&Obj::from_raw_non_owning(default_screen));
            default_screen
        };

        // Now safe to drop the popped view and its screen.
        drop(popped);

        // Non-owning handle — the screen is owned by the ViewEntry, not
        // this temporary. Child suppresses Drop so no screen deletion.
        let container = Obj::from_raw_non_owning(container_handle);
        top.view
            .create(&container)
            .map_err(NavigationError::CreateFailed)?;

        top.view.register_events();
        top.view.did_show();
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

        // Now safe to drop the old view and its screen.
        self.stack.pop();

        boxed
            .create(&new_screen)
            .expect("replaced view create failed");
        boxed.register_events();
        boxed.did_show();

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

        let layer_top = Screen::layer_top();
        boxed
            .create(&layer_top)
            .expect("modal view create failed");
        // For modals, register_events default would register on
        // lv_screen_active() which is the background view's screen.
        // Modal views should override register_events to register on
        // the layer_top container instead, or use EVENT_BUBBLE.
        // We call register_events() and trust the view's override.
        boxed.register_events();
        boxed.did_show();
        self.modal = Some(boxed);
    }

    /// Dismiss the current modal overlay.
    ///
    /// Cleans `lv_layer_top()` children. Returns `Err` if no modal is active.
    pub fn dismiss_modal(&mut self) -> Result<(), NavigationError> {
        if let Some(mut modal) = self.modal.take() {
            modal.will_hide();
            // SAFETY: lv_layer_top() returns the global overlay object (valid
            // after lv_init). lv_obj_clean deletes all children. Any Obj
            // wrappers in the modal view now hold stale pointers — their Drop
            // uses lv_obj_is_valid() as a guard (spec-memory-lifetime §8.1).
            // We clean before dropping the modal so LVGL removes its widgets
            // from the display immediately.
            let layer = unsafe { lv_layer_top() };
            unsafe { lv_obj_clean(layer) };
            // modal is dropped here — Obj::drop guards prevent double-free.
            Ok(())
        } else {
            Err(NavigationError::NoActiveModal)
        }
    }

    /// Whether a modal is currently showing.
    pub fn has_modal(&self) -> bool {
        self.modal.is_some()
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
}

// SPDX-License-Identifier: MIT OR Apache-2.0
//! View trait and navigation primitives.
//!
//! The [`View`](crate::view::View) trait defines a single screen of UI with a repeatable
//! lifecycle: `create` → `update` → `on_event` → `will_hide`, cycling
//! on each navigation transition. See `docs/spec-navigation.md`.

use alloc::boxed::Box;
use core::cell::UnsafeCell;
use core::ffi::c_void;

use core::time::Duration;
use embassy_time::Timer;

use oxivgl_sys::*;

use crate::{
    display::{lvgl_disp_init, LvglBuffers, DISPLAY_READY},
    driver::LvglDriver,
    enums::EventCode,
    event::Event,
    widgets::{AsLvHandle, Obj, ScreenAnim, WidgetError},
};

/// LVGL timer tick interval (ms). `LV_DEF_REFR_PERIOD / 4` yields ~4 ticks
/// per refresh cycle, keeping animations smooth at ~30 fps.
const LVGL_TICK_MS: u64 = LV_DEF_REFR_PERIOD as u64 / 4;

/// A single view of UI (one screen or modal in a navigation stack).
///
/// The lifecycle is:
///
/// 1. **Construction** — caller creates the struct (e.g. `Default::default()`)
/// 2. [`create`](View::create) — build widgets into `container`; may be called
///    multiple times across push/pop cycles
/// 3. [`did_show`](View::did_show) — post-creation setup (optional)
/// 4. [`update`](View::update) — per-tick polling (runs in render loop)
/// 5. [`on_event`](View::on_event) — LVGL event dispatch
/// 6. [`will_hide`](View::will_hide) — save transient state before teardown
///
/// Override [`on_event`](View::on_event) to handle LVGL widget events (clicks,
/// presses, etc.) without writing `unsafe extern "C"` callbacks. Widgets that
/// should deliver events to `on_event` must have `ObjFlag::EVENT_BUBBLE`
/// set so the event reaches the screen-level handler.
///
/// For nested widget trees (e.g. buttons inside a container), override
/// [`register_events_on`](View::register_events_on) to add event handlers
/// on intermediate objects via [`register_event_on`].
pub trait View: Sized + 'static {
    /// Build all LVGL widgets for this view into `container`.
    ///
    /// Called each time this view becomes the active (topmost) view —
    /// both on initial display and when a view above it is popped.
    /// `container` is the LVGL screen object to build into.
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError>;

    /// Refresh widget values from application state. Called every render tick.
    ///
    /// Return [`NavAction::None`] to stay on this view, or a navigation
    /// action to trigger a transition. This is the primary integration
    /// point for external events — poll channels/shared state here.
    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }

    /// Handle a bubbled LVGL event. Return [`NavAction::None`] to stay on
    /// this view, or a navigation action to trigger a transition.
    fn on_event(&mut self, _event: &Event) -> NavAction {
        NavAction::None
    }

    /// Register event handlers. Called once after [`create`](View::create),
    /// with the same `container` that was passed to `create`.
    ///
    /// Default registers the view's event trampoline on `container`, so
    /// bubbled events from any descendant reach [`on_event`](View::on_event).
    /// Override to register on additional objects (e.g. intermediate
    /// containers that catch bubbled events for sub-trees).
    ///
    /// Receiving the container as an argument — rather than reading
    /// `lv_screen_active()` — is what makes the default impl correct for
    /// modals: when the navigator builds a modal, `container` is
    /// `lv_layer_top()`, not the background view's screen.
    fn register_events_on(&mut self, container: &Obj<'static>) {
        register_event_on(self, container.lv_handle());
    }

    /// Called before this view's widget tree is destroyed (navigating away).
    /// Save any transient widget state here. Default is a no-op.
    fn will_hide(&mut self) {}

    /// Called after this view becomes visible again (navigated back to).
    /// Default is a no-op.
    fn did_show(&mut self) {}

    /// Focus group containing this view's focusable widgets.
    ///
    /// When non-`None`, the navigator activates this group on
    /// modal open (sets it as default + binds it to all keyboard /
    /// encoder input devices) and restores the previously active focus
    /// state on dismiss. The view owns the [`Group`](crate::group::Group)
    /// internally; this method just borrows a non-owning handle.
    ///
    /// Default `None` — only OSD-style modals that need key input
    /// usually return `Some`.
    fn input_group(&self) -> Option<crate::group::GroupRef> {
        None
    }
}

// ---------------------------------------------------------------------------
// NavAction
// ---------------------------------------------------------------------------

/// Navigation action requested by a view.
///
/// Returned from [`View::update`] and [`View::on_event`]. The render loop
/// (or [`Navigator`](crate::navigator::Navigator)) processes the action
/// after the method returns.
pub enum NavAction {
    /// No navigation requested.
    None,
    /// Push a new view onto the stack.
    Push(Box<dyn AnyView>, Option<ScreenAnim>),
    /// Pop the current view (return to previous).
    Pop(Option<ScreenAnim>),
    /// Replace the current view (non-reversible transition).
    Replace(Box<dyn AnyView>, Option<ScreenAnim>),
    /// Show a modal overlay on top of the current view.
    Modal(Box<dyn AnyView>),
    /// Dismiss the current modal overlay.
    DismissModal,
    /// Show a global passive status overlay (toast) on the system layer.
    ///
    /// Unlike [`Modal`](Self::Modal), the toast persists across page
    /// switches and registers no input handlers. If the `Duration` is
    /// `Some`, the navigator auto-dismisses on expiry. See
    /// [`Navigator::show_toast`](crate::navigator::Navigator::show_toast).
    ShowToast(Box<dyn AnyView>, Option<Duration>),
    /// Dismiss the active global toast overlay.
    DismissToast,
}

impl NavAction {
    /// Convenience: push a view with an optional animation.
    pub fn push(view: impl View, anim: Option<ScreenAnim>) -> Self {
        Self::Push(Box::new(view), anim)
    }

    /// Convenience: replace the current view.
    pub fn replace(view: impl View, anim: Option<ScreenAnim>) -> Self {
        Self::Replace(Box::new(view), anim)
    }

    /// Convenience: show a modal overlay.
    pub fn modal(view: impl View) -> Self {
        Self::Modal(Box::new(view))
    }

    /// Convenience: show a passive global toast.
    ///
    /// `duration` is an optional auto-dismiss timeout owned by the navigator.
    /// `None` means the toast stays until [`NavAction::DismissToast`] (or
    /// [`Navigator::dismiss_toast`](crate::navigator::Navigator::dismiss_toast)).
    pub fn show_toast(view: impl View, duration: Option<Duration>) -> Self {
        Self::ShowToast(Box::new(view), duration)
    }

    /// Returns `true` if this is [`NavAction::None`].
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }
}

// ---------------------------------------------------------------------------
// AnyView — object-safe trait for type-erased views
// ---------------------------------------------------------------------------

/// Object-safe trait for type-erased views stored in a
/// [`Navigator`](crate::navigator::Navigator) stack.
///
/// Implemented automatically for all [`View`] types via blanket impl.
/// Users should implement [`View`], never `AnyView` directly.
pub trait AnyView: 'static {
    /// Build widgets into `container`. See [`View::create`].
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError>;
    /// Per-tick update. See [`View::update`].
    fn update(&mut self) -> Result<NavAction, WidgetError>;
    /// Handle a bubbled LVGL event. See [`View::on_event`].
    fn on_event(&mut self, event: &Event) -> NavAction;
    /// Register event handlers. See [`View::register_events_on`].
    fn register_events_on(&mut self, container: &Obj<'static>);
    /// Called before widget teardown. See [`View::will_hide`].
    fn will_hide(&mut self);
    /// Called after view becomes visible again. See [`View::did_show`].
    fn did_show(&mut self);
    /// Focus group for this view (modals only). See [`View::input_group`].
    fn input_group(&self) -> Option<crate::group::GroupRef>;
}

impl<T: View> AnyView for T {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {
        View::create(self, container)
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        View::update(self)
    }

    fn on_event(&mut self, event: &Event) -> NavAction {
        View::on_event(self, event)
    }

    fn register_events_on(&mut self, container: &Obj<'static>) {
        View::register_events_on(self, container)
    }

    fn will_hide(&mut self) {
        View::will_hide(self)
    }

    fn did_show(&mut self) {
        View::did_show(self)
    }

    fn input_group(&self) -> Option<crate::group::GroupRef> {
        View::input_group(self)
    }
}

// ---------------------------------------------------------------------------
// NavigationError
// ---------------------------------------------------------------------------

/// Errors from navigation operations.
#[derive(Debug)]
pub enum NavigationError {
    /// Cannot pop the root view — the stack has only one entry.
    StackEmpty,
    /// No modal is currently active.
    NoActiveModal,
    /// No toast overlay is currently active.
    NoActiveToast,
    /// View creation failed during a navigation transition.
    CreateFailed(WidgetError),
}

impl core::fmt::Display for NavigationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::StackEmpty => write!(f, "cannot pop the root view"),
            Self::NoActiveModal => write!(f, "no active modal to dismiss"),
            Self::NoActiveToast => write!(f, "no active toast to dismiss"),
            Self::CreateFailed(e) => write!(f, "view creation failed: {:?}", e),
        }
    }
}

// ---------------------------------------------------------------------------
// Pending event action (single-threaded stash for trampoline → navigator)
// ---------------------------------------------------------------------------

/// SAFETY: LVGL runs on a single task — the event trampoline (which writes
/// to this cell during `lv_timer_handler`) and the render loop (which reads
/// it) execute sequentially within the same async task. No concurrent
/// access occurs. `NavAction` contains `Box<dyn AnyView>` which is `!Send`,
/// but that is fine because the cell never crosses task/thread boundaries.
///
/// This invariant would break if LVGL were driven from multiple threads.
/// The single-task requirement is enforced architecturally (run_app / run_app_nav
/// own both the render loop and the timer handler).
struct SyncCell(UnsafeCell<Option<NavAction>>);
unsafe impl Sync for SyncCell {}

static PENDING_EVENT_ACTION: SyncCell = SyncCell(UnsafeCell::new(None));

/// Take the pending event action stashed by the trampoline, if any.
pub(crate) fn take_pending_event_action() -> Option<NavAction> {
    // SAFETY: single-threaded access — see SyncCell doc.
    unsafe { (*PENDING_EVENT_ACTION.0.get()).take() }
}

/// Register event handlers for the view by delegating to
/// [`View::register_events_on`] with `container` as the target.
///
/// The `view` reference must remain at a stable address for the lifetime of
/// the LVGL display (guaranteed by `run_app` and `host_main!`).
pub fn register_view_events<V: View>(view: &mut V, container: &Obj<'static>) {
    view.register_events_on(container);
}


/// Register the view's event trampoline on a specific LVGL object.
/// Use this from [`View::register_events_on`] to catch events on containers
/// or other intermediate objects that don't bubble to the screen.
///
/// # Address stability (not enforced by the type system)
///
/// `view` must remain at a stable address for the LVGL display lifetime.
/// This is guaranteed by:
/// - `run_app`: view lives in the async task frame (pinned by the executor)
/// - `host_main!`: view is stack-local before the infinite loop
/// - `Navigator`: views live inside `Box<dyn AnyView>` (heap-stable)
///
/// Do not call this on a view that may be moved after registration.
pub fn register_event_on<V: View>(view: &mut V, obj: *mut lv_obj_t) {
    assert!(!obj.is_null(), "register_event_on: obj must not be null");
    let view_ptr = view as *mut V as *mut c_void;
    // SAFETY: obj non-null (asserted above); view_ptr remains valid for the
    // LVGL display lifetime (see address stability guarantee in doc comment).
    // The view lives behind Box indirection (navigator) or in a pinned async
    // frame (run_app), so the pointer survives Vec reallocations.
    unsafe {
        lv_obj_add_event_cb(
            obj,
            Some(view_event_trampoline::<V>),
            EventCode::ALL.0,
            view_ptr,
        );
    };
}

/// SAFETY: `user_data` is a `*mut V` set by `register_event_on`. The pointer
/// remains valid because the view lives behind Box indirection (navigator) or
/// in a pinned async frame (run_app), so address stability is guaranteed even
/// if the navigator's Vec reallocates. See `register_event_on` doc comment.
unsafe extern "C" fn view_event_trampoline<V: View>(e: *mut lv_event_t) {
    if e.is_null() {
        return;
    }
    unsafe {
        let view = lv_event_get_user_data(e) as *mut V;
        if !view.is_null() {
            let event = Event::from_raw(e);
            let action = (*view).on_event(&event);
            if !action.is_none() {
                // Stash the action for the render loop to process.
                // First action per tick wins (subsequent are dropped).
                let slot = &mut *PENDING_EVENT_ACTION.0.get();
                if slot.is_none() {
                    *slot = Some(action);
                }
            }
        }
    }
}

/// Run the LVGL render loop with a [`View`].
///
/// This is an embassy async task. Spawn it alongside your other application
/// tasks. It initialises LVGL, creates the view, then loops: calls
/// `V::update` and drives `lv_timer_handler` every tick.
///
/// `w` and `h` are the display resolution in pixels. `bufs` must be a
/// `'static` caller-allocated [`LvglBuffers`] sized for the screen width.
///
/// `view` is the initial view instance. Its `create` method is called once
/// the display is ready.
///
/// Never returns.
pub async fn run_app<V: View, const BYTES: usize>(
    w: i32,
    h: i32,
    bufs: &'static mut LvglBuffers<BYTES>,
    mut view: V,
) -> ! {
    info!("UI task started");
    let driver = LvglDriver::init(w, h);
    // SAFETY: lv_init() has been called inside LvglDriver::init() above.
    unsafe { lvgl_disp_init(w, h, bufs) };

    DISPLAY_READY.wait().await;
    info!("Display ready");

    // Wrap the active screen in a non-owning handle (Child suppresses Drop,
    // so the LVGL screen is never deleted by Rust).
    let screen_handle = unsafe { lv_screen_active() };
    assert!(!screen_handle.is_null(), "no active screen after display init");
    let container = Obj::from_raw_non_owning(screen_handle);

    if let Err(e) = view.create(&container) {
        warn!("Could not create LVGL widgets: {:?}, disabling UI", e);
        loop {
            Timer::after(embassy_time::Duration::from_secs(60)).await;
        }
    }

    register_view_events(&mut view, &container);

    loop {
        debug!("Rendering UI loop iteration");
        let action = view.update()
            .unwrap_or_else(|e| { warn!("Failed to update widgets: {:?}", e); NavAction::None });
        debug_assert!(action.is_none(), "NavAction ignored in run_app — use run_app_nav for navigation");

        // Drive lv_timer_handler 4× per update cycle (once per refresh period)
        // so LVGL animations stay smooth while update() is called at ~30fps.
        for _ in 0..4 {
            debug!("LVGL tick/timer handler");
            driver.timer_handler();
            Timer::after(embassy_time::Duration::from_millis(LVGL_TICK_MS)).await;
        }

        // Drain any pending event action (stashed by on_event trampoline).
        let _event_action = take_pending_event_action();

        // Note: NavAction processing is handled by Navigator (see
        // run_app_nav). This simple run_app ignores actions —
        // use run_app_nav for multi-screen applications.
    }
}

/// Run the LVGL render loop with navigation support.
///
/// Like [`run_app`], but creates a [`Navigator`](crate::navigator::Navigator)
/// that processes [`NavAction`] values from `update()` and `on_event()`.
/// Use this for multi-screen applications that need push/pop/replace/modal.
///
/// `initial` is the root view. Never returns.
pub async fn run_app_nav<const BYTES: usize>(
    w: i32,
    h: i32,
    bufs: &'static mut LvglBuffers<BYTES>,
    initial: impl View,
) -> ! {
    run_app_nav_inner(w, h, bufs, initial, None, false, no_wake).await
}

/// Like [`run_app_nav`], but also registers a **TIMER-mode** keypad input
/// device driven by `keypad`.
///
/// The navigator routes each active view's
/// [`input_group`](View::input_group) to the keypad, so focusable widgets can
/// be navigated with discrete keys — from a GPIO button task or, on a
/// touchscreen, from on-screen buttons that call
/// [`KeypadState::press`](crate::indev::KeypadState::press) /
/// [`release`](crate::indev::KeypadState::release). LVGL polls the device on its
/// own read timer.
///
/// For an interrupt-driven, poll-free input path (a driver that already decodes
/// long-press/repeat and uses [`KeypadState::send`](crate::indev::KeypadState::send)),
/// use [`run_app_nav_keypad_events`] instead.
///
/// `keypad` must be `'static` (typically a `static KeypadState`). Never returns.
pub async fn run_app_nav_keypad<const BYTES: usize>(
    w: i32,
    h: i32,
    bufs: &'static mut LvglBuffers<BYTES>,
    initial: impl View,
    keypad: &'static crate::indev::KeypadState,
) -> ! {
    run_app_nav_inner(w, h, bufs, initial, Some(keypad), false, no_wake).await
}

/// Like [`run_app_nav_keypad`], but **event-driven and poll-free**.
///
/// Creates the keypad in EVENT mode (no LVGL read timer) and races the
/// inter-tick sleep against `wake`. When `wake` resolves — e.g. your
/// interrupt-driven input task signalled after `KEYPAD.send(key)` — the loop
/// reads the device immediately, so a decoded key reaches the screen with no
/// periodic polling of either the button or the indev.
///
/// `wake` is called fresh each tick to produce a future to race; supply your
/// input signal, e.g. `|| async { WAKE.wait().await }`. Never returns.
pub async fn run_app_nav_keypad_events<const BYTES: usize, Fut>(
    w: i32,
    h: i32,
    bufs: &'static mut LvglBuffers<BYTES>,
    initial: impl View,
    keypad: &'static crate::indev::KeypadState,
    wake: impl Fn() -> Fut,
) -> !
where
    Fut: core::future::Future<Output = ()>,
{
    run_app_nav_inner(w, h, bufs, initial, Some(keypad), true, wake).await
}

/// No-wake closure for the timer-only loops: a future that never resolves, so
/// the inter-tick race always falls through to the normal tick.
fn no_wake() -> core::future::Pending<()> {
    core::future::pending()
}

/// Shared implementation of the navigation render loop.
///
/// `event_mode` selects EVENT mode for the keypad (read only on `wake`) vs
/// TIMER mode (LVGL polls). `wake` is raced against each inter-tick sleep; when
/// it resolves the loop reads the keypad and runs `update()` immediately.
async fn run_app_nav_inner<const BYTES: usize, Fut>(
    w: i32,
    h: i32,
    bufs: &'static mut LvglBuffers<BYTES>,
    initial: impl View,
    keypad: Option<&'static crate::indev::KeypadState>,
    event_mode: bool,
    wake: impl Fn() -> Fut,
) -> !
where
    Fut: core::future::Future<Output = ()>,
{
    info!("UI task started (navigator)");
    let driver = LvglDriver::init(w, h);
    // SAFETY: lv_init() has been called inside LvglDriver::init() above.
    unsafe { lvgl_disp_init(w, h, bufs) };

    DISPLAY_READY.wait().await;
    info!("Display ready");

    // Register the keypad device (if any) BEFORE push_root, so the root view's
    // input_group binds to it. Held for the loop's lifetime; since the loop
    // never returns, its Drop never runs.
    let keypad_dev = keypad.and_then(|state| {
        let res = if event_mode {
            crate::indev::KeypadIndev::new_event(state)
        } else {
            crate::indev::KeypadIndev::new(state)
        };
        match res {
            Ok(kp) => Some(kp),
            Err(e) => {
                warn!("keypad indev create failed: {:?}", e);
                None
            }
        }
    });

    let mut nav = crate::navigator::Navigator::new();
    nav.push_root(initial);

    loop {
        // Poll active view and modal for NavActions.
        let action = nav
            .active_view_mut()
            .map(|v| v.update())
            .unwrap_or(Ok(NavAction::None))
            .unwrap_or_else(|e| {
                warn!("view update: {:?}", e);
                NavAction::None
            });

        let modal_action = nav
            .active_modal_mut()
            .map(|m| m.update())
            .unwrap_or(Ok(NavAction::None))
            .unwrap_or_else(|e| {
                warn!("modal update: {:?}", e);
                NavAction::None
            });

        // Drive lv_timer_handler 4× per update cycle, racing each inter-tick
        // sleep against `wake`. If `wake` resolves first (input arrived), read
        // the keypad now and break out early to run update() sooner.
        for _ in 0..4 {
            driver.timer_handler();
            match embassy_time::with_timeout(
                embassy_time::Duration::from_millis(LVGL_TICK_MS),
                wake(),
            )
            .await
            {
                Ok(()) => {
                    if let Some(kp) = &keypad_dev {
                        kp.read();
                    }
                    break;
                }
                Err(_timeout) => {} // normal tick
            }
        }

        // Event actions (from on_event trampoline) take priority.
        // Only process update/modal actions if no event action fired.
        let event_handled = nav.process_pending_event_action();
        if !event_handled {
            if !action.is_none() {
                nav.process_action(action);
            }
            if !modal_action.is_none() {
                nav.process_action(modal_action);
            }
        }

        // Drain toast requests posted from background tasks via
        // navigator::post_toast / post_dismiss_toast.
        nav.drain_toast_requests();

        // Auto-dismiss expired toasts; self-heal if the slot was orphaned.
        nav.tick_toast();
    }
}

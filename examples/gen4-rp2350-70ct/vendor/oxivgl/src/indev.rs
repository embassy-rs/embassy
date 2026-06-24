// SPDX-License-Identifier: MIT OR Apache-2.0
//! Input devices — non-owning query wrappers plus owning keypad and pointer
//! devices.
//!
//! [`Indev`](crate::indev::Indev) is a read-only handle for inspecting the
//! active device inside an event handler.
//! [`KeypadIndev`](crate::indev::KeypadIndev) is an *owning* KEYPAD input
//! device whose key state is supplied by the application through a
//! [`KeypadState`](crate::indev::KeypadState): a lock-free cell that any task —
//! a debounced GPIO button task, or an on-screen button's event handler on a
//! touchscreen — writes, and LVGL's focus engine reads.
//! [`PointerIndev`](crate::indev::PointerIndev) is the touchscreen analogue: an
//! *owning* POINTER device fed raw `(x, y)` coordinates through a
//! [`PointerState`](crate::indev::PointerState) cell or a polling closure, so a
//! view can be navigated by tapping a widget at a coordinate.
//!
//! Both owning devices take input in oxivgl's own vocabulary — LVGL keys and
//! raw coordinates — never a BSP/MCU/driver type, so they stay portable across
//! boards and MCUs.
//!
//! # Driving focus navigation
//!
//! ```no_run
//! use oxivgl::indev::{KeypadIndev, KeypadState};
//! use oxivgl::enums::Key;
//!
//! // Declare the shared state as a `static` (it must outlive the device).
//! static KEYPAD: KeypadState = KeypadState::new();
//!
//! # fn demo() -> Result<(), oxivgl::widgets::WidgetError> {
//! // Register the device once at startup; bind it to a focus group.
//! let _keypad = KeypadIndev::new(&KEYPAD)?;
//!
//! // From a button task or an on-screen button's PRESSED / RELEASED handler:
//! KEYPAD.press(Key::NEXT);   // advance focus to the next group member
//! KEYPAD.release();          // button up
//! # Ok(())
//! # }
//! ```
//!
//! Reporting the *currently held* key (rather than queuing discrete events)
//! lets LVGL derive long-press and repeat itself — holding a button repeats the
//! key, a tap moves focus once.
//!
//! # Event-driven, poll-free input
//!
//! If your input driver is interrupt-driven and *already* decodes debounce /
//! long-press / repeat, it emits finished, discrete key events. Feed those with
//! [`KeypadState::send`](crate::indev::KeypadState::send) and an **EVENT-mode**
//! device ([`KeypadIndev::new_event`](crate::indev::KeypadIndev::new_event)):
//! each event is one focus step, LVGL adds no repeat of its own, and nothing is
//! polled — the device is only read when you call
//! [`KeypadIndev::read`](crate::indev::KeypadIndev::read).
//!
//! ```no_run
//! use oxivgl::indev::{KeypadIndev, KeypadState};
//! use oxivgl::enums::Key;
//!
//! static KEYPAD: KeypadState = KeypadState::new();
//!
//! # fn demo() -> Result<(), oxivgl::widgets::WidgetError> {
//! let keypad = KeypadIndev::new_event(&KEYPAD)?;   // no read timer
//!
//! // Producer (e.g. an interrupt-driven async task) on each decoded event:
//! KEYPAD.send(Key::NEXT);          // queue one discrete step
//! // …then signal your render loop, which calls:
//! keypad.read();                   // drain the queue into LVGL now
//! # Ok(())
//! # }
//! ```
//!
//! `lv_indev_read` / `lv_timer_handler` must run on the LVGL task, so the
//! interrupt hands off via the lock-free queue + a wake signal — a wake, not a
//! poll. With the built-in render loop, use
//! [`run_app_nav_keypad_events`](crate::view::run_app_nav_keypad_events), which
//! wires the wake for you.

use alloc::boxed::Box;
use core::ffi::c_void;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering};

use oxivgl_sys::*;

use crate::enums::Key;
use crate::group::Group;
use crate::widgets::WidgetError;

/// 2D point (mirrors `lv_point_t`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Point {
    /// Horizontal coordinate.
    pub x: i32,
    /// Vertical coordinate.
    pub y: i32,
}

/// Non-owning handle to an LVGL input device.
///
/// LVGL owns the indev lifecycle — this wrapper only provides read access.
/// Obtain via [`Indev::active()`] inside an event handler.
pub struct Indev {
    ptr: *mut lv_indev_t,
}

impl core::fmt::Debug for Indev {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Indev").finish_non_exhaustive()
    }
}

impl Indev {
    /// Currently active input device (valid only inside an event handler).
    ///
    /// Returns `None` when no indev is being processed.
    pub fn active() -> Option<Self> {
        let ptr = unsafe { lv_indev_active() };
        if ptr.is_null() {
            None
        } else {
            Some(Self { ptr })
        }
    }

    /// Pointer movement vector since last read.
    pub fn get_vect(&self) -> Point {
        let mut pt: lv_point_t = unsafe { core::mem::zeroed() };
        unsafe { lv_indev_get_vect(self.ptr, &mut pt) };
        Point { x: pt.x, y: pt.y }
    }

    /// Consecutive short-click count.
    ///
    /// Updated before `SHORT_CLICKED` fires. Resets after timeout or
    /// movement beyond the short-click distance threshold.
    pub fn short_click_streak(&self) -> u8 {
        unsafe { lv_indev_get_short_click_streak(self.ptr) }
    }
}

/// Capacity of the one-shot key queue (see [`KeypadState::send`]). Eight
/// outstanding discrete events absorb a short burst of decoded key presses
/// without dropping any before the render loop drains them.
const KEYPAD_QUEUE_CAP: usize = 8;

/// Lock-free key state shared between an input producer and a [`KeypadIndev`].
///
/// Supports two producer models — pick one per device, don't mix them:
///
/// **Held** ([`press`](Self::press) / [`release`](Self::release)) — report the
/// key currently held down (`0` = none). LVGL reads the held state and derives
/// press / long-press / repeat / release itself. Use this for **raw momentary
/// buttons** (or on-screen touch buttons): a tap is one step, a hold repeats.
///
/// **One-shot** ([`send`](Self::send)) — post a stream of *discrete* key events;
/// each delivers exactly **one** focus step and LVGL adds **no** repeat of its
/// own. Use this when your input driver **already decodes** debounce /
/// long-press / repeat and emits finished events — feeding those as a held key
/// would double the repeat. Pair with [`KeypadIndev::new_event`] +
/// [`read`](KeypadIndev::read) for a fully event-driven, poll-free path.
///
/// The producer may be an interrupt-driven async task; the consumer is the LVGL
/// task (the read callback). All fields are atomic — `send`/`press`/`release`
/// are safe to call from a different task than the one driving LVGL.
///
/// Declare it as a `static` so it satisfies [`KeypadIndev::new`]'s `'static`
/// requirement (LVGL stores a pointer to it for the device's lifetime).
#[derive(Debug)]
pub struct KeypadState {
    /// Currently-held LVGL key code (`lv_key_t`); `0` = no key held. No real
    /// `Key` constant is `0`, so it is an unambiguous "released" sentinel.
    held: AtomicU32,

    /// Single-producer / single-consumer ring of pending one-shot keys.
    /// `head`/`tail` are monotonic (wrapping) counters; `head == tail` is empty,
    /// `tail - head == CAP` is full. The producer owns `tail`, the consumer
    /// (read callback) owns `head`.
    queue: [AtomicU32; KEYPAD_QUEUE_CAP],
    head: AtomicUsize,
    tail: AtomicUsize,

    /// One-shot release phase: after a queued key is reported `PRESSED`, the
    /// next read reports it `RELEASED` (so the key is never held across reads,
    /// and LVGL never arms its own long-press/repeat).
    release_pending: AtomicBool,
    release_key: AtomicU32,
}

impl KeypadState {
    /// Create a new, empty state.
    ///
    /// `const` so it can initialise a `static`:
    /// `static KEYPAD: KeypadState = KeypadState::new();`
    pub const fn new() -> Self {
        Self {
            held: AtomicU32::new(0),
            queue: [const { AtomicU32::new(0) }; KEYPAD_QUEUE_CAP],
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
            release_pending: AtomicBool::new(false),
            release_key: AtomicU32::new(0),
        }
    }

    // ── Held model (raw momentary buttons) ──────────────────────────────────

    /// Report `key` as currently held (a press edge).
    ///
    /// Overwrites any previously-held key — a single-pointer keypad reports one
    /// key at a time. The next time LVGL reads the device, this key is delivered
    /// to the focused group, and LVGL derives long-press/repeat from the hold.
    pub fn press(&self, key: Key) {
        self.held.store(key.0, Ordering::Relaxed);
    }

    /// Report that no key is held (a release edge).
    pub fn release(&self) {
        self.held.store(0, Ordering::Relaxed);
    }

    // ── One-shot model (pre-decoded discrete events) ────────────────────────

    /// Post one discrete key event: exactly one focus step, with no LVGL-side
    /// repeat (the key is delivered as a single `PRESSED` → `RELEASED`).
    ///
    /// Lock-free and single-producer; best-effort — if the queue (8 outstanding)
    /// is full, the event is dropped rather than blocking (a dropped repeat tick
    /// is preferable to stalling an interrupt-driven producer).
    pub fn send(&self, key: Key) {
        // Producer side: owns `tail`, reads `head` to check for space.
        let head = self.head.load(Ordering::Acquire);
        let tail = self.tail.load(Ordering::Relaxed);
        if tail.wrapping_sub(head) >= KEYPAD_QUEUE_CAP {
            return; // full — drop
        }
        self.queue[tail % KEYPAD_QUEUE_CAP].store(key.0, Ordering::Relaxed);
        self.tail.store(tail.wrapping_add(1), Ordering::Release);
    }

    /// Whether any one-shot event (queued or mid release-phase) is still
    /// undelivered. The render loop uses this to drain the queue via
    /// [`KeypadIndev::read`].
    pub fn has_pending(&self) -> bool {
        self.head.load(Ordering::Acquire) != self.tail.load(Ordering::Acquire)
            || self.release_pending.load(Ordering::Acquire)
    }

    /// Consumer side (read callback): pop the next queued key, or `None`.
    fn dequeue(&self) -> Option<u32> {
        let tail = self.tail.load(Ordering::Acquire);
        let head = self.head.load(Ordering::Relaxed);
        if head == tail {
            return None;
        }
        let key = self.queue[head % KEYPAD_QUEUE_CAP].load(Ordering::Relaxed);
        self.head.store(head.wrapping_add(1), Ordering::Release);
        Some(key)
    }

    /// Consumer side: are there more queued keys after the current one?
    fn queue_nonempty(&self) -> bool {
        self.head.load(Ordering::Relaxed) != self.tail.load(Ordering::Acquire)
    }
}

impl Default for KeypadState {
    fn default() -> Self {
        Self::new()
    }
}

/// Owning KEYPAD input device, backed by a [`KeypadState`].
///
/// Created once at startup and kept alive for the application's lifetime
/// (commonly held by the render task or owned by the
/// [`Navigator`](crate::navigator::Navigator) via
/// [`run_app_nav`](crate::view::run_app_nav)). Dropping it removes the device
/// from LVGL via `lv_indev_delete`.
///
/// Bind it to a focus [`Group`] — either explicitly with
/// [`set_group`](Self::set_group), or automatically by the navigator, which
/// routes each active view's
/// [`input_group`](crate::view::View::input_group) to every registered keypad
/// device.
///
/// # Thread safety
///
/// `KeypadIndev` is `!Send + !Sync` — LVGL must be driven from a single task.
pub struct KeypadIndev {
    ptr: *mut lv_indev_t,
    /// The state this device reads from — kept so [`read`](Self::read) can drain
    /// the one-shot queue. `'static`, so it does not constrain the device.
    state: &'static KeypadState,
    _not_send: PhantomData<*const ()>,
}

impl core::fmt::Debug for KeypadIndev {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("KeypadIndev").finish_non_exhaustive()
    }
}

impl KeypadIndev {
    /// Create a KEYPAD device in **TIMER mode** (LVGL polls it on its own read
    /// timer, ~30 ms). Use with the held model
    /// ([`KeypadState::press`](KeypadState::press) /
    /// [`release`](KeypadState::release)) for raw momentary buttons.
    ///
    /// `state` must be `'static` because LVGL stores a pointer to it (in the
    /// device's user data) and reads it for the device's lifetime — see
    /// `spec-memory-lifetime.md` §1.
    ///
    /// Returns `Err(WidgetError::LvglNullPointer)` if LVGL allocation fails.
    pub fn new(state: &'static KeypadState) -> Result<Self, WidgetError> {
        Self::create(state, false)
    }

    /// Create a KEYPAD device in **EVENT mode** — LVGL does **not** poll it on a
    /// timer; nothing is read until you call [`read`](Self::read).
    ///
    /// Pair with [`KeypadState::send`] for a fully event-driven, poll-free path:
    /// an interrupt-driven producer calls `send` + signals the render loop, the
    /// loop calls `read`, and the key reaches the screen with no periodic
    /// polling of either the button or the device.
    ///
    /// Returns `Err(WidgetError::LvglNullPointer)` if LVGL allocation fails.
    pub fn new_event(state: &'static KeypadState) -> Result<Self, WidgetError> {
        Self::create(state, true)
    }

    /// Shared constructor. `event_mode` selects `LV_INDEV_MODE_EVENT`.
    fn create(state: &'static KeypadState, event_mode: bool) -> Result<Self, WidgetError> {
        // SAFETY: lv_indev_create allocates and registers a new indev in the
        // global indev list; returns NULL on OOM (checked below).
        // See lvgl/src/indev/lv_indev.c — lv_indev_create.
        let ptr = unsafe { lv_indev_create() };
        if ptr.is_null() {
            return Err(WidgetError::LvglNullPointer);
        }
        // SAFETY: ptr is non-null (checked). We mark it a KEYPAD device, point
        // its read_cb at `keypad_read_cb`, store `state` (a `&'static`
        // reference, so it outlives the device) as the user data the callback
        // reads, and optionally switch it to EVENT mode (no read timer).
        // lv_indev_set_* only store these into the indev struct.
        // See lvgl/src/indev/lv_indev.c — lv_indev_set_type/read_cb/user_data/mode.
        unsafe {
            lv_indev_set_type(ptr, lv_indev_type_t_LV_INDEV_TYPE_KEYPAD);
            lv_indev_set_read_cb(ptr, Some(keypad_read_cb));
            lv_indev_set_user_data(ptr, state as *const KeypadState as *mut c_void);
            if event_mode {
                lv_indev_set_mode(ptr, lv_indev_mode_t_LV_INDEV_MODE_EVENT);
            }
        }
        Ok(Self { ptr, state, _not_send: PhantomData })
    }

    /// Enable **hold-to-repeat**: while a key is held (the *held* model —
    /// [`KeypadState::press`] without a matching [`release`](KeypadState::release)),
    /// LVGL re-sends it to the focused group, first after `after`, then once
    /// every `every`. Use this for value/setpoint editing — hold a button to
    /// keep incrementing.
    ///
    /// A thin pass-through to LVGL's `long_press_time` /
    /// `long_press_repeat_time`. Has no effect on the *one-shot* model
    /// ([`KeypadState::send`]), which never holds a key across reads.
    ///
    /// Durations are clamped to `u16::MAX` milliseconds (LVGL's field width).
    /// Builder-style — chain it onto construction:
    ///
    /// ```no_run
    /// use core::time::Duration;
    /// use oxivgl::indev::{KeypadIndev, KeypadState};
    /// static KEYPAD: KeypadState = KeypadState::new();
    /// # fn demo() -> Result<(), oxivgl::widgets::WidgetError> {
    /// let keypad = KeypadIndev::new(&KEYPAD)?
    ///     .with_repeat(Duration::from_millis(400), Duration::from_millis(80));
    /// # let _ = keypad; Ok(()) }
    /// ```
    pub fn with_repeat(self, after: core::time::Duration, every: core::time::Duration) -> Self {
        let after = after.as_millis().min(u16::MAX as u128) as u16;
        let every = every.as_millis().min(u16::MAX as u128) as u16;
        // SAFETY: self.ptr is a live indev created in create(). These setters
        // only store the timing fields into the indev struct.
        // See lvgl/src/indev/lv_indev.c — lv_indev_set_long_press_time/repeat_time.
        unsafe {
            lv_indev_set_long_press_time(self.ptr, after);
            lv_indev_set_long_press_repeat_time(self.ptr, every);
        }
        self
    }

    /// Bind this device to `group` so its keys drive that group's focus.
    ///
    /// Equivalent to adding the device to the group's keyboard/encoder set.
    /// The navigator does this automatically for the active view's
    /// [`input_group`](crate::view::View::input_group); call this only for
    /// manual (non-navigator) setups.
    pub fn set_group(&self, group: &Group) -> &Self {
        // SAFETY: self.ptr is non-null (checked in create()); group.raw_ptr()
        // returns the group's non-null lv_group_t. lv_indev_set_group stores
        // the group pointer into the indev.
        // See lvgl/src/indev/lv_indev.c — lv_indev_set_group.
        unsafe { lv_indev_set_group(self.ptr, group.raw_ptr()) };
        self
    }

    /// Process pending input now, draining the one-shot queue
    /// ([`KeypadState::send`]). Call from your render loop when your input
    /// signal fires — essential in EVENT mode (where LVGL never reads on its
    /// own), harmless in TIMER mode.
    ///
    /// Each queued key is delivered as `PRESSED` then `RELEASED`. The loop
    /// drains a full burst even if the platform ignores `continue_reading`, and
    /// is bounded so it can never spin.
    pub fn read(&self) -> &Self {
        // At most one PRESSED + one RELEASED read per queued key, plus a margin.
        let mut budget = 2 * KEYPAD_QUEUE_CAP + 1;
        loop {
            // SAFETY: self.ptr is a live KEYPAD indev created in create().
            // lv_indev_read invokes our read_cb and processes one input state.
            unsafe { lv_indev_read(self.ptr) };
            budget -= 1;
            if !self.state.has_pending() || budget == 0 {
                break;
            }
        }
        self
    }
}

impl Drop for KeypadIndev {
    fn drop(&mut self) {
        // SAFETY: self.ptr was returned by lv_indev_create and is non-null.
        // lv_indev_delete unlinks the device from the global indev list and
        // any group binding, then frees it. Called exactly once via Drop.
        // See lvgl/src/indev/lv_indev.c — lv_indev_delete.
        unsafe { lv_indev_delete(self.ptr) };
    }
}

/// LVGL read callback for a [`KeypadIndev`].
///
/// Delivers, in priority order: (1) the `RELEASED` half of a one-shot key just
/// reported `PRESSED`; (2) the next queued one-shot key as `PRESSED` (arming its
/// release); (3) the held key ([`KeypadState::press`]/[`KeypadState::release`]). For queued
/// keys it sets `continue_reading` so a whole burst drains in one
/// `lv_indev_read`. Invoked by LVGL on its own task.
unsafe extern "C" fn keypad_read_cb(indev: *mut lv_indev_t, data: *mut lv_indev_data_t) {
    if indev.is_null() || data.is_null() {
        return;
    }
    // SAFETY: indev is non-null (checked). The user data was set in
    // KeypadIndev::new* to a `&'static KeypadState` pointer that outlives the
    // device; NULL only if unset (handled below).
    let state = unsafe { lv_indev_get_user_data(indev) } as *const KeypadState;
    if state.is_null() {
        return;
    }
    // SAFETY: state points to a live `'static` KeypadState (see above). All its
    // fields are atomics, so shared access from this C callback is sound.
    let st = unsafe { &*state };

    // 1. Finish a one-shot: report RELEASED for the key just pressed.
    if st.release_pending.swap(false, Ordering::AcqRel) {
        let k = st.release_key.load(Ordering::Relaxed);
        // SAFETY: data is a valid lv_indev_data_t LVGL gave us to populate.
        unsafe {
            (*data).key = k;
            (*data).state = lv_indev_state_t_LV_INDEV_STATE_RELEASED;
            // Drain the rest of the burst in this same lv_indev_read.
            (*data).continue_reading = st.queue_nonempty();
        }
        return;
    }

    // 2. Start the next queued one-shot key: report PRESSED, arm its release.
    if let Some(k) = st.dequeue() {
        st.release_key.store(k, Ordering::Relaxed);
        st.release_pending.store(true, Ordering::Release);
        // SAFETY: data is valid (as above).
        unsafe {
            (*data).key = k;
            (*data).state = lv_indev_state_t_LV_INDEV_STATE_PRESSED;
            (*data).continue_reading = true; // come back to release it
        }
        return;
    }

    // 3. Held model: report the currently-held key (LVGL derives repeat).
    let h = st.held.load(Ordering::Relaxed);
    // SAFETY: data is valid (as above).
    unsafe {
        (*data).key = h;
        (*data).state = if h != 0 {
            lv_indev_state_t_LV_INDEV_STATE_PRESSED
        } else {
            lv_indev_state_t_LV_INDEV_STATE_RELEASED
        };
        (*data).continue_reading = false;
    }
}

/// Lock-free touch state shared between an input producer and a
/// [`PointerIndev`] — the POINTER analogue of [`KeypadState`].
///
/// A producer (a touch-panel polling task, or an interrupt handler) writes raw
/// `(x, y)` coordinates with [`touch`](Self::touch) and lifts with
/// [`release`](Self::release); LVGL's read callback reads the latest state. All
/// fields are atomic, so the producer may run on a different task than the one
/// driving LVGL.
///
/// The input is plain coordinates — no driver, board, or MCU type — so it stays
/// BSP- and MCU-agnostic. The consumer's binary writes the few-line bridge from
/// its touch driver (e.g. `ft6336u::read_touch() -> Option<(u16, u16)>`) to
/// this cell.
///
/// Declare it as a `static` so it satisfies [`PointerIndev::new`]'s `'static`
/// requirement (LVGL stores a pointer to it for the device's lifetime).
///
/// ```no_run
/// use oxivgl::indev::{PointerIndev, PointerState};
///
/// static TOUCH: PointerState = PointerState::new();
///
/// # fn demo() -> Result<(), oxivgl::widgets::WidgetError> {
/// let _pointer = PointerIndev::new(&TOUCH)?;
///
/// // From a touch-panel task, on each sample:
/// TOUCH.touch(120, 48);   // finger down at (120, 48)
/// TOUCH.release();        // finger up
/// # Ok(()) }
/// ```
#[derive(Debug)]
pub struct PointerState {
    /// Last reported coordinates packed as `(x << 16) | y` (both `u16`), so the
    /// pair is read atomically — `x` and `y` can never tear across an update.
    /// Latched on release so LVGL sees the release at the point of the last
    /// touch (the conventional touchscreen behaviour).
    xy: AtomicU32,
    /// Whether the panel is currently being touched. Stored with `Release`
    /// *after* `xy` and loaded with `Acquire` *before* it, so a reader that
    /// observes a press also observes the coordinates that press was reported
    /// with — there is a happens-before from the coordinate store to the press.
    pressed: AtomicBool,
}

impl PointerState {
    /// Create a new, released state.
    ///
    /// `const` so it can initialise a `static`:
    /// `static TOUCH: PointerState = PointerState::new();`
    pub const fn new() -> Self {
        Self {
            xy: AtomicU32::new(0),
            pressed: AtomicBool::new(false),
        }
    }

    /// Report a touch (press) at `(x, y)`.
    pub fn touch(&self, x: u16, y: u16) {
        // Publish the coordinates first, then the press with Release so the
        // matching `sample()` Acquire-load of `pressed` sees these coords.
        self.xy.store(((x as u32) << 16) | y as u32, Ordering::Relaxed);
        self.pressed.store(true, Ordering::Release);
    }

    /// Report that the panel is no longer touched (release). The last
    /// coordinates are kept, so the release is reported at the touch point.
    pub fn release(&self) {
        self.pressed.store(false, Ordering::Release);
    }

    /// Consumer side (read callback): the current `(x, y, pressed)`.
    ///
    /// Loads `pressed` (Acquire) before the coordinates so a press is paired
    /// with the coordinates it was reported with (single producer).
    fn sample(&self) -> (i32, i32, bool) {
        let pressed = self.pressed.load(Ordering::Acquire);
        let xy = self.xy.load(Ordering::Relaxed);
        (((xy >> 16) & 0xffff) as i32, (xy & 0xffff) as i32, pressed)
    }
}

impl Default for PointerState {
    fn default() -> Self {
        Self::new()
    }
}

/// Owning POINTER (touchscreen) input device — the direct-touch analogue of
/// [`KeypadIndev`].
///
/// A view can be navigated by *tapping a widget at a coordinate*, not only via
/// focus keys. Created once at startup (e.g. in
/// [`View::create`](crate::view::View::create)) and kept alive for as long as
/// touch input is wanted; dropping it removes the device via `lv_indev_delete`.
///
/// LVGL polls the device on its own read timer (TIMER mode), driven by
/// `lv_timer_handler` in the render loop — no group binding and no run-loop
/// wiring is required, unlike the keypad's focus routing.
///
/// Fed in oxivgl's own vocabulary — raw `(x, y)` coordinates — via either a
/// [`PointerState`] cell ([`new`](Self::new)) or a polling closure
/// ([`new_with`](Self::new_with)). No BSP/MCU type is involved.
///
/// # Thread safety
///
/// `PointerIndev` is `!Send + !Sync` — LVGL must be driven from a single task.
pub struct PointerIndev {
    ptr: *mut lv_indev_t,
    /// Owned heap allocation backing a closure-fed device ([`new_with`]): the
    /// `outer` thin pointer to the boxed fat `dyn FnMut` pointer. `Drop`
    /// reclaims both boxes from this — the device's own record, not whatever is
    /// in the LVGL user-data slot at drop time. `None` for the
    /// [`PointerState`]-backed form.
    closure: Option<*mut *mut PointerReadFn>,
    _not_send: PhantomData<*const ()>,
}

/// Boxed polling closure stored as a [`PointerIndev`]'s user data.
type PointerReadFn = dyn FnMut() -> Option<(u16, u16)>;

impl core::fmt::Debug for PointerIndev {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PointerIndev").finish_non_exhaustive()
    }
}

impl PointerIndev {
    /// Create a POINTER device backed by a `'static` [`PointerState`].
    ///
    /// `state` must be `'static` because LVGL stores a pointer to it (in the
    /// device's user data) and reads it for the device's lifetime — see
    /// `spec-memory-lifetime.md` §1.
    ///
    /// Returns `Err(WidgetError::LvglNullPointer)` if LVGL allocation fails.
    pub fn new(state: &'static PointerState) -> Result<Self, WidgetError> {
        // SAFETY: lv_indev_create allocates and registers a new indev; returns
        // NULL on OOM (checked). See lvgl/src/indev/lv_indev.c.
        let ptr = unsafe { lv_indev_create() };
        if ptr.is_null() {
            return Err(WidgetError::LvglNullPointer);
        }
        // SAFETY: ptr non-null (checked). Mark it a POINTER device, point its
        // read_cb at `pointer_state_read_cb`, and store `state` (a `&'static`
        // reference that outlives the device) as the user data the callback
        // reads. lv_indev_set_* only store these into the indev struct.
        unsafe {
            lv_indev_set_type(ptr, lv_indev_type_t_LV_INDEV_TYPE_POINTER);
            lv_indev_set_read_cb(ptr, Some(pointer_state_read_cb));
            lv_indev_set_user_data(ptr, state as *const PointerState as *mut c_void);
        }
        Ok(Self { ptr, closure: None, _not_send: PhantomData })
    }

    /// Create a POINTER device fed by a polling closure.
    ///
    /// `read` is called by LVGL on each read: `Some((x, y))` reports a touch at
    /// that coordinate, `None` reports release. This is the ergonomic form for a
    /// driver that already exposes a poll function, e.g.
    /// `PointerIndev::new_with(|| ft6336u::read_touch())`.
    ///
    /// The closure is heap-allocated and owned by the device; it is reclaimed
    /// when the device is dropped.
    ///
    /// Returns `Err(WidgetError::LvglNullPointer)` if LVGL allocation fails.
    pub fn new_with(read: impl FnMut() -> Option<(u16, u16)> + 'static) -> Result<Self, WidgetError> {
        let boxed: Box<PointerReadFn> = Box::new(read);
        // Box<dyn> is a fat pointer; double-box to get a thin pointer for the
        // single user-data slot. The inner raw pointer is reclaimed in Drop.
        let raw: *mut PointerReadFn = Box::into_raw(boxed);
        let outer = Box::into_raw(Box::new(raw));
        // SAFETY: lv_indev_create allocates a new indev; NULL on OOM (checked).
        let ptr = unsafe { lv_indev_create() };
        if ptr.is_null() {
            // Reclaim both boxes before bailing out.
            // SAFETY: `outer` and `raw` were just produced by Box::into_raw and
            // not yet handed to LVGL, so they are still uniquely owned here.
            unsafe {
                let _ = Box::from_raw(*Box::from_raw(outer));
            }
            return Err(WidgetError::LvglNullPointer);
        }
        // SAFETY: ptr non-null (checked). Store the thin pointer-to-fat-pointer
        // as user data; the device owns it until Drop reclaims it.
        unsafe {
            lv_indev_set_type(ptr, lv_indev_type_t_LV_INDEV_TYPE_POINTER);
            lv_indev_set_read_cb(ptr, Some(pointer_closure_read_cb));
            lv_indev_set_user_data(ptr, outer as *mut c_void);
        }
        Ok(Self { ptr, closure: Some(outer), _not_send: PhantomData })
    }
}

impl Drop for PointerIndev {
    fn drop(&mut self) {
        // SAFETY: self.ptr was returned by lv_indev_create and is non-null.
        // lv_indev_delete unlinks and frees the device. Called once via Drop.
        unsafe { lv_indev_delete(self.ptr) };
        // Reclaim the closure boxes from our own stored pointer (not a re-read
        // of LVGL's user-data slot), if this was a closure-fed device.
        if let Some(outer) = self.closure {
            // SAFETY: `outer` is the pointer produced by Box::into_raw in
            // new_with and never freed; the device is gone, so we now hold
            // unique ownership of both the outer box (a `Box<*mut PointerReadFn>`)
            // and the inner boxed closure it points to.
            unsafe {
                let inner: Box<*mut PointerReadFn> = Box::from_raw(outer);
                let _ = Box::from_raw(*inner);
            }
        }
    }
}

/// LVGL read callback for a [`PointerState`]-backed [`PointerIndev`].
unsafe extern "C" fn pointer_state_read_cb(indev: *mut lv_indev_t, data: *mut lv_indev_data_t) {
    if indev.is_null() || data.is_null() {
        return;
    }
    // SAFETY: indev non-null (checked); user data is a `&'static PointerState`
    // set in PointerIndev::new (NULL only if unset, handled below).
    let state = unsafe { lv_indev_get_user_data(indev) } as *const PointerState;
    if state.is_null() {
        return;
    }
    // SAFETY: state points to a live `'static` PointerState; all fields atomic.
    let (x, y, pressed) = unsafe { &*state }.sample();
    // SAFETY: data is a valid lv_indev_data_t LVGL gave us to populate.
    unsafe {
        (*data).point.x = x;
        (*data).point.y = y;
        (*data).state = if pressed {
            lv_indev_state_t_LV_INDEV_STATE_PRESSED
        } else {
            lv_indev_state_t_LV_INDEV_STATE_RELEASED
        };
        (*data).continue_reading = false;
    }
}

/// LVGL read callback for a closure-fed [`PointerIndev`] (see [`new_with`]).
///
/// [`new_with`]: PointerIndev::new_with
unsafe extern "C" fn pointer_closure_read_cb(indev: *mut lv_indev_t, data: *mut lv_indev_data_t) {
    if indev.is_null() || data.is_null() {
        return;
    }
    // SAFETY: indev non-null (checked); user data is the thin pointer-to-fat-
    // pointer set in new_with (NULL only if unset, handled below).
    let outer = unsafe { lv_indev_get_user_data(indev) } as *mut *mut PointerReadFn;
    if outer.is_null() {
        return;
    }
    // SAFETY: `outer` points to a valid `*mut PointerReadFn` owned by the
    // device; the read callback runs on the LVGL task with exclusive access, so
    // taking `&mut` to the closure for the duration of the call is sound.
    let read: &mut PointerReadFn = unsafe { &mut **outer };
    let touched = read();
    // SAFETY: data is a valid lv_indev_data_t LVGL gave us to populate.
    unsafe {
        match touched {
            Some((x, y)) => {
                (*data).point.x = x as i32;
                (*data).point.y = y as i32;
                (*data).state = lv_indev_state_t_LV_INDEV_STATE_PRESSED;
            }
            None => {
                // Leave point unchanged: report release at the last touch point.
                (*data).state = lv_indev_state_t_LV_INDEV_STATE_RELEASED;
            }
        }
        (*data).continue_reading = false;
    }
}

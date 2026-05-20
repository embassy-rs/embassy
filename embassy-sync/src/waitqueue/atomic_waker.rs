use core::cell::Cell;
#[cfg(target_has_atomic = "32")]
use core::cell::UnsafeCell;
#[cfg(target_has_atomic = "32")]
use core::sync::atomic::AtomicUsize;
#[cfg(target_has_atomic = "32")]
use core::sync::atomic::Ordering::{AcqRel, Acquire, Release};
use core::task::Waker;

use crate::blocking_mutex::Mutex;
use crate::blocking_mutex::raw::RawMutex;

/// Utility struct to register and wake a waker.
/// If a waker is registered, registering another waker will replace the previous one without waking it.
/// Intended to wake a task from an interrupt. Therefore, it is generally not expected,
/// that multiple tasks register try to register a waker simultaneously.
///
/// Most users should prefer [`AtomicWaker`], which uses a small atomic state machine and
/// does **not** enter a critical section on the wake path. See also
/// [`CriticalSectionWaker`](super::CriticalSectionWaker), a non-generic convenience wrapper
/// that hard-codes [`CriticalSectionRawMutex`](crate::blocking_mutex::raw::CriticalSectionRawMutex).
pub struct GenericAtomicWaker<M: RawMutex> {
    waker: Mutex<M, Cell<Option<Waker>>>,
}

impl<M: RawMutex> GenericAtomicWaker<M> {
    /// Create a new `GenericAtomicWaker`.
    pub const fn new(mutex: M) -> Self {
        Self {
            waker: Mutex::const_new(mutex, Cell::new(None)),
        }
    }

    /// Register a waker. Overwrites the previous waker, if any.
    pub fn register(&self, w: &Waker) {
        self.waker.lock(|cell| {
            cell.set(match cell.replace(None) {
                Some(w2) if (w2.will_wake(w)) => Some(w2),
                _ => Some(w.clone()),
            })
        })
    }

    /// Wake the registered waker, if any.
    pub fn wake(&self) {
        self.waker.lock(|cell| {
            if let Some(w) = cell.replace(None) {
                w.wake_by_ref();
                cell.set(Some(w));
            }
        })
    }
}

// Lockless single-slot waker, ported from `futures::task::AtomicWaker` and
// adapted for `no_std`. Coordinates access to a single waker slot via a small
// atomic state machine, so neither `register()` nor `wake()` enters a critical
// section.
//
// Reference: <https://docs.rs/futures/latest/futures/task/struct.AtomicWaker.html>

// The state machine is encoded as a pair of independent bitflags packed into a
// single `AtomicUsize`. They are *not* mutually exclusive: `wake()` uses
// `fetch_or` to set the `WAKING` bit unconditionally, so a `wake()` that races
// an in-flight `register()` legitimately produces the combined state
// `REGISTERING | WAKING` (0b11). All four encodings are valid:
//
// - `WAITING`              (0b00): no operation in flight, waker cell is idle.
// - `REGISTERING`          (0b01): a `register()` call owns the waker cell.
// - `WAKING`               (0b10): a `wake()` call owns the waker cell.
// - `REGISTERING | WAKING` (0b11): a `register()` owns the cell and a
//   concurrent `wake()` has flagged that a wake is pending; the in-flight
//   `register()` is responsible for honoring it before releasing the cell
//   (see `register()` and `wake()` below).
//
// Only the holder of `REGISTERING` *or* `WAKING` (but never both at the same
// time, by virtue of the protocol) may dereference the waker cell.

/// No flags set: waker cell is idle and may be claimed by `register()` or
/// `wake()`.
#[cfg(target_has_atomic = "32")]
const WAITING: usize = 0;
/// Bit set while a `register()` call owns the waker cell.
#[cfg(target_has_atomic = "32")]
const REGISTERING: usize = 0b01;
/// Bit set while a `wake()` call owns the waker cell, or — when combined with
/// `REGISTERING` — to signal an in-flight `register()` that a wake is pending
/// and must be driven before the cell is released.
#[cfg(target_has_atomic = "32")]
const WAKING: usize = 0b10;

/// Utility struct to register and wake a waker.
///
/// Single-slot waker coordinated by an atomic state machine. Unlike
/// [`CriticalSectionWaker`](super::CriticalSectionWaker), `wake()` does **not**
/// enter a critical section, which avoids unnecessary jitter on higher-priority
/// interrupts and improves wake latency for time-critical async drivers.
///
/// If a waker is registered, registering another waker replaces the previous
/// one without waking it. Intended to wake a single task from an interrupt
/// handler.
///
/// On targets without 32-bit atomic CAS support (e.g. Xtensa S2, AVR, RV32I),
/// `AtomicWaker` is a type alias for
/// [`CriticalSectionWaker`](super::CriticalSectionWaker), since the lockless
/// state machine relies on `compare_exchange`/`swap`/`fetch_or`.
#[cfg(target_has_atomic = "32")]
pub struct AtomicWaker {
    state: AtomicUsize,
    waker: UnsafeCell<Option<Waker>>,
}

// SAFETY: All access to the `waker` cell is serialized through the `state`
// machine: only the holder of the REGISTERING or WAKING flag may dereference
// the cell.
#[cfg(target_has_atomic = "32")]
unsafe impl Send for AtomicWaker {}
#[cfg(target_has_atomic = "32")]
unsafe impl Sync for AtomicWaker {}

#[cfg(target_has_atomic = "32")]
impl AtomicWaker {
    /// Create a new `AtomicWaker`.
    pub const fn new() -> Self {
        Self {
            state: AtomicUsize::new(WAITING),
            waker: UnsafeCell::new(None),
        }
    }

    /// Register a waker. Overwrites the previous waker, if any.
    ///
    /// If a [`wake`](Self::wake) call is in flight when this is called, the
    /// supplied waker is woken directly to schedule a re-poll.
    pub fn register(&self, waker: &Waker) {
        match self
            .state
            .compare_exchange(WAITING, REGISTERING, Acquire, Acquire)
            .unwrap_or_else(|x| x)
        {
            WAITING => {
                // SAFETY: We hold the REGISTERING flag; no other call can
                // touch the waker cell while we own it.
                unsafe {
                    match &*self.waker.get() {
                        Some(old) if old.will_wake(waker) => {}
                        _ => *self.waker.get() = Some(waker.clone()),
                    }

                    // Try to release the cell by clearing REGISTERING. This
                    // CAS only succeeds if no `wake()` raced us; otherwise the
                    // observed state is REGISTERING | WAKING (0b11) and the
                    // CAS fails — that failure is precisely the handoff
                    // signal from `wake()`.
                    let res = self.state.compare_exchange(REGISTERING, WAITING, AcqRel, Acquire);
                    match res {
                        Ok(_) => {}
                        Err(_) => {
                            // A concurrent `wake()` observed our REGISTERING
                            // bit and intentionally left the WAKING bit set
                            // instead of touching the waker cell (which we
                            // own). The wake duty has been transferred to us:
                            // take the waker out, clear both bits in a single
                            // `swap` so a new `register()` / `wake()` cycle
                            // can begin, and finally drive the wake.
                            //
                            // This is what makes the "missed wake" race
                            // impossible: the wake is delivered exactly once,
                            // either by `wake()` itself (no race) or by
                            // `register()` here (race), but never dropped.
                            let waker = (*self.waker.get()).take().unwrap();
                            self.state.swap(WAITING, AcqRel);
                            waker.wake();
                        }
                    }
                }
            }
            WAKING => {
                // A wake is currently in flight. Schedule the supplied waker
                // directly so the task does not miss the wakeup.
                waker.wake_by_ref();
            }
            state => {
                // A concurrent `register` is in flight. The single-waiter
                // contract is being violated; drop our call (memory-safe).
                debug_assert!(state == REGISTERING || state == REGISTERING | WAKING);
            }
        }
    }

    /// Wake the registered waker, if any.
    ///
    /// The stored waker is invoked via [`Waker::wake_by_ref`] and remains
    /// registered, matching the semantics of
    /// [`CriticalSectionWaker`](super::CriticalSectionWaker).
    pub fn wake(&self) {
        // `fetch_or` (rather than `compare_exchange`) is load-bearing here.
        // It unconditionally publishes the WAKING bit, even if a `register()`
        // is mid-flight (state == REGISTERING). This is what enables the
        // handoff: the in-flight `register()` will observe the combined
        // REGISTERING | WAKING state on its release-CAS, take ownership of
        // the wake, and drive it.
        //
        // A `compare_exchange(WAITING, WAKING, ...)` here would silently drop
        // the wake whenever it lost the race against `register()`, leaving
        // the task hung — the waker cell holds a stale waker that nothing is
        // about to invoke.
        match self.state.fetch_or(WAKING, AcqRel) {
            WAITING => {
                // No `register()` was in flight, so we now solely own the
                // waker cell via the WAKING bit.
                //
                // SAFETY: We hold the WAKING bit; the protocol guarantees
                // no other call can touch the waker cell while we own it.
                unsafe {
                    if let Some(w) = &*self.waker.get() {
                        w.wake_by_ref();
                    }
                }
                self.state.swap(WAITING, Release);
            }
            _ => {
                // Previous state was REGISTERING (now REGISTERING | WAKING due
                // to `fetch_or` above), WAKING (another wake is in flight), or
                // REGISTERING | WAKING (both). In every case, *somebody else*
                // owns the waker cell and is responsible for delivering the
                // wake:
                //
                // - REGISTERING: the in-flight `register()` will see WAKING
                //   on its release-CAS and wake the registered waker itself.
                // - WAKING / REGISTERING | WAKING: another `wake()` /
                //   `register()` is already going to deliver the wake.
                //
                // So we deliberately do nothing here — including not touching
                // the waker cell, which we do not own.
            }
        }
    }
}

#[cfg(target_has_atomic = "32")]
impl Default for AtomicWaker {
    fn default() -> Self {
        Self::new()
    }
}

/// On targets without 32-bit atomic CAS, fall back to the critical-section
/// based waker. The lockless state machine requires
/// `AtomicUsize::{compare_exchange, swap, fetch_or}`, which are unavailable on
/// such targets (e.g. Xtensa S2, AVR, RV32I).
#[cfg(not(target_has_atomic = "32"))]
pub type AtomicWaker = super::CriticalSectionWaker;

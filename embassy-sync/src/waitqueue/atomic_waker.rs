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
/// If a waker is registered, registering a different waker replaces the
/// previous one and wakes it, giving the displaced task a chance to
/// re-register itself. This matches the semantics of
/// [`WakerRegistration`](super::WakerRegistration); note that
/// [`GenericAtomicWaker`] and
/// [`CriticalSectionWaker`](super::CriticalSectionWaker) instead replace the
/// previous waker *without* waking it.
///
/// Intended to wake a single task from an interrupt handler.
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

    /// Register a waker.
    ///
    /// If a different waker was previously registered, it is replaced and
    /// woken so the displaced task can re-register itself if it is still
    /// interested. This matches
    /// [`WakerRegistration::register`](super::WakerRegistration::register).
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
                // The waker evicted from the cell (if any), to be woken
                // *after* we release REGISTERING so the wake cannot re-enter
                // while we still own the cell.
                let evicted: Option<Waker>;
                // The newly-registered waker, taken back out in the handoff
                // path because a racing `wake()` transferred wake duty to us.
                let pending_wake: Option<Waker>;

                // SAFETY: We hold the REGISTERING flag; no other call can
                // touch the waker cell while we own it.
                unsafe {
                    evicted = match &*self.waker.get() {
                        Some(old) if old.will_wake(waker) => None,
                        _ => (*self.waker.get()).replace(waker.clone()),
                    };

                    // Try to release the cell by clearing REGISTERING. This
                    // CAS only succeeds if no `wake()` raced us; otherwise the
                    // observed state is REGISTERING | WAKING (0b11) and the
                    // CAS fails — that failure is precisely the handoff
                    // signal from `wake()`.
                    let res = self.state.compare_exchange(REGISTERING, WAITING, AcqRel, Acquire);
                    pending_wake = match res {
                        Ok(_) => None,
                        Err(_) => {
                            // A concurrent `wake()` observed our REGISTERING
                            // bit and intentionally left the WAKING bit set
                            // instead of touching the waker cell (which we
                            // own). The wake duty has been transferred to us:
                            // take the waker out and clear both bits in a
                            // single `swap` so a new `register()` / `wake()`
                            // cycle can begin. We drive the wake outside the
                            // unsafe block, below.
                            //
                            // This is what makes the "missed wake" race
                            // impossible: the wake is delivered exactly once,
                            // either by `wake()` itself (no race) or by
                            // `register()` here (race), but never dropped.
                            let w = (*self.waker.get()).take().unwrap();
                            self.state.swap(WAITING, AcqRel);
                            Some(w)
                        }
                    };
                }

                // Cell ownership has been released. It is now safe to invoke
                // user wakers, which may re-enter `register()` / `wake()` on
                // this same `AtomicWaker`.
                if let Some(w) = pending_wake {
                    w.wake();
                }
                if let Some(w) = evicted {
                    w.wake();
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

#[cfg(all(test, target_has_atomic = "32"))]
mod tests {
    extern crate std;

    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::task::{Wake, Waker};

    use super::AtomicWaker;

    /// Test waker that counts how many times it was woken (via either
    /// `wake` or `wake_by_ref`).
    struct CountingWaker {
        count: AtomicUsize,
    }

    impl CountingWaker {
        fn new() -> Arc<Self> {
            Arc::new(Self {
                count: AtomicUsize::new(0),
            })
        }

        fn count(&self) -> usize {
            self.count.load(Ordering::SeqCst)
        }

        fn waker(self: &Arc<Self>) -> Waker {
            Waker::from(self.clone())
        }
    }

    impl Wake for CountingWaker {
        fn wake(self: Arc<Self>) {
            self.count.fetch_add(1, Ordering::SeqCst);
        }
        fn wake_by_ref(self: &Arc<Self>) {
            self.count.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[test]
    fn register_with_distinct_waker_wakes_evicted() {
        let aw = AtomicWaker::new();
        let a = CountingWaker::new();
        let b = CountingWaker::new();

        aw.register(&a.waker());
        assert_eq!(a.count(), 0);
        assert_eq!(b.count(), 0);

        aw.register(&b.waker());

        // The evicted (a) is woken exactly once; the newly-registered (b) is
        // not woken by `register` alone.
        assert_eq!(a.count(), 1, "evicted waker should be woken on eviction");
        assert_eq!(b.count(), 0, "newly-registered waker must not be woken by register");
    }

    #[test]
    fn register_same_waker_no_spurious_wakes() {
        let aw = AtomicWaker::new();
        let a = CountingWaker::new();
        let w = a.waker();

        aw.register(&w);
        aw.register(&w);

        // `Waker::will_wake` is best-effort: it may return `false` even
        // when both wakers wake the same task (the result depends on
        // toolchain-specific const-promotion of the internal vtable).
        // Either branch is correct:
        //   - fast path taken      → 0 wakes
        //   - eviction + re-register → 1 wake (the evicted clone)
        // What we guard against is *spurious extra* wakes.
        assert!(a.count() <= 1);
    }

    #[test]
    fn wake_after_register_wakes_registered() {
        let aw = AtomicWaker::new();
        let a = CountingWaker::new();

        aw.register(&a.waker());
        aw.wake();

        assert_eq!(a.count(), 1);
    }

    #[test]
    fn wake_with_no_registered_waker_is_noop() {
        let aw = AtomicWaker::new();
        aw.wake();
        // Just checking we don't panic / UB.
    }
}

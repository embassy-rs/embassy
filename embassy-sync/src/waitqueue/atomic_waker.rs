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

/// Idle.
#[cfg(target_has_atomic = "32")]
const WAITING: usize = 0;
/// A `register` call is in flight.
#[cfg(target_has_atomic = "32")]
const REGISTERING: usize = 0b01;
/// A `wake` call is in flight (or pending while a register is in flight).
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

                    let res = self.state.compare_exchange(REGISTERING, WAITING, AcqRel, Acquire);
                    match res {
                        Ok(_) => {}
                        Err(_) => {
                            // A concurrent `wake()` observed our REGISTERING
                            // flag and left the WAKING bit set for us to
                            // honor. Take the waker out and wake it ourselves.
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
        match self.state.fetch_or(WAKING, AcqRel) {
            WAITING => {
                // SAFETY: We hold the WAKING flag; no other call can touch
                // the waker cell while we own it.
                unsafe {
                    if let Some(w) = &*self.waker.get() {
                        w.wake_by_ref();
                    }
                }
                self.state.swap(WAITING, Release);
            }
            _ => {
                // Either a register is in progress (it will see the WAKING
                // bit on its CAS-back and wake the supplied waker itself),
                // or another wake is in flight.
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

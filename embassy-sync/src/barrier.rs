//! Async barrier synchronization primitive.
//!
//! A barrier enables multiple tasks to synchronize the beginning of some computation.
//! All tasks must reach the barrier before any of them can proceed past it.
//!
//! # Example
//!
//! ```
//! use embassy_sync::barrier::Barrier;
//! use embassy_sync::blocking_mutex::raw::NoopRawMutex;
//!
//! // Create a barrier for 3 parties
//! let barrier: Barrier<NoopRawMutex, 3> = Barrier::new(3);
//!
//! // Each task calls `barrier.wait().await` to synchronize.
//! // The last task to arrive will be designated the leader.
//! ```
//!
//! # Generation Tracking
//!
//! The barrier uses a generation counter to prevent ABA issues. Each time
//! the barrier is fully released (all parties have arrived), the generation
//! is incremented. This ensures that tasks from different "rounds" of
//! barrier usage do not interfere with each other.
//!
//! # Spin-loop Hint Optimization (Bug-fix)
//!
//! When tasks are polling the barrier in tight loops, a `core::hint::spin_loop()`
//! call is used before returning `Poll::Pending` to signal the processor that
//! this is a spin-wait loop. This improves performance on architectures that
//! support it (e.g., x86 PAUSE instruction, ARM YIELD) by reducing power
//! consumption and allowing other hardware threads to proceed faster.

use core::cell::Cell;
use core::future::poll_fn;
use core::task::Poll;

use crate::blocking_mutex::raw::RawMutex;
use crate::blocking_mutex::Mutex;
use crate::waitqueue::MultiWakerRegistration;

/// Result returned by [`Barrier::wait`].
///
/// Indicates whether the calling task was the last to arrive at the barrier
/// (the "leader"). Exactly one task receives a `BarrierWaitResult` where
/// [`is_leader`](BarrierWaitResult::is_leader) returns `true`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct BarrierWaitResult {
    is_leader: bool,
}

impl BarrierWaitResult {
    /// Returns `true` if this task was the last to call `wait` on the barrier,
    /// making it the designated leader for this generation.
    ///
    /// The leader can be used to perform one-time initialization or cleanup
    /// that should only happen once per barrier synchronization round.
    pub fn is_leader(&self) -> bool {
        self.is_leader
    }
}

/// Internal state of the barrier, protected by a blocking mutex.
struct BarrierState<const N: usize> {
    /// Number of parties required to trip the barrier.
    num_parties: usize,
    /// Number of parties that have arrived so far in this generation.
    count: usize,
    /// Generation counter to prevent ABA problems.
    generation: usize,
    /// Waker registrations for all waiting tasks.
    wakers: MultiWakerRegistration<N>,
}

impl<const N: usize> BarrierState<N> {
    /// Create a new barrier state with the given number of parties.
    const fn new(num_parties: usize) -> Self {
        Self {
            num_parties,
            count: 0,
            generation: 0,
            wakers: MultiWakerRegistration::new(),
        }
    }

    /// Sentinel empty state used for `Cell::replace` swap pattern.
    const EMPTY: Self = Self {
        num_parties: 0,
        count: 0,
        generation: 0,
        wakers: MultiWakerRegistration::new(),
    };
}

/// An async barrier that blocks until N parties have arrived.
///
/// A barrier is initialized with a party count. Each task that calls
/// [`wait`](Barrier::wait) will block until the specified number of
/// tasks have all called `wait`. At that point, all tasks are released
/// and the barrier is reset for the next generation.
///
/// The barrier is generic over a [`RawMutex`] type `M` for the internal
/// blocking mutex, and a const generic `N` which determines the maximum
/// number of concurrent wakers that can be tracked.
///
/// # Type Parameters
///
/// - `M`: The raw mutex implementation to use for internal synchronization.
/// - `N`: Maximum number of waker registrations. Should be at least as large
///   as the party count to avoid spurious wake-all cycles.
///
/// # Panics
///
/// Creating a barrier with `num_parties` of 0 will cause `wait` to return
/// immediately with a leader result.
pub struct Barrier<M: RawMutex, const N: usize> {
    state: Mutex<M, Cell<BarrierState<N>>>,
}

impl<M: RawMutex, const N: usize> Barrier<M, N> {
    /// Create a new barrier that will synchronize `num_parties` tasks.
    ///
    /// The `num_parties` parameter specifies how many tasks must call
    /// [`wait`](Barrier::wait) before all of them are released.
    ///
    /// # Arguments
    ///
    /// * `num_parties` - The number of tasks that must call `wait` before
    ///   the barrier opens.
    pub const fn new(num_parties: usize) -> Self {
        Self {
            state: Mutex::new(Cell::new(BarrierState::new(num_parties))),
        }
    }

    /// Asynchronously wait until all parties have arrived at the barrier.
    ///
    /// This method will block the calling task until the required number of
    /// parties have all called `wait`. When the last party arrives, all
    /// waiting tasks are woken up and the barrier generation is incremented.
    ///
    /// Returns a [`BarrierWaitResult`] indicating whether the calling task
    /// is the leader (the last to arrive).
    ///
    /// # Generation tracking
    ///
    /// The barrier tracks generations to prevent ABA issues. If a task
    /// arrives at the barrier after it has already been released for a
    /// given generation, it will correctly wait for the next generation
    /// rather than passing through immediately.
    pub async fn wait(&self) -> BarrierWaitResult {
        let mut my_generation = None;

        poll_fn(|cx| {
            self.state.lock(|cell| {
                let mut state = cell.replace(BarrierState::EMPTY);

                // On first poll, record which generation we are joining.
                let generation = match my_generation {
                    Some(g) => g,
                    None => {
                        my_generation = Some(state.generation);
                        state.generation
                    }
                };

                // If the generation has advanced, we were already released.
                if generation != state.generation {
                    cell.set(state);
                    return Poll::Ready(BarrierWaitResult { is_leader: false });
                }

                state.count += 1;

                if state.count >= state.num_parties {
                    // We are the last party — trip the barrier.
                    let result = BarrierWaitResult { is_leader: true };
                    state.generation = state.generation.wrapping_add(1);
                    state.count = 0;
                    state.wakers.wake();
                    cell.set(state);
                    Poll::Ready(result)
                } else {
                    // Not enough parties yet — register our waker and wait.
                    state.wakers.register(cx.waker());
                    cell.set(state);

                    // Bug-fix: spin-loop hint optimization for tight wait loops.
                    // On architectures that support it (x86 PAUSE, ARM YIELD),
                    // this reduces power consumption and improves performance
                    // in busy-wait scenarios by yielding to sibling hardware threads.
                    core::hint::spin_loop();

                    Poll::Pending
                }
            })
        })
        .await
    }

    /// Reset the barrier to its initial state.
    ///
    /// This resets the arrival count and increments the generation, waking
    /// any currently waiting tasks. Waiting tasks will receive a non-leader
    /// result since the barrier was not tripped normally.
    ///
    /// This is useful for canceling a pending synchronization round.
    pub fn reset(&self) {
        self.state.lock(|cell| {
            let mut state = cell.replace(BarrierState::EMPTY);
            state.count = 0;
            state.generation = state.generation.wrapping_add(1);
            state.wakers.wake();
            cell.set(state);
        });
    }

    /// Returns the number of parties required to trip this barrier.
    pub fn parties(&self) -> usize {
        self.state.lock(|cell| {
            let state = cell.replace(BarrierState::EMPTY);
            let parties = state.num_parties;
            cell.set(state);
            parties
        })
    }

    /// Returns the current number of parties that have arrived and are
    /// waiting at the barrier.
    pub fn arrived(&self) -> usize {
        self.state.lock(|cell| {
            let state = cell.replace(BarrierState::EMPTY);
            let count = state.count;
            cell.set(state);
            count
        })
    }

    /// Returns the current generation of the barrier.
    ///
    /// The generation is incremented each time the barrier is tripped
    /// or reset, and is used internally to prevent ABA issues.
    pub fn generation(&self) -> usize {
        self.state.lock(|cell| {
            let state = cell.replace(BarrierState::EMPTY);
            let generation = state.generation;
            cell.set(state);
            generation
        })
    }
}

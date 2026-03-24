//! Async interface for dispatching `FnOnce(&mut T) -> R` jobs to a dedicated
//! runner task with exclusive access to `T`.
//!
//! Callers submit an `FnOnce(&mut T) -> R` via [`ContextService::call`].
//! A dedicated runner, started with [`ContextService::run`], executes
//! closures one at a time with exclusive `&mut T` access and sends
//! results back. Closures and return values are stored inline in a
//! fixed-size slot of `S` bytes, checked at compile time.

use core::cell::{Cell, UnsafeCell};
use core::future::Future;
use core::marker::PhantomData;
use core::mem::{self, MaybeUninit};
use core::pin::Pin;
use core::sync::atomic::{AtomicBool, Ordering};
use core::task::{Context, Poll};

use crate::blocking_mutex::Mutex;
use crate::blocking_mutex::raw::RawMutex;
use crate::signal::Signal;
use crate::waitqueue::WakerRegistration;

type RunFn<T, const S: usize> = unsafe fn(&Storage<S>, &mut T);

// It is sometimes useful to dispatch blocking work from async tasks onto a
// dedicated runner — for example to serialize access to a shared resource,
// or to run blocking operations in a lower-priority task without stalling
// the caller's executor.
//
// We would like to enable async callers to run FnOnce(&mut T) -> R on a shared
// T, where T is owned by a single runner task. The interface should remain simple,
// with a call() function to submit a closure and a run() function to drive execution.
//
// ## Design requirements
// Ideally, each call should be able to have different F and R types. Since the
// runner task will be unaware of the exact F and R, we will need to erase the type
// of the closure and its return value.
//
// Like the existing primitives in embassy-sync, both call() and run()
// should be cancel-safe: dropping either future at any await point must
// leave the service in a consistent state and ready for further use.
//
// Particularly, when a call future is dropped, either:
//   - the closure was not yet submitted and is simply dropped, or
//   - the closure was already submitted and will be executed by the
//     runner to completion (the result is discarded).
// In both cases, the service should remain usable and apply backpressure
// correctly s.t new callers are simply blocked until the slot is free.
//
// Since run() can also be cancelled, it would be reasonable for it to also
// be restartable, that is, a new run() call must be able to pick up where the
// previous one left off, recovering any stale work before accepting new work.
//
// ## Implementation
// Fundamentally, we need some shared memory between the caller and the runner for the
// closure (and its captured environment) and the return value.
//
// Stack-pinned memory on the caller side is not an option since after
// submission, the runner may still be reading F or writing R when the
// caller is dropped. We cannot block in Drop to wait for it to finish,
// and also cannot interrupt the runner mid-execution either. So instead
// of living on the stack, our closure and result will live in a shared
// fixed-size byte buffer ("slot"), owned by the ContextService with access
// coordinated by a handshake protocol.
//
// The slot (`Storage<S>`) is a statically sized S-byte buffer. The caller writes
// its closure F into the slot via a pointer cast, then sends an erased function
// pointer to run_job::<T, R, F, S> through the job signal. The runner calls
// this function pointer, which takes F out of the slot, executes it, and
// writes R back. The slot also stores a type-erased drop function so that
// whoever cleans up (runner after ack, or Storage::drop) can drop the
// contents without knowing the concrete type.
//
// Coordination uses a SlotState (behind a blocking mutex) and three signals:
//
//   - state (`Mutex<Cell<SlotState>>`): a boolean `free` flag and a
//     WakerRegistration. When a caller tries to acquire the slot and it
//     is not free, the caller registers its waker here and returns
//     Pending. When the runner finishes a job and marks the slot free,
//     it wakes the registered waker (This follows the same pattern that
//     Channel uses for backpressure (senders_waker / receiver_waker)).
//   - job (`Signal<RunFn>`): caller -> runner. Carries the type-erased
//     function pointer and wakes the runner to start executing.
//   - done (`Signal<()>`): runner -> caller. Tells the caller that R is
//     ready in the slot.
//   - ack (`Signal<()>`): caller -> runner. Tells the runner the caller is
//     done reading R and the slot can be cleaned up.
//
// The protocol:
//```text
//   caller                           runner
//     |                                |
//     |---- acquire slot ------------->|
//     |       store F into slot        |
//     |---- signal job --------------->|
//     |                          take F, execute it, store R
//     |<--- signal done ---------------|
//     |       take R from slot         |
//     |---- signal ack --------------->|
//     |                          drop slot contents, mark slot free
//     |                                |
//```
// Slot ownership according to the protocol:
//   - caller owns the slot between acquiring it and signalling job
//   - runner owns the slot between receiving job and signalling done
//   - caller owns the slot between receiving done and signalling ack
//   - runner owns the slot between receiving ack and marking the slot free
//
// To support cancellation and restartabilty of run(), we can extend the above
// with a few atomic flags:
//   - `running`: prevents concurrent `run()` calls. Cleared by
//     a drop guard so a new `run()` can start after the old one is dropped.
//   - `initialized`: ensures the slot is marked free exactly
//     once across all `run()` calls. Prevents reopening a slot occupied by a
//     stale call on restart.
//   - `needs_recovery`: set while a job is in-flight. If the
//     runner task is dropped while this is true, the next `run()` waits for
//     the caller's ack and cleans up before accepting new work.
//
// ```text
//   caller                          runner
//     |                               |
//     |                         [running = true, drop guard armed]
//     |                         [if !initialized: mark slot free]
//     |                               |
//     |--- acquire slot ------------->|
//     |    store F in slot            |
//     |--- signal job --------------->|
//     |                         [needs_recovery = true]
//     |                         take F, execute it, store R
//     |<-- signal done ---------------|
//     |    take R from slot           |
//     |--- signal ack --------------->|
//     |                         drop slot contents
//     |                         [needs_recovery = false]
//     |                         mark slot free
//     |                               |
// ```
// The runner can be dropped at any await point (job.wait, ack.wait).
// If dropped while needs_recovery is true, the slot may still contain
// data and the caller may still be active. The next run() checks
// needs_recovery, waits for the caller's ack, and cleans up before
// entering the main loop.
//
// The key invariant is that **every job signal is eventually followed by an
// ack signal**, provided the caller is eventually dropped. CallFuture::drop
// sends ack if the closure was already submitted. This guarantees the
// runner (or the next runner, after recovery) can always make progress.
//
// Closures and return types should avoid unwinding. If `f(state)` panics,
// `needs_recovery` will stay set and `done` is never signaled. The caller blocks
// until dropped, at which point its Drop sends ack and the next `run()` can
// recover. This provides us with recovery after drop, but there is no liveness
// guarantee. That is, if the caller is never dropped, the service becomes blocked. If
// `R`'s destructor panics during cleanup, the `FinishGuard` in `wait_ack_and_finish`
// ensures the slot is still freed and `needs_recovery` is cleared. The destructor's
// side effects are lost but the service remains usable.
// TODO: this could probably be circumvented, but is is worth it... :doubt:

struct SlotState {
    free: bool,
    waker: WakerRegistration,
}

impl SlotState {
    const EMPTY: Self = Self {
        free: false,
        waker: WakerRegistration::new(),
    };
}

/// Type-erased storage for closures and return values.
///
/// Invariants:
/// - `drop_fn = Some(drop_glue::<T>)` iff the slot contains a live `T`
/// - `store<T>()` writes a `T` and arms `drop_fn`
/// - `take<T>()` reads a `T` out and clears `drop_fn`
/// - `drop_contents()` drops in place if occupied; no-op if empty
#[repr(C, align(8))]
//TODO: unflexible, but need to choose some value... maybe target_pointer_width is better?
struct Storage<const S: usize> {
    buf: UnsafeCell<MaybeUninit<[u8; S]>>,
    drop_fn: UnsafeCell<Option<unsafe fn(&Self)>>,
}

impl<const S: usize> Storage<S> {
    const fn new() -> Self {
        Self {
            buf: UnsafeCell::new(MaybeUninit::uninit()),
            drop_fn: UnsafeCell::new(None),
        }
    }

    /// # Safety
    /// Slot must be empty. `size_of::<T>() <= S` and `align_of::<T>() <= align_of::<Self>()`.
    unsafe fn store<T>(&self, val: T) {
        // SAFETY: caller guarantees the slot is empty and T fits.
        unsafe {
            (*self.buf.get()).as_mut_ptr().cast::<T>().write(val);
            *self.drop_fn.get() = Some(Self::drop_typed::<T>);
        }
    }

    /// # Safety
    /// Slot must contain a live `T`.
    unsafe fn take<T>(&self) -> T {
        // SAFETY: caller guarantees a live T is in the slot.
        unsafe {
            let val = (*self.buf.get()).as_ptr().cast::<T>().read();
            *self.drop_fn.get() = None;
            val
        }
    }

    /// # Safety
    /// Caller must have exclusive access.
    ///
    /// # Panic behavior
    /// If the stored value's destructor panics, `drop_fn` is already
    /// cleared so double-drop won't occur
    unsafe fn drop_contents(&self) {
        // SAFETY: caller guarantees exclusive access.
        unsafe {
            if let Some(f) = (*self.drop_fn.get()).take() {
                f(self);
            }
        }
    }

    /// # Safety
    /// `slot` must currently contain a live `T`.
    unsafe fn drop_typed<T>(slot: &Self) {
        // SAFETY: caller guarantees the slot contains a live T.
        unsafe {
            core::ptr::drop_in_place((*slot.buf.get()).as_mut_ptr().cast::<T>());
        }
    }
}

impl<const S: usize> Drop for Storage<S> {
    fn drop(&mut self) {
        // SAFETY: &mut self guarantees exclusive access.
        unsafe { self.drop_contents() };
    }
}

/// # Safety
/// - `slot` must currently contain a live `F`.
/// - `R` must fit in the slot.
///
/// After return, slot contains a live `R`.
///
/// # Panic behavior
/// If `f(state)` panics, `F` has already been taken from the slot and `R` is
/// never stored. The slot is left empty (`drop_fn` is `None`).
unsafe fn run_job<T, R, F: FnOnce(&mut T) -> R, const S: usize>(slot: &Storage<S>, state: &mut T) {
    // SAFETY: caller guarantees slot contains a live F and R fits.
    unsafe {
        let f: F = slot.take();
        let res = f(state);
        slot.store(res);
    }
}

struct JobSlot<M: RawMutex, T, const S: usize> {
    storage: Storage<S>,
    state: Mutex<M, Cell<SlotState>>,
    job: Signal<M, RunFn<T, S>>,
    done: Signal<M, ()>,
    ack: Signal<M, ()>,
}

impl<M: RawMutex, T, const S: usize> JobSlot<M, T, S> {
    const fn new() -> Self {
        Self {
            storage: Storage::new(),
            state: Mutex::new(Cell::new(SlotState::EMPTY)),
            job: Signal::new(),
            done: Signal::new(),
            ack: Signal::new(),
        }
    }

    // TODO: should remove this or make debug_assert message clearer.
    fn debug_assert_held(&self) {
        self.state.lock(|cell| {
            let s = cell.replace(SlotState::EMPTY);
            debug_assert!(!s.free, "slot accessed without being held");
            cell.set(s);
        });
    }

    // TODO: documentation needed
    fn poll_acquire(&self, cx: &mut Context<'_>) -> Poll<()> {
        self.state.lock(|cell| {
            let mut s = cell.replace(SlotState::EMPTY);
            if s.free {
                s.free = false;
                cell.set(s);
                Poll::Ready(())
            } else {
                s.waker.register(cx.waker());
                cell.set(s);
                Poll::Pending
            }
        })
    }

    // TODO: documentation needed
    fn try_acquire(&self) -> bool {
        self.state.lock(|cell| {
            let mut s = cell.replace(SlotState::EMPTY);
            if s.free {
                s.free = false;
                cell.set(s);
                true
            } else {
                cell.set(s);
                false
            }
        })
    }

    /// # Safety
    /// Caller must have acquired the slot. F and R must fit.
    unsafe fn submit<R, F: FnOnce(&mut T) -> R>(&self, f: F) {
        // SAFETY: caller guarantees slot is acquired and F/R fit.
        unsafe { self.storage.store(f) };
        self.job.signal(run_job::<T, R, F, S>);
    }

    /// # Safety
    /// Caller must have acquired the slot. F must fit.
    unsafe fn submit_immediate<F: FnOnce(&mut T)>(&self, f: F) {
        // SAFETY: caller guarantees slot is acquired and F fits.
        unsafe { self.storage.store(f) };
        self.ack.signal(());
        self.job.signal(run_job::<T, (), F, S>);
    }

    fn mark_free(&self) {
        self.state.lock(|cell| {
            let mut s = cell.replace(SlotState::EMPTY);
            s.free = true;
            // Set the state to free before waking, so the woken caller
            // sees free=true when it re-enters poll_acquire.
            cell.set(SlotState {
                free: true,
                waker: WakerRegistration::new(),
            });
            s.waker.wake();
        });
    }

    fn poll_result<R>(&self, cx: &mut Context<'_>) -> Poll<R> {
        let mut fut = self.done.wait();
        match Pin::new(&mut fut).poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(()) => {
                self.debug_assert_held();
                // SAFETY: done was signaled, so the runner wrote R into the slot.
                let result = unsafe { self.storage.take::<R>() };
                self.ack.signal(());
                Poll::Ready(result)
            }
        }
    }

    // TODO: maybe more docs
    /// Wait for the caller's ack, then clean up the slot and mark it free.
    ///
    /// If the stored value's destructor panics under unwinding, the slot
    /// is still reset and freed.
    async fn wait_ack_and_finish(&self, needs_recovery: &AtomicBool) {
        struct FinishGuard<'a, M: RawMutex, T, const S: usize> {
            slot: &'a JobSlot<M, T, S>,
            needs_recovery: &'a AtomicBool,
        }
        impl<M: RawMutex, T, const S: usize> Drop for FinishGuard<'_, M, T, S> {
            fn drop(&mut self) {
                self.needs_recovery.store(false, Ordering::Release);
                self.slot.done.reset();
                self.slot.mark_free();
            }
        }

        self.ack.wait().await;
        self.debug_assert_held();

        let _guard = FinishGuard {
            slot: self,
            needs_recovery,
        };
        // SAFETY: ack received, so the caller is done with the slot.
        // drop_contents() already guards against double drop.
        unsafe { self.storage.drop_contents() };
    }
}

/// Dispatch closures for execution on a dedicated runner task.
///
/// Callers submit an `FnOnce(&mut T) -> R` via [`call`](ContextService::call).
/// The runner, started with [`run`](ContextService::run), executes closures one at a
/// time with exclusive `&mut T` access and sends results back. Each call
/// can return a different type.
///
/// Closures and return values are stored inline in a fixed-size slot of `S` bytes.
/// Both the closure and the return type must fit within `S` bytes and require no
/// more alignment than the slot. This is checked at compile time.
///
/// ## Example
///
/// ```rust,ignore
/// static FS: ContextService<CriticalSectionRawMutex, Filesystem, 64> =
///     ContextService::new();
///
/// // runner task
/// FS.run(&mut filesystem).await;
///
/// // caller task
/// let size = FS.call(|fs| fs.read_blocking(path).len()).await;
/// ```
pub struct ContextService<M: RawMutex, T, const S: usize> {
    slot: JobSlot<M, T, S>,
    running: AtomicBool,
    initialized: AtomicBool,
    needs_recovery: AtomicBool,
}

impl<M: RawMutex, T, const S: usize> ContextService<M, T, S> {
    /// Create a new `ContextService`.
    pub const fn new() -> Self {
        Self {
            slot: JobSlot::new(),
            running: AtomicBool::new(false),
            initialized: AtomicBool::new(false),
            needs_recovery: AtomicBool::new(false),
        }
    }

    /// Submit a closure for execution on the runner.
    ///
    /// Fails at compile time if `F` or `R` exceeds the slot capacity `S`.
    ///
    /// **Note:** Under panic=unwind, a panicking closure prevents the returned
    /// future from ever completing.
    ///
    /// ## Cancellation
    ///
    /// The returned future is cancel-safe: dropping it at any point is
    /// sound and leaves the service in a usable state. If dropped before
    /// the closure has been submitted, no work is performed. If dropped
    /// after submission, the closure will still be executed to completion
    /// and the return value is discarded.
    pub fn call<R, F>(&self, f: F) -> CallFuture<'_, M, T, R, F, S>
    where
        F: FnOnce(&mut T) -> R + Send + 'static,
        R: Send + 'static,
    {
        const { assert_slot_fits::<F, R, S>() };

        CallFuture {
            svc: self,
            f: Some(f),
            phase: Phase::Acquiring,
            _marker: PhantomData,
        }
    }

    /// Try to submit a fire-and-forget closure without blocking.
    ///
    /// Returns `true` if the closure was submitted, `false` if the slot is busy.
    /// A submitted closure will be executed during the next [`run()`](Self::run)
    /// call. If no `run()` is currently active, it remains pending until one
    /// starts. There is no way to retrieve a return value.
    pub fn try_call_immediate<F>(&self, f: F) -> bool
    where
        F: FnOnce(&mut T) + Send + 'static,
    {
        const { assert_slot_fits::<F, (), S>() };

        if !self.slot.try_acquire() {
            return false;
        }

        // SAFETY: we just acquired the slot, F fits (compile-time check above).
        unsafe { self.slot.submit_immediate(f) };
        true
    }

    /// Run the service loop, executing closures submitted via [`call`](Self::call)
    /// with exclusive `&mut T` access. This future must be polled (f.ex spawned as
    /// a task) for callers to make progress.
    ///
    /// # Panics
    ///
    /// Panics if called while another `run()` is still active.
    /// Sequential calls after a previous `run()` was dropped are fine.
    ///
    /// # Cancellation
    ///
    /// This future is cancel-safe. A subsequent call to `run()` will recover the
    /// previous state and resume processing any in-flight call. Callers that were
    /// blocked will transparently continue once the new `run()` starts.
    pub async fn run(&self, state: &mut T) -> ! {
        struct RunGuard<'a> {
            running: &'a AtomicBool,
        }
        impl Drop for RunGuard<'_> {
            fn drop(&mut self) {
                self.running.store(false, Ordering::Release);
            }
        }

        if self.running.swap(true, Ordering::AcqRel) {
            panic!("ContextService::run() must not be called concurrently")
        }
        let _guard = RunGuard { running: &self.running };

        // If the previous runner was cancelled mid-job the caller might still
        // be interacting with the slot. We must wait for it to finish (the caller
        // always acks, either explicitly or via its Drop) and then clean up.
        if self.needs_recovery.load(Ordering::Acquire) {
            self.slot.wait_ack_and_finish(&self.needs_recovery).await;
        }

        // Only the first run() may open the slot. On restarts, the slot
        // is either already free or holds a pending job from a caller
        // that submitted while no run() was active, which we may not clear!
        if !self.initialized.swap(true, Ordering::Relaxed) {
            self.slot.mark_free();
        }

        loop {
            // Wait for a caller to submit a closure.
            // This is a clean cancellation point because no job in flight
            let run_fn = self.slot.job.wait().await;

            // From here on we may need to recover if cancellation occurs
            self.needs_recovery.store(true, Ordering::Release);

            // SAFETY: slot contains a live F, run_fn matches its types.
            // No other task can access the slot, because the caller is waiting on done.
            // If the closure panics under unwinding, done is not signaled. The
            // caller would then block until dropped, but dropping sends an ack
            // and then allows for run to make progress.
            unsafe { run_fn(&self.slot.storage, state) };
            self.slot.done.signal(());

            // Wait for the caller to read R and signal ack (or for
            // CallFuture::drop to signal ack on cancellation), then
            // clean up the slot and mark it free for the next caller.
            // If cancelled here, needs_recovery is true and the next
            // run() will wait for ack before touching the slot.
            self.slot.wait_ack_and_finish(&self.needs_recovery).await;
        }
    }
}

// SAFETY: access to Storage is serialized by the call/run handshake protocol
unsafe impl<M: RawMutex, T, const S: usize> Sync for ContextService<M, T, S>
where
    Mutex<M, Cell<SlotState>>: Sync,
    Signal<M, RunFn<T, S>>: Sync,
    Signal<M, ()>: Sync,
{
}

/// State of the [`CallFuture`]
enum Phase {
    Acquiring,
    Submitted,
    Done,
}

/// Future returned by [`ContextService::call`].
///
/// This future is cancel-safe. See [`ContextService::call`] for details.
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct CallFuture<'a, M: RawMutex, T, R, F, const S: usize> {
    svc: &'a ContextService<M, T, S>,
    f: Option<F>,
    phase: Phase,
    _marker: PhantomData<R>,
}

impl<M: RawMutex, T, R, F, const S: usize> Unpin for CallFuture<'_, M, T, R, F, S> {}

impl<M, T, R, F, const S: usize> Future for CallFuture<'_, M, T, R, F, S>
where
    M: RawMutex,
    F: FnOnce(&mut T) -> R + Send + 'static,
    R: Send + 'static,
{
    type Output = R;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<R> {
        loop {
            match self.phase {
                Phase::Acquiring => match self.svc.slot.poll_acquire(cx) {
                    Poll::Pending => return Poll::Pending,
                    Poll::Ready(()) => {
                        let f = self.f.take().unwrap();
                        // SAFETY: we just acquired the slot, F and R fit (compile-time check in call())
                        unsafe { self.svc.slot.submit::<R, F>(f) };
                        self.phase = Phase::Submitted;
                    }
                },
                Phase::Submitted => {
                    return match self.svc.slot.poll_result::<R>(cx) {
                        Poll::Pending => Poll::Pending,
                        Poll::Ready(result) => {
                            self.phase = Phase::Done;
                            Poll::Ready(result)
                        }
                    };
                }
                Phase::Done => panic!("CallFuture polled after completion"),
            }
        }
    }
}

impl<M: RawMutex, T, R, F, const S: usize> Drop for CallFuture<'_, M, T, R, F, S> {
    fn drop(&mut self) {
        if matches!(self.phase, Phase::Submitted) {
            // Future dropped after the job was submitted. The runner will still
            // finish executing the closure, but we cannot touch the slot (the runner
            // may still be using it) and we should not block. Signal ack so the
            // runner can clean up and accept new work after it completes.
            self.svc.slot.ack.signal(());
        }
    }
}

const fn assert_slot_fits<F, R, const S: usize>() {
    assert!(mem::size_of::<F>() <= S, "closure must fit in slot, increase S");
    assert!(mem::size_of::<R>() <= S, "return type must fit in slot, increase S");
    assert!(
        mem::align_of::<F>() <= mem::align_of::<Storage<S>>(),
        "closure alignment must not exceed 8 bytes"
    );
    assert!(
        mem::align_of::<R>() <= mem::align_of::<Storage<S>>(),
        "return type alignment must not exceed 8 bytes"
    );
}

#[cfg(test)]
mod tests {
    extern crate alloc;

    use alloc::string::String;
    use alloc::sync::Arc;
    use alloc::vec::Vec;
    use core::sync::atomic::{AtomicUsize, Ordering};

    use super::*;
    use crate::blocking_mutex::raw::{CriticalSectionRawMutex, NoopRawMutex};
    use core::pin::pin;

    /// Run `caller` against `runner` until the caller completes.
    async fn drive<R, N>(caller: impl Future<Output = R>, runner: impl Future<Output = N>) -> R {
        let caller = pin!(caller);
        let runner = pin!(runner);
        match futures_util::future::select(caller, runner).await {
            futures_util::future::Either::Left((r, _)) => r,
            futures_util::future::Either::Right(_) => unreachable!(),
        }
    }

    fn add(n: i32) -> impl FnOnce(&mut i32) -> i32 + Send + 'static {
        move |s| {
            *s += n;
            *s
        }
    }

    #[futures_test::test]
    async fn basic() {
        static SVC: ContextService<CriticalSectionRawMutex, i32, 64> = ContextService::new();
        let mut state = 0i32;
        assert_eq!(drive(SVC.call(add(10)), SVC.run(&mut state)).await, 10);
    }

    #[futures_test::test]
    async fn different_return_types() {
        let svc: ContextService<NoopRawMutex, Vec<String>, 256> = ContextService::new();
        let mut state = Vec::from([String::from("hello")]);
        drive(
            async {
                assert_eq!(svc.call(|s| s.len()).await, 1);
                svc.call(|s| s.push("world".into())).await;
                assert_eq!(svc.call(|s| s.join(" ")).await, "hello world");
            },
            svc.run(&mut state),
        )
        .await;
    }

    #[futures_test::test]
    async fn cancel_before_acquire_then_next_call() {
        let svc: ContextService<NoopRawMutex, i32, 64> = ContextService::new();
        let mut state = 0i32;
        let r = drive(
            async {
                {
                    let fut = svc.call(add(100));
                    let mut fut = pin!(fut);
                    assert!(
                        futures_util::poll!(fut.as_mut()).is_pending(),
                        "slot should not be free without a runner"
                    );
                }
                svc.call(add(1)).await
            },
            svc.run(&mut state),
        )
        .await;
        assert_eq!(r, 1);
    }

    #[futures_test::test]
    #[should_panic(expected = "must not be called concurrently")]
    async fn concurrent_run_panics() {
        let svc: ContextService<NoopRawMutex, i32, 64> = ContextService::new();
        let mut s1 = 0;
        let mut s2 = 0;
        let a = svc.run(&mut s1);
        let b = svc.run(&mut s2);
        let mut a = pin!(a);
        let mut b = pin!(b);
        let _ = futures_util::poll!(a.as_mut());
        let _ = futures_util::poll!(b.as_mut());
    }

    #[futures_test::test]
    async fn restart_after_cancel_mid_job() {
        let svc: ContextService<NoopRawMutex, (), 64> = ContextService::new();
        let mut state = ();

        let drop_count = Arc::new(AtomicUsize::new(0));

        struct Tracked(Arc<AtomicUsize>);
        impl Drop for Tracked {
            fn drop(&mut self) {
                self.0.fetch_add(1, Ordering::Relaxed);
            }
        }

        {
            let runner = svc.run(&mut state);
            let mut runner = pin!(runner);
            let _ = futures_util::poll!(runner.as_mut());

            {
                let dc = drop_count.clone();
                let fut = svc.call(move |_| Tracked(dc));
                let mut fut = pin!(fut);
                let _ = futures_util::poll!(fut.as_mut());
                let _ = futures_util::poll!(runner.as_mut());
            }
            // Runner dropped while needs_recovery is true.
        }

        assert_eq!(
            drop_count.load(Ordering::Relaxed),
            0,
            "R should not be dropped before recovery"
        );

        {
            let runner = svc.run(&mut state);
            let mut runner = pin!(runner);
            let _ = futures_util::poll!(runner.as_mut());

            assert_eq!(drop_count.load(Ordering::Relaxed), 1, "recovery should drop R");

            assert_eq!(drive(svc.call(|_| 42u32), runner).await, 42);
        }
    }

    #[futures_test::test]
    async fn restart_many_cycles() {
        let svc: ContextService<NoopRawMutex, i32, 64> = ContextService::new();
        let mut state = 0i32;

        for _ in 0..10 {
            drive(svc.call(add(1)), svc.run(&mut state)).await;
        }

        assert_eq!(state, 10);
    }

    #[futures_test::test]
    async fn zero_sized_return() {
        let svc: ContextService<NoopRawMutex, i32, 64> = ContextService::new();
        let mut state = 0i32;
        drive(svc.call(|s| { *s += 1; }), svc.run(&mut state)).await;
    }

    #[futures_test::test]
    async fn drop_glue_on_service_drop() {
        let drop_count = Arc::new(AtomicUsize::new(0));

        struct Payload(Arc<AtomicUsize>);
        impl Drop for Payload {
            fn drop(&mut self) {
                self.0.fetch_add(1, Ordering::Relaxed);
            }
        }

        {
            let svc: ContextService<NoopRawMutex, (), 64> = ContextService::new();
            let mut state = ();
            let dc = drop_count.clone();
            let caller = async {
                let fut = svc.call(move |_| Payload(dc));
                let mut fut = pin!(fut);
                assert!(futures_util::poll!(fut.as_mut()).is_pending());
            };
            let runner = svc.run(&mut state);
            let mut runner = pin!(runner);
            let _ = futures_util::poll!(runner.as_mut());
            let mut caller = pin!(caller);
            let _ = futures_util::poll!(caller.as_mut());
            let _ = futures_util::poll!(runner.as_mut());
            drop(caller);
            let _ = futures_util::poll!(runner.as_mut());
        }
        assert_eq!(
            drop_count.load(Ordering::Relaxed),
            1,
            "service drop should drop slot contents"
        );
    }

    #[futures_test::test]
    async fn cancel_after_submit_no_leak() {
        let drop_count = Arc::new(AtomicUsize::new(0));

        struct Heavy(Arc<AtomicUsize>, #[allow(dead_code)] [u8; 4096]);
        impl Drop for Heavy {
            fn drop(&mut self) {
                self.0.fetch_add(1, Ordering::Relaxed);
            }
        }

        let svc: ContextService<NoopRawMutex, (), 4160> = ContextService::new();
        let mut state = ();

        let runner = svc.run(&mut state);
        let mut runner = pin!(runner);
        let _ = futures_util::poll!(runner.as_mut());

        {
            let dc = drop_count.clone();
            let fut = svc.call(move |_| Heavy(dc, [0xAB; 4096]));
            let mut fut = pin!(fut);
            let _ = futures_util::poll!(fut.as_mut());
            let _ = futures_util::poll!(runner.as_mut());
        }
        assert_eq!(
            drop_count.load(Ordering::Relaxed),
            0,
            "R should not be dropped before runner cleanup"
        );

        let _ = futures_util::poll!(runner.as_mut());

        drive(svc.call(|_| 42u32), runner).await;

        assert_eq!(
            drop_count.load(Ordering::Relaxed),
            1,
            "runner should drop R during cleanup"
        );
    }

    #[futures_test::test]
    async fn try_call_immediate() {
        let svc: ContextService<NoopRawMutex, i32, 64> = ContextService::new();
        let mut state = 0i32;

        assert!(!svc.try_call_immediate(|s| *s += 100)); // no runner yet

        {
            let runner = svc.run(&mut state);
            let mut runner = pin!(runner);
            let _ = futures_util::poll!(runner.as_mut());

            assert!(svc.try_call_immediate(|s| *s += 1));
            assert!(!svc.try_call_immediate(|s| *s += 100)); // slot busy

            let _ = futures_util::poll!(runner.as_mut());
            let _ = futures_util::poll!(runner.as_mut());

            assert_eq!(drive(svc.call(|_| 42u32), runner).await, 42);
        }

        assert_eq!(state, 1);
    }

    #[test]
    #[cfg(feature = "std")]
    fn destructor_panic_recovery() {
        // Destructor of return value panics during cleanup after caller was dropped.
        // FinishGuard should recover the service
        extern crate std;
        use std::panic::{AssertUnwindSafe, catch_unwind};

        struct PanicOnDrop;
        impl Drop for PanicOnDrop {
            fn drop(&mut self) {
                panic!("destructor panic");
            }
        }

        let svc: ContextService<NoopRawMutex, (), 64> = ContextService::new();
        let mut state = ();

        let result = catch_unwind(AssertUnwindSafe(|| {
            futures_executor::block_on(async {
                let runner = svc.run(&mut state);
                let mut runner = pin!(runner);
                let _ = futures_util::poll!(runner.as_mut());

                {
                    let fut = svc.call(|_| PanicOnDrop);
                    let mut fut = pin!(fut);
                    let _ = futures_util::poll!(fut.as_mut());
                    let _ = futures_util::poll!(runner.as_mut());
                }

                let _ = futures_util::poll!(runner.as_mut());
            });
        }));
        assert!(result.is_err(), "destructor should have panicked");

        futures_executor::block_on(async {
            assert_eq!(drive(svc.call(|_| 42u32), svc.run(&mut state)).await, 42);
        });
    }

    #[test]
    #[cfg(feature = "std")]
    fn closure_panic_recovery() {
        // The closure panics during execution. Caller is dropped during
        // unwind, should send ack, and the service recovers
        extern crate std;
        use std::panic::{AssertUnwindSafe, catch_unwind};

        let svc: ContextService<NoopRawMutex, i32, 64> = ContextService::new();
        let mut state = 0i32;

        let result = catch_unwind(AssertUnwindSafe(|| {
            futures_executor::block_on(async {
                let fut = svc.call(|_: &mut i32| -> i32 { panic!("closure panic") });
                let mut fut = pin!(fut);

                let runner = svc.run(&mut state);
                let mut runner = pin!(runner);
                let _ = futures_util::poll!(runner.as_mut());
                let _ = futures_util::poll!(fut.as_mut());
                let _ = futures_util::poll!(runner.as_mut());
            });
        }));
        assert!(result.is_err(), "closure should have panicked");

        futures_executor::block_on(async {
            assert_eq!(drive(svc.call(add(1)), svc.run(&mut state)).await, 1);
        });
    }

    #[futures_test::test]
    #[should_panic(expected = "polled after completion")]
    async fn poll_after_completion_panics() {
        let svc: ContextService<NoopRawMutex, i32, 64> = ContextService::new();
        let mut state = 0i32;
        let runner = svc.run(&mut state);
        let mut runner = pin!(runner);
        let _ = futures_util::poll!(runner.as_mut());

        let fut = svc.call(add(1));
        let mut fut = pin!(fut);

        let _ = futures_util::poll!(fut.as_mut());
        let _ = futures_util::poll!(runner.as_mut());
        let _ = futures_util::poll!(fut.as_mut());
        let _ = futures_util::poll!(runner.as_mut());

        let _ = futures_util::poll!(fut.as_mut()); // poll after Done
    }

    #[futures_test::test]
    async fn restart_with_pending_job() {
        // Runner is dropped with a pending job in the slot. The
        // new runner should process it and not mark the slot free.
        let svc: ContextService<NoopRawMutex, i32, 64> = ContextService::new();
        let mut state = 0i32;

        {
            let runner = svc.run(&mut state);
            let mut runner = pin!(runner);
            let _ = futures_util::poll!(runner.as_mut());
        }

        let caller = svc.call(add(1));
        let mut caller = pin!(caller);
        let _ = futures_util::poll!(caller.as_mut());

        assert_eq!(drive(caller, svc.run(&mut state)).await, 1);
    }
}

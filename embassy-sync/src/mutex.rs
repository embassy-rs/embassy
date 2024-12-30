//! Async mutex.
//!
//! This module provides a mutex that can be used to synchronize data between asynchronous tasks.
use core::cell::{Cell, UnsafeCell};
use core::future::{poll_fn, Future};
use core::ops::{Deref, DerefMut};
use core::task::Poll;
use core::{fmt, mem};

use crate::blocking_mutex::raw::RawMutex;
use crate::blocking_mutex::Mutex as BlockingMutex;
use crate::waitqueue::NonSyncWakerRegistration;

/// Error returned by [`Mutex::try_lock`]
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct TryLockError;

struct State {
    locked: Cell<bool>,
    waker: NonSyncWakerRegistration,
}

impl State {
    const fn new() -> Self {
        Self {
            locked: Cell::new(false),
            waker: NonSyncWakerRegistration::new(),
        }
    }

    fn lock(&self, waker: &core::task::Waker) -> bool {
        if self.locked.replace(true) {
            self.waker.register(waker);
            false
        } else {
            true
        }
    }

    fn try_lock(&self) -> Result<(), TryLockError> {
        if self.locked.replace(true) {
            Err(TryLockError)
        } else {
            Ok(())
        }
    }

    fn unlock(&self) {
        self.waker.wake();
        self.locked.set(false);
    }
}

/// Async mutex.
///
/// The mutex is generic over a blocking [`RawMutex`].
/// The raw mutex is used to guard access to the internal "is locked" flag. It
/// is held for very short periods only, while locking and unlocking. It is *not* held
/// for the entire time the async Mutex is locked.
///
/// Which implementation you select depends on the context in which you're using the mutex.
///
/// Use [`CriticalSectionRawMutex`](crate::blocking_mutex::raw::CriticalSectionRawMutex) when data can be shared between threads and interrupts.
///
/// Use [`NoopRawMutex`](crate::blocking_mutex::raw::NoopRawMutex) when data is only shared between tasks running on the same executor.
///
/// Use [`ThreadModeRawMutex`](crate::blocking_mutex::raw::ThreadModeRawMutex) when data is shared between tasks running on the same executor but you want a singleton.
///
pub struct Mutex<M, T>
where
    M: RawMutex,
    T: ?Sized,
{
    state: BlockingMutex<M, State>,
    inner: UnsafeCell<T>,
}

unsafe impl<M: RawMutex + Send, T: ?Sized + Send> Send for Mutex<M, T> {}
unsafe impl<M: RawMutex + Sync, T: ?Sized + Send> Sync for Mutex<M, T> {}

/// Async mutex.
impl<M, T> Mutex<M, T>
where
    M: RawMutex,
{
    /// Create a new mutex with the given value.
    pub const fn new(value: T) -> Self {
        Self {
            inner: UnsafeCell::new(value),
            state: BlockingMutex::new(State::new()),
        }
    }
}

impl<M, T> Mutex<M, T>
where
    M: RawMutex,
    T: ?Sized,
{
    /// Lock the mutex.
    ///
    /// This will wait for the mutex to be unlocked if it's already locked.
    pub fn lock(&self) -> impl Future<Output = MutexGuard<'_, M, T>> {
        poll_fn(|cx| {
            let ready = self.state.lock(|s| s.lock(cx.waker()));

            if ready {
                Poll::Ready(MutexGuard { mutex: self })
            } else {
                Poll::Pending
            }
        })
    }

    /// Attempt to immediately lock the mutex.
    ///
    /// If the mutex is already locked, this will return an error instead of waiting.
    pub fn try_lock(&self) -> Result<MutexGuard<'_, M, T>, TryLockError> {
        self.state.lock(|s| s.try_lock())?;

        Ok(MutexGuard { mutex: self })
    }

    /// Consumes this mutex, returning the underlying data.
    pub fn into_inner(self) -> T
    where
        T: Sized,
    {
        self.inner.into_inner()
    }

    /// Returns a mutable reference to the underlying data.
    ///
    /// Since this call borrows the Mutex mutably, no actual locking needs to
    /// take place -- the mutable borrow statically guarantees no locks exist.
    pub fn get_mut(&mut self) -> &mut T {
        self.inner.get_mut()
    }
}

impl<M: RawMutex, T> From<T> for Mutex<M, T> {
    fn from(from: T) -> Self {
        Self::new(from)
    }
}

impl<M, T> Default for Mutex<M, T>
where
    M: RawMutex,
    T: Default,
{
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<M, T> fmt::Debug for Mutex<M, T>
where
    M: RawMutex,
    T: ?Sized + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut d = f.debug_struct("Mutex");
        match self.try_lock() {
            Ok(value) => {
                d.field("inner", &&*value);
            }
            Err(TryLockError) => {
                d.field("inner", &format_args!("<locked>"));
            }
        }

        d.finish_non_exhaustive()
    }
}

/// Async mutex guard.
///
/// Owning an instance of this type indicates having
/// successfully locked the mutex, and grants access to the contents.
///
/// Dropping it unlocks the mutex.
#[clippy::has_significant_drop]
#[must_use = "if unused the Mutex will immediately unlock"]
pub struct MutexGuard<'a, M, T>
where
    M: RawMutex,
    T: ?Sized,
{
    mutex: &'a Mutex<M, T>,
}

impl<'a, M, T> MutexGuard<'a, M, T>
where
    M: RawMutex,
    T: ?Sized,
{
    /// Returns a locked view over a portion of the locked data.
    pub fn map<U>(this: Self, fun: impl FnOnce(&mut T) -> &mut U) -> MappedMutexGuard<'a, M, U> {
        let mutex = this.mutex;
        let value = fun(unsafe { &mut *this.mutex.inner.get() });
        // Don't run the `drop` method for MutexGuard. The ownership of the underlying
        // locked state is being moved to the returned MappedMutexGuard.
        mem::forget(this);
        MappedMutexGuard {
            state: &mutex.state,
            value,
        }
    }
}

impl<'a, M, T> Drop for MutexGuard<'a, M, T>
where
    M: RawMutex,
    T: ?Sized,
{
    fn drop(&mut self) {
        self.mutex.state.lock(|s| s.unlock())
    }
}

impl<'a, M, T> Deref for MutexGuard<'a, M, T>
where
    M: RawMutex,
    T: ?Sized,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        // Safety: the MutexGuard represents exclusive access to the contents
        // of the mutex, so it's OK to get it.
        unsafe { &*(self.mutex.inner.get() as *const T) }
    }
}

impl<'a, M, T> DerefMut for MutexGuard<'a, M, T>
where
    M: RawMutex,
    T: ?Sized,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        // Safety: the MutexGuard represents exclusive access to the contents
        // of the mutex, so it's OK to get it.
        unsafe { &mut *(self.mutex.inner.get()) }
    }
}

impl<'a, M, T> fmt::Debug for MutexGuard<'a, M, T>
where
    M: RawMutex,
    T: ?Sized + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<'a, M, T> fmt::Display for MutexGuard<'a, M, T>
where
    M: RawMutex,
    T: ?Sized + fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

/// A handle to a held `Mutex` that has had a function applied to it via [`MutexGuard::map`] or
/// [`MappedMutexGuard::map`].
///
/// This can be used to hold a subfield of the protected data.
#[clippy::has_significant_drop]
pub struct MappedMutexGuard<'a, M, T>
where
    M: RawMutex,
    T: ?Sized,
{
    state: &'a BlockingMutex<M, State>,
    value: *mut T,
}

impl<'a, M, T> MappedMutexGuard<'a, M, T>
where
    M: RawMutex,
    T: ?Sized,
{
    /// Returns a locked view over a portion of the locked data.
    pub fn map<U>(this: Self, fun: impl FnOnce(&mut T) -> &mut U) -> MappedMutexGuard<'a, M, U> {
        let state = this.state;
        let value = fun(unsafe { &mut *this.value });
        // Don't run the `drop` method for MutexGuard. The ownership of the underlying
        // locked state is being moved to the returned MappedMutexGuard.
        mem::forget(this);
        MappedMutexGuard { state, value }
    }
}

impl<'a, M, T> Deref for MappedMutexGuard<'a, M, T>
where
    M: RawMutex,
    T: ?Sized,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        // Safety: the MutexGuard represents exclusive access to the contents
        // of the mutex, so it's OK to get it.
        unsafe { &*self.value }
    }
}

impl<'a, M, T> DerefMut for MappedMutexGuard<'a, M, T>
where
    M: RawMutex,
    T: ?Sized,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        // Safety: the MutexGuard represents exclusive access to the contents
        // of the mutex, so it's OK to get it.
        unsafe { &mut *self.value }
    }
}

impl<'a, M, T> Drop for MappedMutexGuard<'a, M, T>
where
    M: RawMutex,
    T: ?Sized,
{
    fn drop(&mut self) {
        self.state.lock(|s| s.unlock())
    }
}

unsafe impl<M, T> Send for MappedMutexGuard<'_, M, T>
where
    M: RawMutex + Sync,
    T: Send + ?Sized,
{
}

unsafe impl<M, T> Sync for MappedMutexGuard<'_, M, T>
where
    M: RawMutex + Sync,
    T: Sync + ?Sized,
{
}

impl<'a, M, T> fmt::Debug for MappedMutexGuard<'a, M, T>
where
    M: RawMutex,
    T: ?Sized + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<'a, M, T> fmt::Display for MappedMutexGuard<'a, M, T>
where
    M: RawMutex,
    T: ?Sized + fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

#[cfg(test)]
mod tests {
    use futures_util::future::join;
    use futures_util::FutureExt;

    use crate::blocking_mutex::raw::NoopRawMutex;
    use crate::mutex::{Mutex, MutexGuard};

    async fn increment_once(mutex: &Mutex<NoopRawMutex, i32>) {
        use core::future::poll_fn;
        use core::task::Poll;

        let mut guard = mutex.lock().await;
        let value = &mut *guard;

        // yield once to allow the other future to run while
        // we are holding an exclusive borrow
        let mut called = false;
        poll_fn(|cx| {
            if called {
                return Poll::Ready(());
            }
            called = true;
            cx.waker().wake_by_ref();
            Poll::Pending
        })
        .await;

        *value += 1;
    }

    #[futures_test::test]
    async fn mutex_actually_provides_exclusive_access() {
        let mutex: Mutex<NoopRawMutex, i32> = Mutex::new(0);

        let fut1 = async {
            for _ in 0..5 {
                increment_once(&mutex).await;
            }
        };

        let fut2 = async {
            for _ in 0..5 {
                increment_once(&mutex).await;
            }
        };

        join(fut1, fut2).await;

        assert_eq!(*mutex.lock().await, 10);
    }

    #[futures_test::test]
    async fn mapped_guard_releases_lock_when_dropped() {
        let mutex: Mutex<NoopRawMutex, [i32; 2]> = Mutex::new([0, 1]);

        {
            let guard = mutex.lock().await;
            assert_eq!(*guard, [0, 1]);
            let mut mapped = MutexGuard::map(guard, |this| &mut this[1]);
            assert_eq!(*mapped, 1);
            *mapped = 2;
        }

        {
            let guard = mutex.lock().await;
            assert_eq!(*guard, [0, 2]);
            let mut mapped = MutexGuard::map(guard, |this| &mut this[1]);
            assert_eq!(*mapped, 2);
            *mapped = 3;
        }

        assert_eq!(*mutex.lock().await, [0, 3]);
    }

    /// Test that user code does not cause UB.
    #[cfg(miri)]
    #[test]
    fn locking_in_a_waker_is_not_ub() {
        use core::future::Future;
        use core::pin::Pin;
        use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

        // Set up the same test as in `mutex_actually_provides_exclusive_access` but with a custom
        // evil block_on implementation that tries to lock the mutex inside a waker.
        let mutex: Mutex<NoopRawMutex, i32> = Mutex::new(0);

        let fut1 = async {
            for _ in 0..5 {
                increment_once(&mutex).await;
            }
        };

        let fut2 = async {
            for _ in 0..5 {
                increment_once(&mutex).await;
            }
        };

        let mut fut = join(fut1, fut2);

        // block_on impl

        unsafe fn wake(ctx: *const ()) {
            static NOOP_VTABLE: RawWakerVTable = RawWakerVTable::new(
                |_| RawWaker::new(core::ptr::null(), &NOOP_VTABLE),
                |_| {},
                |_| {},
                |_| {},
            );
            static NOOP_WAKER: Waker = unsafe { Waker::from_raw(RawWaker::new(core::ptr::null(), &NOOP_VTABLE)) };

            let mutex = ctx.cast::<Mutex<NoopRawMutex, i32>>().as_ref().unwrap();

            // Let's ask miri whether locking the same mutex inside a waker is UB.
            let mut fut = async {
                let _g = mutex.lock().await;
            };

            let mut fut = unsafe { Pin::new_unchecked(&mut fut) };
            _ = fut.poll_unpin(&mut Context::from_waker(&NOOP_WAKER));
        }

        static VTABLE: RawWakerVTable = RawWakerVTable::new(|ptr| RawWaker::new(ptr, &VTABLE), wake, wake, |_| {});

        let raw_waker = RawWaker::new(&mutex as *const Mutex<NoopRawMutex, i32> as *const _, &VTABLE);
        let waker = unsafe { Waker::from_raw(raw_waker) };
        let mut cx = Context::from_waker(&waker);

        let mut fut = unsafe { Pin::new_unchecked(&mut fut) };
        loop {
            if let Poll::Ready(_) = fut.as_mut().poll(&mut cx) {
                break;
            }
        }
    }
}

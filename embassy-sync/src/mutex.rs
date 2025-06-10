//! Async mutex.
//!
//! This module provides a mutex that can be used to synchronize data between asynchronous tasks.
use core::cell::{RefCell, UnsafeCell};
use core::future::{poll_fn, Future};
use core::ops::{Deref, DerefMut};
use core::task::Poll;
use core::{fmt, mem};

use crate::blocking_mutex::raw::RawMutex;
use crate::blocking_mutex::Mutex as BlockingMutex;
use crate::waitqueue::WakerRegistration;

/// Error returned by [`Mutex::try_lock`]
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct TryLockError;

struct State {
    locked: bool,
    waker: WakerRegistration,
}

/// Async mutex.
///
/// The mutex is generic over a blocking [`RawMutex`](crate::blocking_mutex::raw::RawMutex).
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
    state: BlockingMutex<M, RefCell<State>>,
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
            state: BlockingMutex::new(RefCell::new(State {
                locked: false,
                waker: WakerRegistration::new(),
            })),
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
            let ready = self.state.lock(|s| {
                let mut s = s.borrow_mut();
                if s.locked {
                    s.waker.register(cx.waker());
                    false
                } else {
                    s.locked = true;
                    true
                }
            });

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
        self.state.lock(|s| {
            let mut s = s.borrow_mut();
            if s.locked {
                Err(TryLockError)
            } else {
                s.locked = true;
                Ok(())
            }
        })?;

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
        self.mutex.state.lock(|s| {
            let mut s = unwrap!(s.try_borrow_mut());
            s.locked = false;
            s.waker.wake();
        })
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
    state: &'a BlockingMutex<M, RefCell<State>>,
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
        self.state.lock(|s| {
            let mut s = unwrap!(s.try_borrow_mut());
            s.locked = false;
            s.waker.wake();
        })
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
    use crate::blocking_mutex::raw::NoopRawMutex;
    use crate::mutex::{Mutex, MutexGuard};

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
}

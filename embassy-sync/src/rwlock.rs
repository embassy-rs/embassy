//! Async read-write lock.
//!
//! This module provides a read-write lock that can be used to synchronize data between asynchronous tasks.
use core::cell::{RefCell, UnsafeCell};
use core::fmt;
use core::future::{poll_fn, Future};
use core::ops::{Deref, DerefMut};
use core::task::Poll;

use crate::blocking_rwlock::raw::RawRwLock;
use crate::blocking_rwlock::RwLock as BlockingRwLock;
use crate::waitqueue::WakerRegistration;

/// Error returned by [`RwLock::try_read_lock`] and [`RwLock::try_write_lock`]
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct TryLockError;

struct State {
    readers: usize,
    writer: bool,
    waker: WakerRegistration,
}

/// Async read-write lock.
///
/// The read-write lock is generic over a blocking [`RawRwLock`](crate::blocking_mutex::raw_rwlock::RawRwLock).
/// The raw read-write lock is used to guard access to the internal state. It
/// is held for very short periods only, while locking and unlocking. It is *not* held
/// for the entire time the async RwLock is locked.
///
/// Which implementation you select depends on the context in which you're using the read-write lock.
///
/// Use [`CriticalSectionRawRwLock`](crate::blocking_mutex::raw_rwlock::CriticalSectionRawRwLock) when data can be shared between threads and interrupts.
///
/// Use [`NoopRawRwLock`](crate::blocking_mutex::raw_rwlock::NoopRawRwLock) when data is only shared between tasks running on the same executor.
///
/// Use [`ThreadModeRawRwLock`](crate::blocking_mutex::raw_rwlock::ThreadModeRawRwLock) when data is shared between tasks running on the same executor but you want a singleton.
///
pub struct RwLock<R, T>
where
    R: RawRwLock,
    T: ?Sized,
{
    state: BlockingRwLock<R, RefCell<State>>,
    inner: UnsafeCell<T>,
}

unsafe impl<R: RawRwLock + Send, T: ?Sized + Send> Send for RwLock<R, T> {}
unsafe impl<R: RawRwLock + Sync, T: ?Sized + Send> Sync for RwLock<R, T> {}

/// Async read-write lock.
impl<R, T> RwLock<R, T>
where
    R: RawRwLock,
{
    /// Create a new read-write lock with the given value.
    pub const fn new(value: T) -> Self {
        Self {
            inner: UnsafeCell::new(value),
            state: BlockingRwLock::new(RefCell::new(State {
                readers: 0,
                writer: false,
                waker: WakerRegistration::new(),
            })),
        }
    }
}

impl<R, T> RwLock<R, T>
where
    R: RawRwLock,
    T: ?Sized,
{
    /// Lock the read-write lock for reading.
    ///
    /// This will wait for the lock to be available if it's already locked for writing.
    pub fn read_lock(&self) -> impl Future<Output = RwLockReadGuard<'_, R, T>> {
        poll_fn(|cx| {
            let ready = self.state.write_lock(|s| {
                let mut s = s.borrow_mut();
                if s.writer {
                    s.waker.register(cx.waker());
                    false
                } else {
                    s.readers += 1;
                    true
                }
            });

            if ready {
                Poll::Ready(RwLockReadGuard { rwlock: self })
            } else {
                Poll::Pending
            }
        })
    }

    /// Lock the read-write lock for writing.
    ///
    /// This will wait for the lock to be available if it's already locked for reading or writing.
    pub fn write_lock(&self) -> impl Future<Output = RwLockWriteGuard<'_, R, T>> {
        poll_fn(|cx| {
            let ready = self.state.write_lock(|s| {
                let mut s = s.borrow_mut();
                if s.readers > 0 || s.writer {
                    s.waker.register(cx.waker());
                    false
                } else {
                    s.writer = true;
                    true
                }
            });

            if ready {
                Poll::Ready(RwLockWriteGuard { rwlock: self })
            } else {
                Poll::Pending
            }
        })
    }

    /// Attempt to immediately lock the read-write lock for reading.
    ///
    /// If the lock is already locked for writing, this will return an error instead of waiting.
    pub fn try_read_lock(&self) -> Result<RwLockReadGuard<'_, R, T>, TryLockError> {
        self.state.read_lock(|s| {
            let mut s = s.borrow_mut();
            if s.writer {
                Err(TryLockError)
            } else {
                s.readers += 1;
                Ok(())
            }
        })?;

        Ok(RwLockReadGuard { rwlock: self })
    }

    /// Attempt to immediately lock the read-write lock for writing.
    ///
    /// If the lock is already locked for reading or writing, this will return an error instead of waiting.
    pub fn try_write_lock(&self) -> Result<RwLockWriteGuard<'_, R, T>, TryLockError> {
        self.state.write_lock(|s| {
            let mut s = s.borrow_mut();
            if s.readers > 0 || s.writer {
                Err(TryLockError)
            } else {
                s.writer = true;
                Ok(())
            }
        })?;

        Ok(RwLockWriteGuard { rwlock: self })
    }

    /// Consumes this read-write lock, returning the underlying data.
    pub fn into_inner(self) -> T
    where
        T: Sized,
    {
        self.inner.into_inner()
    }

    /// Returns a mutable reference to the underlying data.
    ///
    /// Since this call borrows the RwLock mutably, no actual locking needs to
    /// take place -- the mutable borrow statically guarantees no locks exist.
    pub fn get_mut(&mut self) -> &mut T {
        self.inner.get_mut()
    }
}

impl<R: RawRwLock, T> From<T> for RwLock<R, T> {
    fn from(from: T) -> Self {
        Self::new(from)
    }
}

impl<R, T> Default for RwLock<R, T>
where
    R: RawRwLock,
    T: Default,
{
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<R, T> fmt::Debug for RwLock<R, T>
where
    R: RawRwLock,
    T: ?Sized + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut d = f.debug_struct("RwLock");
        match self.try_write_lock() {
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

/// Async read lock guard.
///
/// Owning an instance of this type indicates having
/// successfully locked the read-write lock for reading, and grants access to the contents.
///
/// Dropping it unlocks the read-write lock.
#[clippy::has_significant_drop]
#[must_use = "if unused the RwLock will immediately unlock"]
pub struct RwLockReadGuard<'a, R, T>
where
    R: RawRwLock,
    T: ?Sized,
{
    rwlock: &'a RwLock<R, T>,
}

impl<'a, R, T> Drop for RwLockReadGuard<'a, R, T>
where
    R: RawRwLock,
    T: ?Sized,
{
    fn drop(&mut self) {
        self.rwlock.state.write_lock(|s| {
            let mut s = unwrap!(s.try_borrow_mut());
            s.readers -= 1;
            if s.readers == 0 {
                s.waker.wake();
            }
        })
    }
}

impl<'a, R, T> Deref for RwLockReadGuard<'a, R, T>
where
    R: RawRwLock,
    T: ?Sized,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        // Safety: the RwLockReadGuard represents shared access to the contents
        // of the read-write lock, so it's OK to get it.
        unsafe { &*(self.rwlock.inner.get() as *const T) }
    }
}

impl<'a, R, T> fmt::Debug for RwLockReadGuard<'a, R, T>
where
    R: RawRwLock,
    T: ?Sized + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<'a, R, T> fmt::Display for RwLockReadGuard<'a, R, T>
where
    R: RawRwLock,
    T: ?Sized + fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

/// Async write lock guard.
///
/// Owning an instance of this type indicates having
/// successfully locked the read-write lock for writing, and grants access to the contents.
///
/// Dropping it unlocks the read-write lock.
#[clippy::has_significant_drop]
#[must_use = "if unused the RwLock will immediately unlock"]
pub struct RwLockWriteGuard<'a, R, T>
where
    R: RawRwLock,
    T: ?Sized,
{
    rwlock: &'a RwLock<R, T>,
}

impl<'a, R, T> Drop for RwLockWriteGuard<'a, R, T>
where
    R: RawRwLock,
    T: ?Sized,
{
    fn drop(&mut self) {
        self.rwlock.state.write_lock(|s| {
            let mut s = unwrap!(s.try_borrow_mut());
            s.writer = false;
            s.waker.wake();
        })
    }
}

impl<'a, R, T> Deref for RwLockWriteGuard<'a, R, T>
where
    R: RawRwLock,
    T: ?Sized,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        // Safety: the RwLockWriteGuard represents exclusive access to the contents
        // of the read-write lock, so it's OK to get it.
        unsafe { &*(self.rwlock.inner.get() as *mut T) }
    }
}

impl<'a, R, T> DerefMut for RwLockWriteGuard<'a, R, T>
where
    R: RawRwLock,
    T: ?Sized,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        // Safety: the RwLockWriteGuard represents exclusive access to the contents
        // of the read-write lock, so it's OK to get it.
        unsafe { &mut *(self.rwlock.inner.get()) }
    }
}

impl<'a, R, T> fmt::Debug for RwLockWriteGuard<'a, R, T>
where
    R: RawRwLock,
    T: ?Sized + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<'a, R, T> fmt::Display for RwLockWriteGuard<'a, R, T>
where
    R: RawRwLock,
    T: ?Sized + fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

#[cfg(test)]
mod tests {
    use crate::blocking_rwlock::raw::NoopRawRwLock;
    use crate::rwlock::RwLock;

    #[futures_test::test]
    async fn read_guard_releases_lock_when_dropped() {
        let rwlock: RwLock<NoopRawRwLock, [i32; 2]> = RwLock::new([0, 1]);

        {
            let guard = rwlock.read_lock().await;
            assert_eq!(*guard, [0, 1]);
        }

        {
            let guard = rwlock.read_lock().await;
            assert_eq!(*guard, [0, 1]);
        }

        assert_eq!(*rwlock.read_lock().await, [0, 1]);
    }

    #[futures_test::test]
    async fn write_guard_releases_lock_when_dropped() {
        let rwlock: RwLock<NoopRawRwLock, [i32; 2]> = RwLock::new([0, 1]);

        {
            let mut guard = rwlock.write_lock().await;
            assert_eq!(*guard, [0, 1]);
            guard[1] = 2;
        }

        {
            let guard = rwlock.read_lock().await;
            assert_eq!(*guard, [0, 2]);
        }

        assert_eq!(*rwlock.read_lock().await, [0, 2]);
    }
}

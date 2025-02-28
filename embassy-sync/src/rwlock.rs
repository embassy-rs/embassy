use core::cell::RefCell;
use core::future::{poll_fn, Future};
use core::ops::{Deref, DerefMut};
use core::task::Poll;

use crate::blocking_mutex::raw::RawMutex;
use crate::blocking_mutex::Mutex as BlockingMutex;
use crate::waitqueue::MultiWakerRegistration;

/// Error returned by [`RwLock::try_read`] and [`RwLock::try_write`]
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct TryLockError;

/// Async read-write lock.
///
/// The lock is generic over a blocking [`RawMutex`](crate::blocking_mutex::raw::RawMutex).
/// The raw mutex is used to guard access to the internal state. It
/// is held for very short periods only, while locking and unlocking. It is *not* held
/// for the entire time the async RwLock is locked.
///
/// Which implementation you select depends on the context in which you're using the lock.
///
/// Use [`CriticalSectionRawMutex`](crate::blocking_mutex::raw::CriticalSectionRawMutex) when data can be shared between threads and interrupts.
///
/// Use [`NoopRawMutex`](crate::blocking_mutex::raw::NoopRawMutex) when data is only shared between tasks running on the same executor.
///
/// Use [`ThreadModeRawMutex`](crate::blocking_mutex::raw::ThreadModeRawMutex) when data is shared between tasks running on the same executor but you want a singleton.
///
pub struct RwLock<M, T>
where
    M: RawMutex,
    T: ?Sized,
{
    state: BlockingMutex<M, RefCell<State>>,
    inner: RefCell<T>,
}

struct State {
    readers: usize,
    writer: bool,
    writer_waker: MultiWakerRegistration<1>,
    reader_wakers: MultiWakerRegistration<8>,
}

impl State {
    fn new() -> Self {
        Self {
            readers: 0,
            writer: false,
            writer_waker: MultiWakerRegistration::new(),
            reader_wakers: MultiWakerRegistration::new(),
        }
    }
}

impl<M, T> RwLock<M, T>
where
    M: RawMutex,
{
    /// Create a new read-write lock with the given value.
    pub const fn new(value: T) -> Self {
        Self {
            inner: RefCell::new(value),
            state: BlockingMutex::new(RefCell::new(State::new())),
        }
    }
}

impl<M, T> RwLock<M, T>
where
    M: RawMutex,
    T: ?Sized,
{
    /// Acquire a read lock.
    ///
    /// This will wait for the lock to be available if it's already locked for writing.
    pub fn read(&self) -> impl Future<Output = RwLockReadGuard<'_, M, T>> {
        poll_fn(|cx| {
            let mut state = self.state.lock(|s| s.borrow_mut());
            if state.writer {
                state.reader_wakers.register(cx.waker());
                Poll::Pending
            } else {
                state.readers += 1;
                Poll::Ready(RwLockReadGuard { lock: self })
            }
        })
    }

    /// Acquire a write lock.
    ///
    /// This will wait for the lock to be available if it's already locked for reading or writing.
    pub fn write(&self) -> impl Future<Output = RwLockWriteGuard<'_, M, T>> {
        poll_fn(|cx| {
            let mut state = self.state.lock(|s| s.borrow_mut());
            if state.writer || state.readers > 0 {
                state.writer_waker.register(cx.waker());
                Poll::Pending
            } else {
                state.writer = true;
                Poll::Ready(RwLockWriteGuard { lock: self })
            }
        })
    }

    /// Attempt to immediately acquire a read lock.
    ///
    /// If the lock is already locked for writing, this will return an error instead of waiting.
    pub fn try_read(&self) -> Result<RwLockReadGuard<'_, M, T>, TryLockError> {
        let mut state = self.state.lock(|s| s.borrow_mut());
        if state.writer {
            Err(TryLockError)
        } else {
            state.readers += 1;
            Ok(RwLockReadGuard { lock: self })
        }
    }

    /// Attempt to immediately acquire a write lock.
    ///
    /// If the lock is already locked for reading or writing, this will return an error instead of waiting.
    pub fn try_write(&self) -> Result<RwLockWriteGuard<'_, M, T>, TryLockError> {
        let mut state = self.state.lock(|s| s.borrow_mut());
        if state.writer || state.readers > 0 {
            Err(TryLockError)
        } else {
            state.writer = true;
            Ok(RwLockWriteGuard { lock: self })
        }
    }

    /// Consumes this lock, returning the underlying data.
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

impl<M, T> From<T> for RwLock<M, T>
where
    M: RawMutex,
{
    fn from(from: T) -> Self {
        Self::new(from)
    }
}

impl<M, T> Default for RwLock<M, T>
where
    M: RawMutex,
    T: Default,
{
    fn default() -> Self {
        Self::new(Default::default())
    }
}

/// Async read lock guard.
///
/// Owning an instance of this type indicates having
/// successfully locked the RwLock for reading, and grants access to the contents.
///
/// Dropping it unlocks the RwLock.
#[must_use = "if unused the RwLock will immediately unlock"]
pub struct RwLockReadGuard<'a, M, T>
where
    M: RawMutex,
    T: ?Sized,
{
    lock: &'a RwLock<M, T>,
}

impl<'a, M, T> Drop for RwLockReadGuard<'a, M, T>
where
    M: RawMutex,
    T: ?Sized,
{
    fn drop(&mut self) {
        let mut state = self.lock.state.lock(|s| s.borrow_mut());
        state.readers -= 1;
        if state.readers == 0 {
            state.writer_waker.wake();
        }
    }
}

impl<'a, M, T> Deref for RwLockReadGuard<'a, M, T>
where
    M: RawMutex,
    T: ?Sized,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.lock.inner.borrow()
    }
}

/// Async write lock guard.
///
/// Owning an instance of this type indicates having
/// successfully locked the RwLock for writing, and grants access to the contents.
///
/// Dropping it unlocks the RwLock.
#[must_use = "if unused the RwLock will immediately unlock"]
pub struct RwLockWriteGuard<'a, M, T>
where
    M: RawMutex,
    T: ?Sized,
{
    lock: &'a RwLock<M, T>,
}

impl<'a, M, T> Drop for RwLockWriteGuard<'a, M, T>
where
    M: RawMutex,
    T: ?Sized,
{
    fn drop(&mut self) {
        let mut state = self.lock.state.lock(|s| s.borrow_mut());
        state.writer = false;
        state.reader_wakers.wake();
        state.writer_waker.wake();
    }
}

impl<'a, M, T> Deref for RwLockWriteGuard<'a, M, T>
where
    M: RawMutex,
    T: ?Sized,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.lock.inner.borrow()
    }
}

impl<'a, M, T> DerefMut for RwLockWriteGuard<'a, M, T>
where
    M: RawMutex,
    T: ?Sized,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.lock.inner.borrow_mut()
    }
}

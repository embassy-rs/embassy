use core::cell::UnsafeCell;
use core::future::poll_fn;
use core::ops::{Deref, DerefMut};
use core::task::Poll;

use crate::blocking_mutex::Mutex as BlockingMutex;
use crate::waitqueue::WakerRegistration;
use crate::raw_rwlock::RawRwLock;

pub struct RwLock<M, T>
where
    M: RawRwLock,
    T: ?Sized,
{
    state: BlockingMutex<M, RwLockState>,
    inner: UnsafeCell<T>,
}

unsafe impl<M: RawRwLock + Send, T: ?Sized + Send> Send for RwLock<M, T> {}
unsafe impl<M: RawRwLock + Sync, T: ?Sized + Send> Sync for RwLock<M, T> {}

impl<M, T> RwLock<M, T>
where
    M: RawRwLock,
{
    pub const fn new(value: T) -> Self {
        Self {
            inner: UnsafeCell::new(value),
            state: BlockingMutex::new(RwLockState {
                locked: LockedState::Unlocked,
                writer_pending: 0,
                readers_pending: 0,
                waker: WakerRegistration::new(),
            }),
        }
    }
}

impl<M, T> RwLock<M, T>
where
    M: RawRwLock,
    T: ?Sized,
{
    pub fn read(&self) -> impl Future<Output = RwLockReadGuard<'_, M, T>> {
        poll_fn(|cx| {
            let ready = self.state.lock(|s| {
                let mut s = s.borrow_mut();
                match s.locked {
                    LockedState::Unlocked => {
                        s.locked = LockedState::ReadLocked(1);
                        true
                    }
                    LockedState::ReadLocked(ref mut count) => {
                        *count += 1;
                        true
                    }
                    LockedState::WriteLocked => {
                        s.readers_pending += 1;
                        s.waker.register(cx.waker());
                        false
                    }
                }
            });

            if ready {
                Poll::Ready(RwLockReadGuard { lock: self })
            } else {
                Poll::Pending
            }
        })
    }

    pub fn write(&self) -> impl Future<Output = RwLockWriteGuard<'_, M, T>> {
        poll_fn(|cx| {
            let ready = self.state.lock(|s| {
                let mut s = s.borrow_mut();
                match s.locked {
                    LockedState::Unlocked => {
                        s.locked = LockedState::WriteLocked;
                        true
                    }
                    _ => {
                        s.writer_pending += 1;
                        s.waker.register(cx.waker());
                        false
                    }
                }
            });

            if ready {
                Poll::Ready(RwLockWriteGuard { lock: self })
            } else {
                Poll::Pending
            }
        })
    }

    pub fn try_read(&self) -> Result<RwLockReadGuard<'_, M, T>, TryLockError> {
        self.state.lock(|s| {
            let mut s = s.borrow_mut();
            match s.locked {
                LockedState::Unlocked => {
                    s.locked = LockedState::ReadLocked(1);
                    Ok(())
                }
                LockedState::ReadLocked(ref mut count) => {
                    *count += 1;
                    Ok(())
                }
                LockedState::WriteLocked => Err(TryLockError),
            }
        })?;

        Ok(RwLockReadGuard { lock: self })
    }

    pub fn try_write(&self) -> Result<RwLockWriteGuard<'_, M, T>, TryLockError> {
        self.state.lock(|s| {
            let mut s = s.borrow_mut();
            match s.locked {
                LockedState::Unlocked => {
                    s.locked = LockedState::WriteLocked;
                    Ok(())
                }
                _ => Err(TryLockError),
            }
        })?;

        Ok(RwLockWriteGuard { lock: self })
    }

    pub fn into_inner(self) -> T
    where
        T: Sized,
    {
        self.inner.into_inner()
    }

    pub fn get_mut(&mut self) -> &mut T {
        self.inner.get_mut()
    }
}

impl<M: RawRwLock, T> From<T> for RwLock<M, T> {
    fn from(from: T) -> Self {
        Self::new(from)
    }
}

impl<M, T> Default for RwLock<M, T>
where
    M: RawRwLock,
    T: Default,
{
    fn default() -> Self {
        Self::new(Default::default())
    }
}

pub struct RwLockReadGuard<'a, M, T>
where
    M: RawRwLock,
    T: ?Sized,
{
    lock: &'a RwLock<M, T>,
}

impl<'a, M, T> Deref for RwLockReadGuard<'a, M, T>
where
    M: RawRwLock,
    T: ?Sized,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.inner.get() }
    }
}

impl<'a, M, T> Drop for RwLockReadGuard<'a, M, T>
where
    M: RawRwLock,
    T: ?Sized,
{
    fn drop(&mut self) {
        self.lock.state.lock(|s| {
            let mut s = s.borrow_mut();
            match s.locked {
                LockedState::ReadLocked(ref mut count) => {
                    *count -= 1;
                    if *count == 0 {
                        s.locked = LockedState::Unlocked;
                        s.waker.wake();
                    }
                }
                _ => unreachable!(),
            }
        });
    }
}

pub struct RwLockWriteGuard<'a, M, T>
where
    M: RawRwLock,
    T: ?Sized,
{
    lock: &'a RwLock<M, T>,
}

impl<'a, M, T> Deref for RwLockWriteGuard<'a, M, T>
where
    M: RawRwLock,
    T: ?Sized,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.inner.get() }
    }
}

impl<'a, M, T> DerefMut for RwLockWriteGuard<'a, M, T>
where
    M: RawRwLock,
    T: ?Sized,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.lock.inner.get() }
    }
}

impl<'a, M, T> Drop for RwLockWriteGuard<'a, M, T>
where
    M: RawRwLock,
    T: ?Sized,
{
    fn drop(&mut self) {
        self.lock.state.lock(|s| {
            let mut s = s.borrow_mut();
            s.locked = LockedState::Unlocked;
            s.waker.wake();
        });
    }
}

struct RwLockState {
    locked: LockedState,
    writer_pending: usize,
    readers_pending: usize,
    waker: WakerRegistration,
}

enum LockedState {
    Unlocked,
    ReadLocked(usize),
    WriteLocked,
}

pub struct TryLockError;

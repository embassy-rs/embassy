//! A collection of objects that may be shared between tasks.
//!
//! Multiple tasks may share a reference to the pool and acquire resources when required.
//! Acquired resources may be kept or moved between tasks before they are released.
use core::cell::RefCell;
use core::future::poll_fn;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
use core::task::Poll;

use heapless::Vec;

use crate::blocking_mutex::Mutex;
use crate::blocking_mutex::raw::RawMutex;
use crate::waitqueue::WakerRegistration;

/// Resource pool
pub struct ResourcePool<'a, M: RawMutex, T, const N: usize> {
    buf: BufferPtr<T>,
    phantom: PhantomData<&'a mut T>,
    state: Mutex<M, RefCell<State<N>>>,
}

impl<'a, M: RawMutex, T, const N: usize> ResourcePool<'a, M, T, N> {
    /// Crate a new resource pool, taking an array of resources which will be managed.
    pub fn new(buf: &'a mut [T]) -> Self {
        let mut available = Vec::new();
        available.extend(0..buf.len());
        Self {
            buf: BufferPtr(buf.as_mut_ptr()),
            phantom: PhantomData,
            state: Mutex::new(RefCell::new(State {
                available,
                waker: WakerRegistration::new(),
            })),
        }
    }

    /// Attempt to acquire one instance of the resource.
    ///
    /// If no instance is available, return None immediately.
    pub fn try_take<'guard>(&'guard self) -> Option<ResourceGuard<'guard, 'a, M, T, N>> {
        self.state.lock(|state| {
            let state = &mut *state.borrow_mut();
            let index = state.available.pop()?;
            Some(ResourceGuard { store: self, index })
        })
    }

    /// Acquire one instance of the resource.
    ///
    /// If no instance is available, wait for an instance to be returned to the pool.
    pub fn take<'guard>(&'guard self) -> impl Future<Output = ResourceGuard<'guard, 'a, M, T, N>> {
        poll_fn(|cx| {
            self.state.lock(|state| {
                let state = &mut *state.borrow_mut();
                let Some(index) = state.available.pop() else {
                    state.waker.register(cx.waker());
                    return Poll::Pending;
                };
                Poll::Ready(ResourceGuard { store: self, index })
            })
        })
    }
}

#[repr(transparent)]
#[derive(Debug)]
struct BufferPtr<T>(*mut T);

unsafe impl<T> Send for BufferPtr<T> {}
unsafe impl<T> Sync for BufferPtr<T> {}

struct State<const N: usize> {
    available: Vec<usize, N>,
    waker: WakerRegistration,
}

/// Resource guard
///
/// Owning this guard provides mutable access to the underlying resource.
///
/// Dropping the guard returns the resource back to the pool.
pub struct ResourceGuard<'guard, 'buffer, M: RawMutex, T, const N: usize> {
    store: &'guard ResourcePool<'buffer, M, T, N>,
    index: usize,
}

impl<'guard, 'buffer, M: RawMutex, T, const N: usize> Drop for ResourceGuard<'guard, 'buffer, M, T, N> {
    fn drop(&mut self) {
        self.store.state.lock(|state| {
            let state = &mut *state.borrow_mut();
            state.available.push(self.index).unwrap();
            state.waker.wake();
        });
    }
}

impl<'guard, 'buffer, M: RawMutex, T, const N: usize> Deref for ResourceGuard<'guard, 'buffer, M, T, N> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.store.buf.0.add(self.index) }
        // unsafe { &*self.element.0 }
    }
}

impl<'guard, 'buffer, M: RawMutex, T, const N: usize> DerefMut for ResourceGuard<'guard, 'buffer, M, T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.store.buf.0.add(self.index) }
    }
}

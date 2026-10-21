//! A collection of objects that can be shared between tasks.
//!
//! Multiple tasks may share a reference to the pool and acquire resources when required.
//! Acquired resources may be kept or moved between tasks before they are released.
//!
//! Since only references are shared, this can be used to implement a zero-copy message passing system.
use core::cell::RefCell;
use core::future::poll_fn;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
use core::task::Poll;

use heapless::Vec;

use crate::blocking_mutex::Mutex;
use crate::blocking_mutex::raw::RawMutex;
use crate::waitqueue::WakerRegistration;

/// A collection of objects that can be shared between tasks.
///
/// Multiple tasks may share a reference to the pool and acquire resources when required.
/// Acquired resources may be kept or moved between tasks before they are released.
///
/// When the reference to a resource is dropped, the resource is returned to the pool and may be acquired by another task.
pub struct ResourcePool<'a, M: RawMutex, T, const N: usize> {
    buf: BufferPtr<T>,
    phantom: PhantomData<&'a mut T>,
    state: Mutex<M, RefCell<State<N>>>,
}

impl<'a, M: RawMutex, T, const N: usize> ResourcePool<'a, M, T, N> {
    /// Crate a new [`ResourcePool`], taking an array of resources which will be managed.
    ///
    /// The function will panic if the length of the array is larger than `N`.
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
struct BufferPtr<T: ?Sized>(*mut T);

unsafe impl<T: ?Sized> Send for BufferPtr<T> {}
unsafe impl<T: ?Sized> Sync for BufferPtr<T> {}

struct State<const N: usize> {
    available: Vec<usize, N>,
    waker: WakerRegistration,
}

/// Resource guard
///
/// Owning this guard provides mutable access to an instance of the underlying resource.
/// Dropping the guard returns the resource back to the [`ResourcePool`].
/// The guard can be mapped to a different type, referencing the original resource, using [`ResourceGuard::map`].
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
    }
}

impl<'guard, 'buffer, M: RawMutex, T, const N: usize> DerefMut for ResourceGuard<'guard, 'buffer, M, T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.store.buf.0.add(self.index) }
    }
}

impl<'guard, 'buffer, M: RawMutex, T, const N: usize> ResourceGuard<'guard, 'buffer, M, T, N> {
    /// Maps the managed resource to another type, referencing the original value. Does not take "self" to avoid shadowing any functions of the wrapped type.
    pub fn map<U: ?Sized>(
        orig: Self,
        fun: impl FnOnce(&mut T) -> &mut U,
    ) -> MappedResourceGuard<'guard, 'buffer, M, T, U, N> {
        let store = orig.store;
        let index = orig.index;
        let value = fun(unsafe { &mut *store.buf.0.add(index) });
        // Don't run the `drop` method for MutexGuard. The ownership of the underlying
        // locked state is being moved to the returned MappedMutexGuard.
        core::mem::forget(orig);
        MappedResourceGuard {
            store,
            value: BufferPtr(value),
            index,
        }
    }
}

/// Mapped resource guard
///
/// Owning this guard provides mutable access to the underlying resource.
/// This guard is created by mapping a [`ResourceGuard`] to a different type, referencing the original resource.
/// Dropping the guard returns the resource back to the [`ResourcePool`].
pub struct MappedResourceGuard<'guard, 'buffer, M: RawMutex, T, U: ?Sized, const N: usize> {
    store: &'guard ResourcePool<'buffer, M, T, N>,
    index: usize,
    value: BufferPtr<U>,
}

impl<'guard, 'buffer, M: RawMutex, T, U: ?Sized, const N: usize> Drop
    for MappedResourceGuard<'guard, 'buffer, M, T, U, N>
{
    fn drop(&mut self) {
        self.store.state.lock(|state| {
            let state = &mut *state.borrow_mut();
            state.available.push(self.index).unwrap();
            state.waker.wake();
        });
    }
}

impl<'guard, 'buffer, M: RawMutex, T, U: ?Sized, const N: usize> Deref
    for MappedResourceGuard<'guard, 'buffer, M, T, U, N>
{
    type Target = U;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.value.0 }
    }
}

impl<'guard, 'buffer, M: RawMutex, T, U: ?Sized, const N: usize> DerefMut
    for MappedResourceGuard<'guard, 'buffer, M, T, U, N>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.value.0 }
    }
}

impl<'guard, 'buffer, M: RawMutex, T, U: ?Sized, const N: usize> MappedResourceGuard<'guard, 'buffer, M, T, U, N> {
    /// Maps the managed resource to another type, referencing the original value. Does not take "self" to avoid shadowing any functions of the wrapped type.
    pub fn map<V: ?Sized>(
        orig: Self,
        fun: impl FnOnce(&mut U) -> &mut V,
    ) -> MappedResourceGuard<'guard, 'buffer, M, T, V, N> {
        let store = orig.store;
        let index = orig.index;
        let value = fun(unsafe { &mut *orig.value.0 });
        // Don't run the `drop` method for MutexGuard. The ownership of the underlying
        // locked state is being moved to the returned MappedMutexGuard.
        core::mem::forget(orig);
        MappedResourceGuard {
            store,
            value: BufferPtr(value),
            index,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blocking_mutex::raw::NoopRawMutex;

    #[test]
    fn resources_returned_to_pool_when_dropped() {
        let mut resources = [0, 1];
        let pool = ResourcePool::<NoopRawMutex, _, 2>::new(&mut resources);

        {
            let a = pool.try_take().expect("Failed to take resource");
            let b = pool.try_take().expect("Failed to take resource");
            let c = pool.try_take();
            assert!(c.is_none(), "Expected no more resources to be available");
        }

        let d = pool.try_take().expect("Resource should have been returned to the pool");
    }

    #[test]
    fn mapped_resources_not_returned_to_pool() {
        let mut resources = [[0]];
        let pool = ResourcePool::<NoopRawMutex, _, 1>::new(&mut resources);

        {
            let a = pool.try_take().expect("Failed to take resource");
            let mapped = ResourceGuard::map(a, |r| &mut r[0]);

            let b = pool.try_take();
            assert!(b.is_none(), "Expected no more resources to be available");
        }

        let c = pool.try_take().expect("Resource should have been returned to the pool");
    }
}

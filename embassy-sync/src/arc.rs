//! Thread-safe reference-counting pointer.
//!
//! This module provides [`Arc`], a thread-safe reference-counting pointer. 'Arc' stands for
//! 'Atomically Reference Counted'.
//!
//! This implementation uses CriticalSection for thread safety rather than atomic operations.

use core::cmp::Ordering as CmpOrdering;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::ops::Deref;
use core::ptr::NonNull;

extern crate alloc;
use alloc::boxed::Box;

use crate::blocking_mutex::raw::RawMutex;
use crate::blocking_mutex::Mutex;

/// A thread-safe reference-counting pointer. 'Arc' stands for 'Atomically Reference Counted'.
///
/// This implementation uses embassy-sync blocking Mutex for thread safety rather than atomic operations.
pub struct Arc<T: ?Sized, M: RawMutex> {
    ptr: NonNull<ArcInner<T, M>>,
}

struct ArcInner<T: ?Sized, M: RawMutex> {
    count: Mutex<M, usize>,
    data: T,
}

unsafe impl<T: ?Sized + Sync + Send, M: RawMutex> Send for Arc<T, M> {}
unsafe impl<T: ?Sized + Sync + Send, M: RawMutex> Sync for Arc<T, M> {}

impl<T, M: RawMutex> Arc<T, M> {
    /// Constructs a new `Arc<T>`.
    #[inline]
    pub fn new(data: T) -> Arc<T, M> {
        let inner = Box::new(ArcInner {
            count: Mutex::new(1),
            data,
        });

        Arc {
            ptr: NonNull::new(Box::into_raw(inner)).unwrap(),
        }
    }

    /// Returns a mutable reference to the inner value if this is the only `Arc` pointer to the value.
    ///
    /// Returns `None` otherwise.
    #[inline]
    pub fn get_mut(this: &mut Self) -> Option<&mut T> {
        let inner = unsafe { this.ptr.as_ref() };
        inner.count.lock(|count| {
            if *count == 1 {
                // More than one Arc, so we can't return a mutable reference
                Some(unsafe { &mut this.ptr.as_mut().data })
            } else {
                None
            }
        })
    }

    /// Unwraps this `Arc` pointer, returning the inner value if this is the only `Arc` pointer to the value.
    ///
    /// Returns `Err(this)` otherwise.
    #[inline]
    pub fn try_unwrap(this: Self) -> Result<T, Self> {
        if let Some(inner) = Arc::get_mut(&mut Arc::clone(&this)) {
            unsafe {
                let result = core::ptr::read(inner);
                core::mem::forget(this); // Don't run the destructor
                Ok(result)
            }
        } else {
            Err(this)
        }
    }
}

impl<T: ?Sized, M: RawMutex> Arc<T, M> {
    /// Gets the number of strong references to this value.
    #[inline]
    pub fn strong_count(this: &Self) -> usize {
        let inner = unsafe { this.ptr.as_ref() };
        inner.count.lock(|count| *count)
    }

    /// Creates a new `Arc` pointer to the same allocation.
    #[inline]
    pub fn clone_from(this: &mut Self, source: &Self) {
        if this.ptr.as_ptr() as *const () != source.ptr.as_ptr() as *const () {
            // Drop the original reference
            *this = source.clone();
        }
    }
}

impl<T: ?Sized, M: RawMutex> Clone for Arc<T, M> {
    #[inline]
    fn clone(&self) -> Self {
        let inner = unsafe { self.ptr.as_ref() };
        unsafe {
            inner.count.lock_mut(|count| *count += 1);
        }

        Self { ptr: self.ptr }
    }
}

impl<T: ?Sized, M: RawMutex> Deref for Arc<T, M> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        let inner = unsafe { self.ptr.as_ref() };
        &inner.data
    }
}

impl<T: ?Sized, M: RawMutex> Drop for Arc<T, M> {
    #[inline]
    fn drop(&mut self) {
        let inner = unsafe { self.ptr.as_ref() };
        unsafe {
            inner.count.lock_mut(|count| *count -= 1);
        }

        inner.count.lock(|count| unsafe {
            if *count == 0 {
                // Drop the inner data
                drop(Box::from_raw(self.ptr.as_ptr()));
            }
        });
    }
}

impl<T: ?Sized + PartialEq, M: RawMutex> PartialEq for Arc<T, M> {
    #[inline]
    fn eq(&self, other: &Arc<T, M>) -> bool {
        **self == **other
    }
}

impl<T: ?Sized + Eq, M: RawMutex> Eq for Arc<T, M> {}

impl<T: ?Sized + PartialOrd, M: RawMutex> PartialOrd for Arc<T, M> {
    #[inline]
    fn partial_cmp(&self, other: &Arc<T, M>) -> Option<CmpOrdering> {
        (**self).partial_cmp(&**other)
    }
}

impl<T: ?Sized + Ord, M: RawMutex> Ord for Arc<T, M> {
    #[inline]
    fn cmp(&self, other: &Arc<T, M>) -> CmpOrdering {
        (**self).cmp(&**other)
    }
}

impl<T: ?Sized + Hash, M: RawMutex> Hash for Arc<T, M> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (**self).hash(state)
    }
}

impl<T: ?Sized + fmt::Display, M: RawMutex> fmt::Display for Arc<T, M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

impl<T: ?Sized + fmt::Debug, M: RawMutex> fmt::Debug for Arc<T, M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blocking_mutex::raw::CriticalSectionRawMutex;

    #[test]
    fn test_arc_basic() {
        let a = Arc::<_, CriticalSectionRawMutex>::new(5);
        assert_eq!(*a, 5);

        let b = Arc::clone(&a);
        assert_eq!(*b, 5);

        assert_eq!(Arc::strong_count(&a), 2);
        assert_eq!(Arc::strong_count(&b), 2);

        drop(a);
        assert_eq!(Arc::strong_count(&b), 1);
    }

    #[test]
    fn test_arc_get_mut() {
        let mut a = Arc::<_, CriticalSectionRawMutex>::new(5);
        assert_eq!(Arc::strong_count(&a), 1);

        *Arc::get_mut(&mut a).unwrap() = 10;
        assert_eq!(*a, 10);

        let b = Arc::clone(&a);
        assert!(Arc::get_mut(&mut a).is_none());

        drop(b);
        assert!(Arc::get_mut(&mut a).is_some());
    }
}

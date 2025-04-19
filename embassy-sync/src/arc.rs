//! Thread-safe reference-counting pointer.
//!
//! This module provides [`Arc`], a thread-safe reference-counting pointer, and [`Weak`], a non-owning
//! reference to the allocation of an [`Arc`].
//! 'Arc' stands for 'Atomically Reference Counted'.
//!
//! This implementation uses Mutex for thread safety rather than atomic operations.

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

/// A non-owning reference to the allocation of an [`Arc`].
pub struct Weak<T: ?Sized, M: RawMutex> {
    ptr: NonNull<ArcInner<T, M>>,
}

struct ArcInner<T: ?Sized, M: RawMutex> {
    strong: Mutex<M, usize>,
    weak: Mutex<M, usize>,
    data: T,
}

unsafe impl<T: ?Sized + Sync + Send, M: RawMutex> Send for Arc<T, M> {}
unsafe impl<T: ?Sized + Sync + Send, M: RawMutex> Sync for Arc<T, M> {}

unsafe impl<T: ?Sized + Sync + Send, M: RawMutex> Send for Weak<T, M> {}
unsafe impl<T: ?Sized + Sync + Send, M: RawMutex> Sync for Weak<T, M> {}

impl<T, M: RawMutex> Arc<T, M> {
    /// Constructs a new `Arc<T>`.
    #[inline]
    pub fn new(data: T) -> Arc<T, M> {
        let inner = Box::new(ArcInner {
            strong: Mutex::new(1),
            weak: Mutex::new(1), // Initialize weak count to 1
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
        inner.strong.lock(|count| {
            if *count == 1 {
                // Only one strong reference, so we can provide a mutable reference
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

    /// Consumes this `Arc` pointer, returning the raw pointer to the inner value.
    ///
    /// This is the opposite of [`Arc::from_raw`].
    pub fn into_raw(this: Self) -> *const T {
        let inner = unsafe { this.ptr.as_ref() };
        let ptr = &inner.data as *const T;
        core::mem::forget(this); // Don't run the destructor
        ptr
    }

    /// Constructs an `Arc` pointer from a raw pointer to the inner value.
    ///    
    /// This is the opposite of [`Arc::into_raw`].
    pub unsafe fn from_raw(ptr: *const T) -> Self {
        let offset = core::mem::offset_of!(ArcInner<T, M>, data);
        let inner = (ptr as *const u8).sub(offset) as *mut ArcInner<T, M>;
        let inner = NonNull::new(inner).expect("Null pointer passed to Arc::from_raw");
        Arc { ptr: inner }
    }

    /// Returns a raw pointer to the inner value.
    pub fn as_ptr(this: &Self) -> *const T {
        let inner = unsafe { this.ptr.as_ref() };
        &inner.data as *const T
    }

    /// Creates a new [`Weak`] pointer to this allocation.
    pub fn downgrade(this: &Self) -> Weak<T, M> {
        unsafe {
            let inner = this.ptr.as_ref();
            inner.weak.lock_mut(|count| *count += 1);
        };

        Weak { ptr: this.ptr }
    }

    /// Decrements the strong reference count of this `Arc` pointer.
    #[inline]
    pub unsafe fn decrement_strong_count(ptr: *const T) {
        let offset = core::mem::offset_of!(ArcInner<T, M>, data);
        let inner_ptr = (ptr as *const u8).sub(offset) as *mut ArcInner<T, M>;
        let inner = unsafe { &*inner_ptr };

        let should_drop = inner.strong.lock_mut(|count| {
            *count -= 1;
            *count == 0
        });

        if should_drop {
            // Drop the inner data
            unsafe {
                core::ptr::drop_in_place(&mut (*inner_ptr).data);
            }

            // Check if we should deallocate the memory
            let should_dealloc = inner.weak.lock_mut(|count| {
                *count -= 1;
                *count == 0
            });

            if should_dealloc {
                unsafe {
                    drop(Box::from_raw(inner_ptr));
                }
            }
        }
    }
}

impl<T: ?Sized, M: RawMutex> Arc<T, M> {
    /// Gets the number of strong references to this value.
    #[inline]
    pub fn strong_count(this: &Self) -> usize {
        let inner = unsafe { this.ptr.as_ref() };
        inner.strong.lock(|count| *count)
    }

    /// Gets the number of weak references to this value.
    #[inline]
    pub fn weak_count(this: &Self) -> usize {
        let inner = unsafe { this.ptr.as_ref() };
        inner.weak.lock(|count| {
            // Subtract 1 to not count the implicit weak reference held by all Arc instances
            if *count > 0 {
                *count - 1
            } else {
                0
            }
        })
    }

    /// Creates a new `Arc` pointer to the same allocation.
    #[inline]
    pub fn clone_from(this: &mut Self, source: &Self) {
        if this.ptr.as_ptr() as *const () != source.ptr.as_ptr() as *const () {
            // Drop the original reference
            *this = source.clone();
        }
    }

    /// Increments the strong reference count of this `Arc` pointer.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it directly manipulates the reference count without
    /// proper synchronization. It is the caller's responsibility to ensure that this function
    /// is called in a thread-safe manner and that the reference count is not manipulated
    /// concurrently from multiple threads.
    #[inline]
    pub unsafe fn increment_strong_count(this: &Self) {
        let inner = unsafe { this.ptr.as_ref() };
        inner.strong.lock_mut(|count| *count += 1);
    }
}

impl<T: ?Sized, M: RawMutex> Clone for Arc<T, M> {
    #[inline]
    fn clone(&self) -> Self {
        let inner = unsafe { self.ptr.as_ref() };
        unsafe {
            inner.strong.lock_mut(|count| *count += 1);
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
        // SAFETY: the pointer is always valid as we're dropping a valid Arc
        unsafe {
            let inner = self.ptr.as_ref();

            // Decrement the strong count and check if we should drop the inner data
            let should_drop = inner.strong.lock_mut(|count| {
                *count -= 1;
                *count == 0
            });

            // SAFETY: We know this is the last strong reference, so it's safe to drop the data
            if should_drop {
                // Drop the inner data first
                core::ptr::drop_in_place(&mut self.ptr.as_mut().data);

                // Now check if we should deallocate the memory
                let should_dealloc = inner.weak.lock_mut(|count| {
                    *count -= 1;
                    *count == 0
                });

                // SAFETY: Both strong and weak counts are zero, so we can deallocate
                if should_dealloc {
                    // Convert to Box and drop to properly deallocate
                    let _ = Box::from_raw(self.ptr.as_ptr());
                }
            }
        };
    }
}

impl<T: ?Sized, M: RawMutex> Weak<T, M> {
    /// Attempts to upgrade this `Weak` pointer to an [`Arc`].
    ///
    /// Returns `None` if the inner value has been dropped.
    pub fn upgrade(&self) -> Option<Arc<T, M>> {
        unsafe {
            let inner = self.ptr.as_ref(); // Try to increment the strong count
            inner.strong.lock_mut(|count| {
                if *count == 0 {
                    None
                } else {
                    *count += 1;
                    Some(Arc { ptr: self.ptr })
                }
            })
        }
    }

    /// Gets the number of strong references this `Weak` pointer could upgrade to.
    ///
    /// Returns 0 if the inner value has been dropped.
    pub fn strong_count(&self) -> usize {
        let inner = unsafe { self.ptr.as_ref() };
        inner.strong.lock(|count| *count)
    }

    /// Gets the number of weak references to this allocation.
    ///
    /// This includes the implicit weak reference held by all `Arc` instances.
    pub fn weak_count(&self) -> usize {
        let inner = unsafe { self.ptr.as_ref() };
        inner.weak.lock(|count| *count)
    }
}

impl<T: ?Sized, M: RawMutex> Clone for Weak<T, M> {
    #[inline]
    fn clone(&self) -> Self {
        unsafe {
            let inner = self.ptr.as_ref();
            inner.weak.lock_mut(|count| *count += 1);
        };

        Self { ptr: self.ptr }
    }
}

impl<T: ?Sized, M: RawMutex> Drop for Weak<T, M> {
    #[inline]
    fn drop(&mut self) {
        // SAFETY: the pointer is always valid as we're dropping a valid Weak
        unsafe {
            let inner = self.ptr.as_ref();

            // Decrement the weak count and check if we should deallocate
            let should_dealloc = inner.weak.lock_mut(|count| {
                *count -= 1;
                *count == 0
            });

            // SAFETY: There are no more references, either strong or weak, so we can deallocate
            if should_dealloc {
                // Convert to Box and drop to properly deallocate
                let _ = Box::from_raw(self.ptr.as_ptr());
            }
        };
    }
}

impl<T: ?Sized + fmt::Debug, M: RawMutex> fmt::Debug for Weak<T, M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.upgrade() {
            Some(arc) => f.debug_tuple("Weak").field(&arc).finish(),
            None => f.debug_tuple("Weak").field(&"(dropped)").finish(),
        }
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

    #[test]
    fn test_weak() {
        let a = Arc::<_, CriticalSectionRawMutex>::new(5);
        let weak = Arc::downgrade(&a);

        assert_eq!(Arc::strong_count(&a), 1);
        assert_eq!(Arc::weak_count(&a), 1);

        // Upgrade works while strong references exist
        let upgraded = weak.upgrade().unwrap();
        assert_eq!(*upgraded, 5);
        assert_eq!(Arc::strong_count(&a), 2);

        // Drop all strong references
        drop(a);
        drop(upgraded);

        // Weak can't be upgraded anymore
        assert!(weak.upgrade().is_none());
    }
}

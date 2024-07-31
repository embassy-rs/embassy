//! Synchronization primitive for initializing a value once, allowing others to get a reference to the value.

use core::cell::UnsafeCell;
use core::mem::ManuallyDrop;
use core::sync::atomic::{AtomicBool, Ordering};

/// The `LazyLock` is a synchronization primitive that allows for
/// initializing a value once, and allowing others to obtain a
/// reference to the value. This is useful for lazy initialization of
/// a static value.
///
/// # Example
/// ```
/// use futures_executor::block_on;
/// use embassy_sync::lazy_lock::LazyLock;
///
/// // Define a static value that will be lazily initialized
/// // at runtime at the first access.
/// static VALUE: LazyLock<u32> = LazyLock::new(|| 20);
///
/// let reference = VALUE.get();
/// assert_eq!(reference, &20);
/// ```
pub struct LazyLock<T, F = fn() -> T> {
    init: AtomicBool,
    data: UnsafeCell<Data<T, F>>,
}

union Data<T, F> {
    value: ManuallyDrop<T>,
    f: ManuallyDrop<F>,
}

unsafe impl<T, F> Sync for LazyLock<T, F> {}

impl<T, F: FnOnce() -> T> LazyLock<T, F> {
    /// Create a new uninitialized `StaticLock`.
    pub const fn new(init_fn: F) -> Self {
        Self {
            init: AtomicBool::new(false),
            data: UnsafeCell::new(Data {
                f: ManuallyDrop::new(init_fn),
            }),
        }
    }

    /// Get a reference to the underlying value, initializing it if it
    /// has not been done already.
    #[inline]
    pub fn get(&self) -> &T {
        self.ensure_init_fast();
        unsafe { &(*self.data.get()).value }
    }

    /// Consume the `LazyLock`, returning the underlying value. The
    /// initialization function will be called if it has not been
    /// already.
    #[inline]
    pub fn into_inner(self) -> T {
        self.ensure_init_fast();
        let this = ManuallyDrop::new(self);
        let data = unsafe { core::ptr::read(&this.data) }.into_inner();

        ManuallyDrop::into_inner(unsafe { data.value })
    }

    /// Initialize the `LazyLock` if it has not been initialized yet.
    /// This function is a fast track to [`Self::ensure_init`]
    /// which does not require a critical section in most cases when
    /// the value has been initialized already.
    /// When this function returns, `self.data` is guaranteed to be
    /// initialized and visible on the current core.
    #[inline]
    fn ensure_init_fast(&self) {
        if !self.init.load(Ordering::Acquire) {
            self.ensure_init();
        }
    }

    /// Initialize the `LazyLock` if it has not been initialized yet.
    /// When this function returns, `self.data` is guaranteed to be
    /// initialized and visible on the current core.
    fn ensure_init(&self) {
        critical_section::with(|_| {
            if !self.init.load(Ordering::Acquire) {
                let data = unsafe { &mut *self.data.get() };
                let f = unsafe { ManuallyDrop::take(&mut data.f) };
                let value = f();
                data.value = ManuallyDrop::new(value);

                self.init.store(true, Ordering::Release);
            }
        });
    }
}

impl<T, F> Drop for LazyLock<T, F> {
    fn drop(&mut self) {
        if self.init.load(Ordering::Acquire) {
            unsafe { ManuallyDrop::drop(&mut self.data.get_mut().value) };
        } else {
            unsafe { ManuallyDrop::drop(&mut self.data.get_mut().f) };
        }
    }
}

#[cfg(test)]
mod tests {
    use core::sync::atomic::{AtomicU32, Ordering};

    use super::*;

    #[test]
    fn test_lazy_lock() {
        static VALUE: LazyLock<u32> = LazyLock::new(|| 20);
        let reference = VALUE.get();
        assert_eq!(reference, &20);
    }
    #[test]
    fn test_lazy_lock_into_inner() {
        let lazy: LazyLock<u32> = LazyLock::new(|| 20);
        let value = lazy.into_inner();
        assert_eq!(value, 20);
    }

    static DROP_CHECKER: AtomicU32 = AtomicU32::new(0);
    struct DropCheck;

    impl Drop for DropCheck {
        fn drop(&mut self) {
            DROP_CHECKER.fetch_add(1, Ordering::Acquire);
        }
    }

    #[test]
    fn test_lazy_drop() {
        let lazy: LazyLock<DropCheck> = LazyLock::new(|| DropCheck);
        assert_eq!(DROP_CHECKER.load(Ordering::Acquire), 0);
        lazy.get();
        drop(lazy);
        assert_eq!(DROP_CHECKER.load(Ordering::Acquire), 1);

        let dropper = DropCheck;
        let lazy_fn: LazyLock<u32, _> = LazyLock::new(move || {
            let _a = dropper;
            20
        });
        assert_eq!(DROP_CHECKER.load(Ordering::Acquire), 1);
        drop(lazy_fn);
        assert_eq!(DROP_CHECKER.load(Ordering::Acquire), 2);
    }
}

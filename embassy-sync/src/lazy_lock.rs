//! Synchronization primitive for initializing a value once, allowing others to get a reference to the value.

use core::cell::Cell;
use core::mem::MaybeUninit;
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
    init_fn: Cell<Option<F>>,
    data: Cell<MaybeUninit<T>>,
}

unsafe impl<T, F> Sync for LazyLock<T, F> {}

impl<T, F: FnOnce() -> T> LazyLock<T, F> {
    /// Create a new uninitialized `StaticLock`.
    pub const fn new(init_fn: F) -> Self {
        Self {
            init: AtomicBool::new(false),
            init_fn: Cell::new(Some(init_fn)),
            data: Cell::new(MaybeUninit::zeroed()),
        }
    }

    /// Get a reference to the underlying value, initializing it if it
    /// has not been done already.
    #[inline]
    pub fn get(&self) -> &T {
        self.ensure_init_fast();
        unsafe { (*self.data.as_ptr()).assume_init_ref() }
    }

    /// Consume the `LazyLock`, returning the underlying value. The
    /// initialization function will be called if it has not been
    /// already.
    #[inline]
    pub fn into_inner(self) -> T {
        self.ensure_init_fast();
        unsafe { self.data.into_inner().assume_init() }
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
                let init_fn = self.init_fn.take().unwrap();
                self.data.set(MaybeUninit::new(init_fn()));
                self.init.store(true, Ordering::Release);
            }
        });
    }
}

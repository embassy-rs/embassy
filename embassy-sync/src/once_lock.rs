//! Synchronization primitive for initializing a value once, allowing others to await a reference to the value.

use core::cell::Cell;
use core::future::{poll_fn, Future};
use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicBool, Ordering};
use core::task::Poll;

/// The `OnceLock` is a synchronization primitive that allows for
/// initializing a value once, and allowing others to `.await` a
/// reference to the value. This is useful for lazy initialization of
/// a static value.
///
/// **Note**: this implementation uses a busy loop to poll the value,
/// which is not as efficient as registering a dedicated `Waker`.
/// However, if the usecase for it is to initialize a static variable
/// relatively early in the program life cycle, it should be fine.
///
/// # Example
/// ```
/// use futures_executor::block_on;
/// use embassy_sync::once_lock::OnceLock;
///
/// // Define a static value that will be lazily initialized
/// static VALUE: OnceLock<u32> = OnceLock::new();
///
/// let f = async {
///
/// // Initialize the value
/// let reference = VALUE.get_or_init(|| 20);
/// assert_eq!(reference, &20);
///
/// // Wait for the value to be initialized
/// // and get a static reference it
/// assert_eq!(VALUE.get().await, &20);
///
/// };
/// block_on(f)
/// ```
pub struct OnceLock<T> {
    init: AtomicBool,
    data: Cell<MaybeUninit<T>>,
}

unsafe impl<T> Sync for OnceLock<T> {}

impl<T> OnceLock<T> {
    /// Create a new uninitialized `OnceLock`.
    pub const fn new() -> Self {
        Self {
            init: AtomicBool::new(false),
            data: Cell::new(MaybeUninit::zeroed()),
        }
    }

    /// Get a reference to the underlying value, waiting for it to be set.
    /// If the value is already set, this will return immediately.
    pub fn get(&self) -> impl Future<Output = &T> {
        poll_fn(|cx| match self.try_get() {
            Some(data) => Poll::Ready(data),
            None => {
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        })
    }

    /// Try to get a reference to the underlying value if it exists.
    pub fn try_get(&self) -> Option<&T> {
        if self.init.load(Ordering::Relaxed) {
            Some(unsafe { self.get_ref_unchecked() })
        } else {
            None
        }
    }

    /// Set the underlying value. If the value is already set, this will return an error with the given value.
    pub fn init(&self, value: T) -> Result<(), T> {
        // Critical section is required to ensure that the value is
        // not simultaneously initialized elsewhere at the same time.
        critical_section::with(|_| {
            // If the value is not set, set it and return Ok.
            if !self.init.load(Ordering::Relaxed) {
                self.data.set(MaybeUninit::new(value));
                self.init.store(true, Ordering::Relaxed);
                Ok(())

            // Otherwise return an error with the given value.
            } else {
                Err(value)
            }
        })
    }

    /// Get a reference to the underlying value, initializing it if it does not exist.
    pub fn get_or_init<F>(&self, f: F) -> &T
    where
        F: FnOnce() -> T,
    {
        // Critical section is required to ensure that the value is
        // not simultaneously initialized elsewhere at the same time.
        critical_section::with(|_| {
            // If the value is not set, set it.
            if !self.init.load(Ordering::Relaxed) {
                self.data.set(MaybeUninit::new(f()));
                self.init.store(true, Ordering::Relaxed);
            }
        });

        // Return a reference to the value.
        unsafe { self.get_ref_unchecked() }
    }

    /// Consume the `OnceLock`, returning the underlying value if it was initialized.
    pub fn into_inner(self) -> Option<T> {
        if self.init.load(Ordering::Relaxed) {
            Some(unsafe { self.data.into_inner().assume_init() })
        } else {
            None
        }
    }

    /// Take the underlying value if it was initialized, uninitializing the `OnceLock` in the process.
    pub fn take(&mut self) -> Option<T> {
        // If the value is set, uninitialize the lock and return the value.
        critical_section::with(|_| {
            if self.init.load(Ordering::Relaxed) {
                let val = unsafe { self.data.replace(MaybeUninit::zeroed()).assume_init() };
                self.init.store(false, Ordering::Relaxed);
                Some(val)

            // Otherwise return None.
            } else {
                None
            }
        })
    }

    /// Check if the value has been set.
    pub fn is_set(&self) -> bool {
        self.init.load(Ordering::Relaxed)
    }

    /// Get a reference to the underlying value.
    /// # Safety
    /// Must only be used if a value has been set.
    unsafe fn get_ref_unchecked(&self) -> &T {
        (*self.data.as_ptr()).assume_init_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn once_lock() {
        let lock = OnceLock::new();
        assert_eq!(lock.try_get(), None);
        assert_eq!(lock.is_set(), false);

        let v = 42;
        assert_eq!(lock.init(v), Ok(()));
        assert_eq!(lock.is_set(), true);
        assert_eq!(lock.try_get(), Some(&v));
        assert_eq!(lock.try_get(), Some(&v));

        let v = 43;
        assert_eq!(lock.init(v), Err(v));
        assert_eq!(lock.is_set(), true);
        assert_eq!(lock.try_get(), Some(&42));
    }

    #[test]
    fn once_lock_get_or_init() {
        let lock = OnceLock::new();
        assert_eq!(lock.try_get(), None);
        assert_eq!(lock.is_set(), false);

        let v = lock.get_or_init(|| 42);
        assert_eq!(v, &42);
        assert_eq!(lock.is_set(), true);
        assert_eq!(lock.try_get(), Some(&42));

        let v = lock.get_or_init(|| 43);
        assert_eq!(v, &42);
        assert_eq!(lock.is_set(), true);
        assert_eq!(lock.try_get(), Some(&42));
    }

    #[test]
    fn once_lock_static() {
        static LOCK: OnceLock<i32> = OnceLock::new();

        let v: &'static i32 = LOCK.get_or_init(|| 42);
        assert_eq!(v, &42);

        let v: &'static i32 = LOCK.get_or_init(|| 43);
        assert_eq!(v, &42);
    }

    #[futures_test::test]
    async fn once_lock_async() {
        static LOCK: OnceLock<i32> = OnceLock::new();

        assert!(LOCK.init(42).is_ok());

        let v: &'static i32 = LOCK.get().await;
        assert_eq!(v, &42);
    }

    #[test]
    fn once_lock_into_inner() {
        let lock: OnceLock<i32> = OnceLock::new();

        let v = lock.get_or_init(|| 42);
        assert_eq!(v, &42);

        assert_eq!(lock.into_inner(), Some(42));
    }

    #[test]
    fn once_lock_take_init() {
        let mut lock: OnceLock<i32> = OnceLock::new();

        assert_eq!(lock.get_or_init(|| 42), &42);
        assert_eq!(lock.is_set(), true);

        assert_eq!(lock.take(), Some(42));
        assert_eq!(lock.is_set(), false);

        assert_eq!(lock.get_or_init(|| 43), &43);
        assert_eq!(lock.is_set(), true);
    }
}

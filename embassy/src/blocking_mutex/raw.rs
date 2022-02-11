use core::marker::PhantomData;

pub trait RawMutex {
    const INIT: Self;

    fn lock<R>(&self, f: impl FnOnce() -> R) -> R;
}

pub struct CriticalSectionRawMutex {
    _phantom: PhantomData<()>,
}
unsafe impl Send for CriticalSectionRawMutex {}
unsafe impl Sync for CriticalSectionRawMutex {}

impl CriticalSectionRawMutex {
    pub const fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl RawMutex for CriticalSectionRawMutex {
    const INIT: Self = Self::new();

    fn lock<R>(&self, f: impl FnOnce() -> R) -> R {
        critical_section::with(|_| f())
    }
}

// ================

pub struct NoopRawMutex {
    _phantom: PhantomData<*mut ()>,
}

unsafe impl Send for NoopRawMutex {}

impl NoopRawMutex {
    pub const fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl RawMutex for NoopRawMutex {
    const INIT: Self = Self::new();
    fn lock<R>(&self, f: impl FnOnce() -> R) -> R {
        f()
    }
}

// ================

#[cfg(any(cortex_m, feature = "std"))]
mod thread_mode {
    use super::*;

    pub struct ThreadModeRawMutex {
        _phantom: PhantomData<()>,
    }

    unsafe impl Send for ThreadModeRawMutex {}
    unsafe impl Sync for ThreadModeRawMutex {}

    impl ThreadModeRawMutex {
        pub const fn new() -> Self {
            Self {
                _phantom: PhantomData,
            }
        }
    }

    impl RawMutex for ThreadModeRawMutex {
        const INIT: Self = Self::new();
        fn lock<R>(&self, f: impl FnOnce() -> R) -> R {
            assert!(
                in_thread_mode(),
                "ThreadModeMutex can only be locked from thread mode."
            );

            f()
        }
    }

    impl Drop for ThreadModeRawMutex {
        fn drop(&mut self) {
            // Only allow dropping from thread mode. Dropping calls drop on the inner `T`, so
            // `drop` needs the same guarantees as `lock`. `ThreadModeMutex<T>` is Send even if
            // T isn't, so without this check a user could create a ThreadModeMutex in thread mode,
            // send it to interrupt context and drop it there, which would "send" a T even if T is not Send.
            assert!(
                in_thread_mode(),
                "ThreadModeMutex can only be dropped from thread mode."
            );

            // Drop of the inner `T` happens after this.
        }
    }

    pub(crate) fn in_thread_mode() -> bool {
        #[cfg(feature = "std")]
        return Some("main") == std::thread::current().name();

        #[cfg(not(feature = "std"))]
        return cortex_m::peripheral::SCB::vect_active()
            == cortex_m::peripheral::scb::VectActive::ThreadMode;
    }
}
#[cfg(any(cortex_m, feature = "std"))]
pub use thread_mode::*;

use core::cell::UnsafeCell;
use core::mem::MaybeUninit;

use atomic_polyfill::{AtomicBool, Ordering};

/// Type with static lifetime that may be written to once at runtime.
///
/// This may be used to initialize static objects at runtime, typically in the init routine.
/// This is useful for objects such as Embassy's RTC, which cannot be initialized in a const
/// context.
///
/// Note: IF a global mutable variable is desired, use a CriticalSectionMutex or ThreadModeMutex instead.
///
/// ```
/// use embassy::util::Forever;
/// // Using an integer for the sake of keeping this example self-contained,
/// // see https://github.com/embassy-rs/embassy/wiki/Getting-Started for a more "proper" example.
/// static SOME_INT: Forever<u32> =Forever::new();
///
/// // put returns a mutable pointer to the object stored in the forever, which may then be passed
/// // around.
/// let mut x = SOME_INT.put(42);
/// assert_eq!(*x, 42);
/// ```
pub struct Forever<T> {
    used: AtomicBool,
    t: UnsafeCell<MaybeUninit<T>>,
}

unsafe impl<T> Send for Forever<T> {}
unsafe impl<T> Sync for Forever<T> {}

impl<T> Forever<T> {
    /// Create a new `Forever`.
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            used: AtomicBool::new(false),
            t: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }

    /// Store a value in this `Forever`, returning a mutable reference to it.
    ///
    /// Using this method, the compiler usually constructs `val` in the stack and then moves
    /// it into the `Forever`. If `T` is big, this is likely to cause stack overflows.
    /// Considering using [`Signal::put_with`] instead, which will construct it in-place inside the `Forever`.
    ///
    /// # Panics
    ///
    /// Panics if this `Forever` already has a value stored in it.
    #[inline(always)]
    #[allow(clippy::mut_from_ref)]
    pub fn put(&'static self, val: T) -> &'static mut T {
        self.put_with(|| val)
    }

    /// Store the closure return value in this `Forever`, returning a mutable reference to it.
    ///
    /// The advantage over [`Forever::put`] is that this method allows the closure to construct
    /// the `T` value in-place directly inside the `Forever`, saving stack space.
    ///
    /// # Panics
    ///
    /// Panics if this `Forever` already has a value stored in it.
    #[inline(always)]
    #[allow(clippy::mut_from_ref)]
    pub fn put_with(&'static self, val: impl FnOnce() -> T) -> &'static mut T {
        if self
            .used
            .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
            .is_err()
        {
            panic!("Forever.put() called multiple times");
        }

        unsafe {
            let p = self.t.get();
            let p = (&mut *p).as_mut_ptr();
            p.write(val());
            &mut *p
        }
    }

    /// Unsafely get a mutable reference to the contents of this Forever.
    ///
    /// # Safety
    ///
    /// This is undefined behavior if:
    ///
    /// - The `Forever` has not been initialized yet (with `put' or `put_with`), or
    /// - A reference to the contents (mutable or not) already exists.
    #[inline(always)]
    #[allow(clippy::mut_from_ref)]
    pub unsafe fn steal(&self) -> &mut T {
        let p = self.t.get();
        let p = (&mut *p).as_mut_ptr();
        &mut *p
    }
}

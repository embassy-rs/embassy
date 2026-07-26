use core::sync::atomic::{AtomicBool, Ordering};

use crate::pac::common::{Read, Reg, Write};

#[allow(dead_code)]
pub trait AtomicModify<T: Sized> {
    /// Atomically mask bits
    ///
    /// Call `set_xxx(true)` inside the closure
    fn set_bits(&self, f: impl FnOnce(&mut T));
    /// Atomically unmask bits
    ///
    /// Call `set_xxx(false)` inside the closure
    fn clear_bits(&self, f: impl FnOnce(&mut T));
}

impl<T: Copy, A: Read + Write> AtomicModify<T> for Reg<T, A> {
    fn set_bits(&self, f: impl FnOnce(&mut T)) {
        unsafe {
            #[cfg(target_has_atomic = "32")]
            use core::sync::atomic::{AtomicU32, Ordering, compiler_fence};

            let mut v = u32::MIN;

            core::assert_eq!(size_of::<u32>(), size_of::<T>());
            core::assert_eq!(align_of::<u32>(), align_of::<T>());

            f(&mut *(&raw mut v as *mut T));

            let ptr = self.as_ptr() as *mut u32;

            #[cfg(not(target_has_atomic = "32"))]
            critical_section::with(|_| {
                let val = ptr.read_volatile();
                ptr.write_volatile(val | v);
            });

            #[cfg(target_has_atomic = "32")]
            compiler_fence(Ordering::Release);

            #[cfg(target_has_atomic = "32")]
            AtomicU32::from_ptr(ptr).fetch_or(v, Ordering::Relaxed);

            #[cfg(target_has_atomic = "32")]
            compiler_fence(Ordering::Release);
        }
    }

    fn clear_bits(&self, f: impl FnOnce(&mut T)) {
        unsafe {
            #[cfg(target_has_atomic = "32")]
            use core::sync::atomic::{AtomicU32, Ordering, compiler_fence};

            let mut v = u32::MAX;

            core::assert_eq!(size_of::<u32>(), size_of::<T>());
            core::assert_eq!(align_of::<u32>(), align_of::<T>());

            f(&mut *(&raw mut v as *mut T));

            let ptr = self.as_ptr() as *mut u32;

            #[cfg(not(target_has_atomic = "32"))]
            critical_section::with(|_| {
                let val = ptr.read_volatile();
                ptr.write_volatile(val & v);
            });

            #[cfg(target_has_atomic = "32")]
            compiler_fence(Ordering::Release);

            #[cfg(target_has_atomic = "32")]
            AtomicU32::from_ptr(ptr).fetch_and(v, Ordering::Relaxed);

            #[cfg(target_has_atomic = "32")]
            compiler_fence(Ordering::Release);
        }
    }
}

#[allow(dead_code)]
pub trait AtomicClear {
    /// Clear the boolean value and return its state
    fn clear(&self) -> bool;
}

impl AtomicClear for AtomicBool {
    #[cfg(target_has_atomic = "8")]
    fn clear(&self) -> bool {
        self.swap(false, Ordering::Acquire)
    }

    #[cfg(not(target_has_atomic = "8"))]
    fn clear(&self) -> bool {
        if self.load(Ordering::Acquire) {
            self.store(false, Ordering::Relaxed);

            true
        } else {
            false
        }
    }
}

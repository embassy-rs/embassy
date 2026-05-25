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
            #[cfg(target_has_atomic = "ptr")]
            use core::sync::atomic::{AtomicUsize, Ordering};

            let mut v = usize::MIN;

            core::assert_eq!(size_of::<usize>(), size_of::<T>());
            core::assert_eq!(align_of::<usize>(), align_of::<T>());

            f(&mut *(&raw mut v as *mut T));

            let ptr = self.as_ptr() as *mut usize;

            #[cfg(not(target_has_atomic = "ptr"))]
            critical_section::with(|_| {
                let val = ptr.read_volatile();
                ptr.write_volatile(val | v);
            });

            #[cfg(target_has_atomic = "ptr")]
            AtomicUsize::from_ptr(ptr).fetch_or(v, Ordering::Relaxed);
        }
    }

    fn clear_bits(&self, f: impl FnOnce(&mut T)) {
        unsafe {
            #[cfg(target_has_atomic = "ptr")]
            use core::sync::atomic::{AtomicUsize, Ordering};

            let mut v = usize::MAX;

            core::assert_eq!(size_of::<usize>(), size_of::<T>());
            core::assert_eq!(align_of::<usize>(), align_of::<T>());

            f(&mut *(&raw mut v as *mut T));

            let ptr = self.as_ptr() as *mut usize;

            #[cfg(not(target_has_atomic = "ptr"))]
            critical_section::with(|_| {
                let val = ptr.read_volatile();
                ptr.write_volatile(val & v);
            });

            #[cfg(target_has_atomic = "ptr")]
            AtomicUsize::from_ptr(ptr).fetch_and(v, Ordering::Relaxed);
        }
    }
}

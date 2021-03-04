use core::cell::UnsafeCell;
use core::mem::MaybeUninit;

use crate::atomic::{AtomicBool, Ordering};

pub struct Forever<T> {
    used: AtomicBool,
    t: UnsafeCell<MaybeUninit<T>>,
}

unsafe impl<T> Send for Forever<T> {}
unsafe impl<T> Sync for Forever<T> {}

impl<T> Forever<T> {
    pub const fn new() -> Self {
        Self {
            used: AtomicBool::new(false),
            t: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }

    pub fn put(&'static self, val: T) -> &'static mut T {
        if self
            .used
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            panic!("Forever.put() called multiple times");
        }

        unsafe {
            let p = self.t.get();
            let p = (&mut *p).as_mut_ptr();
            p.write(val);
            &mut *p
        }
    }

    pub unsafe fn steal(&'static self) -> &'static mut T {
        let p = self.t.get();
        let p = (&mut *p).as_mut_ptr();
        &mut *p
    }
}

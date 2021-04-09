pub use cs::{critical_section, CriticalSection};

#[cfg(feature = "std")]
mod cs {
    static INIT: std::sync::Once = std::sync::Once::new();
    static mut BKL: Option<std::sync::Mutex<()>> = None;

    pub type CriticalSection = std::sync::MutexGuard<'static, ()>;
    pub fn critical_section<F, R>(f: F) -> R
    where
        F: FnOnce(&CriticalSection) -> R,
    {
        INIT.call_once(|| unsafe {
            BKL.replace(std::sync::Mutex::new(()));
        });
        let guard = unsafe { BKL.as_ref().unwrap().lock().unwrap() };
        f(&guard)
    }
}

#[cfg(not(feature = "std"))]
mod cs {
    pub use cortex_m::interrupt::CriticalSection;
    pub fn critical_section<F, R>(f: F) -> R
    where
        F: FnOnce(&CriticalSection) -> R,
    {
        cortex_m::interrupt::free(f)
    }
}

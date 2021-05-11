struct GlobalLock;

static INIT: std::sync::Once = std::sync::Once::new();
static mut GLOBAL_LOCK: Option<std::sync::Mutex<()>> = None;
static mut GLOBAL_GUARD: Option<std::sync::MutexGuard<'static, ()>> = None;

critical_section::custom_impl!(GlobalLock);

unsafe impl critical_section::Impl for GlobalLock {
    unsafe fn acquire() -> u8 {
        INIT.call_once(|| unsafe {
            GLOBAL_LOCK.replace(std::sync::Mutex::new(()));
        });

        let guard = GLOBAL_LOCK.as_ref().unwrap().lock().unwrap();
        GLOBAL_GUARD.replace(guard);
        1
    }

    unsafe fn release(token: u8) {
        if token == 1 {
            GLOBAL_GUARD.take();
        }
    }
}

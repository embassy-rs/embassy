use embassy_sync::lazy_lock::LazyLock;

fn main() {
    let x = 128u8;
    let x_ptr: *const u8 = core::ptr::addr_of!(x);
    let closure_capturing_non_sync_variable = || unsafe { core::ptr::read(x_ptr) };

    check_sync(LazyLock::new(closure_capturing_non_sync_variable));
}

fn check_sync<T: Sync>(_lazy_lock: T) {}

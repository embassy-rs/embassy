use embassy_sync::lazy_lock::LazyLock;

fn main() {
    let x = 128u8;
    let x_ptr: *const u8 = core::ptr::addr_of!(x);

    let closure_capturing_non_sync_variable = || {
        unsafe {
            core::ptr::read(x_ptr)
        }
    };

    // This should fail to compile because the closure captures a non-Sync variable: x_ptr.
    let _lazy_u8: LazyLock<u8, _> = LazyLock::new(closure_capturing_non_sync_variable);
}

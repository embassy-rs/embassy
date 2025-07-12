use embassy_sync::lazy_lock::LazyLock;

// *mut u8 is not Sync, so LazyLock should not implement Sync for this type. This should fail to compile.
static GLOBAL: LazyLock<*mut u8> = LazyLock::new(|| core::ptr::null_mut());

fn main() {}

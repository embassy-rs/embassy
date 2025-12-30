use embassy_sync::once_lock::OnceLock;

// *mut u8 is not Sync, so OnceLock should not implement Sync for this type. This should fail to compile.
static GLOBAL: OnceLock<*mut u8> = OnceLock::new();

fn main() {}

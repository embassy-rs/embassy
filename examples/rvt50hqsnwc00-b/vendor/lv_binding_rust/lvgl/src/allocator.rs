use core::alloc::{GlobalAlloc, Layout};

// Register the global allocator
#[global_allocator]
static ALLOCATOR: LvglAlloc = LvglAlloc;

/// LVGL allocator. Enabled by toggling the `lvgl_alloc` or `alloc` features.
pub struct LvglAlloc;

unsafe impl GlobalAlloc for LvglAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // Make sure LVGL is initialized!
        crate::init();
        lvgl_sys::lv_mem_alloc(layout.size() as cty::size_t) as *mut u8
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        crate::init();
        lvgl_sys::lv_mem_free(ptr as *mut cty::c_void)
    }
}

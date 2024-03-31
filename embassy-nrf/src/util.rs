#![allow(dead_code)]
use core::mem;

const SRAM_LOWER: usize = 0x2000_0000;
const SRAM_UPPER: usize = 0x3000_0000;

// #![feature(const_slice_ptr_len)]
// https://github.com/rust-lang/rust/issues/71146
pub(crate) fn slice_ptr_len<T>(ptr: *const [T]) -> usize {
    use core::ptr::NonNull;
    let ptr = ptr.cast_mut();
    if let Some(ptr) = NonNull::new(ptr) {
        ptr.len()
    } else {
        // We know ptr is null, so we know ptr.wrapping_byte_add(1) is not null.
        NonNull::new(ptr.wrapping_byte_add(1)).unwrap().len()
    }
}

// TODO: replace transmutes with core::ptr::metadata once it's stable
pub(crate) fn slice_ptr_parts<T>(slice: *const [T]) -> (*const T, usize) {
    unsafe { mem::transmute(slice) }
}

pub(crate) fn slice_ptr_parts_mut<T>(slice: *mut [T]) -> (*mut T, usize) {
    unsafe { mem::transmute(slice) }
}

/// Does this slice reside entirely within RAM?
pub(crate) fn slice_in_ram<T>(slice: *const [T]) -> bool {
    let (ptr, len) = slice_ptr_parts(slice);
    let ptr = ptr as usize;
    ptr >= SRAM_LOWER && (ptr + len * core::mem::size_of::<T>()) < SRAM_UPPER
}

/// Return an error if slice is not in RAM. Skips check if slice is zero-length.
pub(crate) fn slice_in_ram_or<T, E>(slice: *const [T], err: E) -> Result<(), E> {
    let (_, len) = slice_ptr_parts(slice);
    if len == 0 || slice_in_ram(slice) {
        Ok(())
    } else {
        Err(err)
    }
}

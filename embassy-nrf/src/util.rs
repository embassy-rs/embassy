const SRAM_LOWER: usize = 0x2000_0000;
const SRAM_UPPER: usize = 0x3000_0000;

/// Does this slice reside entirely within RAM?
pub(crate) fn slice_in_ram<T>(slice: &[T]) -> bool {
    let ptr = slice.as_ptr() as usize;
    ptr >= SRAM_LOWER && (ptr + slice.len() * core::mem::size_of::<T>()) < SRAM_UPPER
}

/// Return an error if slice is not in RAM.
#[cfg(not(feature = "nrf51"))]
pub(crate) fn slice_in_ram_or<T, E>(slice: &[T], err: E) -> Result<(), E> {
    if slice.is_empty() || slice_in_ram(slice) {
        Ok(())
    } else {
        Err(err)
    }
}

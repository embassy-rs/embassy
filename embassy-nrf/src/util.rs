#![allow(dead_code)]

const SRAM_LOWER: usize = 0x2000_0000;
const SRAM_UPPER: usize = 0x3000_0000;

/// Does this slice reside entirely within RAM?
pub(crate) fn slice_in_ram<T>(slice: *const [T]) -> bool {
    if slice.is_empty() {
        return true;
    }

    let ptr = slice as *const T as usize;
    ptr >= SRAM_LOWER && (ptr + slice.len() * core::mem::size_of::<T>()) < SRAM_UPPER
}

/// Return an error if slice is not in RAM. Skips check if slice is zero-length.
pub(crate) fn slice_in_ram_or<T, E>(slice: *const [T], err: E) -> Result<(), E> {
    if slice_in_ram(slice) { Ok(()) } else { Err(err) }
}

/// Compute the maximum value of an EasyDMA `MAXCNT`-style register field.
///
/// Writes all-ones through the PAC's (masking) field setter and reads the field
/// back, so the result is exactly the largest value the hardware field can hold.
#[cfg(not(feature = "_nrf51"))]
macro_rules! easy_dma_max {
    ($reg:path, $set:ident, $get:ident) => {{
        let mut r = $reg(0);
        r.$set(!0);
        r.$get() as usize
    }};
}
#[cfg(not(feature = "_nrf51"))]
pub(crate) use easy_dma_max;

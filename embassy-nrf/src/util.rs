const SRAM_LOWER: usize = 0x2000_0000;
const SRAM_UPPER: usize = 0x3000_0000;

/// Does this slice reside entirely within RAM?
pub(crate) fn slice_in_ram(slice: &[u8]) -> bool {
    let ptr = slice.as_ptr() as usize;
    ptr >= SRAM_LOWER && (ptr + slice.len()) < SRAM_UPPER
}

/// Return an error if slice is not in RAM.
#[cfg(not(feature = "nrf51"))]
pub(crate) fn slice_in_ram_or<T>(slice: &[u8], err: T) -> Result<(), T> {
    if slice.len() == 0 || slice_in_ram(slice) {
        Ok(())
    } else {
        Err(err)
    }
}

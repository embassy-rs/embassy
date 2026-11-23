//! Pure eSPI wire-format encodings.

pub(super) fn saf_input(raw: u32) -> (u32, u8) {
    // RM 52.6.1 and 52.6.3 define the SAF-only view. The generic PAC view
    // deliberately follows the conflicting endpoint register table instead.
    ((raw & 0xfff) + 1, ((raw >> 26) & 0xf) as u8)
}

pub(super) fn oob_len(length: usize, window: usize) -> Option<u8> {
    if length == 0 || length > window || length > 124 {
        None
    } else {
        Some((length - 1) as u8)
    }
}

pub(super) fn ram_window_valid(offset: usize, length: usize, alignment: usize) -> bool {
    alignment != 0 && offset % alignment == 0 && offset.checked_add(length).is_some_and(|end| end <= 4096)
}

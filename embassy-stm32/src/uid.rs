//! Unique ID (UID)

use once_cell::sync::Lazy;

struct UID {
    bytes: [u8; 12],
    hex: [u8; 24],
}

static UID: Lazy<UID> = Lazy::new(|| {
    let mut bytes = [0u8; 12];
    for (idx, chunk) in bytes.chunks_mut(4).enumerate() {
        chunk.copy_from_slice(&read_word(idx).to_le_bytes());
    }

    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let mut hex = [0u8; 24];
    for (idx, v) in bytes.iter().enumerate() {
        let lo = v & 0x0f;
        let hi = (v & 0xf0) >> 4;
        hex[idx * 2] = HEX[hi as usize];
        hex[idx * 2 + 1] = HEX[lo as usize];
    }

    UID { bytes, hex }
});

/// Read word `idx` (0..3) of the 96-bit unique ID.
#[cfg(not(stm32l0))]
fn read_word(idx: usize) -> u32 {
    crate::pac::UID.uid(idx).read()
}

/// Read word `idx` (0..3) of the 96-bit unique ID.
///
/// On STM32L0 the three words are not contiguous: RM0367 section 34.2 documents U_ID(31:0) at
/// offset 0x00, U_ID(63:32) at 0x04 and U_ID(95:64) at 0x14. The word at 0x08 is a factory
/// constant, so reading with a 4-byte stride gives every chip from the same wafer and lot the
/// same "unique" ID (#3046).
#[cfg(stm32l0)]
fn read_word(idx: usize) -> u32 {
    const OFFSETS: [usize; 3] = [0x00, 0x04, 0x14];
    let base = crate::pac::UID.as_ptr() as usize;
    unsafe { core::ptr::read_volatile((base + OFFSETS[idx]) as *const u32) }
}

/// Get this device's unique 96-bit ID.
pub fn uid() -> &'static [u8; 12] {
    &UID.bytes
}

/// Get this device's unique 96-bit ID, encoded into a string of 24 hexadecimal ASCII digits.
pub fn uid_hex() -> &'static str {
    unsafe { core::str::from_utf8_unchecked(uid_hex_bytes()) }
}

/// Get this device's unique 96-bit ID, encoded into 24 hexadecimal ASCII bytes.
pub fn uid_hex_bytes() -> &'static [u8; 24] {
    &UID.hex
}

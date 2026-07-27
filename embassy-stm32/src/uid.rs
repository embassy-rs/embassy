//! Unique ID (UID)

use once_cell::sync::Lazy;

struct UID {
    bytes: [u8; 12],
    hex: [u8; 24],
}

static UID: Lazy<UID> = Lazy::new(|| {
    let mut bytes = [0u8; 12];
    for (idx, chunk) in bytes.chunks_mut(4).enumerate() {
        chunk.copy_from_slice(&crate::pac::UID.uid(idx).read().to_le_bytes());
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

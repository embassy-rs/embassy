//! Unique ID (UID)

/// Get this device's unique 96-bit ID.
pub fn uid() -> &'static [u8; 12] {
    unsafe { &*crate::pac::UID.uid(0).as_ptr().cast::<[u8; 12]>() }
}

/// Get this device's unique 96-bit ID, encoded into a string of 24 hexadecimal ASCII digits.
pub fn uid_hex() -> &'static str {
    unsafe { core::str::from_utf8_unchecked(uid_hex_bytes()) }
}

/// Get this device's unique 96-bit ID, encoded into 24 hexadecimal ASCII bytes.
pub fn uid_hex_bytes() -> &'static [u8; 24] {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    static mut UID_HEX: [u8; 24] = [0; 24];
    static mut LOADED: bool = false;
    critical_section::with(|_| unsafe {
        if !LOADED {
            let uid = uid();
            for (idx, v) in uid.iter().enumerate() {
                let lo = v & 0x0f;
                let hi = (v & 0xf0) >> 4;
                UID_HEX[idx * 2] = HEX[hi as usize];
                UID_HEX[idx * 2 + 1] = HEX[lo as usize];
            }
            LOADED = true;
        }
    });
    unsafe { &UID_HEX }
}

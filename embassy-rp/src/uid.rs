//! Unique ID (UID)
//!
//! The ID is read once, on first use, and cached.
//!
//! The source differs per chip:
//!
//! - **RP2350**: just reads the chip ID from OTP. Same as bootrom USB serial number.
//!
//! - **RP2040**: no chip ID, so use the flash chip's unique ID, as per `pico_get_unique_board_id`
//!   in pico-sdk. Some flash chips don't even support this.
//!
//! Infallible, so if the ID cannot be read, return a fixed value (`0xEE`), also as per `pico-sdk`.

use embassy_sync::lazy_lock::LazyLock;

const UNAVAILABLE: u8 = 0xEE;

struct Uid {
    bytes: [u8; 8],
    hex: [u8; 16],
}

static UID: LazyLock<Uid> = LazyLock::new(|| {
    let bytes = read_uid();

    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let mut hex = [0u8; 16];
    for (idx, v) in bytes.iter().enumerate() {
        let lo = v & 0x0f;
        let hi = (v & 0xf0) >> 4;
        hex[idx * 2] = HEX[hi as usize];
        hex[idx * 2 + 1] = HEX[lo as usize];
    }

    Uid { bytes, hex }
});

#[cfg(feature = "rp2040")]
fn read_uid() -> [u8; 8] {
    let mut bytes = [0u8; 8];
    // `in_ram` pauses core 1 and runs the command in a critical section
    // to avoid problems with XIP.
    match unsafe { crate::flash::in_ram(|| crate::flash::ram_helpers::flash_unique_id(&mut bytes)) } {
        Ok(()) => bytes,
        Err(_) => [UNAVAILABLE; 8],
    }
}

#[cfg(feature = "_rp235x")]
fn read_uid() -> [u8; 8] {
    match crate::otp::get_chipid() {
        Ok(id) => id.to_be_bytes(),
        Err(_) => [UNAVAILABLE; 8],
    }
}

/// Get this device's unique 64-bit ID.
pub fn uid() -> &'static [u8; 8] {
    &UID.get().bytes
}

/// Get this device's unique 64-bit ID as a string of 16 hexadecimal ASCII digits.
pub fn uid_hex() -> &'static str {
    unsafe { core::str::from_utf8_unchecked(uid_hex_bytes()) }
}

/// Get this device's unique 64-bit ID, encoded into 16 hexadecimal ASCII bytes.
pub fn uid_hex_bytes() -> &'static [u8; 16] {
    &UID.get().hex
}

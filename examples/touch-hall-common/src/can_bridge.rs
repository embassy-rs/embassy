//! CAN bridge for hall touch UI — ports the Python `CanBridge` bitmask protocol.
//!
//! TX: one-hot 6-byte bitmask on `CAN_TX_ID` while a button is held.
//! RX: `minp` entries map feedback bits on `MINP_RX_ID` to button highlight state.

use core::sync::atomic::{AtomicU8, Ordering};

use crate::{BUTTON_COUNT, MINP_COUNT};

/// CAN TX data length (one-hot bitmask across six bytes).
pub const TX_PAYLOAD_LEN: usize = 6;

/// Highest button index currently held (255 = none).
pub static ACTIVE_BUTTON: AtomicU8 = AtomicU8::new(255);

/// Build a 6-byte one-hot TX payload for the given button index.
pub fn command_payload(button_index: u8) -> [u8; 6] {
    let mut data = [0u8; 6];
    let bit = button_index as usize;
    if bit >= 48 {
        return data;
    }
    let byte = bit / 8;
    let bit_in_byte = bit % 8;
    if crate::CAN_TX_LITTLE_ENDIAN {
        data[byte] = 1 << bit_in_byte;
    } else {
        let be_bit = 47 - bit;
        let be_byte = be_bit / 8;
        let be_bit_in_byte = 7 - (be_bit % 8);
        data[be_byte] = 1 << be_bit_in_byte;
    }
    data
}

/// Release frame payload: six zero bytes.
pub fn release_payload() -> [u8; TX_PAYLOAD_LEN] {
    [0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
}

/// Byte and bit position for a button index in the TX bitmask.
pub fn button_byte_bit(index: usize) -> Option<(usize, u8)> {
    if index >= 48 {
        return None;
    }
    if crate::CAN_TX_LITTLE_ENDIAN {
        Some((index / 8, (index % 8) as u8))
    } else {
        let be_bit = 47 - index;
        let be_byte = be_bit / 8;
        let be_bit_in_byte = 7 - (be_bit % 8);
        Some((be_byte, be_bit_in_byte as u8))
    }
}

/// Set or clear one bit in a payload byte.
pub fn set_bit_in_byte(byte: u8, bit_index: u8, active: bool) -> u8 {
    if bit_index >= 8 {
        return byte;
    }
    let mask = 1 << bit_index;
    if active {
        byte | mask
    } else {
        byte & !mask
    }
}

/// Set or clear the one-hot bit for `index` in `payload`.
pub fn apply_button_bit(payload: &mut [u8; TX_PAYLOAD_LEN], index: usize, active: bool) {
    if let Some((byte, bit)) = button_byte_bit(index) {
        if byte < TX_PAYLOAD_LEN {
            payload[byte] = set_bit_in_byte(payload[byte], bit, active);
        }
    }
}

/// True when every TX byte is zero (release / idle keepalive).
pub fn payload_is_release(payload: &[u8; TX_PAYLOAD_LEN]) -> bool {
    payload.iter().all(|&b| b == 0)
}

/// Map PLC/Rust TX index to CAN data (255 = release).
pub fn tx_payload(button_index: u8) -> [u8; 6] {
    if button_index == 255 {
        release_payload()
    } else {
        command_payload(button_index)
    }
}

/// Apply minp feedback bits into `button_status` (sustained CAN RX levels).
pub fn handle_minp_frame(can_id: u16, data: &[u8], button_status: &mut [u8]) {
    crate::can_input::store_rx(can_id, data);
    let count = button_status.len().min(MINP_COUNT).min(BUTTON_COUNT);
    for (i, slot) in button_status.iter_mut().enumerate().take(count) {
        *slot = crate::can_input::minp_raw(i) as u8;
    }
}

/// Mark which button is held for the repeat sender task.
pub fn set_active_button(button_index: Option<u8>) {
    ACTIVE_BUTTON.store(button_index.unwrap_or(255), Ordering::Relaxed);
}

/// Read the held button index, if any.
pub fn active_button() -> Option<u8> {
    match ACTIVE_BUTTON.load(Ordering::Relaxed) {
        255 => None,
        index => Some(index),
    }
}

/// Token string for a button index (`field:…` / `group:…`), if in range.
pub fn button_token(index: usize) -> &'static str {
    crate::BUTTON_TOKENS.get(index).copied().unwrap_or("?")
}

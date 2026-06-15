//! CAN bridge for hall touch UI — ports the Python `CanBridge` bitmask protocol.
//!
//! TX: one-hot 6-byte bitmask on `CAN_TX_ID` while a button is held.
//! RX: `minp` entries map feedback bits on `MINP_RX_ID` to button highlight state.

use core::sync::atomic::{AtomicU8, Ordering};

use crate::{BUTTON_COUNT, MINP_COUNT};

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
pub fn release_payload() -> [u8; 6] {
    [0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
}

/// Map PLC/Rust TX index to CAN data (255 = release).
pub fn tx_payload(button_index: u8) -> [u8; 6] {
    if button_index == 255 {
        release_payload()
    } else {
        command_payload(button_index)
    }
}

/// Apply debounced `minp` feedback into `button_status`.
pub fn handle_minp_frame(can_id: u16, data: &[u8], button_status: &mut [u8]) {
    crate::can_input::store_rx(can_id, data);
    let count = button_status.len().min(MINP_COUNT).min(BUTTON_COUNT);
    for (i, slot) in button_status.iter_mut().enumerate().take(count) {
        *slot = crate::can_input::minp_active(i) as u8;
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

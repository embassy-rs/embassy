//! CAN bridge for hall touch UI — ports the Python `CanBridge` bitmask protocol.
//!
//! TX: one-hot 6-byte bitmask on `CAN_TX_ID` while a button is held.
//! RX: `minp` entries map feedback bits on `MINP_RX_ID` to button highlight state.

use core::sync::atomic::{AtomicU8, Ordering};

use crate::{BUTTON_COUNT, MINP, MINP_COUNT};

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

/// Apply `minp` feedback bits from a received CAN frame.
pub fn handle_minp_frame(can_id: u16, data: &[u8], button_status: &mut [u8]) {
    let count = button_status.len().min(MINP_COUNT).min(BUTTON_COUNT);
    for (i, entry) in MINP.iter().enumerate().take(count) {
        if entry.active_value == 0 || entry.can_id != can_id {
            continue;
        }
        let byte_index = entry.byte_index as usize;
        let bit_index = entry.bit_index;
        if byte_index >= data.len() {
            continue;
        }
        let active = (data[byte_index] >> bit_index) & 1;
        button_status[i] = active;
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

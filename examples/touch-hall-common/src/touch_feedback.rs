//! Touch hold highlight — steady while latched (no blink).

use crate::input_state;
use crate::touch_hold;

/// Highlight overrides CAN/PLC status while the button is latched or held.
pub fn hold_highlight(index: usize) -> Option<bool> {
    if touch_hold::latched() == Some(index as u8) || input_state::held(index) {
        Some(true)
    } else {
        None
    }
}

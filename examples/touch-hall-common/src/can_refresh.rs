//! Idle CAN keepalive — cyclically re-send release frame `00×6`.

use crate::can_bridge;
use crate::touch_hold;

/// Refresh interval for the idle release frame.
pub const CAN_REFRESH_MS: u32 = 1000;

/// Idle keepalive: release (`255` → `00×6`) every second while no touch hold.
pub fn idle_refresh_button() -> Option<u8> {
    if can_bridge::active_button().is_some() || crate::input_state::any_held() || touch_hold::is_latched() {
        None
    } else {
        Some(255)
    }
}

/// Track last TX for diagnostics (optional callers).
pub fn note_tx(_button_index: u8) {}

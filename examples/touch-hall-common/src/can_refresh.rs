//! CAN refresh interval — PLC mode runs `cycle()` in `on_periodic_refresh`; legacy uses `idle_refresh_payload`.

use crate::button_status;
use crate::can_bridge::{payload_is_release, release_payload, TX_PAYLOAD_LEN};
use crate::touch_hold;

/// Refresh interval (`on_periodic_refresh` / legacy idle keepalive).
pub const CAN_REFRESH_MS: u32 = 1000;

/// Legacy non-PLC idle keepalive (`00×6` while nothing active). PLC uses Rhai `on_periodic_refresh`.
pub fn idle_refresh_payload() -> Option<[u8; TX_PAYLOAD_LEN]> {
    if crate::can_bridge::active_button().is_some()
        || crate::input_state::any_held()
        || touch_hold::is_latched()
        || button_status::any_plc_active()
    {
        None
    } else {
        Some(release_payload())
    }
}

/// Legacy helper — release sentinel for button-index TX paths.
pub fn idle_refresh_button() -> Option<u8> {
    idle_refresh_payload().map(|_| 255)
}

/// Track last TX for diagnostics (optional callers).
pub fn note_tx(_button_index: u8) {}

/// Track last TX payload for diagnostics (optional callers).
pub fn note_tx_payload(payload: &[u8; TX_PAYLOAD_LEN]) {
    let _ = payload_is_release(payload);
}

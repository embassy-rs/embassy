//! CAN TX scheduling — Rust defines WHEN; Rhai defines WHAT (`can_tx`).
//!
//! | Trigger              | Rust schedule                                      |
//! |----------------------|----------------------------------------------------|
//! | Periodic (1 s)       | `cycle()` then TX                                  |
//! | CAN RX (new data)    | `cycle()` then TX immediately                      |
//! | Touch press/hold     | `cycle()` then TX (`command_repeat_ms` while hold) |
//! | Touch release        | `cycle()` then TX immediately                      |
//!
//! Every bus frame is sent only after `plc.cycle()` — payload is always Rhai `can_tx`.

use crate::can_bridge::{button_token, payload_is_release, set_active_button, TX_PAYLOAD_LEN};
use crate::input_state;
use crate::touch_hold;
use crate::BUTTON_COUNT;

#[cfg(feature = "rhai")]
use crate::button_status;
#[cfg(feature = "rhai")]
use crate::rhai_state::Plc;

/// Rhai `can_tx` payload length on the bus.
pub type PlcCanTx = [u8; TX_PAYLOAD_LEN];

/// PLC TX arm — every scheduled send goes on the bus (no payload dedupe).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlcTxState;

impl PlcTxState {
    pub const fn new() -> Self {
        Self
    }

    fn arm_send_repeat(&self, payload: Option<PlcCanTx>) -> Option<PlcCanTx> {
        payload
    }
}

/// Snapshot `ui[]`, run one PLC scan, return Rhai `can_tx`.
#[cfg(feature = "rhai")]
fn scan_plc_cycle(plc: &mut Plc, scratch: &mut [u8]) -> (Option<PlcCanTx>, [u8; 64], bool) {
    let minp_event = crate::can_input::any_minp_pending();
    let mut before = [0u8; 64];
    button_status::snapshot(&mut before, BUTTON_COUNT);
    let payload = cycle_output(plc, scratch);
    (payload, before, minp_event)
}

#[cfg(feature = "rhai")]
fn log_plc_ui(scratch: &[u8]) {
    let count = BUTTON_COUNT.min(scratch.len());
    log_plc_ui_slice(&scratch[..count]);
}

#[cfg(all(feature = "rhai", feature = "log"))]
fn log_plc_ui_slice(scratch: &[u8]) {
    log::debug!("PLC ui[{}] = {:02?}", scratch.len(), scratch);
    for (i, &v) in scratch.iter().enumerate() {
        if v != 0 {
            log::debug!("  ui[{i}] true ({})", button_token(i));
        }
    }
}

/// Log active `ui[]` entries at info level (e.g. after a state change).
#[cfg(feature = "rhai")]
pub fn log_plc_ui_info(scratch: &[u8]) {
    let count = BUTTON_COUNT.min(scratch.len());
    log_plc_ui_info_slice(&scratch[..count]);
}

#[cfg(all(feature = "rhai", feature = "log"))]
fn log_plc_ui_info_slice(scratch: &[u8]) {
    log::info!("PLC ui[{}] = {:02?}", scratch.len(), scratch);
    for (i, &v) in scratch.iter().enumerate() {
        if v != 0 {
            log::info!("  ui[{i}] true ({})", button_token(i));
        }
    }
}

#[cfg(all(feature = "rhai", feature = "defmt", not(feature = "log")))]
fn log_plc_ui_info_slice(scratch: &[u8]) {
    defmt::info!("PLC ui len={}", scratch.len());
    for (i, &v) in scratch.iter().enumerate() {
        if v != 0 {
            defmt::info!("PLC ui[{}] true ({})", i, button_token(i));
        }
    }
}

#[cfg(all(feature = "rhai", not(any(feature = "log", feature = "defmt"))))]
fn log_plc_ui_info_slice(_scratch: &[u8]) {}

#[cfg(all(feature = "rhai", feature = "defmt", not(feature = "log")))]
fn log_plc_ui_slice(scratch: &[u8]) {
    defmt::debug!("PLC ui len={}", scratch.len());
    for (i, &v) in scratch.iter().enumerate() {
        if v != 0 {
            defmt::debug!("PLC ui[{}] true ({})", i, button_token(i));
        }
    }
}

#[cfg(all(feature = "rhai", not(any(feature = "log", feature = "defmt"))))]
fn log_plc_ui_slice(_scratch: &[u8]) {}

/// Periodic — `cycle()` then TX every `CAN_REFRESH_MS`.
#[cfg(feature = "rhai")]
pub fn on_periodic_refresh(plc: &mut Plc, scratch: &mut [u8], tx_state: &PlcTxState) -> Option<PlcCanTx> {
    tx_state.arm_send_repeat(cycle_output(plc, scratch))
}

/// CAN RX — `cycle()` then immediate TX (CanRx only fires when RX data changed).
#[cfg(feature = "rhai")]
pub fn on_can_rx(plc: &mut Plc, scratch: &mut [u8], tx_state: &PlcTxState) -> Option<PlcCanTx> {
    let (payload, before, minp_event) = scan_plc_cycle(plc, scratch);
    let count = BUTTON_COUNT.min(scratch.len()).min(before.len());
    log_minp_changes(&before[..count], &scratch[..count]);
    if minp_event {
        log_plc_ui_info(scratch);
    }
    tx_state.arm_send_repeat(payload)
}

/// Run one PLC scan, update Rhai `ui[]`. Returns Rhai `can_tx` payload.
#[cfg(feature = "rhai")]
pub fn cycle_output(plc: &mut Plc, scratch: &mut [u8]) -> Option<PlcCanTx> {
    if plc.cycle().is_err() {
        log_cycle_failed();
        return None;
    }
    let payload = plc.can_tx_payload();
    button_status::apply_from_plc(plc, scratch);
    log_plc_ui(scratch);
    log_rhai_can_tx(&payload);
    Some(payload)
}

#[cfg(feature = "rhai")]
fn log_rhai_can_tx(payload: &PlcCanTx) {
    log_rhai_can_tx_inner(payload);
}

#[cfg(all(feature = "rhai", feature = "log"))]
fn log_rhai_can_tx_inner(payload: &PlcCanTx) {
    if payload_is_release(payload) {
        log::debug!("Rhai can_tx = RELEASE");
    } else {
        log::debug!("Rhai can_tx = {:02x?}", payload);
    }
}

#[cfg(all(feature = "rhai", feature = "defmt", not(feature = "log")))]
fn log_rhai_can_tx_inner(payload: &PlcCanTx) {
    if payload_is_release(payload) {
        defmt::debug!("Rhai can_tx RELEASE");
    } else {
        defmt::debug!("Rhai can_tx {:?}", payload);
    }
}

#[cfg(all(feature = "rhai", not(any(feature = "log", feature = "defmt"))))]
fn log_rhai_can_tx_inner(_payload: &PlcCanTx) {}

/// Touch press — latch inputs, one scan, send Rhai `can_tx` once.
#[cfg(feature = "rhai")]
pub fn on_touch_press(
    plc: &mut Plc,
    scratch: &mut [u8],
    tx_state: &PlcTxState,
    index: u8,
) -> Option<PlcCanTx> {
    touch_begin(index);
    let (payload, _, _) = scan_plc_cycle(plc, scratch);
    tx_state.arm_send_repeat(payload)
}

/// Touch release — clear inputs, `cycle()`, send Rhai `can_tx` immediately.
#[cfg(feature = "rhai")]
pub fn on_touch_release(
    plc: &mut Plc,
    scratch: &mut [u8],
    tx_state: &PlcTxState,
    held: u8,
    long_fired: bool,
) -> Option<PlcCanTx> {
    touch_end(held, long_fired);
    let (payload, _, _) = scan_plc_cycle(plc, scratch);
    tx_state.arm_send_repeat(payload)
}

/// Switch held button — one scan, send Rhai `can_tx` once.
#[cfg(feature = "rhai")]
pub fn on_touch_switch(
    plc: &mut Plc,
    scratch: &mut [u8],
    tx_state: &PlcTxState,
    from: u8,
    to: u8,
) -> Option<PlcCanTx> {
    touch_switch(from, to);
    let (payload, _, _) = scan_plc_cycle(plc, scratch);
    tx_state.arm_send_repeat(payload)
}

/// Hold tick — long-press pulse + `cycle()` + TX at touch repeat interval.
#[cfg(feature = "rhai")]
pub fn on_touch_hold_tick(
    plc: &mut Plc,
    scratch: &mut [u8],
    tx_state: &PlcTxState,
    held: u8,
    press_elapsed_ms: u64,
    long_fired: &mut bool,
) -> Option<PlcCanTx> {
    touch_maybe_long(held, press_elapsed_ms, long_fired);
    let (payload, _, _) = scan_plc_cycle(plc, scratch);
    tx_state.arm_send_repeat(payload)
}

/// Release outside active hold loop.
#[cfg(feature = "rhai")]
pub fn on_idle_release(plc: &mut Plc, scratch: &mut [u8], tx_state: &PlcTxState) -> Option<PlcCanTx> {
    if touch_hold::is_latched() || input_state::any_held() {
        return None;
    }
    set_active_button(None);
    tx_state.arm_send_repeat(cycle_output(plc, scratch))
}

pub fn touch_begin(index: u8) {
    touch_hold::latch_press(index);
    input_state::set_held(index as usize, true);
    set_active_button(Some(index));
}

pub fn touch_end(held: u8, long_fired: bool) {
    touch_hold::latch_clear();
    input_state::set_held(held as usize, false);
    if !long_fired {
        input_state::pulse_short(held as usize);
    }
    input_state::pulse_release(held as usize);
    set_active_button(None);
}

pub fn touch_switch(from: u8, to: u8) {
    input_state::set_held(from as usize, false);
    input_state::pulse_release(from as usize);
    touch_hold::latch_press(to);
    input_state::set_held(to as usize, true);
    set_active_button(Some(to));
}

pub fn touch_maybe_long(held: u8, press_elapsed_ms: u64, long_fired: &mut bool) {
    if !*long_fired && press_elapsed_ms >= crate::LONG_PRESS_MS as u64 {
        *long_fired = true;
        input_state::pulse_long(held as usize);
    }
}

fn log_minp_changes(before: &[u8], after: &[u8]) {
    for (i, (prev, next)) in before.iter().zip(after.iter()).enumerate() {
        if prev != next {
            log_minp_btn(i, *prev != 0, *next != 0);
        }
    }
}

#[cfg(feature = "rhai")]
pub fn log_minp_scratch(before: &[u8], after: &[u8]) {
    log_minp_changes(before, after);
}

#[cfg(feature = "log")]
fn log_cycle_failed() {
    log::warn!("PLC cycle failed");
}

#[cfg(all(feature = "defmt", not(feature = "log")))]
fn log_cycle_failed() {
    defmt::warn!("PLC cycle failed");
}

#[cfg(not(any(feature = "log", feature = "defmt")))]
fn log_cycle_failed() {}

#[cfg(feature = "log")]
fn log_minp_btn(i: usize, prev: bool, next: bool) {
    log::info!(
        "CAN minp btn={i} token={} active {prev} -> {next}",
        button_token(i),
    );
}

#[cfg(all(feature = "defmt", not(feature = "log")))]
fn log_minp_btn(i: usize, prev: bool, next: bool) {
    defmt::info!(
        "CAN minp btn={} token={} active {} -> {}",
        i,
        button_token(i),
        prev,
        next
    );
}

#[cfg(not(any(feature = "log", feature = "defmt")))]
fn log_minp_btn(_i: usize, _prev: bool, _next: bool) {}

//! CAN TX scheduling — all timing lives here (Rust). Rhai supplies `can_tx` only.
//!
//! | Trigger        | Rust schedule                            |
//! |----------------|------------------------------------------|
//! | Idle           | `idle_refresh_button` every 1 s, no touch |
//! | CAN RX changed | `on_can_rx` once per changed frame       |
//! | Touch hold     | `cycle_output` every `command_repeat_ms` (no CAN) |
//! | Touch press/release/switch | `cycle_output` only (no CAN)       |
//!
//! Touch: GUI + Rhai `cycle()` only — **no CAN TX** (CAN TX = Rhai `can_tx` on RX + idle refresh).

use core::sync::atomic::{AtomicU8, Ordering};

use crate::can_bridge::{button_token, set_active_button};
use crate::can_refresh;
use crate::input_state;
use crate::touch_hold;
use crate::BUTTON_COUNT;

#[cfg(feature = "rhai")]
use crate::button_status;
#[cfg(feature = "rhai")]
use crate::rhai_state::Plc;

/// Sentinel: no PLC command transmitted yet.
const LAST_SENT_UNSET: u8 = 254;

static LAST_SENT: AtomicU8 = AtomicU8::new(LAST_SENT_UNSET);

/// Tracks last transmitted Rhai `can_tx` (CAN RX / idle dedupe).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlcTxState;

impl PlcTxState {
    pub const fn new() -> Self {
        Self
    }

    fn last_sent() -> Option<u8> {
        match LAST_SENT.load(Ordering::Relaxed) {
            LAST_SENT_UNSET => None,
            cmd => Some(cmd),
        }
    }

    fn set_last_sent(cmd: u8) {
        LAST_SENT.store(cmd, Ordering::Relaxed);
    }

    pub fn note_sent(&self, cmd: u8) {
        Self::set_last_sent(cmd);
    }

    /// Rhai `can_tx` dedupe (CAN RX / release paths only).
    pub fn arm_send_rhai(&self, cmd: Option<u8>) -> Option<u8> {
        let cmd = cmd?;
        if Self::last_sent() == Some(cmd) {
            return None;
        }
        Self::set_last_sent(cmd);
        Some(cmd)
    }
}

fn ui_unchanged(before: &[u8], after: &[u8]) -> bool {
    let count = before.len().min(after.len()).min(BUTTON_COUNT);
    before[..count] == after[..count]
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

/// Idle keepalive button index (`255` = release), or `None` if touch blocks refresh.
pub fn idle_refresh_button() -> Option<u8> {
    can_refresh::idle_refresh_button()
}

/// Run one PLC scan, update Rhai `ui[]`. Returns Rhai `can_tx` (CAN/minp only).
#[cfg(feature = "rhai")]
pub fn cycle_output(plc: &mut Plc, scratch: &mut [u8]) -> Option<u8> {
    if plc.cycle().is_err() {
        log_cycle_failed();
        return None;
    }
    let cmd = plc.can_tx();
    button_status::apply_from_plc(plc, scratch);
    log_plc_ui(scratch);
    log_rhai_can_tx(cmd);
    cmd
}

#[cfg(feature = "rhai")]
fn log_rhai_can_tx(cmd: Option<u8>) {
    log_rhai_can_tx_inner(cmd);
}

#[cfg(all(feature = "rhai", feature = "log"))]
fn log_rhai_can_tx_inner(cmd: Option<u8>) {
    match cmd {
        None => log::debug!("Rhai can_tx = none"),
        Some(255) => log::debug!("Rhai can_tx = RELEASE"),
        Some(btn) => log::debug!("Rhai can_tx = {btn} ({})", button_token(btn as usize)),
    }
}

#[cfg(all(feature = "rhai", feature = "defmt", not(feature = "log")))]
fn log_rhai_can_tx_inner(cmd: Option<u8>) {
    match cmd {
        None => defmt::debug!("Rhai can_tx none"),
        Some(255) => defmt::debug!("Rhai can_tx RELEASE"),
        Some(btn) => defmt::debug!("Rhai can_tx {}", btn),
    }
}

#[cfg(all(feature = "rhai", not(any(feature = "log", feature = "defmt"))))]
fn log_rhai_can_tx_inner(_cmd: Option<u8>) {}

/// CAN RX changed — one scan; TX only when PLC `ui[]` changed.
#[cfg(feature = "rhai")]
pub fn on_can_rx(plc: &mut Plc, scratch: &mut [u8], tx_state: &PlcTxState) -> Option<u8> {
    let mut before = [0u8; 64];
    button_status::snapshot(&mut before, BUTTON_COUNT);
    let cmd = cycle_output(plc, scratch);
    log_minp_changes(&before[..BUTTON_COUNT], &scratch[..BUTTON_COUNT]);
    if ui_unchanged(&before[..BUTTON_COUNT], &scratch[..BUTTON_COUNT]) {
        return None;
    }
    log_plc_ui_info(scratch);
    tx_state.arm_send_rhai(cmd)
}

/// Touch: run PLC scan for GUI / Rhai inputs — no CAN TX.
#[cfg(feature = "rhai")]
fn touch_cycle(plc: &mut Plc, scratch: &mut [u8]) {
    let _ = cycle_output(plc, scratch);
}

/// Touch press — latch inputs, one scan (no CAN).
#[cfg(feature = "rhai")]
pub fn on_touch_press(plc: &mut Plc, scratch: &mut [u8], index: u8) {
    touch_begin(index);
    touch_cycle(plc, scratch);
}

/// Touch release — clear inputs, one scan (no CAN).
#[cfg(feature = "rhai")]
pub fn on_touch_release(plc: &mut Plc, scratch: &mut [u8], held: u8, long_fired: bool) {
    touch_end(held, long_fired);
    touch_cycle(plc, scratch);
}

/// Switch held button — one scan (no CAN).
#[cfg(feature = "rhai")]
pub fn on_touch_switch(plc: &mut Plc, scratch: &mut [u8], from: u8, to: u8) {
    touch_switch(from, to);
    touch_cycle(plc, scratch);
}

/// Hold tick — long-press pulse + PLC scan every `command_repeat_ms` (no CAN).
#[cfg(feature = "rhai")]
pub fn on_touch_hold_tick(
    plc: &mut Plc,
    scratch: &mut [u8],
    held: u8,
    press_elapsed_ms: u64,
    long_fired: &mut bool,
) {
    touch_maybe_long(held, press_elapsed_ms, long_fired);
    touch_cycle(plc, scratch);
}

/// Release outside active hold loop.
#[cfg(feature = "rhai")]
pub fn on_idle_release(plc: &mut Plc, scratch: &mut [u8], tx_state: &PlcTxState) -> Option<u8> {
    if touch_hold::is_latched() || input_state::any_held() {
        return None;
    }
    set_active_button(None);
    tx_state.arm_send_rhai(cycle_output(plc, scratch))
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
        next,
    );
}

#[cfg(not(any(feature = "log", feature = "defmt")))]
fn log_minp_btn(_i: usize, _prev: bool, _next: bool) {}

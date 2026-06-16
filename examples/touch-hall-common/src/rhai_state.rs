//! PLC-style Rhai controller — single `cycle()` scan entry point.
//!
//! Rhai defines WHAT to send (`can_tx` bytes via natives). Rust decides WHEN (see [`crate::can_scheduler`]).

extern crate alloc;

use alloc::boxed::Box;

use rhai::{Array, AST, Dynamic, Engine, EvalAltResult, Scope, packages::BasicMathPackage, packages::Package};

use crate::can_bridge::{
    apply_button_bit, button_byte_bit, release_payload, set_bit_in_byte, TX_PAYLOAD_LEN,
};
use crate::can_input;
use crate::input_state;
use crate::{BUTTON_COUNT, CAN_TX_ID, LONG_PRESS_MS, MINP, MINP_LEVEL_ON_EVEN, MINP_RX_ID, STATE_SCRIPT, STATE_SCRIPT_ENABLED};

const OUT_UI: &str = "ui";

/// Active PLC scan buffer — set for the duration of `Plc::cycle()`.
static mut ACTIVE_CAN_TX: *mut [u8; TX_PAYLOAD_LEN] = core::ptr::null_mut();

fn with_active<F: FnOnce(&mut [u8; TX_PAYLOAD_LEN])>(f: F) {
    // SAFETY: `ACTIVE_CAN_TX` is only set from `Plc::cycle` on the PLC thread.
    let ptr = core::ptr::addr_of!(ACTIVE_CAN_TX);
    // SAFETY: single writer during cycle; readers are native fns on same thread.
    let active = unsafe { *ptr };
    if !active.is_null() {
        // SAFETY: pointer valid for the duration of `cycle()`.
        f(unsafe { &mut *active });
    }
}

fn can_tx_clear_buf() {
    with_active(|buf| *buf = release_payload());
}

fn can_tx_set_byte(index: i32, value: i32) {
    with_active(|buf| {
        if index >= 0 && index < TX_PAYLOAD_LEN as i32 {
            buf[index as usize] = value as u8;
        }
    });
}

/// Button index → set the payload byte that contains that button's bit.
fn can_tx_set(button_index: i32, byte_value: i32) {
    if let Some((byte, _)) = button_byte_bit(button_index as usize) {
        can_tx_set_byte(byte as i32, byte_value);
    }
}

/// Button index → read the payload byte that contains that button's bit.
fn can_tx(button_index: i32) -> i32 {
    if let Some((byte, _)) = button_byte_bit(button_index as usize) {
        can_tx_byte(byte as i32)
    } else {
        0
    }
}

/// Button index → set one-hot bit (OR into byte).
fn can_tx_on(button_index: i32) {
    can_tx_set_btn(button_index, true);
}

/// Button index → clear one-hot bit.
fn can_tx_off(button_index: i32) {
    can_tx_set_btn(button_index, false);
}

fn can_tx_byte(index: i32) -> i32 {
    if index >= 0 && index < TX_PAYLOAD_LEN as i32 {
        let mut byte = 0u8;
        with_active(|buf| byte = buf[index as usize]);
        byte as i32
    } else {
        0
    }
}

fn can_tx_set_bit(byte_index: i32, bit_index: i32, active: bool) {
    with_active(|buf| {
        let bi = byte_index as usize;
        if bi < TX_PAYLOAD_LEN {
            buf[bi] = set_bit_in_byte(buf[bi], bit_index as u8, active);
        }
    });
}

fn can_tx_set_btn(button_index: i32, active: bool) {
    with_active(|buf| apply_button_bit(buf, button_index as usize, active));
}

fn minp_in(index: usize) -> bool {
    let level = if MINP_LEVEL_ON_EVEN {
        index % 2 == 0
    } else {
        index % 2 != 0
    };
    if level {
        can_input::minp_raw(index)
    } else {
        can_input::minp_active(index)
    }
}

fn register_api(engine: &mut Engine) {
    engine.register_fn("button_indices", || -> Array {
        let mut indices = Array::new();
        for i in 0..BUTTON_COUNT {
            indices.push(Dynamic::from(i as i32));
        }
        indices
    });

    engine.register_fn("can_in_len", |id: i64| -> i64 { can_input::frame_len(id as u16) as i64 });
    engine.register_fn("can_in_byte", |id: i64, byte_index: i64| -> i64 {
        can_input::frame_byte(id as u16, byte_index as usize) as i64
    });
    engine.register_fn("can_in_bit", |id: i64, byte_index: i64, bit_index: i64| -> bool {
        can_input::frame_bit(id as u16, byte_index as usize, bit_index as u8)
    });

    engine.register_fn("minp_in", |index: i32| -> bool { minp_in(index as usize) });
    engine.register_fn("minp_can_id", |index: i32| -> i64 {
        MINP.get(index as usize)
            .map(|entry| entry.can_id as i64)
            .unwrap_or(-1)
    });
    engine.register_fn("minp_byte", |index: i32| -> i64 {
        MINP.get(index as usize)
            .map(|entry| entry.byte_index as i64)
            .unwrap_or(-1)
    });
    engine.register_fn("minp_bit", |index: i32| -> i64 {
        MINP.get(index as usize)
            .map(|entry| entry.bit_index as i64)
            .unwrap_or(-1)
    });
    engine.register_fn("minp_active_val", |index: i32| -> i64 {
        MINP.get(index as usize)
            .map(|entry| entry.active_value as i64)
            .unwrap_or(0)
    });
    engine.register_fn("minp_rx_id", || -> i64 { MINP_RX_ID as i64 });

    engine.register_fn("btn_held", |index: i32| -> bool { input_state::held(index as usize) });
    engine.register_fn("btn_short_take", |index: i32| -> bool { input_state::take_short(index as usize) });
    engine.register_fn("btn_long_take", |index: i32| -> bool { input_state::take_long(index as usize) });
    engine.register_fn("btn_release_take", |index: i32| -> bool { input_state::take_release(index as usize) });

    engine.register_fn("button_count", || -> i32 { BUTTON_COUNT as i32 });
    engine.register_fn("long_press_ms", || -> i64 { LONG_PRESS_MS as i64 });
    engine.register_fn("can_tx_id", || -> i64 { CAN_TX_ID as i64 });

    engine.register_fn("can_tx_len", || -> i64 { TX_PAYLOAD_LEN as i64 });
    engine.register_fn("can_tx_bit", |byte: i64, bit_index: i64, active: bool| -> i64 {
        set_bit_in_byte(byte as u8, bit_index as u8, active) as i64
    });
    engine.register_fn("can_tx_btn_byte", |index: i32| -> i64 {
        button_byte_bit(index as usize)
            .map(|(byte, _)| byte as i64)
            .unwrap_or(-1)
    });
    engine.register_fn("can_tx_btn_bit", |index: i32| -> i64 {
        button_byte_bit(index as usize)
            .map(|(_, bit)| bit as i64)
            .unwrap_or(-1)
    });
    engine.register_fn("can_tx_clear", can_tx_clear_buf);
    engine.register_fn("can_tx", can_tx);
    engine.register_fn("can_tx_set", can_tx_set);
    engine.register_fn("can_tx_on", can_tx_on);
    engine.register_fn("can_tx_off", can_tx_off);
    engine.register_fn("can_tx_byte", can_tx_byte);
    engine.register_fn("can_tx_set_byte", can_tx_set_byte);
    engine.register_fn("can_tx_set_bit", can_tx_set_bit);
    engine.register_fn("can_tx_set_btn", can_tx_set_btn);
}

/// PLC Rhai controller — call `cycle()` each scan; read outputs from script scope.
pub struct Plc {
    engine: Engine,
    scope: Scope<'static>,
    ast: AST,
    can_tx_out: [u8; TX_PAYLOAD_LEN],
}

impl Plc {
    pub fn new() -> Option<Self> {
        if !STATE_SCRIPT_ENABLED {
            return None;
        }

        let mut engine = Engine::new();
        BasicMathPackage::new().register_into_engine(&mut engine);
        register_api(&mut engine);

        let ast = match engine.compile(STATE_SCRIPT) {
            Ok(ast) => ast,
            Err(err) => {
                log_compile_error(err);
                return None;
            }
        };

        if !ast.iter_functions().any(|func| func.name == "cycle") {
            log_missing_cycle_fn();
            return None;
        }

        let mut scope = Scope::new();
        if let Err(err) = engine.run_ast_with_scope(&mut scope, &ast) {
            log_load_error(err);
            return None;
        }

        if ast.iter_functions().any(|func| func.name == "init") {
            if let Err(err) = engine.call_fn::<Dynamic>(&mut scope, &ast, "init", ()) {
                log_init_error(err);
                return None;
            }
        }

        let mut plc = Self {
            engine,
            scope,
            ast,
            can_tx_out: release_payload(),
        };
        if let Err(err) = plc.cycle() {
            log_cycle_error(err);
            return None;
        }
        Some(plc)
    }

    /// Run one PLC scan. Script must define `fn cycle()`.
    pub fn cycle(&mut self) -> Result<(), Box<EvalAltResult>> {
        self.can_tx_out = release_payload();
        let ptr = core::ptr::addr_of_mut!(ACTIVE_CAN_TX);
        // SAFETY: PLC runs on one thread; pointer cleared before return.
        unsafe {
            *ptr = &mut self.can_tx_out;
        }
        let result = self.engine.call_fn::<Dynamic>(&mut self.scope, &self.ast, "cycle", ());
        unsafe {
            *ptr = core::ptr::null_mut();
        }
        let _ = result?;
        Ok(())
    }

    /// UI highlight for one button (reads `ui` output array from script scope).
    pub fn ui_active(&self, index: usize) -> bool {
        let Some(ui) = self.scope.get_value::<Dynamic>(OUT_UI) else {
            return false;
        };
        let Some(array) = ui.clone().try_cast::<Array>() else {
            return false;
        };
        let Some(value) = array.get(index) else {
            return false;
        };
        value.clone().try_cast::<bool>().unwrap_or(false)
    }

    /// Copy all `ui` outputs into `out`.
    pub fn read_ui(&self, out: &mut [u8]) {
        let count = out.len().min(BUTTON_COUNT);
        let Some(ui) = self.scope.get_value::<Dynamic>(OUT_UI) else {
            return;
        };
        let Some(array) = ui.clone().try_cast::<Array>() else {
            return;
        };
        for (i, slot) in out.iter_mut().enumerate().take(count) {
            let active = if let Some(value) = array.get(i) {
                let value: Dynamic = value.clone();
                value.try_cast::<bool>().unwrap_or(false)
            } else {
                false
            };
            *slot = active as u8;
        }
    }

    /// CAN TX payload from this scan.
    pub fn can_tx_payload(&self) -> [u8; TX_PAYLOAD_LEN] {
        self.can_tx_out
    }
}

#[cfg(feature = "log")]
fn log_cycle_error(err: Box<EvalAltResult>) {
    log::error!("PLC script cycle error: {err}");
}

#[cfg(all(feature = "defmt", not(feature = "log")))]
fn log_cycle_error(_err: Box<EvalAltResult>) {
    defmt::warn!("PLC script cycle error");
}

#[cfg(not(any(feature = "log", feature = "defmt")))]
fn log_cycle_error(_err: Box<EvalAltResult>) {}

#[cfg(feature = "log")]
fn log_compile_error(err: rhai::ParseError) {
    log::error!("PLC script compile error: {err}");
}

#[cfg(feature = "log")]
fn log_load_error(err: Box<EvalAltResult>) {
    log::error!("PLC script load error: {err}");
}

#[cfg(feature = "log")]
fn log_init_error(err: Box<EvalAltResult>) {
    log::error!("PLC script init error: {err}");
}

#[cfg(feature = "log")]
fn log_missing_cycle_fn() {
    log::error!("PLC script must define fn cycle()");
}

#[cfg(all(feature = "defmt", not(feature = "log")))]
fn log_compile_error(_err: rhai::ParseError) {
    defmt::warn!("PLC script compile error");
}

#[cfg(all(feature = "defmt", not(feature = "log")))]
fn log_load_error(_err: Box<EvalAltResult>) {
    defmt::warn!("PLC script load error");
}

#[cfg(all(feature = "defmt", not(feature = "log")))]
fn log_init_error(_err: Box<EvalAltResult>) {
    defmt::warn!("PLC script init error");
}

#[cfg(all(feature = "defmt", not(feature = "log")))]
fn log_missing_cycle_fn() {
    defmt::warn!("PLC script must define fn cycle()");
}

#[cfg(not(any(feature = "log", feature = "defmt")))]
fn log_compile_error(_err: rhai::ParseError) {}

#[cfg(not(any(feature = "log", feature = "defmt")))]
fn log_load_error(_err: Box<EvalAltResult>) {}

#[cfg(not(any(feature = "log", feature = "defmt")))]
fn log_init_error(_err: Box<EvalAltResult>) {}

#[cfg(not(any(feature = "log", feature = "defmt")))]
fn log_missing_cycle_fn() {}

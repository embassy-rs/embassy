//! PLC-style Rhai controller — single `cycle()` scan entry point.
//!
//! Rhai defines WHAT to send (`can_tx`, `ui`). Rust decides WHEN (see [`crate::can_scheduler`]).

extern crate alloc;

use alloc::boxed::Box;

use rhai::{Array, AST, Dynamic, Engine, EvalAltResult, Scope, packages::BasicMathPackage, packages::Package};

use crate::can_input;
use crate::input_state;
use crate::{BUTTON_COUNT, CAN_TX_ID, LONG_PRESS_MS, MINP, MINP_RX_ID, STATE_SCRIPT, STATE_SCRIPT_ENABLED};

const OUT_UI: &str = "ui";
const OUT_CAN_TX: &str = "can_tx";

const CAN_TX_NONE: i64 = -1;
const CAN_TX_RELEASE: i64 = 255;

fn minp_in(index: usize) -> bool {
    can_input::minp_active(index)
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
}

fn seed_scope(scope: &mut Scope<'_>) {
    scope.push_constant("CAN_TX_NONE", -1i32);
    scope.push_constant("CAN_TX_RELEASE", 255i32);
    scope.push(OUT_CAN_TX, -1i32);
}

/// PLC Rhai controller — call `cycle()` each scan; read outputs from script scope.
pub struct Plc {
    engine: Engine,
    scope: Scope<'static>,
    ast: AST,
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
        seed_scope(&mut scope);
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

        let mut plc = Self { engine, scope, ast };
        if let Err(err) = plc.cycle() {
            log_cycle_error(err);
            return None;
        }
        Some(plc)
    }

    /// Run one PLC scan. Script must define `fn cycle()`.
    pub fn cycle(&mut self) -> Result<(), Box<EvalAltResult>> {
        let _ = self.engine.call_fn::<Dynamic>(&mut self.scope, &self.ast, "cycle", ())?;
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

    /// CAN TX command from this scan (`can_tx` output variable).
    pub fn can_tx(&self) -> Option<u8> {
        let value = self
            .scope
            .get_value::<Dynamic>(OUT_CAN_TX)
            .and_then(|v| v.as_int().ok())
            .unwrap_or(CAN_TX_NONE as i32) as i64;
        match value {
            CAN_TX_NONE => None,
            CAN_TX_RELEASE => Some(255),
            btn if (0..=254).contains(&(btn as i32)) => Some(btn as u8),
            _ => None,
        }
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

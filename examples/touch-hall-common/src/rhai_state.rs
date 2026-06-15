//! Rhai-driven button state machine for CAN feedback.
//!
//! Scripts define `button_active(index)` (and optionally `on_can_rx(id)`) to map
//! incoming CAN frames to per-button highlight state using logic expressions.

extern crate alloc;

use alloc::boxed::Box;
use core::ptr::NonNull;

use rhai::{
    AST, Dynamic, Engine, EvalAltResult, Scope, packages::BasicMathPackage, packages::Package,
};

use crate::{BUTTON_COUNT, MINP, MINP_COUNT, STATE_SCRIPT, STATE_SCRIPT_ENABLED};

const MAX_TRACKED_IDS: usize = 8;
const MAX_FRAME_BYTES: usize = 8;

#[derive(Clone, Copy)]
struct FrameSlot {
    id: u16,
    len: u8,
    data: [u8; MAX_FRAME_BYTES],
}

impl FrameSlot {
    const fn empty() -> Self {
        Self {
            id: 0,
            len: 0,
            data: [0; MAX_FRAME_BYTES],
        }
    }
}

struct FrameStore {
    slots: [FrameSlot; MAX_TRACKED_IDS],
}

impl FrameStore {
    const fn new() -> Self {
        Self {
            slots: [FrameSlot::empty(); MAX_TRACKED_IDS],
        }
    }

    fn store(&mut self, id: u16, data: &[u8]) {
        let len = data.len().min(MAX_FRAME_BYTES) as u8;
        if let Some(slot) = self.slots.iter_mut().find(|slot| slot.id == id) {
            slot.len = len;
            slot.data[..len as usize].copy_from_slice(&data[..len as usize]);
            return;
        }
        if let Some(slot) = self.slots.iter_mut().find(|slot| slot.id == 0) {
            slot.id = id;
            slot.len = len;
            slot.data[..len as usize].copy_from_slice(&data[..len as usize]);
        }
    }

    fn find(&self, id: u16) -> Option<&FrameSlot> {
        self.slots.iter().find(|slot| slot.id == id)
    }

    fn len(&self, id: u16) -> usize {
        self.find(id).map(|slot| slot.len as usize).unwrap_or(0)
    }

    fn byte(&self, id: u16, index: usize) -> u8 {
        self.find(id)
            .and_then(|slot| slot.data.get(index).copied())
            .unwrap_or(0)
    }

    fn bit(&self, id: u16, byte_index: usize, bit_index: u8) -> bool {
        if bit_index >= 8 {
            return false;
        }
        (self.byte(id, byte_index) >> bit_index) & 1 != 0
    }
}

fn register_api(engine: &mut Engine, frames: NonNull<FrameStore>) {
    engine.register_fn("can_len", move |id: i64| -> i64 {
        unsafe { frames.as_ref().len(id as u16) as i64 }
    });
    engine.register_fn("can_byte", move |id: i64, byte_index: i64| -> i64 {
        unsafe { frames.as_ref().byte(id as u16, byte_index as usize) as i64 }
    });
    engine.register_fn("can_bit", move |id: i64, byte_index: i64, bit_index: i64| -> bool {
        unsafe { frames.as_ref().bit(id as u16, byte_index as usize, bit_index as u8) }
    });
    engine.register_fn("minp_can_id", |index: i64| -> i64 {
        MINP.get(index as usize)
            .map(|entry| entry.can_id as i64)
            .unwrap_or(-1)
    });
    engine.register_fn("minp_byte", |index: i64| -> i64 {
        MINP.get(index as usize)
            .map(|entry| entry.byte_index as i64)
            .unwrap_or(-1)
    });
    engine.register_fn("minp_bit", |index: i64| -> i64 {
        MINP.get(index as usize)
            .map(|entry| entry.bit_index as i64)
            .unwrap_or(-1)
    });
    engine.register_fn("minp_active_val", |index: i64| -> i64 {
        MINP.get(index as usize)
            .map(|entry| entry.active_value as i64)
            .unwrap_or(0)
    });
    engine.register_fn("button_count", || -> i64 { BUTTON_COUNT as i64 });
    engine.register_fn("minp_count", || -> i64 { MINP_COUNT as i64 });
}

fn default_minp_bit(frames: &FrameStore, index: usize) -> bool {
    let Some(entry) = MINP.get(index) else {
        return false;
    };
    if entry.active_value == 0 {
        return false;
    }
    if entry.byte_index as usize >= frames.len(entry.can_id) {
        return false;
    }
    frames.bit(entry.can_id, entry.byte_index as usize, entry.bit_index)
}

/// Rhai state machine for CAN-driven button highlight state.
pub struct StateMachine {
    frames: FrameStore,
    engine: Engine,
    scope: Scope<'static>,
    ast: AST,
    has_on_can_rx: bool,
}

impl StateMachine {
    /// Build the state machine from the generated `STATE_SCRIPT`, if enabled.
    pub fn new() -> Option<Self> {
        if !STATE_SCRIPT_ENABLED {
            return None;
        }

        let mut frames = FrameStore::new();
        let frames_ptr = NonNull::from(&mut frames);
        let mut engine = Engine::new();
        BasicMathPackage::new().register_into_engine(&mut engine);
        register_api(&mut engine, frames_ptr);

        let ast = match engine.compile(STATE_SCRIPT) {
            Ok(ast) => ast,
            Err(err) => {
                log_compile_error(err);
                return None;
            }
        };

        let has_button_fn = ast
            .iter_functions()
            .any(|func| func.name == "button_active");
        if !has_button_fn {
            log_missing_button_fn();
            return None;
        }

        let has_on_can_rx = ast.iter_functions().any(|func| func.name == "on_can_rx");

        let mut scope = Scope::new();
        if let Err(err) = engine.run_ast_with_scope(&mut scope, &ast) {
            log_load_error(err);
            return None;
        }

        Some(Self {
            frames,
            engine,
            scope,
            ast,
            has_on_can_rx,
        })
    }

    /// Store the latest payload for `id` and run the optional `on_can_rx` hook.
    pub fn on_can_rx(&mut self, id: u16, data: &[u8]) {
        self.frames.store(id, data);
        if self.has_on_can_rx {
            let _ = self.engine.call_fn::<Dynamic>(
                &mut self.scope,
                &self.ast,
                "on_can_rx",
                (id as i64,),
            );
        }
    }

    /// Evaluate highlight state for one button via the Rhai `button_active` function.
    pub fn button_active(&mut self, index: usize) -> bool {
        if index >= BUTTON_COUNT {
            return default_minp_bit(&self.frames, index);
        }
        match self.engine.call_fn::<bool>(
            &mut self.scope,
            &self.ast,
            "button_active",
            (index as i64,),
        ) {
            Ok(active) => active,
            Err(_) => default_minp_bit(&self.frames, index),
        }
    }

    /// Evaluate all configured buttons into `out`.
    pub fn eval_button_status(&mut self, out: &mut [u8]) {
        let count = out.len().min(BUTTON_COUNT);
        for (i, slot) in out.iter_mut().enumerate().take(count) {
            *slot = self.button_active(i) as u8;
        }
    }
}

#[cfg(feature = "log")]
fn log_compile_error(err: rhai::ParseError) {
    log::error!("failed to compile state script: {err}");
}

#[cfg(feature = "log")]
fn log_load_error(err: Box<EvalAltResult>) {
    log::error!("failed to load state script: {err}");
}

#[cfg(feature = "log")]
fn log_missing_button_fn() {
    log::error!("state script must define fn button_active(index)");
}

#[cfg(feature = "defmt")]
fn log_compile_error(_err: rhai::ParseError) {
    defmt::warn!("failed to compile state script");
}

#[cfg(feature = "defmt")]
fn log_load_error(_err: Box<EvalAltResult>) {
    defmt::warn!("failed to load state script");
}

#[cfg(feature = "defmt")]
fn log_missing_button_fn() {
    defmt::warn!("state script must define fn button_active(index)");
}

#[cfg(not(any(feature = "log", feature = "defmt")))]
fn log_compile_error(_err: rhai::ParseError) {}

#[cfg(not(any(feature = "log", feature = "defmt")))]
fn log_load_error(_err: Box<EvalAltResult>) {}

#[cfg(not(any(feature = "log", feature = "defmt")))]
fn log_missing_button_fn() {}

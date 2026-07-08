//! JSON-driven Rhai node (PoC for CANbossTouch) on the STM32H573I-DK — with FDCAN.
//!
//! `json_node.json` describes ONE CANopen-style node the same way
//! CANbossTouch's `eds/network.json` describes the network: an object
//! dictionary (index/subindex/type/access/initial value) plus Rhai scripts
//! (`once` at boot, `cyclic` with a period). This firmware is fully generic —
//! all node behaviour lives in the JSON:
//!
//! 1. parse the embedded JSON (serde_json, no_std + alloc)
//! 2. build the object dictionary in RAM
//! 3. set up a Rhai engine with the OD + board API
//! 4. bring up FDCAN1 as configured in the JSON (`"can"` section)
//! 5. run the `once` scripts, then schedule the `cyclic` scripts
//!
//! Rhai API (see README):
//! - `od_read(index, sub)` / `od_write(index, sub, value)` — OD access by index
//! - `get("name")` / `set("name", value)` — OD access by datapoint name
//! - `od_dump()`, `node_id()`, `node_name()`, `uptime_ms()`, `sleep(ms)`
//! - `led(n, on)`, `leds(mask)`, `button()` — board I/O as process data
//!
//! CAN (H5 has FDCAN — this is why the PoC moved here from the WBA65):
//! - every OD value change is sent as a "TPDO" frame, COB-ID = tpdo base + node id
//!   payload: index u16 LE, sub u8, dtype u8 (0=int 1=f32 2=bool), value 4 B LE
//! - received frames on COB-ID = rpdo base + node id write into the OD with the
//!   same layout — only `rw`/`wo` entries are writable from the bus
//! - `"mode": "loopback"` works standalone (TX frames echo back and are logged);
//!   `"mode": "normal"` talks to a real bus via the on-board transceiver
//!
//! Board (STM32H573I-DK, MB1677):
//! - user LEDs (active-low): LD1 green PI9, LD2 orange PI8, LD3 red PF1, LD4 blue PF4
//! - user button PC13 (active-low, pull-up)
//! - FDCAN1 on PA11 (RX) / PA12 (TX), HSE 25 MHz crystal as FDCAN kernel clock

#![no_std]
#![no_main]

extern crate alloc;

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::cell::RefCell;
use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};

use defmt::{debug, error, info, unwrap, warn};
use embassy_executor::Spawner;
use embassy_futures::block_on;
use embassy_stm32::gpio::{Input, Level, Output, Pull, Speed};
use embassy_stm32::time::Hertz;
use embassy_stm32::{bind_interrupts, can, peripherals, rcc};
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::{Duration, Instant, Timer};
use embedded_alloc::LlffHeap as Heap;
use rhai::packages::{BasicIteratorPackage, BasicMathPackage, BasicStringPackage, MoreStringPackage, Package};
use rhai::{AST, Dynamic, Engine, EvalAltResult};
use serde::Deserialize;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    FDCAN1_IT0 => can::IT0InterruptHandler<peripherals::FDCAN1>;
    FDCAN1_IT1 => can::IT1InterruptHandler<peripherals::FDCAN1>;
});

#[global_allocator]
static HEAP: Heap = Heap::empty();
const HEAP_SIZE: usize = 128 * 1024;

/// Node description compiled into the firmware.
/// Edit the JSON and rebuild — no Rust changes needed for a different node.
const NODE_JSON: &str = include_str!("json_node.json");

/// Abort a runaway script (endless loop) after this many Rhai operations.
const MAX_SCRIPT_OPS: u64 = 500_000;

/// Board: LEDs are active-low, the user button is active-low (pull-up).
const LED_ACTIVE_LOW: bool = true;
const BUTTON_ACTIVE_LOW: bool = true;

// ---------------------------------------------------------------------------
// JSON schema
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct NodeConfig {
    node_id: u8,
    name: String,
    #[serde(default)]
    #[allow(dead_code)]
    description: String,
    #[serde(default)]
    can: Option<CanSection>,
    datapoints: Vec<DpConfig>,
    scripts: Vec<ScriptConfig>,
}

#[derive(Deserialize)]
struct CanSection {
    /// "loopback" (standalone demo), "normal" (real bus) or "off".
    mode: String,
    #[serde(default = "default_bitrate")]
    bitrate: u32,
    /// COB-ID bases; the node id is added (CANopen style: 0x180 + id, 0x200 + id).
    #[serde(default = "default_tpdo")]
    tpdo: String,
    #[serde(default = "default_rpdo")]
    rpdo: String,
}

fn default_bitrate() -> u32 {
    250_000
}
fn default_tpdo() -> String {
    "0x180".to_string()
}
fn default_rpdo() -> String {
    "0x200".to_string()
}

#[derive(Deserialize)]
struct DpConfig {
    /// Object dictionary index, hex string ("0x6200") or decimal number string.
    index: String,
    #[serde(default)]
    sub: u8,
    name: String,
    #[serde(rename = "type")]
    ty: DpType,
    #[serde(default)]
    access: Access,
    value: JsonValue,
}

#[derive(Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum DpType {
    Bool,
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    F32,
    Str,
}

impl DpType {
    fn name(self) -> &'static str {
        match self {
            Self::Bool => "bool",
            Self::U8 => "u8",
            Self::I8 => "i8",
            Self::U16 => "u16",
            Self::I16 => "i16",
            Self::U32 => "u32",
            Self::I32 => "i32",
            Self::F32 => "f32",
            Self::Str => "str",
        }
    }

    /// Value range for the integer types (clamped on write, like an OD limit).
    fn int_range(self) -> Option<(i32, i32)> {
        match self {
            Self::U8 => Some((0, u8::MAX as i32)),
            Self::I8 => Some((i8::MIN as i32, i8::MAX as i32)),
            Self::U16 => Some((0, u16::MAX as i32)),
            Self::I16 => Some((i16::MIN as i32, i16::MAX as i32)),
            // With Rhai `only_i32` the scripting side cannot express the full
            // u32 range; clamp to 0..=i32::MAX for the PoC.
            Self::U32 => Some((0, i32::MAX)),
            Self::I32 => Some((i32::MIN, i32::MAX)),
            _ => None,
        }
    }
}

#[derive(Deserialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
enum Access {
    Ro,
    Wo,
    #[default]
    Rw,
    Const,
}

impl Access {
    fn name(self) -> &'static str {
        match self {
            Self::Ro => "ro",
            Self::Wo => "wo",
            Self::Rw => "rw",
            Self::Const => "const",
        }
    }
}

/// Initial value in JSON — type is resolved against the datapoint's `type`.
#[derive(Deserialize)]
#[serde(untagged)]
enum JsonValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
}

#[derive(Deserialize)]
struct ScriptConfig {
    name: String,
    /// "once" (default when no period is given) or "cyclic".
    #[serde(default)]
    run: Option<String>,
    #[serde(default)]
    period_ms: Option<u32>,
    script: ScriptSource,
}

/// Rhai source as a single string or — much more readable in JSON — an array
/// of lines that gets joined with newlines.
#[derive(Deserialize)]
#[serde(untagged)]
enum ScriptSource {
    One(String),
    Lines(Vec<String>),
}

impl ScriptSource {
    fn join(&self) -> String {
        match self {
            Self::One(s) => s.clone(),
            Self::Lines(lines) => lines.join("\n"),
        }
    }
}

// ---------------------------------------------------------------------------
// Object dictionary (runtime)
// ---------------------------------------------------------------------------

#[derive(Clone, PartialEq)]
enum Value {
    Int(i32),
    Float(f32),
    Bool(bool),
    Str(String),
}

impl Value {
    fn from_json(v: &JsonValue) -> Self {
        match v {
            JsonValue::Bool(b) => Self::Bool(*b),
            JsonValue::Int(i) => Self::Int((*i).clamp(i32::MIN as i64, i32::MAX as i64) as i32),
            JsonValue::Float(f) => Self::Float(*f as f32),
            JsonValue::Str(s) => Self::Str(s.clone()),
        }
    }

    fn to_dynamic(&self) -> Dynamic {
        match self {
            Self::Int(i) => Dynamic::from(*i),
            Self::Float(f) => Dynamic::from(*f),
            Self::Bool(b) => Dynamic::from(*b),
            Self::Str(s) => Dynamic::from(s.clone()),
        }
    }

    fn to_log_string(&self) -> String {
        match self {
            Self::Int(i) => format!("{}", i),
            Self::Float(f) => format!("{}", f),
            Self::Bool(b) => format!("{}", b),
            Self::Str(s) => format!("\"{}\"", s),
        }
    }

    /// Wire encoding for the CAN "PDO" payload: (dtype, 4 bytes LE).
    fn to_wire(&self) -> Option<(u8, [u8; 4])> {
        match self {
            Self::Int(i) => Some((0, i.to_le_bytes())),
            Self::Float(f) => Some((1, f.to_le_bytes())),
            Self::Bool(b) => Some((2, [*b as u8, 0, 0, 0])),
            Self::Str(_) => None, // strings do not fit a classic frame; not sent
        }
    }

    fn from_wire(dtype: u8, raw: [u8; 4]) -> Self {
        match dtype {
            1 => Self::Float(f32::from_le_bytes(raw)),
            2 => Self::Bool(raw[0] != 0),
            _ => Self::Int(i32::from_le_bytes(raw)),
        }
    }
}

struct Datapoint {
    index: u16,
    sub: u8,
    name: String,
    ty: DpType,
    access: Access,
    value: Value,
}

/// Who is writing: scripts are device-internal (may update `ro` inputs),
/// the bus (RPDO) only gets `rw`/`wo` entries.
#[derive(Clone, Copy, PartialEq)]
enum Writer {
    Script,
    Bus,
}

/// The object dictionary, shared between Rhai closures, main and the CAN tasks.
static OD: Mutex<CriticalSectionRawMutex, RefCell<Vec<Datapoint>>> = Mutex::new(RefCell::new(Vec::new()));

static NODE_ID: AtomicU8 = AtomicU8::new(0);
static NODE_NAME: Mutex<CriticalSectionRawMutex, RefCell<String>> = Mutex::new(RefCell::new(String::new()));

fn od_with<R>(index: u16, sub: u8, f: impl FnOnce(&mut Datapoint) -> R) -> Option<R> {
    OD.lock(|cell| {
        let mut od = cell.borrow_mut();
        od.iter_mut().find(|dp| dp.index == index && dp.sub == sub).map(f)
    })
}

fn od_with_name<R>(name: &str, f: impl FnOnce(&mut Datapoint) -> R) -> Option<R> {
    OD.lock(|cell| {
        let mut od = cell.borrow_mut();
        od.iter_mut().find(|dp| dp.name == name).map(f)
    })
}

/// Coerce a script/JSON/bus value to the datapoint's declared type.
/// Integers are clamped to the type's range (with a warning), like OD limits.
fn coerce(ty: DpType, v: Value) -> Result<Value, String> {
    match ty {
        DpType::Bool => match v {
            Value::Bool(b) => Ok(Value::Bool(b)),
            Value::Int(i) => Ok(Value::Bool(i != 0)),
            _ => Err("expected bool".to_string()),
        },
        DpType::F32 => match v {
            Value::Float(f) => Ok(Value::Float(f)),
            Value::Int(i) => Ok(Value::Float(i as f32)),
            _ => Err("expected f32".to_string()),
        },
        DpType::Str => match v {
            Value::Str(s) => Ok(Value::Str(s)),
            Value::Int(i) => Ok(Value::Str(format!("{}", i))),
            Value::Float(f) => Ok(Value::Str(format!("{}", f))),
            Value::Bool(b) => Ok(Value::Str(format!("{}", b))),
        },
        _ => {
            let (min, max) = ty.int_range().unwrap();
            let i = match v {
                Value::Int(i) => i,
                Value::Bool(b) => b as i32,
                Value::Float(f) => f as i32,
                Value::Str(_) => return Err(format!("expected {}", ty.name())),
            };
            let clamped = i.clamp(min, max);
            if clamped != i {
                warn!("od: value {} clamped to {} ({=str})", i, clamped, ty.name());
            }
            Ok(Value::Int(clamped))
        }
    }
}

// "TPDO" queue: OD changes are sent on CAN by can_tx_task. try_send from
// write_dp — when the queue is full (bus stalled) changes are dropped with a
// warning instead of blocking the scripts.
struct Tpdo {
    index: u16,
    sub: u8,
    dtype: u8,
    raw: [u8; 4],
}
static CAN_TX_CHAN: Channel<CriticalSectionRawMutex, Tpdo, 32> = Channel::new();
static CAN_ON: AtomicBool = AtomicBool::new(false);

/// Write a datapoint; logs every change and queues a "TPDO" CAN frame.
fn write_dp(dp: &mut Datapoint, new: Value, writer: Writer) -> Result<(), String> {
    if dp.access == Access::Const {
        return Err(format!("od 0x{:04x}.{:02x} '{}' is const", dp.index, dp.sub, dp.name));
    }
    if writer == Writer::Bus && dp.access == Access::Ro {
        return Err(format!(
            "od 0x{:04x}.{:02x} '{}' is read-only from the bus",
            dp.index, dp.sub, dp.name
        ));
    }
    let coerced =
        coerce(dp.ty, new).map_err(|e| format!("od 0x{:04x}.{:02x} '{}': {}", dp.index, dp.sub, dp.name, e))?;
    if coerced != dp.value {
        info!(
            "{=str}",
            format!(
                "od 0x{:04x}.{:02x} {} = {} (was {})",
                dp.index,
                dp.sub,
                dp.name,
                coerced.to_log_string(),
                dp.value.to_log_string()
            )
            .as_str()
        );
        if CAN_ON.load(Ordering::Relaxed) {
            if let Some((dtype, raw)) = coerced.to_wire() {
                let msg = Tpdo {
                    index: dp.index,
                    sub: dp.sub,
                    dtype,
                    raw,
                };
                if CAN_TX_CHAN.try_send(msg).is_err() {
                    warn!("can: tx queue full, tpdo dropped");
                }
            }
        }
        dp.value = coerced;
    }
    Ok(())
}

fn dump_od() {
    OD.lock(|cell| {
        let od = cell.borrow();
        info!("object dictionary ({} entries):", od.len());
        for dp in od.iter() {
            info!(
                "{=str}",
                format!(
                    "  0x{:04x}.{:02x} {:<14} {:>5} {:<5} = {}",
                    dp.index,
                    dp.sub,
                    dp.name,
                    dp.ty.name(),
                    dp.access.name(),
                    dp.value.to_log_string()
                )
                .as_str()
            );
        }
    });
}

/// Parse an OD index / COB-ID given as "0x6200" (hex) or "25088" (decimal).
fn parse_index(s: &str) -> Option<u16> {
    let s = s.trim();
    if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        u16::from_str_radix(hex, 16).ok()
    } else {
        s.parse().ok()
    }
}

fn build_od(cfg: &NodeConfig) -> Vec<Datapoint> {
    let mut od: Vec<Datapoint> = Vec::with_capacity(cfg.datapoints.len());
    for d in &cfg.datapoints {
        let Some(index) = parse_index(&d.index) else {
            defmt::panic!("json_node.json: invalid index '{=str}'", d.index.as_str());
        };
        if od.iter().any(|dp| dp.index == index && dp.sub == d.sub) {
            defmt::panic!("json_node.json: duplicate index 0x{=u16:04x}.{=u8:02x}", index, d.sub);
        }
        if od.iter().any(|dp| dp.name == d.name) {
            defmt::panic!("json_node.json: duplicate name '{=str}'", d.name.as_str());
        }
        let value = match coerce(d.ty, Value::from_json(&d.value)) {
            Ok(v) => v,
            Err(e) => defmt::panic!("json_node.json: '{=str}': {=str}", d.name.as_str(), e.as_str()),
        };
        od.push(Datapoint {
            index,
            sub: d.sub,
            name: d.name.clone(),
            ty: d.ty,
            access: d.access,
            value,
        });
    }
    od
}

// ---------------------------------------------------------------------------
// Rhai API
// ---------------------------------------------------------------------------

type RhaiResult<T> = Result<T, alloc::boxed::Box<EvalAltResult>>;

fn od_write_rhai(index: i32, sub: i32, v: Value) -> RhaiResult<i32> {
    match od_with(index as u16, sub as u8, |dp| write_dp(dp, v, Writer::Script)) {
        None => Err(format!("od_write: 0x{:04x}.{:02x} not in object dictionary", index, sub).into()),
        Some(Err(e)) => Err(e.into()),
        Some(Ok(())) => Ok(0),
    }
}

fn set_rhai(name: &str, v: Value) -> RhaiResult<i32> {
    match od_with_name(name, |dp| write_dp(dp, v, Writer::Script)) {
        None => Err(format!("set: '{}' not in object dictionary", name).into()),
        Some(Err(e)) => Err(e.into()),
        Some(Ok(())) => Ok(0),
    }
}

// User LEDs, shared between the Rhai closures and main.
static LEDS: Mutex<CriticalSectionRawMutex, RefCell<Vec<Output<'static>>>> = Mutex::new(RefCell::new(Vec::new()));
// User button (read by button() in scripts).
static BUTTON: Mutex<CriticalSectionRawMutex, RefCell<Option<Input<'static>>>> = Mutex::new(RefCell::new(None));

fn led_apply(index: i32, on: bool) {
    LEDS.lock(|cell| {
        if let Some(pin) = cell.borrow_mut().get_mut(index as usize) {
            let level = if on == LED_ACTIVE_LOW { Level::Low } else { Level::High };
            pin.set_level(level);
        }
    });
}

fn leds_apply(mask: i32) {
    LEDS.lock(|cell| {
        for (i, pin) in cell.borrow_mut().iter_mut().enumerate() {
            let on = (mask >> i) & 1 != 0;
            let level = if on == LED_ACTIVE_LOW { Level::Low } else { Level::High };
            pin.set_level(level);
        }
    });
}

fn button_read() -> i32 {
    BUTTON.lock(|cell| {
        cell.borrow()
            .as_ref()
            .map(|b| {
                let pressed = if BUTTON_ACTIVE_LOW { b.is_low() } else { b.is_high() };
                pressed as i32
            })
            .unwrap_or(0)
    })
}

fn register_api(engine: &mut Engine) {
    // Object dictionary — by index/subindex.
    engine.register_fn("od_read", |index: i32, sub: i32| -> RhaiResult<Dynamic> {
        od_with(index as u16, sub as u8, |dp| dp.value.to_dynamic())
            .ok_or_else(|| format!("od_read: 0x{:04x}.{:02x} not in object dictionary", index, sub).into())
    });
    engine.register_fn("od_write", |i: i32, s: i32, v: i32| od_write_rhai(i, s, Value::Int(v)));
    engine.register_fn("od_write", |i: i32, s: i32, v: bool| od_write_rhai(i, s, Value::Bool(v)));
    engine.register_fn("od_write", |i: i32, s: i32, v: f32| od_write_rhai(i, s, Value::Float(v)));
    engine.register_fn("od_write", |i: i32, s: i32, v: &str| od_write_rhai(i, s, Value::Str(v.to_string())));

    // Object dictionary — by datapoint name.
    engine.register_fn("get", |name: &str| -> RhaiResult<Dynamic> {
        od_with_name(name, |dp| dp.value.to_dynamic())
            .ok_or_else(|| format!("get: '{}' not in object dictionary", name).into())
    });
    engine.register_fn("set", |name: &str, v: i32| set_rhai(name, Value::Int(v)));
    engine.register_fn("set", |name: &str, v: bool| set_rhai(name, Value::Bool(v)));
    engine.register_fn("set", |name: &str, v: f32| set_rhai(name, Value::Float(v)));
    engine.register_fn("set", |name: &str, v: &str| set_rhai(name, Value::Str(v.to_string())));

    engine.register_fn("od_dump", || -> i32 {
        dump_od();
        0
    });

    // Node identity + time.
    engine.register_fn("node_id", || -> i32 { NODE_ID.load(Ordering::Relaxed) as i32 });
    engine.register_fn("node_name", || -> String { NODE_NAME.lock(|c| c.borrow().clone()) });
    engine.register_fn("uptime_ms", || -> i32 { Instant::now().as_millis() as i32 });
    engine.register_fn("sleep", |ms: i32| -> i32 {
        let ms = ms.clamp(0, 10_000) as u64;
        block_on(Timer::after_millis(ms));
        0
    });

    // Board I/O (process data): LEDs + user button.
    engine.register_fn("led", |index: i32, on: bool| -> i32 {
        led_apply(index, on);
        if on { 1 } else { 0 }
    });
    engine.register_fn("led", |index: i32, on: i32| -> i32 {
        led_apply(index, on != 0);
        if on != 0 { 1 } else { 0 }
    });
    engine.register_fn("leds", |mask: i32| -> i32 {
        leds_apply(mask);
        mask
    });
    engine.register_fn("button", || -> i32 { button_read() });

    engine.on_print(|s| info!("[script] {=str}", s));
    engine.on_debug(|s, _src, _pos| defmt::debug!("[script] {=str}", s));
}

// ---------------------------------------------------------------------------
// CAN tasks
// ---------------------------------------------------------------------------

#[embassy_executor::task]
async fn can_tx_task(mut tx: can::CanTx<'static>, tpdo_id: u16) {
    loop {
        let pdo = CAN_TX_CHAN.receive().await;
        let mut data = [0u8; 8];
        data[0..2].copy_from_slice(&pdo.index.to_le_bytes());
        data[2] = pdo.sub;
        data[3] = pdo.dtype;
        data[4..8].copy_from_slice(&pdo.raw);
        let frame = unwrap!(can::frame::Frame::new_standard(tpdo_id, &data));
        _ = tx.write(&frame).await;
        debug!(
            "{=str}",
            format!("can: tpdo 0x{:03x} -> 0x{:04x}.{:02x}", tpdo_id, pdo.index, pdo.sub).as_str()
        );
    }
}

#[embassy_executor::task]
async fn can_rx_task(mut rx: can::CanRx<'static>, rpdo_id: u16) {
    loop {
        match rx.read().await {
            Ok(envelope) => {
                let (frame, _ts) = envelope.parts();
                let id = match frame.id() {
                    embedded_can::Id::Standard(sid) => sid.as_raw(),
                    embedded_can::Id::Extended(_) => continue,
                };
                if id != rpdo_id {
                    // In loopback mode our own TPDOs come back here — ignore.
                    debug!("can: rx 0x{=u16:03x} ignored (not rpdo)", id);
                    continue;
                }
                let data = frame.data();
                if data.len() < 8 {
                    warn!("can: rpdo 0x{=u16:03x} too short ({} bytes)", id, data.len());
                    continue;
                }
                let index = u16::from_le_bytes([data[0], data[1]]);
                let sub = data[2];
                let value = Value::from_wire(data[3], [data[4], data[5], data[6], data[7]]);
                match od_with(index, sub, |dp| write_dp(dp, value, Writer::Bus)) {
                    None => warn!("can: rpdo for unknown od 0x{=u16:04x}.{=u8:02x}", index, sub),
                    Some(Err(e)) => warn!("can: rpdo rejected: {=str}", e.as_str()),
                    Some(Ok(())) => {}
                }
            }
            Err(err) => {
                warn!("can: bus error {:?}", err);
                Timer::after_millis(100).await;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Main: parse JSON → build OD → bring up CAN → run scripts
// ---------------------------------------------------------------------------

struct CyclicScript {
    name: String,
    period: Duration,
    next: Instant,
    ast: AST,
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // HSE 25 MHz crystal as FDCAN kernel clock (like examples/stm32h5 can.rs).
    let mut config = embassy_stm32::Config::default();
    config.rcc.hse = Some(rcc::Hse {
        freq: Hertz(25_000_000),
        mode: rcc::HseMode::Oscillator,
    });
    config.rcc.mux.fdcan12sel = rcc::mux::Fdcansel::Hse;
    let p = embassy_stm32::init(config);

    {
        use core::mem::MaybeUninit;
        static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe { HEAP.init(core::ptr::addr_of!(HEAP_MEM) as usize, HEAP_SIZE) }
    }

    info!("json_node: JSON-driven Rhai node PoC (STM32H573I-DK)");

    // Board I/O: LD1 green PI9, LD2 orange PI8, LD3 red PF1, LD4 blue PF4 (active-low),
    // user button PC13 (active-low, pull-up).
    LEDS.lock(|cell| {
        let mut leds = cell.borrow_mut();
        leds.push(Output::new(p.PI9, Level::High, Speed::Low));
        leds.push(Output::new(p.PI8, Level::High, Speed::Low));
        leds.push(Output::new(p.PF1, Level::High, Speed::Low));
        leds.push(Output::new(p.PF4, Level::High, Speed::Low));
    });
    BUTTON.lock(|cell| {
        cell.replace(Some(Input::new(p.PC13, Pull::Up)));
    });

    // 1. Parse the node description.
    let cfg: NodeConfig = match serde_json::from_str(NODE_JSON) {
        Ok(cfg) => cfg,
        Err(e) => defmt::panic!("json_node.json: parse error: {=str}", format!("{}", e).as_str()),
    };
    NODE_ID.store(cfg.node_id, Ordering::Relaxed);
    NODE_NAME.lock(|c| *c.borrow_mut() = cfg.name.clone());
    info!(
        "node '{=str}' (id {}): {} datapoints, {} scripts",
        cfg.name.as_str(),
        cfg.node_id,
        cfg.datapoints.len(),
        cfg.scripts.len()
    );

    // 2. Build the object dictionary.
    let od = build_od(&cfg);
    OD.lock(|cell| *cell.borrow_mut() = od);

    // 3. FDCAN1 as configured in the JSON.
    if let Some(can_cfg) = &cfg.can {
        if can_cfg.mode != "off" {
            let tpdo_base = parse_index(&can_cfg.tpdo).unwrap_or(0x180);
            let rpdo_base = parse_index(&can_cfg.rpdo).unwrap_or(0x200);
            let tpdo_id = tpdo_base + cfg.node_id as u16;
            let rpdo_id = rpdo_base + cfg.node_id as u16;

            let mut configurator = can::CanConfigurator::new(p.FDCAN1, p.PA11, p.PA12, Irqs);
            configurator.set_bitrate(can_cfg.bitrate);
            let can = match can_cfg.mode.as_str() {
                "loopback" => configurator.into_internal_loopback_mode(),
                "normal" => configurator.into_normal_mode(),
                other => defmt::panic!("json_node.json: unknown can mode '{=str}'", other),
            };
            let (tx, rx, _props) = can.split();
            CAN_ON.store(true, Ordering::Relaxed);
            spawner.spawn(unwrap!(can_tx_task(tx, tpdo_id)));
            spawner.spawn(unwrap!(can_rx_task(rx, rpdo_id)));
            info!(
                "{=str}",
                format!(
                    "can: {} @ {} bit/s, tpdo 0x{:03x}, rpdo 0x{:03x}",
                    can_cfg.mode, can_cfg.bitrate, tpdo_id, rpdo_id
                )
                .as_str()
            );
        } else {
            info!("can: off");
        }
    } else {
        info!("can: not configured");
    }

    // 4. Rhai engine with the same slim package set as the WBA playground.
    let mut engine = Engine::new_raw();
    BasicMathPackage::new().register_into_engine(&mut engine);
    BasicIteratorPackage::new().register_into_engine(&mut engine);
    BasicStringPackage::new().register_into_engine(&mut engine);
    MoreStringPackage::new().register_into_engine(&mut engine);
    engine.set_max_operations(MAX_SCRIPT_OPS);
    register_api(&mut engine);

    // 5. Run `once` scripts, compile `cyclic` scripts.
    let mut cyclics: Vec<CyclicScript> = Vec::new();
    let now = Instant::now();
    for s in &cfg.scripts {
        let cyclic = match s.run.as_deref() {
            Some("cyclic") => true,
            Some("once") => false,
            None => s.period_ms.is_some(),
            Some(other) => {
                defmt::panic!("script '{=str}': unknown run mode '{=str}'", s.name.as_str(), other)
            }
        };
        let src = s.script.join();
        if cyclic {
            let period = Duration::from_millis(s.period_ms.unwrap_or(1000).max(10) as u64);
            match engine.compile(&src) {
                Ok(ast) => cyclics.push(CyclicScript {
                    name: s.name.clone(),
                    period,
                    next: now,
                    ast,
                }),
                Err(e) => error!(
                    "script '{=str}': compile error: {=str}",
                    s.name.as_str(),
                    format!("{}", e).as_str()
                ),
            }
        } else {
            info!("running once script '{=str}'", s.name.as_str());
            if let Err(e) = engine.run(&src) {
                error!("script '{=str}': {=str}", s.name.as_str(), format!("{}", e).as_str());
            }
        }
    }

    if cyclics.is_empty() {
        info!("no cyclic scripts — idle");
        loop {
            Timer::after_secs(3600).await;
        }
    }
    info!("scheduler: {} cyclic scripts, heap free {} KB", cyclics.len(), HEAP.free() / 1024);

    // 6. Cooperative fixed-rate scheduler (PLC-style: scripts run to completion).
    loop {
        let now = Instant::now();
        let mut next_due = now + Duration::from_secs(3600);
        for c in cyclics.iter_mut() {
            if now >= c.next {
                if let Err(e) = engine.run_ast(&c.ast) {
                    error!("script '{=str}': {=str}", c.name.as_str(), format!("{}", e).as_str());
                }
                c.next += c.period;
                if c.next <= now {
                    // Fell behind (long script) — skip missed cycles instead of bursting.
                    c.next = now + c.period;
                }
            }
            if c.next < next_due {
                next_due = c.next;
            }
        }
        Timer::at(next_due).await;
    }
}

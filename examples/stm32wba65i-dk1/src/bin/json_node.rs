//! JSON-driven Rhai node (PoC for CANbossTouch) — no LVGL, no BLE.
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
//! 4. run the `once` scripts, then schedule the `cyclic` scripts
//!
//! Rhai API (see README):
//! - `od_read(index, sub)` / `od_write(index, sub, value)` — OD access by index
//! - `get("name")` / `set("name", value)` — OD access by datapoint name
//! - `od_dump()`, `node_id()`, `node_name()`, `uptime_ms()`, `sleep(ms)`
//! - `led(n, on)`, `rgb(r, g, b)`, `joy()` — board I/O as process data
//!
//! Every OD value change is logged over defmt (a simulated PDO). `access`
//! describes the future bus view (SDO/PDO): scripts are device-internal and
//! may write `ro` entries — that is how a device updates its own inputs —
//! only `const` is rejected.

#![no_std]
#![no_main]

extern crate alloc;

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::cell::RefCell;
use core::sync::atomic::{AtomicU8, Ordering};

use defmt::{error, info, unwrap, warn};
use embassy_executor::Spawner;
use embassy_futures::block_on;
use embassy_stm32::adc::{Adc, adc4};
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::{Peri, peripherals};
use embassy_stm32wba65i_dk1_examples::board::{JoyDir, LedBank, LedId};
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_time::{Duration, Instant, Timer};
use embedded_alloc::LlffHeap as Heap;
use rhai::packages::{BasicIteratorPackage, BasicMathPackage, BasicStringPackage, MoreStringPackage, Package};
use rhai::{AST, Dynamic, Engine, EvalAltResult};
use serde::Deserialize;
use {defmt_rtt as _, panic_probe as _};

#[global_allocator]
static HEAP: Heap = Heap::empty();
const HEAP_SIZE: usize = 128 * 1024;

/// Node description compiled into the firmware.
/// Edit the JSON and rebuild — no Rust changes needed for a different node.
const NODE_JSON: &str = include_str!("json_node.json");

/// Abort a runaway script (endless loop) after this many Rhai operations.
const MAX_SCRIPT_OPS: u64 = 500_000;

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
    datapoints: Vec<DpConfig>,
    scripts: Vec<ScriptConfig>,
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
}

struct Datapoint {
    index: u16,
    sub: u8,
    name: String,
    ty: DpType,
    access: Access,
    value: Value,
}

/// The object dictionary, shared between the Rhai closures and main.
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

/// Coerce a script/JSON value to the datapoint's declared type.
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

/// Write a datapoint; logs every change (the "simulated PDO").
/// Scripts are device-internal: `ro`/`wo`/`rw` all writable, `const` rejected.
fn write_dp(dp: &mut Datapoint, new: Value) -> Result<(), String> {
    if dp.access == Access::Const {
        return Err(format!("od 0x{:04x}.{:02x} '{}' is const", dp.index, dp.sub, dp.name));
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

/// Parse an OD index given as "0x6200" (hex) or "25088" (decimal).
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
    match od_with(index as u16, sub as u8, |dp| write_dp(dp, v)) {
        None => Err(format!("od_write: 0x{:04x}.{:02x} not in object dictionary", index, sub).into()),
        Some(Err(e)) => Err(e.into()),
        Some(Ok(())) => Ok(0),
    }
}

fn set_rhai(name: &str, v: Value) -> RhaiResult<i32> {
    match od_with_name(name, |dp| write_dp(dp, v)) {
        None => Err(format!("set: '{}' not in object dictionary", name).into()),
        Some(Err(e)) => Err(e.into()),
        Some(Ok(())) => Ok(0),
    }
}

// Latest joystick direction, updated by input_task (read by joy()).
static JOY_STATE: AtomicU8 = AtomicU8::new(JoyDir::None as u8);

// RGB LED bank shared between the Rhai closures and main.
static LED_BANK: Mutex<CriticalSectionRawMutex, RefCell<Option<LedBank>>> = Mutex::new(RefCell::new(None));

fn led_apply(index: i32, on: bool) {
    if let Some(id) = LedId::from_i32(index) {
        LED_BANK.lock(|cell| {
            if let Some(bank) = cell.borrow_mut().as_mut() {
                bank.set(id, on);
            }
        });
    }
}

fn rgb_apply(r: bool, g: bool, b: bool) {
    LED_BANK.lock(|cell| {
        if let Some(bank) = cell.borrow_mut().as_mut() {
            bank.set_rgb(r, g, b);
        }
    });
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

    // Board I/O (process data): LEDs + joystick.
    engine.register_fn("led", |index: i32, on: bool| -> i32 {
        led_apply(index, on);
        if on { 1 } else { 0 }
    });
    engine.register_fn("led", |index: i32, on: i32| -> i32 {
        led_apply(index, on != 0);
        if on != 0 { 1 } else { 0 }
    });
    engine.register_fn("rgb", |r: bool, g: bool, b: bool| -> i32 {
        rgb_apply(r, g, b);
        0
    });
    engine.register_fn("rgb", |r: i32, g: i32, b: i32| -> i32 {
        rgb_apply(r != 0, g != 0, b != 0);
        0
    });
    engine.register_fn("joy", || -> i32 { JOY_STATE.load(Ordering::Relaxed) as i32 });

    engine.on_print(|s| info!("[script] {=str}", s));
    engine.on_debug(|s, _src, _pos| defmt::debug!("[script] {=str}", s));
}

// ---------------------------------------------------------------------------
// Tasks
// ---------------------------------------------------------------------------

#[embassy_executor::task]
async fn input_task(mut adc: Adc<'static, peripherals::ADC4>, mut joy_pin: Peri<'static, peripherals::PA3>) {
    adc.set_resolution_adc4(adc4::Resolution::Bits12);
    let max = adc4::resolution_to_max_count(adc4::Resolution::Bits12) as u16;
    loop {
        let raw = adc.blocking_read(&mut joy_pin, adc4::SampleTime::Cycles15);
        let dir = JoyDir::from_raw(raw, max);
        JOY_STATE.store(dir as u8, Ordering::Relaxed);
        Timer::after_millis(80).await;
    }
}

// ---------------------------------------------------------------------------
// Main: parse JSON → build OD → run scripts
// ---------------------------------------------------------------------------

struct CyclicScript {
    name: String,
    period: Duration,
    next: Instant,
    ast: AST,
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    {
        use core::mem::MaybeUninit;
        static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe { HEAP.init(core::ptr::addr_of!(HEAP_MEM) as usize, HEAP_SIZE) }
    }

    info!("json_node: JSON-driven Rhai node PoC");

    // Board I/O: LEDs (active-low) + joystick ADC.
    LED_BANK.lock(|cell| {
        cell.replace(Some(LedBank::new(
            Output::new(p.PD8, Level::High, Speed::Low),
            Output::new(p.PD9, Level::High, Speed::Low),
            Output::new(p.PB10, Level::High, Speed::Low),
        )));
    });
    let adc = Adc::new_adc4(p.ADC4);
    spawner.spawn(unwrap!(input_task(adc, p.PA3)));

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

    // 3. Rhai engine with the same slim package set as ble_rhai.
    let mut engine = Engine::new_raw();
    BasicMathPackage::new().register_into_engine(&mut engine);
    BasicIteratorPackage::new().register_into_engine(&mut engine);
    BasicStringPackage::new().register_into_engine(&mut engine);
    MoreStringPackage::new().register_into_engine(&mut engine);
    engine.set_max_operations(MAX_SCRIPT_OPS);
    register_api(&mut engine);

    // 4. Run `once` scripts, compile `cyclic` scripts.
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

    // 5. Cooperative fixed-rate scheduler (PLC-style: scripts run to completion).
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

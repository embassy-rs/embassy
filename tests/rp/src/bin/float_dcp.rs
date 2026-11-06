//! Checks the `dcp-intrinsics` `f64` routines against results computed by the compiler at compile time.
#![no_std]
#![no_main]
#[cfg(feature = "rp235xb")]
teleprobe_meta::target!(b"pimoroni-pico-plus-2");

use core::hint::black_box;
use core::sync::atomic::{AtomicU32, Ordering};

use cortex_m::peripheral::syst::SystClkSource;
use cortex_m_rt::exception;
use defmt::{assert, assert_eq, *};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_rp::multicore::{Stack, spawn_core1};
use panic_probe as _;

// Avoid denormal inputs and results: the DCP flushes them to zero, unlike rustc's const evaluation.
const CASES: [(f64, f64); 24] = [
    (0.0, 0.0),
    (-0.0, 0.0),
    (1.0, 1.0),
    (1.0, -1.0),
    (1.0, 3.0),
    (2.0, 3.0),
    (-7.5, 0.25),
    (core::f64::consts::PI, core::f64::consts::E),
    (1.0e300, 1.0e10),
    (1.0e-300, 1.0e10),
    (1.0e-150, 1.0e150),
    (1.7976931348623157e308, 2.0),
    (123456789.12345679, -0.000123456789),
    (0.1, 0.2),
    (1.0 / 3.0, 3.0),
    (9007199254740993.0, 1.0),
    (f64::INFINITY, 1.0),
    (f64::NEG_INFINITY, f64::INFINITY),
    (f64::INFINITY, f64::INFINITY),
    (0.0, f64::INFINITY),
    (f64::NAN, 1.0),
    (1.0, f64::NAN),
    (-2147483648.5, 4294967295.75),
    (2147483647.9, -1.5),
];

const N: usize = CASES.len();

macro_rules! const_map {
    (|$a:ident, $b:ident| $e:expr, $ty:ty, $zero:expr) => {{
        let mut out: [$ty; N] = [$zero; N];
        let mut i = 0;
        while i < N {
            let ($a, $b) = CASES[i];
            out[i] = $e;
            i += 1;
        }
        out
    }};
}

const ADD: [f64; N] = const_map!(|a, b| a + b, f64, 0.0);
const SUB: [f64; N] = const_map!(|a, b| a - b, f64, 0.0);
const MUL: [f64; N] = const_map!(|a, b| a * b, f64, 0.0);
const DIV: [f64; N] = const_map!(|a, b| a / b, f64, 0.0);
const CMP: [[bool; 6]; N] = const_map!(
    |a, b| [a == b, a < b, a <= b, a > b, a >= b, a.is_nan() || b.is_nan()],
    [bool; 6],
    [false; 6]
);
const TO_F32: [f32; N] = const_map!(|a, _b| a as f32, f32, 0.0);
const TO_I32: [i32; N] = const_map!(|a, _b| a as i32, i32, 0);
const TO_U32: [u32; N] = const_map!(|a, _b| a as u32, u32, 0);

const INTS: [i32; 6] = [0, 1, -1, 123456789, i32::MIN, i32::MAX];

fn same(a: f64, b: f64) -> bool {
    a.to_bits() == b.to_bits() || (a.is_nan() && b.is_nan())
}

fn check_all() {
    for i in 0..N {
        let (a, b) = (black_box(CASES[i].0), black_box(CASES[i].1));
        assert!(same(a + b, ADD[i]), "add {}: {} + {} = {}", i, a, b, a + b);
        assert!(same(a - b, SUB[i]), "sub {}: {} - {} = {}", i, a, b, a - b);
        assert!(same(a * b, MUL[i]), "mul {}: {} * {} = {}", i, a, b, a * b);
        assert!(same(a / b, DIV[i]), "div {}: {} / {} = {}", i, a, b, a / b);
        assert_eq!(
            [a == b, a < b, a <= b, a > b, a >= b, a.is_nan() || b.is_nan()],
            CMP[i],
            "cmp {}",
            i
        );
        let f = a as f32;
        assert!(
            f.to_bits() == TO_F32[i].to_bits() || (f.is_nan() && TO_F32[i].is_nan()),
            "d2f {}",
            i
        );
        assert_eq!(a as i32, TO_I32[i], "d2i {}", i);
        assert_eq!(a as u32, TO_U32[i], "d2u {}", i);
    }
    for &i in INTS.iter() {
        let i = black_box(i);
        assert_eq!(black_box(i as f64) as i32, i);
        assert_eq!(black_box(i as u32 as f64) as u32, i as u32);
    }
}

/// A short computation whose result is known, to run while other code is using the DCP.
fn interrupt_work() -> bool {
    let x = black_box(1.0f64 / 3.0);
    same(x * black_box(3.0) - black_box(1.0), 0.0) && same(black_box(10.0f64) / black_box(4.0), 2.5)
}

static SYSTICK_RUNS: AtomicU32 = AtomicU32::new(0);
static SYSTICK_PREEMPTED: AtomicU32 = AtomicU32::new(0);
static SYSTICK_ERRORS: AtomicU32 = AtomicU32::new(0);

#[exception]
fn SysTick() {
    // PCMP: N is set if this interrupt pre-empted a canned DCP sequence.
    let apsr: u32;
    unsafe { core::arch::asm!("mrc2 p4, #0, apsr_nzcv, c0, c0, #1", "mrs {0}, apsr", out(reg) apsr) };
    if apsr & (1 << 31) != 0 {
        SYSTICK_PREEMPTED.fetch_add(1, Ordering::Relaxed);
    }
    if !interrupt_work() {
        SYSTICK_ERRORS.fetch_add(1, Ordering::Relaxed);
    }
    SYSTICK_RUNS.fetch_add(1, Ordering::Relaxed);
}

static mut CORE1_STACK: Stack<4096> = Stack::new();
static CORE1_RESULT: AtomicU32 = AtomicU32::new(0);

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    info!("Hello World!");

    check_all();
    info!("results match");

    // Fire SysTick at an odd interval, so it lands at many different points of the canned
    // sequences, and check that both the interrupted code and the interrupt get correct results.
    let mut cp = unsafe { cortex_m::Peripherals::steal() };
    cp.SYST.set_clock_source(SystClkSource::Core);
    cp.SYST.set_reload(1009);
    cp.SYST.clear_current();
    cp.SYST.enable_interrupt();
    cp.SYST.enable_counter();
    for _ in 0..200 {
        check_all();
    }
    cp.SYST.disable_counter();
    cp.SYST.disable_interrupt();
    let runs = SYSTICK_RUNS.load(Ordering::Relaxed);
    let preempted = SYSTICK_PREEMPTED.load(Ordering::Relaxed);
    info!("systick ran {} times, {} mid-sequence", runs, preempted);
    assert_eq!(SYSTICK_ERRORS.load(Ordering::Relaxed), 0);
    assert!(preempted > 0);

    // Core 1 needs the DCP enabled separately.
    spawn_core1(
        p.CORE1,
        unsafe { &mut *core::ptr::addr_of_mut!(CORE1_STACK) },
        move || {
            check_all();
            CORE1_RESULT.store(1, Ordering::Release);
            loop {
                cortex_m::asm::wfe();
            }
        },
    );
    while CORE1_RESULT.load(Ordering::Acquire) == 0 {
        core::hint::spin_loop();
    }

    info!("Test OK");
    cortex_m::asm::bkpt();
}

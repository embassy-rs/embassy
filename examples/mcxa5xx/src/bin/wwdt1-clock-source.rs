//! WWDT1 clock-source selection and verification (MCXA5xx only).
//!
//! WWDT0 is hardwired to `clk_1m` and has no `CLKSEL` register. WWDT1 has
//! one: a four-way source mux (`MRCC_WWDT1_CLKSEL[MUX]`, MCXA5xx RM Rev 1
//! 22.5.2.36), alongside the divider (22.5.2.37) that both instances have.
//! This example selects a source through the public
//! [`hal::wwdt::ClockConfig`] API and then proves, on hardware, that the
//! selection actually reached the mux and that the driver's timing arithmetic
//! follows it.
//!
//! It works by sampling the watchdog counter (`WWDT1.TV`, which counts down)
//! across a known `embassy_time` interval, and comparing the measured tick
//! rate against the rate implied by the resolved clock tree. The expected
//! value is read back from [`hal::clocks::with_clocks`] rather than hardcoded,
//! so the check stays honest for every source.
//!
//! The WWDT applies a fixed divide-by-4 prescaler after the mux and divider
//! (22.5.2.2, 34.3, Figure 172), hence `source / div / 4`.
//!
//! Only one source can be exercised per boot: [`Watchdog::new`] consumes the
//! peripheral, and `MOD[LOCK]` is set during setup so the selection cannot be
//! changed again until reset. Edit [`SOURCE`] and re-flash to check another.
//!
//! Expected results on an FRDM-MCXA577 with the default clock tree:
//!
//! | `SOURCE`         | `MUX` | tick rate |
//! |------------------|-------|-----------|
//! | `Clk1M`          | 2     | 250000 Hz |
//! | `Clk16kVddCore`  | 0     |   4096 Hz |

#![no_std]
#![no_main]

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_mcxa as hal;
use embassy_mcxa::clocks::PoweredClock;
use embassy_mcxa::clocks::periph_helpers::{Div4, WwdtClockSel};
use embassy_time::{Duration, Instant, Timer};
use hal::bind_interrupts;
use hal::config::Config;
use hal::peripherals::WWDT1;
use hal::wwdt::{ClockConfig, InterruptHandler, Watchdog};
use panic_probe as _;

bind_interrupts!(
    struct Irqs {
        WWDT1 => InterruptHandler<WWDT1>;
    }
);

/// The source under test. Change this and re-flash to exercise another leg.
const SOURCE: WwdtClockSel = WwdtClockSel::Clk1M;
/// The divider under test.
const DIV: Div4 = Div4::no_div();
/// How long to sample for. Longer is more accurate.
const SAMPLE: Duration = Duration::from_millis(2000);
/// Tolerance, in percent. FRO16K is an untrimmed RC oscillator.
const TOLERANCE_PCT: u64 = 2;

/// The WWDT's fixed post-mux prescaler (MCXA5xx RM Rev 1 34.3, Figure 172).
const WWDT_PRESCALER: u32 = 4;

/// Read the selected source's frequency back out of the resolved clock tree.
fn source_frequency(sel: WwdtClockSel) -> Option<u32> {
    hal::clocks::with_clocks(|c| {
        let clk = match sel {
            WwdtClockSel::Clk16kVddCore => c.clk_16k_vdd_core.as_ref(),
            WwdtClockSel::FroHfDiv => c.fro_hf_div.as_ref(),
            WwdtClockSel::Clk1M => c.clk_1m.as_ref(),
        };
        clk.map(|c| c.frequency)
    })
    .flatten()
}

fn dump_mux(tag: &str) {
    let mrcc = hal::pac::MRCC0;
    let mux = mrcc.mrcc_wwdt1_clksel().read().mux().to_bits();
    let div = mrcc.mrcc_wwdt1_clkdiv().read();
    defmt::info!(
        "{=str}: CLKSEL.MUX={=u8} (0=CLK_16K 1=FRO_HF_DIV 2=CLK_1M), CLKDIV.DIV={=u8}, UNSTAB={=u8}",
        tag,
        mux,
        div.div(),
        div.unstab().to_bits(),
    );
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = hal::init(Config::default());

    defmt::info!("WWDT1 clock-source verification, SOURCE={}", SOURCE);

    // Reset state, before the HAL touches the mux.
    dump_mux("before Watchdog::new");

    // A long timeout keeps the counter well clear of expiry for the whole
    // measurement, and `warning: Some(..)` selects interrupt mode rather than
    // reset mode so a misconfiguration does not reboot the board mid-test.
    let cfg = hal::wwdt::Config {
        timeout: Duration::from_secs(8),
        warning: Some(Duration::from_micros(4000)),
        clock: ClockConfig {
            power: PoweredClock::NormalEnabledDeepSleepDisabled,
            source: SOURCE,
            div: DIV,
        },
    };

    let mut watchdog = match Watchdog::new(p.WWDT1, Irqs, cfg) {
        Ok(w) => w,
        Err(e) => {
            defmt::error!("Watchdog::new(WWDT1) failed: {:?}", e);
            defmt::panic!("WWDT1 clock setup rejected");
        }
    };

    dump_mux("after Watchdog::new ");
    watchdog.start();

    // What the resolved clock tree says this source should produce.
    let Some(source_hz) = source_frequency(SOURCE) else {
        defmt::panic!("selected source is not active in the resolved clock tree");
    };
    let expected = (source_hz / DIV.into_divisor() / WWDT_PRESCALER) as u64;
    defmt::info!(
        "source = {=u32} Hz, div = {=u32}, prescaler = {=u32} -> expect {=u64} Hz",
        source_hz,
        DIV.into_divisor(),
        WWDT_PRESCALER,
        expected
    );

    // Let the counter move away from its reload value before sampling.
    Timer::after_millis(50).await;

    let regs = hal::pac::WWDT1;
    let t0 = Instant::now();
    let tv0 = regs.tv().read().count();
    Timer::after(SAMPLE).await;
    let tv1 = regs.tv().read().count();
    let elapsed_us = t0.elapsed().as_micros();

    // The WWDT counter counts down.
    let ticks = tv0.saturating_sub(tv1) as u64;
    let measured = (ticks * 1_000_000) / elapsed_us;

    defmt::info!(
        "tv0={=u32} tv1={=u32} ticks={=u64} elapsed_us={=u64}",
        tv0,
        tv1,
        ticks,
        elapsed_us
    );
    defmt::info!("measured {=u64} Hz, expected {=u64} Hz", measured, expected);

    let lo = (expected * (100 - TOLERANCE_PCT)) / 100;
    let hi = (expected * (100 + TOLERANCE_PCT)) / 100;

    if measured >= lo && measured <= hi {
        defmt::info!("PASS: WWDT1 runs at the rate the selected source implies");
    } else {
        defmt::error!("FAIL: {=u64} Hz outside [{=u64}, {=u64}]", measured, lo, hi);
    }

    // Keep feeding so the watchdog never fires.
    loop {
        watchdog.feed();
        Timer::after_millis(500).await;
    }
}

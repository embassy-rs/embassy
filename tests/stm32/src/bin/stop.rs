// required-features: stop,jiff

//! Stop mode with the RTC and TIM time driver.
//!
//! Every board runs from a PLL where it has one, so a clock not restored after Stop shows up: the
//! timer then counts at the wrong rate and embassy-time drifts from the RTC.
#![no_std]
#![no_main]
#[path = "../common.rs"]
mod common;

use common::*;
use defmt::assert;
use embassy_executor::Spawner;
use embassy_stm32::rcc::{LsConfig, StopMode, get_stop_mode};
use embassy_stm32::rtc::{Rtc, RtcConfig, RtcReader};
use embassy_time::{Duration, Instant, Timer, block_for};
use jiff::civil::{DateTime, date};

teleprobe_meta::timeout!(60);

/// Get whether the core is ready to enter the given stop mode.
///
/// This will return false if some peripheral driver is in use that
/// prevents entering the given stop mode.
fn stop_ready(stop_mode: StopMode) -> bool {
    critical_section::with(|cs| match get_stop_mode(cs) {
        Some(mode) => mode.at_least(stop_mode),
        None => false,
    })
}

#[embassy_executor::task]
async fn task_1() {
    for _ in 0..9 {
        info!("task 1: waiting for 500ms...");
        assert!(stop_ready(StopMode::Stop2));
        Timer::after_millis(500).await;
        // Run on the restored clocks for a while: if they're wrong, the timer drifts from the RTC.
        block_for(Duration::from_millis(20));
    }
}

#[embassy_executor::task]
async fn task_2(reader: RtcReader<'static>, start_rtc: DateTime, start: Instant) {
    for _ in 0..5 {
        info!("task 2: waiting for 1000ms...");
        assert!(stop_ready(StopMode::Stop2));
        Timer::after_millis(1000).await;
        block_for(Duration::from_millis(20));
    }

    // The time spent in stop mode is measured with the RTC, so embassy-time must track it.
    let rtc_elapsed = reader.read().unwrap().duration_since(start_rtc).as_micros() as i64;
    let elapsed = start.elapsed().as_micros() as i64;
    info!("elapsed: rtc {} us, embassy-time {} us", rtc_elapsed, elapsed);
    assert!((rtc_elapsed - elapsed).abs() < 20_000);

    info!("Test OK");
    cortex_m::asm::bkpt();
}

/// Whether a debugger attached since power-on.
fn debugger_attached() -> bool {
    cfg_select! {
        // Cortex-M0+ software can't read DHCSR. probe-rs turns on the DBGMCU clock and sets
        // DBG_STOP when it attaches, and only a power-on reset clears DBGMCU_CR (RM0444 40.10.2,
        // RM0367 33.9.3).
        feature = "stm32g071rb" => {
            embassy_stm32::pac::RCC.apbenr1().modify(|w| w.set_dbgen(true));
            embassy_stm32::pac::DBGMCU.cr().read().dbg_stop()
        }
        feature = "stm32l073rz" => {
            embassy_stm32::pac::RCC.apb2enr().modify(|w| w.set_dbgen(true));
            embassy_stm32::pac::DBGMCU.cr().read().dbg_stop()
        }
        // probe-rs doesn't set up DBGMCU on U0.
        feature = "stm32u083rc" => true,
        _ => cortex_m::peripheral::DCB::is_debugger_attached(),
    }
}

#[derive(Clone, Copy)]
struct Irq0;

unsafe impl cortex_m::interrupt::InterruptNumber for Irq0 {
    fn number(self) -> u16 {
        0
    }
}

/// Exercise the `did_enter_stop() == Some(false)` ("aborted WFI") path cleanly.
///
/// With a pending, unmasked NVIC interrupt while interrupts are masked inside a critical section,
/// Cortex-M `WFI` returns immediately without entering Stop. Clocks, flash wait states, and the
/// time driver must remain intact and ready for subsequent Stop transitions.
fn test_aborted_wfi() {
    info!("testing aborted WFI...");
    critical_section::with(|cs| {
        assert!(get_stop_mode(cs).is_some());
        let latency_before = embassy_stm32::pac::FLASH.acr().read().latency();
        let was_enabled = unsafe { (*cortex_m::peripheral::NVIC::PTR).iser[0].read() & 1 != 0 };
        let was_pending = cortex_m::peripheral::NVIC::is_pending(Irq0);

        unsafe { cortex_m::peripheral::NVIC::unmask(Irq0) };
        cortex_m::peripheral::NVIC::pend(Irq0);

        unsafe { embassy_stm32::low_power::sleep(cs) };

        if !was_pending {
            cortex_m::peripheral::NVIC::unpend(Irq0);
        }
        if !was_enabled {
            cortex_m::peripheral::NVIC::mask(Irq0);
        }

        assert!(embassy_stm32::pac::FLASH.acr().read().latency() == latency_before);
    });
}

#[embassy_executor::main(executor = "common::Executor", entry = "cortex_m_rt::entry")]
async fn main(spawner: Spawner) {
    // This test stays in flash, so it also runs when the farm power-cycles the board to load the
    // next test, and loading it sometimes fails while the chip goes in and out of Stop. Without a
    // debugger at boot, this is that leftover image: park for good, teleprobe takes over.
    if !debugger_attached() {
        loop {
            core::hint::spin_loop();
        }
    }

    let mut config = config();
    config.rcc.ls = LsConfig::default_lse();

    // CRS would keep the chip out of Stop 2.
    #[cfg(any(feature = "stm32u585ai", feature = "stm32u5a5zj", feature = "stm32u083rc"))]
    {
        config.rcc.hsi48 = Some(embassy_stm32::rcc::Hsi48Config { sync_from_usb: false });
    }

    // The other tests run WBA from HSI. Run from the PLL here, so it has to be restored after Stop.
    #[cfg(any(feature = "stm32wba52cg", feature = "stm32wba65ri"))]
    {
        use embassy_stm32::rcc::*;
        config.rcc.hse = Some(Hse {
            prescaler: HsePrescaler::Div1,
            trim: None,
        });
        config.rcc.pll1 = Some(Pll {
            source: PllSource::Hse,
            prediv: PllPreDiv::Div2, // 16 MHz
            mul: PllMul::Mul30,      // 480 MHz
            divp: None,
            divq: None,
            divr: Some(PllDiv::Div5), // 96 MHz
            frac: None,
        });
        config.rcc.sys = Sysclk::Pll1R;
        config.rcc.ahb5_pre = AHB5Prescaler::Div4; // 24 MHz
        config.rcc.voltage_scale = VoltageScale::Range1;
    }

    let p = init_with_config(config);
    info!("Hello World!");

    let now = date(2020, 5, 15).at(10, 30, 15, 0);

    let mut rtc = Rtc::new(p.RTC, RtcConfig::default());
    let reader = rtc.reader();

    info!("set datetime");
    rtc.set(now).unwrap();

    test_aborted_wfi();

    let start_rtc = reader.read().unwrap();
    let start = Instant::now();

    info!("spawn");
    spawner.spawn(task_1().unwrap());
    spawner.spawn(task_2(reader, start_rtc, start).unwrap());
}

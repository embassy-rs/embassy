//! BLE Direct Test Mode (DTM) example for STM32WBA55CG
//!
//! Demonstrates RF testing using BLE DTM HCI commands.
//!
//! To use with two Nucleo-WBA55CG boards:
//!  - Change DTM_MODE to DtmMode::Tx on one board, DtmMode::Rx on the other
//!  - Both boards must use the same DTM_CHANNEL
//!  - Flash both, observe defmt logs
//!  - Press USER button (B1) on each board when ready to start
//!  - RX board reports received packet count and PER after DTM_TEST_DURATION_SECS
//!
//! Hardware: STM32WBA55CG (Nucleo-WBA55CG)
//!
//! Note: The BLE runner task is not needed for DTM. DTM commands are synchronous
//! HCI calls handled directly by the controller. The RADIO and HASH interrupt
//! handlers are sufficient to drive the link layer.

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::Pull;
use embassy_stm32::peripherals::RNG;
use embassy_stm32::rng::{self, Rng};
use embassy_stm32::{Config, bind_interrupts, exti, interrupt, rcc};
use embassy_stm32_wpan::bluetooth::HCI;
use embassy_stm32_wpan::bluetooth::hci::types::DtmPacketPayload;
use embassy_stm32_wpan::{HighInterruptHandler, LowInterruptHandler, Platform, new_platform};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

// ---- Test configuration ----
#[allow(dead_code)]
enum DtmMode {
    Tx,
    Rx,
}
const DTM_MODE: DtmMode = DtmMode::Tx; // change to Rx on the other board
const DTM_CHANNEL: u8 = 19; // 2440 MHz — same on both boards
const DTM_DATA_LENGTH: u8 = 37; // bytes per packet
const DTM_TEST_DURATION_SECS: u64 = 10;
// ----------------------------

bind_interrupts!(struct Irqs {
    RNG => rng::InterruptHandler<RNG>;
    EXTI13 => exti::InterruptHandler<interrupt::typelevel::EXTI13>;
    RADIO => HighInterruptHandler;
    HASH => LowInterruptHandler;
});

/// RNG runner task
#[embassy_executor::task]
async fn rng_runner_task(platform: &'static Platform) {
    platform.run_rng().await
}

/// BLE runner task - drives the BLE stack sequencer
#[embassy_executor::task]
async fn ble_runner_task(platform: &'static Platform) {
    platform.run_ble().await
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut config = Config::default();
    config.rcc = rcc::Config::new_wpan();

    let p = embassy_stm32::init(config);

    let mut button = ExtiInput::new(p.PC13, p.EXTI13, Pull::Up, Irqs);

    info!("Embassy STM32WBA BLE DTM Example");
    match DTM_MODE {
        DtmMode::Tx => info!("Mode: TX"),
        DtmMode::Rx => info!("Mode: RX"),
    }

    info!("Press USER button (B1) to start...");
    button.wait_for_falling_edge().await;
    info!("Button pressed — initialising BLE");

    // Initialize hardware peripherals required by BLE stack
    let (platform, runtime) = new_platform!(Rng::new(p.RNG, Irqs), 8);

    info!("Hardware peripherals initialized (RNG, AES, PKA)");

    // Spawn the RNG runner task
    spawner.spawn(rng_runner_task(platform).expect("Failed to spawn rng runner"));

    // Spawn the BLE runner task (required for proper BLE operation)
    spawner.spawn(ble_runner_task(platform).expect("Failed to spawn BLE runner"));

    // Initialize BLE stack
    let mut dtm_ble = HCI::new_dtm(platform, runtime, Irqs)
        .await
        .expect("BLE initialization failed");

    // DTM packet interval is 625 µs (1 BLE slot) per Vol 6, Part F, Section 4.1.6.
    // Expected packets = duration_s × (1_000_000 µs/s ÷ 625 µs/packet) = duration_s × 1600.
    let expected: u32 = DTM_TEST_DURATION_SECS as u32 * 1_000_000 / 625;

    match DTM_MODE {
        DtmMode::Tx => {
            let freq_mhz = 2402 + 2 * DTM_CHANNEL as u32;
            info!(
                "DTM TX: channel {} ({}MHz), {} byte payload, PRBS9, {}s",
                DTM_CHANNEL, freq_mhz, DTM_DATA_LENGTH, DTM_TEST_DURATION_SECS
            );
            // BLE spec Vol 6 Part F §3.4.2: TX packet count is always 0 by spec.
            // Expected rate: ~1600 packets/s (625 µs interval).
            info!(
                "Expected ~{} packets over {}s (~1600 packets/s at 625us interval)",
                expected, DTM_TEST_DURATION_SECS
            );

            dtm_ble
                .dtm_transmit(DTM_CHANNEL, DTM_DATA_LENGTH, DtmPacketPayload::Prbs9)
                .expect("DTM TX start failed");

            Timer::after_secs(DTM_TEST_DURATION_SECS).await;

            match dtm_ble.dtm_end() {
                Ok(_) => info!("DTM TX test ended after {}s", DTM_TEST_DURATION_SECS),
                Err(e) => error!("le_test_end failed: {:?}", e),
            }
        }

        DtmMode::Rx => {
            let freq_mhz = 2402 + 2 * DTM_CHANNEL as u32;
            info!(
                "DTM RX: channel {} ({}MHz), {}s",
                DTM_CHANNEL, freq_mhz, DTM_TEST_DURATION_SECS
            );
            info!(
                "Expected ~{} packets over {}s (~1600 packets/s at 625us interval)",
                expected, DTM_TEST_DURATION_SECS
            );

            dtm_ble.dtm_receive(DTM_CHANNEL).expect("DTM RX start failed");

            Timer::after_secs(DTM_TEST_DURATION_SECS).await;

            match dtm_ble.dtm_end() {
                Ok(received) => {
                    let received = received as u32;
                    // Packet Error Rate = lost / expected × 100
                    // lost = expected − received (clamped to 0 if received > expected)
                    let lost = expected.saturating_sub(received);
                    let per_pct = lost * 100 / expected;
                    let per_frac = (lost * 10000 / expected) % 100;
                    info!("--- DTM RX Results ---");
                    info!("  Expected : {} packets", expected);
                    info!("  Received : {} packets", received);
                    info!("  Lost     : {} packets", lost);
                    info!("  PER      : {}.{:02}%  (lost/expected × 100)", per_pct, per_frac);
                    info!(
                        "  Math     : {} / {} × 100 = {}.{:02}%",
                        lost, expected, per_pct, per_frac
                    );
                }
                Err(e) => error!("le_test_end failed: {:?}", e),
            }
        }
    }

    info!("Done.");
    loop {
        Timer::after_secs(86400).await;
    }
}

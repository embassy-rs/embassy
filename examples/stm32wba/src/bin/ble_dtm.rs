//! BLE Direct Test Mode (DTM) example for STM32WBA55CG
//!
//! Demonstrates RF testing using BLE DTM HCI commands.
//!
//! To use with two Nucleo-WBA55CG boards:
//!  - Change DTM_MODE to DtmMode::Tx on one board, DtmMode::Rx on the other
//!  - Both boards must use the same DTM_CHANNEL
//!  - Flash both, observe defmt logs
//!  - TX board logs packet count while running
//!  - RX board reports received packet count after DTM_TEST_DURATION_SECS
//!
//! Hardware: STM32WBA55CG (Nucleo-WBA55CG)

#![no_std]
#![no_main]

use core::cell::RefCell;

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::peripherals::RNG;
use embassy_stm32::rng::{self, Rng};
use embassy_stm32::time::Hertz;
use embassy_stm32::{Config, bind_interrupts, interrupt};
use embassy_stm32_wpan::hci::command::{
    aci_hal_tx_test_packet_number, le_receiver_test, le_test_end, le_transmitter_test,
};
use embassy_stm32_wpan::hci::types::DtmPacketPayload;
use embassy_stm32_wpan::{Ble, ble_runner, run_radio_high_isr, run_radio_sw_low_isr};
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_time::Timer;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

// ---- Test configuration ----
// Change these to configure the test
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
});

// RADIO interrupt handler - required for BLE stack operation
#[interrupt]
unsafe fn RADIO() {
    unsafe { run_radio_high_isr() };
}

// HASH interrupt handler - used as software low-priority interrupt for BLE
#[interrupt]
unsafe fn HASH() {
    unsafe { run_radio_sw_low_isr() };
}

/// BLE runner task - drives the BLE stack sequencer
#[embassy_executor::task]
async fn ble_runner_task() {
    ble_runner().await
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        // Enable HSE (32 MHz external crystal) - REQUIRED for BLE radio
        config.rcc.hse = Some(Hse {
            prescaler: HsePrescaler::Div1,
        });

        // Enable LSE (32.768 kHz external crystal) - REQUIRED for BLE radio sleep timer
        config.rcc.ls = LsConfig {
            rtc: RtcClockSource::Lse,
            lsi: false,
            lse: Some(LseConfig {
                frequency: Hertz(32_768),
                mode: LseMode::Oscillator(LseDrive::MediumLow),
                peripherals_clocked: true,
            }),
        };
        // Configure PLL1 (required on WBA)
        config.rcc.pll1 = Some(Pll {
            source: PllSource::Hsi,
            prediv: PllPreDiv::Div1,
            mul: PllMul::Mul30,
            divr: Some(PllDiv::Div5),
            divq: None,
            divp: Some(PllDiv::Div30),
            frac: Some(0),
        });
        config.rcc.ahb_pre = AHBPrescaler::Div1;
        config.rcc.apb1_pre = APBPrescaler::Div1;
        config.rcc.apb2_pre = APBPrescaler::Div1;
        config.rcc.apb7_pre = APBPrescaler::Div1;
        config.rcc.ahb5_pre = AHB5Prescaler::Div4;
        config.rcc.voltage_scale = VoltageScale::Range1;
        config.rcc.sys = Sysclk::Pll1R;
        config.rcc.mux.rngsel = mux::Rngsel::Hsi;
    }

    let p = embassy_stm32::init(config);
    // Configure radio sleep timer to use LSE
    {
        use embassy_stm32::pac::RCC;
        use embassy_stm32::pac::rcc::vals::Radiostsel;
        RCC.bdcr().modify(|w| w.set_radiostsel(Radiostsel::Lse));
    }

    info!("Embassy STM32WBA BLE DTM (Direct Test Mode) Example");

    // Initialize hardware peripherals required
    static RNG_INST: StaticCell<Mutex<CriticalSectionRawMutex, RefCell<Rng<'static, RNG>>>> = StaticCell::new();
    let rng = RNG_INST.init(Mutex::new(RefCell::new(Rng::new(p.RNG, Irqs))));

    info!("Hardware peripherals initialized (RNG)");

    // Initialize BLE-DTM stack
    let mut ble = Ble::new_dtm(rng);
    ble.dtm_init().expect("DTM init failed");

    // Spawn the BLE runner task (required for proper BLE operation)
    spawner.spawn(ble_runner_task().expect("Failed to spawn BLE runner"));
    embassy_futures::yield_now().await;

    match DTM_MODE {
        DtmMode::Tx => {
            info!(
                "Starting DTM TX on channel {} ({}MHz), {} byte payload, PRBS9",
                DTM_CHANNEL,
                2402 + 2 * DTM_CHANNEL as u32,
                DTM_DATA_LENGTH
            );

            le_transmitter_test(DTM_CHANNEL, DTM_DATA_LENGTH, DtmPacketPayload::Prbs9).expect("DTM TX start failed");

            for elapsed in 1..=DTM_TEST_DURATION_SECS {
                Timer::after_secs(1).await;
                match aci_hal_tx_test_packet_number() {
                    Ok(n) => info!("  {}s: {} TX packets sent", elapsed, n),
                    Err(e) => warn!("  packet count unavailable: {:?}", e),
                }
            }

            match le_test_end() {
                Ok(_) => info!("DTM TX test ended"),
                Err(e) => error!("le_test_end failed: {:?}", e),
            }
        }

        DtmMode::Rx => {
            info!(
                "Starting DTM RX on channel {} ({}MHz)",
                DTM_CHANNEL,
                2402 + 2 * DTM_CHANNEL as u32
            );

            le_receiver_test(DTM_CHANNEL).expect("DTM RX start failed");

            Timer::after_secs(DTM_TEST_DURATION_SECS).await;

            match le_test_end() {
                Ok(n) => info!("DTM RX test ended: {} packets received", n),
                Err(e) => error!("le_test_end failed: {:?}", e),
            }
        }
    }

    info!("Done.");
    loop {
        Timer::after_secs(86400).await;
    }
}

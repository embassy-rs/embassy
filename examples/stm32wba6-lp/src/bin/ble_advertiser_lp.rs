//! Low-power BLE Advertiser Example for STM32WBA65
//!
//! This example demonstrates the integration of the BLE stack with the low-power
//! STOP mode executor on the STM32WBA65.
//!
//! Between advertising ticks and task executions, the embassy executor will yield
//! and automatically put the microcontroller into STOP mode.
//!
//! To achieve lowest current draw:
//! - Debug peripherals are disabled during sleep (`enable_debug_during_sleep = false`)
//! - Clocks are dynamically managed
//! - The BLE stack is clocked by HSE (32 MHz) and sleep timer uses LSE (32.768 kHz)
//!
//! To test:
//! 1. Flash this example to your STM32WBA6 board
//! 2. Use a BLE scanner app (nRF Connect, LightBlue, etc.)
//! 3. Look for "Embassy-LP-WBA6" in the scan results

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::aes::{self, Aes};
use embassy_stm32::peripherals::{AES, PKA, RNG};
use embassy_stm32::pka::{self, Pka};
use embassy_stm32::rcc::{Config as RccConfig, LseDrive, LseMode};
use embassy_stm32::rng::{self, Rng};
use embassy_stm32::{Config, bind_interrupts};
use embassy_stm32_wpan::bluetooth::HCI;
use embassy_stm32_wpan::bluetooth::gap::types::OwnAddressType;
use embassy_stm32_wpan::bluetooth::gap::{AdvData, AdvParams, AdvType};
use embassy_stm32_wpan::bluetooth::gap_init::{AddressType, GapInitParams};
use embassy_stm32_wpan::bluetooth::gatt::{CharProperties, GattEventMask, SecurityPermissions, ServiceType, Uuid};
use embassy_stm32_wpan::{HighInterruptHandler, LowInterruptHandler, Platform, new_platform};
use embassy_time::Duration;
use {defmt_rtt as _, panic_probe as _};

// ---- Test configuration ----
const ADDR_TYPE: OwnAddressType = OwnAddressType::Random;
const MEASURE_POWER: bool = true;
const ULTRA_LOW_POWER_PRESET: bool = false;
const AGGRESSIVE_SRAM_POWERDOWN: bool = false;
const TUNE_LSE_DRIVE: bool = false;
const LSE_DRIVE_SETTING: LseDrive = LseDrive::Low;

// Conservative defaults for low-power measurements.
const ADV_INTERVAL_UNITS: u16 = 0x0800; // 1.28 s (0.625 ms units)
const MIN_STOP_PAUSE_MS: u64 = 100;

// Suggested measurement matrix (run each for ~30-60s average current):
// A) Baseline:
//    ULTRA_LOW_POWER_PRESET=false, TUNE_LSE_DRIVE=false, AGGRESSIVE_SRAM_POWERDOWN=false
// B) BOR ultra-low-power:
//    ULTRA_LOW_POWER_PRESET=true,  TUNE_LSE_DRIVE=false, AGGRESSIVE_SRAM_POWERDOWN=false
// C) LSE drive sweep:
//    ULTRA_LOW_POWER_PRESET=true,  TUNE_LSE_DRIVE=true,  LSE_DRIVE_SETTING=MediumLow/Low
// D) SRAM power-down:
//    ULTRA_LOW_POWER_PRESET=true,  TUNE_LSE_DRIVE=true,  AGGRESSIVE_SRAM_POWERDOWN=true
// Keep ADV interval and RF conditions fixed while comparing.

bind_interrupts!(struct Irqs {
    RNG => rng::InterruptHandler<RNG>;
    AES => aes::InterruptHandler<AES>;
    PKA => pka::InterruptHandler<PKA>;
    RADIO => HighInterruptHandler;
    HASH => LowInterruptHandler;
});

/// BLE runner task - drives the BLE stack sequencer
#[embassy_executor::task]
async fn ble_runner_task(platform: &'static Platform) {
    platform.run_ble().await
}

#[embassy_executor::main(executor = "embassy_stm32::executor::Executor", entry = "cortex_m_rt::entry")]
async fn main(spawner: Spawner) {
    let mut config = Config::default();

    // Enable HSE/LSE clocks and configure system at 96 MHz via PLL1
    config.rcc = RccConfig::new_wpan();

    // Optional board-specific LSE drive tuning.
    // RM guidance: lower drive can reduce current after startup, but may hurt startup margin.
    if TUNE_LSE_DRIVE {
        if let Some(lse) = &mut config.rcc.ls.lse {
            lse.mode = LseMode::Oscillator(LSE_DRIVE_SETTING);
        }
    }

    // Two practical profiles:
    // - MEASURE_POWER=true: best current draw, limited STOP-time debug visibility.
    // - MEASURE_POWER=false: easier bring-up/logging, higher sleep current.
    if MEASURE_POWER {
        config.enable_debug_during_sleep = false;
        // Keep this comfortably below the advertising period to maximize STOP residency.
        config.min_stop_pause = Duration::from_millis(MIN_STOP_PAUSE_MS);

        // Optional preset aligned with RM0515 guidance for lowest Stop/Standby current.
        // Keep disabled by default: this can be board/application sensitive.
        if ULTRA_LOW_POWER_PRESET {
            // Enable BOR0 ultra-low-power monitoring mode in Stop1/Standby.
            // RM constraints: avoid when autonomous peripherals rely on HSI kernels.
            config.enable_ulpmen = true;

            // Keep flash in low-power mode during stop (lowest current, slower wake).
            config.flash_fast_wakeup = false;

            // Optional aggressive SRAM power-down in Stop modes.
            // WARNING: Any powered-down SRAM content is lost across STOP entry.
            if AGGRESSIVE_SRAM_POWERDOWN {
                config.stop_mode_sram.sram1_page2 = true;
                config.stop_mode_sram.sram1_page3 = true;
                config.stop_mode_sram.sram1_pages567 = true;
                config.stop_mode_sram.sram2 = true;
                config.stop_mode_sram.icache_sram = true;
                config.stop_mode_sram.otg_sram = true;
            }
        }
    } else {
        config.enable_debug_during_sleep = true;
        config.min_stop_pause = Duration::from_millis(20);
    }

    let p = embassy_stm32::init(config);

    info!("Embassy STM32WBA6 Low-Power BLE Advertiser Example");

    // Initialize hardware peripherals required by BLE stack
    let (platform, runtime) = new_platform!(
        Rng::new(p.RNG, Irqs),
        Pka::new(p.PKA, Irqs),
        Aes::new_blocking(p.AES, Irqs),
        8
    );

    info!("Hardware peripherals initialized (RNG, AES, PKA)");

    // Spawn the BLE runner task (required for proper BLE operation)
    spawner.spawn(ble_runner_task(platform).expect("Failed to spawn BLE runner"));

    let mut ble = match ADDR_TYPE {
        OwnAddressType::Public => {
            let gap_params = GapInitParams {
                bd_addr: [0x02, 0x00, 0x00, 0xE1, 0x80, 0x00],
                address_type: AddressType::Public,
                ..GapInitParams::default()
            };
            HCI::new_with_gap_params(platform, runtime, Irqs, gap_params).await
        }
        _ => HCI::new(platform, runtime, Irqs).await,
    }
    .expect("BLE initialization failed");

    info!("BLE stack initialized successfully under low-power executor");

    // Initialize GATT server
    let mut gatt = ble.gatt_server();

    // Add a custom service (128-bit UUID)
    let service_uuid = Uuid::from_u128_le([
        0x00, 0x00, 0x12, 0x34, 0x00, 0x00, 0x10, 0x00, 0x80, 0x00, 0x00, 0x80, 0x5f, 0x9b, 0x34, 0xfb,
    ]);
    let service_handle = gatt
        .add_service(service_uuid, ServiceType::Primary, 8)
        .expect("Failed to add service");
    info!("Created Service with handle: 0x{:04X}", service_handle.0);

    // Add a read/write characteristic
    let char_uuid = Uuid::from_u128_le([
        0x00, 0x00, 0x56, 0x78, 0x00, 0x00, 0x10, 0x00, 0x80, 0x00, 0x00, 0x80, 0x5f, 0x9b, 0x34, 0xfb,
    ]);
    let char_handle = gatt
        .add_characteristic(
            service_handle,
            char_uuid,
            16, // Max length 16 bytes
            CharProperties::READ | CharProperties::WRITE,
            SecurityPermissions::NONE,
            GattEventMask::NONE,
            16,   // Key size
            true, // Variable length
        )
        .expect("Failed to add characteristic");
    info!("Created characteristic with handle: 0x{:04X}", char_handle.0);

    // Set initial characteristic value
    let initial_value = b"Low Power BLE!";
    gatt.update_characteristic_value(service_handle, char_handle, 0, initial_value)
        .expect("Failed to set characteristic value");
    info!("Set initial characteristic value");

    // Create advertising data
    let mut adv_data = AdvData::new();
    adv_data
        .add_flags(0x06) // General discoverable, BR/EDR not supported
        .expect("Failed to add flags");
    adv_data.add_name("Embassy-LP-WBA6").expect("Failed to add name");
    adv_data
        .add_service_uuid_16(0x1234) // Advertise our custom service
        .expect("Failed to add service UUID");

    info!("Advertising data created ({} bytes)", adv_data.len());

    // Configure advertising parameters
    // Slower advertising intervals significantly reduce average radio duty cycle.
    let adv_params = AdvParams {
        interval_min: ADV_INTERVAL_UNITS,
        interval_max: ADV_INTERVAL_UNITS,
        adv_type: AdvType::ConnectableUndirected,
        own_addr_type: ADDR_TYPE,
        ..AdvParams::default()
    };

    // Start advertising
    ble.start_advertising(adv_params, adv_data, None)
        .await
        .expect("Failed to start advertising");

    info!("BLE advertising started!");
    info!("Device is visible as 'Embassy-LP-WBA6'");
    if MEASURE_POWER {
        info!("Power profile: measurement (debug-in-sleep OFF)");
        info!(
            "Ultra-low-power preset: {}",
            if ULTRA_LOW_POWER_PRESET { "enabled" } else { "disabled" }
        );
        info!(
            "Aggressive SRAM power-down: {}",
            if ULTRA_LOW_POWER_PRESET && AGGRESSIVE_SRAM_POWERDOWN {
                "enabled"
            } else {
                "disabled"
            }
        );
        info!(
            "LSE drive tuning: {}",
            if TUNE_LSE_DRIVE { "enabled" } else { "disabled" }
        );
    } else {
        info!("Power profile: debug-friendly (debug-in-sleep ON)");
    }
    info!("Waking for BLE events/timers, then returning to STOP when possible...");

    // Main loop - handle BLE events in a power-efficient manner.
    // Keep draining BLE events; pending events/interrupts can otherwise reduce STOP residency.
    loop {
        let event = ble.read_event().await;
        // Keep host/log overhead low: avoid formatting every event unless debugging.
        if !MEASURE_POWER {
            info!("BLE Event received: {:?}", event);
        } else {
            let _ = event;
        }
    }
}

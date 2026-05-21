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
use embassy_stm32::rcc::Config as RccConfig;
use embassy_stm32::rng::{self, Rng};
use embassy_stm32::{Config, bind_interrupts};
use embassy_stm32_wpan::bluetooth::HCI;
use embassy_stm32_wpan::bluetooth::gap::types::OwnAddressType;
use embassy_stm32_wpan::bluetooth::gap::{AdvData, AdvParams, AdvType};
use embassy_stm32_wpan::bluetooth::gap_init::{AddressType, GapInitParams};
use embassy_stm32_wpan::bluetooth::gatt::{CharProperties, GattEventMask, SecurityPermissions, ServiceType, Uuid};
use embassy_stm32_wpan::{HighInterruptHandler, LowInterruptHandler, Platform, new_platform};
use {defmt_rtt as _, panic_probe as _};

// ---- Test configuration ----
const ADDR_TYPE: OwnAddressType = OwnAddressType::Random;

bind_interrupts!(struct Irqs {
    RNG => rng::InterruptHandler<RNG>;
    AES => aes::InterruptHandler<AES>;
    PKA => pka::InterruptHandler<PKA>;
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

#[embassy_executor::main(executor = "embassy_stm32::executor::Executor", entry = "cortex_m_rt::entry")]
async fn main(spawner: Spawner) {
    let mut config = Config::default();

    // Enable HSE/LSE clocks and configure system at 96 MHz via PLL1
    config.rcc = RccConfig::new_wpan();

    // Disable debug peripherals during STOP to achieve lowest current draw
    // If you need active RTT/probe-rs debugging during STOP, set this to true
    config.enable_debug_during_sleep = false;

    let p = embassy_stm32::init(config);

    info!("Embassy STM32WBA6 Low-Power BLE Advertiser Example");

    // Initialize hardware peripherals required by BLE stack
    let (platform, runtime) = new_platform!(
        Rng::new(p.RNG, Irqs),
        Aes::new_blocking(p.AES, Irqs),
        Pka::new_blocking(p.PKA, Irqs),
        8
    );

    info!("Hardware peripherals initialized (RNG, AES, PKA)");

    // Spawn the RNG runner task
    spawner.spawn(rng_runner_task(platform).expect("Failed to spawn rng runner"));

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
    // Using a 100 ms advertising interval. The device will wake up, transmit an ad packet,
    // and immediately return to STOP mode within less than 5 milliseconds.
    let adv_params = AdvParams {
        interval_min: 0x00A0, // 100 ms
        interval_max: 0x00A0, // 100 ms
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
    info!("Waking up only for BLE events, then automatically returning to STOP mode...");

    // Main loop - handle BLE events in a power-efficient manner
    loop {
        let event = ble.read_event().await;
        info!("BLE Event received: {:?}", event);
    }
}

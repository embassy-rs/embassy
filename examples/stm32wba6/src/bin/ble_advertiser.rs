//! BLE Advertiser Example
//!
//! This example demonstrates the Phase 1 BLE stack implementation:
//! - Initializes the BLE stack
//! - Creates a simple GATT service with a characteristic
//! - Starts BLE advertising
//! - The device will appear as "Embassy-WBA6" in BLE scanner apps
//!
//! Hardware: STM32WBA65 or compatible
//!
//! To test:
//! 1. Flash this example to your STM32WBA6 board
//! 2. Use a BLE scanner app (nRF Connect, LightBlue, etc.)
//! 3. Look for "Embassy-WBA6" in the scan results
//! 4. Connect to see the GATT service

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
use embassy_stm32_wpan::bluetooth::gap::{AdvData, AdvParams, AdvType};
use embassy_stm32_wpan::bluetooth::gap::types::OwnAddressType;
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

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut config = Config::default();
    config.rcc = RccConfig::new_wpan();

    let p = embassy_stm32::init(config);

    info!("Embassy STM32WBA6 BLE Advertiser Example");

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
                bd_addr: [0x01, 0x00, 0x00, 0xE1, 0x80, 0x00],
                address_type: AddressType::Public,
                ..GapInitParams::default()
            };
            HCI::new_with_gap_params(platform, runtime, Irqs, gap_params).await
        }
        _ => HCI::new(platform, runtime, Irqs).await,
    }
    .expect("BLE initialization failed");

    info!("BLE stack initialized");

    // Initialize GATT server
    let mut gatt = ble.gatt_server();
    info!("GATT server initialized");

    // Create a custom service (UUID: 0x1234)
    let service_uuid = Uuid::from_u16(0x1234);
    let service_handle = gatt
        .add_service(service_uuid, ServiceType::Primary, 5)
        .expect("Failed to add service");
    info!("Created service with handle: 0x{:04X}", service_handle.0);

    // Add a read/write characteristic (UUID: 0x5678)
    let char_uuid = Uuid::from_u16(0x5678);
    let char_properties = CharProperties::READ | CharProperties::WRITE | CharProperties::NOTIFY;
    let char_handle = gatt
        .add_characteristic(
            service_handle,
            char_uuid,
            20, // Max 20 bytes
            char_properties,
            SecurityPermissions::NONE,
            GattEventMask::ATTRIBUTE_MODIFIED,
            0,    // No encryption
            true, // Variable length
        )
        .expect("Failed to add characteristic");
    info!("Created characteristic with handle: 0x{:04X}", char_handle.0);

    // Set initial characteristic value
    let initial_value = b"Hello BLE!";
    gatt.update_characteristic_value(service_handle, char_handle, 0, initial_value)
        .expect("Failed to set characteristic value");
    info!("Set initial characteristic value");

    // Create advertising data
    let mut adv_data = AdvData::new();
    adv_data
        .add_flags(0x06) // General discoverable, BR/EDR not supported
        .expect("Failed to add flags");
    adv_data.add_name("Embassy-WBA6").expect("Failed to add name");
    adv_data
        .add_service_uuid_16(0x1234) // Advertise our custom service
        .expect("Failed to add service UUID");

    info!("Advertising data created ({} bytes)", adv_data.len());

    // Configure advertising parameters
    let adv_params = AdvParams {
        interval_min: 0x0080, // 80 ms
        interval_max: 0x0080, // 80 ms
        adv_type: AdvType::ConnectableUndirected,
        own_addr_type: ADDR_TYPE,
        ..AdvParams::default()
    };

    // Start advertising
    ble.start_advertising(adv_params, adv_data, None)
        .await
        .expect("Failed to start advertising");

    info!("BLE advertising started!");
    info!("Device is visible as 'Embassy-WBA6'");
    info!("Use a BLE scanner app to discover and connect");

    // Main loop - handle BLE events
    loop {
        let event = ble.read_event().await;
        info!("BLE Event: {:?}", event);

        // In a real application, you would handle connection events,
        // characteristic writes, etc. here
    }
}

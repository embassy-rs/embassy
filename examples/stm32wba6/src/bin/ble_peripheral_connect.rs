//! BLE Peripheral with Connection Handling Example
//!
//! This example demonstrates BLE connection management:
//! - Advertises as a connectable peripheral
//! - Handles connection and disconnection events
//! - Tracks active connections
//! - Allows disconnection via button press (if available)
//!
//! Hardware: STM32WBA65 or compatible
//!
//! To test:
//! 1. Flash this example to your STM32WBA6 board
//! 2. Use a BLE scanner app (nRF Connect, LightBlue, etc.)
//! 3. Connect to "Embassy-Peripheral"
//! 4. Observe connection/disconnection events in logs

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::aes::{self, Aes};
use embassy_stm32::peripherals::{AES, PKA, RNG};
use embassy_stm32::pka::{self, Pka};
use embassy_stm32::rng::{self, Rng};
use embassy_stm32::{Config, bind_interrupts, rcc};
use embassy_stm32_wpan::bluetooth::HCI;
use embassy_stm32_wpan::bluetooth::gap::types::OwnAddressType;
use embassy_stm32_wpan::bluetooth::gap::{AdvData, AdvParams, AdvType, GapEvent};
use embassy_stm32_wpan::bluetooth::gap_init::{AddressType, GapInitParams};
use embassy_stm32_wpan::bluetooth::gatt::{CharProperties, GattEventMask, SecurityPermissions, ServiceType, Uuid};
use embassy_stm32_wpan::{HighInterruptHandler, LowInterruptHandler, Platform, new_platform};
use stm32wb_hci::event::ConnectionRole;
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
    info!("Embassy STM32WBA6 BLE Peripheral Connection Example");

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
            // Initialize BLE stack with a known public address for testing.
            // Address: 00:80:E1:00:00:01.
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

    // Initialize GATT server with a simple service
    let mut gatt = ble.gatt_server();

    // Create a simple service for demonstration
    let service_uuid = Uuid::from_u16(0x180F); // Battery Service UUID
    let service_handle = gatt
        .add_service(service_uuid, ServiceType::Primary, 4)
        .expect("Failed to add service");

    // Add battery level characteristic
    let char_uuid = Uuid::from_u16(0x2A19); // Battery Level UUID
    let char_handle = gatt
        .add_characteristic(
            service_handle,
            char_uuid,
            1, // 1 byte for battery level
            CharProperties::READ | CharProperties::NOTIFY,
            SecurityPermissions::NONE,
            GattEventMask::ATTRIBUTE_MODIFIED,
            0,
            false,
        )
        .expect("Failed to add characteristic");

    // Set initial battery level (100%)
    gatt.update_characteristic_value(service_handle, char_handle, 0, &[100])
        .expect("Failed to set battery level");

    info!("GATT service created (Battery Service)");

    // Create advertising data
    let mut adv_data = AdvData::new();
    adv_data.add_flags(0x06).expect("Failed to add flags");
    adv_data.add_name("Embassy-Peripheral").expect("Failed to add name");
    adv_data
        .add_service_uuid_16(0x180F)
        .expect("Failed to add service UUID");

    // Configure advertising parameters for connectable advertising
    let adv_params = AdvParams {
        interval_min: 0x0050, // 50 ms
        interval_max: 0x0050,
        adv_type: AdvType::ConnectableUndirected,
        own_addr_type: ADDR_TYPE,
        ..AdvParams::default()
    };

    // Start advertising
    {
        ble.start_advertising(adv_params.clone(), adv_data.clone(), None)
            .await
            .expect("Failed to start advertising");
    }

    info!("BLE advertising started as 'Embassy-Peripheral'");
    info!("Waiting for connections...");

    // Main event loop
    loop {
        let event = ble.read_event().await;

        // Process the event and update connection state
        if let Some(gap_event) = ble.process_event(&event) {
            match gap_event {
                GapEvent::Connected(conn) => {
                    info!("=== CONNECTION ESTABLISHED ===");
                    info!("  Handle: 0x{:04X}", conn.handle.0);
                    info!(
                        "  Role: {}",
                        match conn.role {
                            ConnectionRole::Central => "Central",
                            ConnectionRole::Peripheral => "Peripheral",
                        }
                    );
                    info!("  Peer Address: {}", conn.peer_address);
                    info!("  Interval: {} ", conn.interval.interval());
                    info!("  Latency: {}", conn.interval.conn_latency());
                    info!("  Timeout: {}", conn.interval.supervision_timeout());
                    info!("  Active connections: {}", ble.connections().count());

                    // Note: Advertising typically stops automatically on connection
                    // If you want to support multiple connections, restart advertising here
                }

                GapEvent::Disconnected { handle, reason } => {
                    info!("=== DISCONNECTION ===");
                    info!("  Handle: 0x{:04X}", handle.0);
                    info!("  Reason: 0x{:02X} ({})", reason.as_u8(), Display2Format(&reason));
                    info!("  Active connections: {}", ble.connections().count());

                    // Restart advertising after disconnection.
                    // Advertising parameters are still configured, just re-enable.
                    info!("Restarting advertising...");
                    match ble.start_advertising(adv_params.clone(), adv_data.clone(), None).await {
                        Ok(()) => info!("Advertising restarted"),
                        Err(e) => error!("Failed to restart advertising: {:?}", e),
                    }
                }

                GapEvent::ConnectionParamsUpdated { handle, interval } => {
                    info!("=== CONNECTION PARAMS UPDATED ===");
                    info!("  Handle: 0x{:04X}", handle.0);
                    info!("  New Interval: {}", interval.interval());
                    info!("  New Latency: {}", interval.conn_latency());
                    info!("  New Timeout: {}", interval.supervision_timeout());
                }

                GapEvent::PhyUpdated { handle, tx_phy, rx_phy } => {
                    info!("=== PHY UPDATED ===");
                    info!("  Handle: 0x{:04X}", handle.0);
                    info!("  TX PHY: {:?}", tx_phy);
                    info!("  RX PHY: {:?}", rx_phy);
                }

                GapEvent::DataLengthChanged {
                    handle,
                    max_tx_octets,
                    max_rx_octets,
                    ..
                } => {
                    info!("=== DATA LENGTH CHANGED ===");
                    info!("  Handle: 0x{:04X}", handle.0);
                    info!("  Max TX: {} bytes", max_tx_octets);
                    info!("  Max RX: {} bytes", max_rx_octets);
                }
            }
        } else {
            // Log other events for debugging
            debug!("Other BLE Event: {:?}", event);
        }
    }
}

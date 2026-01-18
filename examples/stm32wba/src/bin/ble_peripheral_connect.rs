//! BLE Peripheral with Connection Handling Example
//!
//! This example demonstrates BLE connection management:
//! - Advertises as a connectable peripheral
//! - Handles connection and disconnection events
//! - Tracks active connections
//! - Allows disconnection via button press (if available)
//!
//! Hardware: STM32WBA52 or compatible
//!
//! To test:
//! 1. Flash this example to your STM32WBA board
//! 2. Use a BLE scanner app (nRF Connect, LightBlue, etc.)
//! 3. Connect to "Embassy-Peripheral"
//! 4. Observe connection/disconnection events in logs

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::rcc::{
    AHB5Prescaler, AHBPrescaler, APBPrescaler, PllDiv, PllMul, PllPreDiv, PllSource, Sysclk, VoltageScale, mux,
};
use embassy_stm32::rng::{self, Rng};
use embassy_stm32::{Config, bind_interrupts};
use embassy_stm32_wpan::gap::{AdvData, AdvParams, AdvType, GapEvent};
use embassy_stm32_wpan::gatt::{CharProperties, GattEventMask, GattServer, SecurityPermissions, ServiceType, Uuid};
use embassy_stm32_wpan::{Ble, ble_runner, set_rng_instance};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    RNG => rng::InterruptHandler<embassy_stm32::peripherals::RNG>;
});

/// BLE runner task - drives the BLE stack sequencer
#[embassy_executor::task]
async fn ble_runner_task() {
    ble_runner().await
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut config = Config::default();

    // Configure PLL1 (required on WBA)
    config.rcc.pll1 = Some(embassy_stm32::rcc::Pll {
        source: PllSource::HSI,
        prediv: PllPreDiv::DIV1,
        mul: PllMul::MUL30,
        divr: Some(PllDiv::DIV5),
        divq: None,
        divp: Some(PllDiv::DIV30),
        frac: Some(0),
    });

    config.rcc.ahb_pre = AHBPrescaler::DIV1;
    config.rcc.apb1_pre = APBPrescaler::DIV1;
    config.rcc.apb2_pre = APBPrescaler::DIV1;
    config.rcc.apb7_pre = APBPrescaler::DIV1;
    config.rcc.ahb5_pre = AHB5Prescaler::DIV4;
    config.rcc.voltage_scale = VoltageScale::RANGE1;
    config.rcc.sys = Sysclk::PLL1_R;
    config.rcc.mux.rngsel = mux::Rngsel::HSI;

    let p = embassy_stm32::init(config);
    info!("Embassy STM32WBA BLE Peripheral Connection Example");

    // Initialize RNG (required by BLE stack)
    let mut rng = Rng::new(p.RNG, Irqs);
    set_rng_instance(&mut rng as *mut _ as *mut ());
    info!("RNG initialized");

    // Initialize BLE stack
    let mut ble = Ble::new();
    ble.init().expect("BLE initialization failed");
    info!("BLE stack initialized");

    // Spawn the BLE runner task (required for proper BLE operation)
    spawner.spawn(ble_runner_task().expect("Failed to create BLE runner task"));

    // Initialize GATT server with a simple service
    let mut gatt = GattServer::new();
    gatt.init().expect("GATT initialization failed");

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
        ..AdvParams::default()
    };

    // Start advertising
    {
        let mut advertiser = ble.advertiser();
        advertiser
            .start(adv_params.clone(), adv_data.clone(), None)
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
                            embassy_stm32_wpan::gap::ConnectionRole::Central => "Central",
                            embassy_stm32_wpan::gap::ConnectionRole::Peripheral => "Peripheral",
                        }
                    );
                    info!(
                        "  Peer Address: {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                        conn.peer_address.0[5],
                        conn.peer_address.0[4],
                        conn.peer_address.0[3],
                        conn.peer_address.0[2],
                        conn.peer_address.0[1],
                        conn.peer_address.0[0]
                    );
                    info!(
                        "  Interval: {} ({}ms)",
                        conn.params.interval,
                        (conn.params.interval as u32 * 125) / 100
                    );
                    info!("  Latency: {}", conn.params.latency);
                    info!(
                        "  Timeout: {} ({}ms)",
                        conn.params.supervision_timeout,
                        conn.params.supervision_timeout as u32 * 10
                    );
                    info!("  Active connections: {}", ble.connections().count());

                    // Note: Advertising typically stops automatically on connection
                    // If you want to support multiple connections, restart advertising here
                }

                GapEvent::Disconnected { handle, reason } => {
                    info!("=== DISCONNECTION ===");
                    info!("  Handle: 0x{:04X}", handle.0);
                    info!("  Reason: 0x{:02X} ({})", reason, disconnect_reason_str(reason));
                    info!("  Active connections: {}", ble.connections().count());

                    // Restart advertising after disconnection
                    info!("Restarting advertising...");
                    let mut advertiser = ble.advertiser();
                    if let Err(e) = advertiser.start(adv_params.clone(), adv_data.clone(), None) {
                        error!("Failed to restart advertising: {:?}", e);
                    } else {
                        info!("Advertising restarted");
                    }
                }

                GapEvent::ConnectionParamsUpdated {
                    handle,
                    interval,
                    latency,
                    supervision_timeout,
                } => {
                    info!("=== CONNECTION PARAMS UPDATED ===");
                    info!("  Handle: 0x{:04X}", handle.0);
                    info!("  New Interval: {} ({}ms)", interval, (interval as u32 * 125) / 100);
                    info!("  New Latency: {}", latency);
                    info!(
                        "  New Timeout: {} ({}ms)",
                        supervision_timeout,
                        supervision_timeout as u32 * 10
                    );
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

/// Convert disconnect reason code to human-readable string
fn disconnect_reason_str(reason: u8) -> &'static str {
    match reason {
        0x08 => "Connection Timeout",
        0x13 => "Remote User Terminated",
        0x14 => "Remote Low Resources",
        0x15 => "Remote Power Off",
        0x16 => "Local Host Terminated",
        0x1A => "Unsupported Remote Feature",
        0x3B => "Unacceptable Connection Parameters",
        0x3D => "MIC Failure",
        0x3E => "Connection Failed to Establish",
        _ => "Unknown",
    }
}

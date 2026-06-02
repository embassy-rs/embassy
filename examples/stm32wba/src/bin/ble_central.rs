//! BLE Central Example
//!
//! This example demonstrates BLE central role functionality:
//! - Scans for nearby BLE peripherals
//! - Connects to a device (either specific address or first discovered)
//! - Handles connection and disconnection events
//! - Demonstrates connection parameter management
//!
//! Hardware: STM32WBA52 or compatible
//!
//! To test:
//! 1. Flash this example to your STM32WBA board
//! 2. Have a BLE peripheral device advertising nearby
//! 3. Observe the scan, connection, and event handling

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::aes::{self, Aes};
use embassy_stm32::peripherals::{AES as AesPeriph, PKA as PkaPeriph};
use embassy_stm32::pka::{self, Pka};
use embassy_stm32::rcc::{self};
use embassy_stm32::rng::{self, Rng};
use embassy_stm32::{Config, bind_interrupts};
use embassy_stm32_wpan::bluetooth::HCI;
use embassy_stm32_wpan::bluetooth::gap::types::OwnAddressType;
use embassy_stm32_wpan::bluetooth::gap::{ConnectionInitParams, GapEvent, ParsedAdvData, ScanParams, ScanType};
use embassy_stm32_wpan::bluetooth::gap_init::{AddressType, GapInitParams};
use embassy_stm32_wpan::{HighInterruptHandler, LowInterruptHandler, Platform, new_platform};
use stm32wb_hci::event::ConnectionRole;
use stm32wb_hci::{BdAddrType, Event};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    RNG => rng::InterruptHandler<embassy_stm32::peripherals::RNG>;
    AES => aes::InterruptHandler<AesPeriph>;
    PKA => pka::InterruptHandler<PkaPeriph>;
    RADIO => HighInterruptHandler;
    HASH => LowInterruptHandler;
});

/// BLE runner task - drives the BLE stack sequencer
#[embassy_executor::task]
async fn ble_runner_task(platform: &'static Platform) {
    platform.run_ble().await
}

/// Target device name to connect to (set to None to connect to first discovered device)
const TARGET_DEVICE_NAME: Option<&str> = None; // e.g., Some("Embassy-Peripheral")

// ---- Test configuration ----
const ADDR_TYPE: OwnAddressType = OwnAddressType::Random;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut config = Config::default();
    config.rcc = rcc::Config::new_wpan();

    let p = embassy_stm32::init(config);

    info!("Embassy STM32WBA BLE Central Example");

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

    // Initialize BLE stack
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

    // State machine for central role
    let mut state = CentralState::Scanning;

    // Configure scan parameters - active scanning for names
    let scan_params = ScanParams::new()
        .with_scan_type(ScanType::Active)
        .with_interval(0x0050) // 50ms
        .with_window(0x0030) // 30ms
        .with_filter_duplicates(false); // Want to see devices multiple times to catch scan responses

    // Start scanning
    ble.start_scan_observation(scan_params.clone())
        .expect("Failed to start scanning");

    info!("=== BLE Central Started ===");
    if let Some(name) = TARGET_DEVICE_NAME {
        info!("Looking for device: \"{}\"", name);
    } else {
        info!("Will connect to first suitable device found");
    }
    info!("");

    // Main event loop
    loop {
        let event = ble.read_event().await;

        match state {
            CentralState::Scanning => {
                // Process advertising reports
                if let Event::LeAdvertisingReport(reports) = event {
                    for report in reports.iter() {
                        // Skip weak signals
                        if report.rssi.is_none() {
                            continue;
                        }

                        // Parse advertising data
                        let parsed = ParsedAdvData::parse(&report.data);

                        // Check if this device matches our criteria
                        let should_connect = if let Some(target_name) = TARGET_DEVICE_NAME {
                            // Looking for specific name
                            parsed.name == Some(target_name)
                        } else {
                            // Connect to any device that has a name (likely a real peripheral)
                            parsed.name.is_some()
                        };

                        if should_connect {
                            let report_address = match report.address {
                                BdAddrType::Public(addr) => addr,
                                BdAddrType::Random(addr) => addr,
                            };

                            info!("=== Found Target Device ===");
                            info!(
                                "  Address: {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                                report_address.0[5],
                                report_address.0[4],
                                report_address.0[3],
                                report_address.0[2],
                                report_address.0[1],
                                report_address.0[0]
                            );
                            if let Some(name) = parsed.name {
                                info!("  Name: \"{}\"", name);
                            }
                            info!("  RSSI: {} dBm", report.rssi);

                            // Stop scanning
                            ble.stop_scan().expect("Failed to stop scanning");
                            info!("Scanning stopped");

                            // Initiate connection
                            let conn_params = ConnectionInitParams {
                                peer_address: report.address,
                                ..ConnectionInitParams::default()
                            };

                            info!("Initiating connection...");
                            if let Err(e) = ble.connect(&conn_params) {
                                error!("Failed to initiate connection: {:?}", e);
                                // Restart scanning on failure
                                ble.start_scan_observation(scan_params.clone())
                                    .expect("Failed to restart scanning");
                            } else {
                                state = CentralState::Connecting;
                            }
                            break;
                        } else if parsed.name.is_some() {
                            // Log other named devices we see
                            debug!(
                                "Found device: \"{}\" at {} dBm",
                                parsed.name.unwrap_or("?"),
                                report.rssi
                            );
                        }
                    }
                }
            }

            CentralState::Connecting => {
                // Wait for connection complete event
                if let Some(gap_event) = ble.process_event(&event) {
                    match gap_event {
                        GapEvent::Connected(conn) => {
                            info!("=== CONNECTED ===");
                            info!("  Handle: 0x{:04X}", conn.handle.0);
                            info!(
                                "  Role: {}",
                                match conn.role {
                                    ConnectionRole::Central => "Central",
                                    ConnectionRole::Peripheral => "Peripheral",
                                }
                            );
                            info!("  Interval: {}", conn.interval.interval(),);
                            info!("  Latency: {}", conn.interval.conn_latency());
                            info!("  Timeout: {} ", conn.interval.supervision_timeout());

                            state = CentralState::Connected;
                            info!("");
                            info!("Connection established! As a central, you can now:");
                            info!("  - Discover services");
                            info!("  - Read/write characteristics");
                            info!("  - Subscribe to notifications");

                            // Kick off a simple GATT client procedure for demo purposes.
                            let gatt_client = ble.gatt_client();
                            if let Err(e) = gatt_client.discover_all_primary_services(conn.handle.0) {
                                warn!("Failed to start primary service discovery: {:?}", e);
                            }
                        }

                        GapEvent::Disconnected { handle, reason } => {
                            error!("Connection failed or disconnected during setup");
                            error!("  Handle: 0x{:04X}, Reason: 0x{:02X}", handle.0, reason);

                            // Go back to scanning
                            ble.start_scan_observation(scan_params.clone())
                                .expect("Failed to restart scanning");
                            state = CentralState::Scanning;
                            info!("Restarted scanning...");
                        }

                        _ => {}
                    }
                }
            }

            CentralState::Connected => {
                // Handle events while connected
                if let Some(gap_event) = ble.process_event(&event) {
                    match gap_event {
                        GapEvent::Disconnected { handle, reason } => {
                            info!("=== DISCONNECTED ===");
                            info!("  Handle: 0x{:04X}", handle.0);
                            info!("  Reason: 0x{:02X} ({})", reason, disconnect_reason_str(reason));

                            // Go back to scanning
                            ble.start_scan_observation(scan_params.clone())
                                .expect("Failed to restart scanning");
                            state = CentralState::Scanning;
                            info!("Restarted scanning...");
                        }

                        GapEvent::ConnectionParamsUpdated { handle, interval } => {
                            info!("=== CONNECTION PARAMS UPDATED ===");
                            info!("  Handle: 0x{:04X}", handle.0);
                            info!("  New Interval: {}", interval.interval());
                            info!("  New Latency: {}", interval.conn_latency());
                            info!("  New Timeout: {} ", interval.supervision_timeout());
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

                        _ => {}
                    }
                }

                // Log GATT client-side responses (service discovery/read/etc).
                for client_event in ble.process_gatt_client_events(&event) {
                    match client_event {
                        embassy_stm32_wpan::bluetooth::gatt::GattClientEvent::PrimaryServiceFound {
                            conn_handle,
                            start_handle,
                            end_handle,
                            uuid,
                        } => info!(
                            "Service: conn=0x{:04X} start=0x{:04X} end=0x{:04X} uuid={=[u8]:02X}",
                            conn_handle.0, start_handle, end_handle, uuid
                        ),
                        embassy_stm32_wpan::bluetooth::gatt::GattClientEvent::ProcedureComplete {
                            conn_handle,
                            success,
                        } => info!("GATT procedure complete: conn=0x{:04X} success={}", conn_handle.0, success),
                        embassy_stm32_wpan::bluetooth::gatt::GattClientEvent::ErrorResponse {
                            conn_handle,
                            request_opcode,
                            attribute_handle,
                            error_code,
                        } => warn!(
                            "GATT error: conn=0x{:04X} req=0x{:02X} attr=0x{:04X} err=0x{:02X}",
                            conn_handle.0, request_opcode, attribute_handle, error_code
                        ),
                        _ => {}
                    }
                }
            }
        }
    }
}

/// State machine for central role
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CentralState {
    /// Scanning for devices
    Scanning,
    /// Connection initiated, waiting for connection complete
    Connecting,
    /// Connected to a peripheral
    Connected,
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

//! BLE Central Example
//!
//! This example demonstrates BLE central role functionality:
//! - Scans for nearby BLE peripherals
//! - Connects to a device (either specific address or first discovered)
//! - Handles connection and disconnection events
//! - Demonstrates connection parameter management
//!
//! Hardware: STM32WBA65 or compatible
//!
//! To test:
//! 1. Flash this example to your STM32WBA6 board
//! 2. Have a BLE peripheral device advertising nearby
//! 3. Observe the scan, connection, and event handling

#![no_std]
#![no_main]

use core::cell::RefCell;

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::peripherals::RNG;
use embassy_stm32::rcc::{
    AHB5Prescaler, AHBPrescaler, APBPrescaler, PllDiv, PllMul, PllPreDiv, PllSource, Sysclk, VoltageScale, mux,
};
use embassy_stm32::rng::{self, Rng};
use embassy_stm32::{Config, bind_interrupts};
use embassy_stm32_wpan::gap::{ConnectionInitParams, GapEvent, ParsedAdvData, ScanParams, ScanType};
use embassy_stm32_wpan::hci::event::EventParams;
use embassy_stm32_wpan::{Ble, ble_runner};
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    RNG => rng::InterruptHandler<embassy_stm32::peripherals::RNG>;
});

/// BLE runner task - drives the BLE stack sequencer
#[embassy_executor::task]
async fn ble_runner_task() {
    ble_runner().await
}

/// Target device name to connect to (set to None to connect to first discovered device)
const TARGET_DEVICE_NAME: Option<&str> = None; // e.g., Some("Embassy-Peripheral")

/// Minimum RSSI to consider a device (helps filter out far away devices)
const MIN_RSSI: i8 = -80;

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
    info!("Embassy STM32WBA6 BLE Central Example");

    // Initialize RNG (required by BLE stack)
    static RNG: StaticCell<Mutex<CriticalSectionRawMutex, RefCell<Rng<'static, RNG>>>> = StaticCell::new();
    let rng = RNG.init(Mutex::new(RefCell::new(Rng::new(p.RNG, Irqs))));
    info!("RNG initialized");

    // Initialize BLE stack
    let mut ble = Ble::new(rng);
    ble.init().expect("BLE initialization failed");
    info!("BLE stack initialized");

    // Spawn the BLE runner task (required for proper BLE operation)
    spawner.spawn(ble_runner_task().unwrap());

    // State machine for central role
    let mut state = CentralState::Scanning;

    // Configure scan parameters - active scanning for names
    let scan_params = ScanParams::new()
        .with_scan_type(ScanType::Active)
        .with_interval(0x0050) // 50ms
        .with_window(0x0030) // 30ms
        .with_filter_duplicates(false); // Want to see devices multiple times to catch scan responses

    // Start scanning
    {
        let mut scanner = ble.scanner();
        scanner.start(scan_params.clone()).expect("Failed to start scanning");
    }

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
                if let EventParams::LeAdvertisingReport { reports } = &event.params {
                    for report in reports.iter() {
                        // Skip weak signals
                        if report.rssi < MIN_RSSI {
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
                            info!("=== Found Target Device ===");
                            info!(
                                "  Address: {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                                report.address.0[5],
                                report.address.0[4],
                                report.address.0[3],
                                report.address.0[2],
                                report.address.0[1],
                                report.address.0[0]
                            );
                            if let Some(name) = parsed.name {
                                info!("  Name: \"{}\"", name);
                            }
                            info!("  RSSI: {} dBm", report.rssi);

                            // Stop scanning
                            {
                                let mut scanner = ble.scanner();
                                scanner.stop().expect("Failed to stop scanning");
                            }
                            info!("Scanning stopped");

                            // Initiate connection
                            let conn_params = ConnectionInitParams {
                                peer_address_type: report.address_type,
                                peer_address: report.address,
                                ..ConnectionInitParams::default()
                            };

                            info!("Initiating connection...");
                            if let Err(e) = ble.connect(&conn_params) {
                                error!("Failed to initiate connection: {:?}", e);
                                // Restart scanning on failure
                                let mut scanner = ble.scanner();
                                scanner.start(scan_params.clone()).expect("Failed to restart scanning");
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
                                    embassy_stm32_wpan::gap::ConnectionRole::Central => "Central",
                                    embassy_stm32_wpan::gap::ConnectionRole::Peripheral => "Peripheral",
                                }
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

                            state = CentralState::Connected;
                            info!("");
                            info!("Connection established! As a central, you can now:");
                            info!("  - Discover services (not implemented in this example)");
                            info!("  - Read/write characteristics");
                            info!("  - Subscribe to notifications");
                        }

                        GapEvent::Disconnected { handle, reason } => {
                            error!("Connection failed or disconnected during setup");
                            error!("  Handle: 0x{:04X}, Reason: 0x{:02X}", handle.0, reason);

                            // Go back to scanning
                            let mut scanner = ble.scanner();
                            scanner.start(scan_params.clone()).expect("Failed to restart scanning");
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
                            let mut scanner = ble.scanner();
                            scanner.start(scan_params.clone()).expect("Failed to restart scanning");
                            state = CentralState::Scanning;
                            info!("Restarted scanning...");
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

                        _ => {}
                    }
                }

                // Log other interesting events
                match &event.params {
                    EventParams::AttExchangeMtuResponse {
                        conn_handle,
                        server_mtu,
                    } => {
                        info!("MTU Exchange: conn 0x{:04X}, MTU={}", conn_handle.0, server_mtu);
                    }
                    _ => {}
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

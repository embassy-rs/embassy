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

use core::cell::RefCell;

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::aes::{self, Aes};
use embassy_stm32::mode::Blocking;
use embassy_stm32::peripherals::{AES as AesPeriph, PKA as PkaPeriph, RNG};
use embassy_stm32::pka::{self, Pka};
use embassy_stm32::rcc::{
    AHB5Prescaler, AHBPrescaler, APBPrescaler, Hse, HsePrescaler, LsConfig, LseConfig, LseDrive, LseMode, PllDiv,
    PllMul, PllPreDiv, PllSource, RtcClockSource, Sysclk, VoltageScale, mux,
};
use embassy_stm32::rng::{self, Rng};
use embassy_stm32::time::Hertz;
use embassy_stm32::{Config, bind_interrupts};
use embassy_stm32_wpan::bluetooth::ble::Ble;
use embassy_stm32_wpan::bluetooth::gap::{ConnectionInitParams, GapEvent, ParsedAdvData, ScanParams, ScanType};
use embassy_stm32_wpan::{ChannelPacket, Controller, HighInterruptHandler, LowInterruptHandler, ble_runner};
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::zerocopy_channel;
use static_cell::StaticCell;
use stm32wb_hci::event::ConnectionRole;
use stm32wb_hci::vendor::event::{AttExchangeMtuResponse, VendorEvent};
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
async fn ble_runner_task() {
    ble_runner().await
}

/// Target device name to connect to (set to None to connect to first discovered device)
const TARGET_DEVICE_NAME: Option<&str> = None; // e.g., Some("Embassy-Peripheral")

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut config = Config::default();

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
    config.rcc.pll1 = Some(embassy_stm32::rcc::Pll {
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

    let p = embassy_stm32::init(config);

    // Configure radio sleep timer to use LSE
    {
        use embassy_stm32::pac::RCC;
        use embassy_stm32::pac::rcc::vals::Radiostsel;
        RCC.bdcr().modify(|w| w.set_radiostsel(Radiostsel::Lse));
    }

    info!("Embassy STM32WBA BLE Central Example");

    // Initialize hardware peripherals required by BLE stack
    static RNG_INST: StaticCell<Mutex<CriticalSectionRawMutex, RefCell<Rng<'static, RNG>>>> = StaticCell::new();
    let rng = RNG_INST.init(Mutex::new(RefCell::new(Rng::new(p.RNG, Irqs))));

    static AES_INST: StaticCell<Mutex<CriticalSectionRawMutex, RefCell<Aes<'static, AesPeriph, Blocking>>>> =
        StaticCell::new();
    let aes = AES_INST.init(Mutex::new(RefCell::new(Aes::new_blocking(p.AES, Irqs))));

    static PKA_INST: StaticCell<Mutex<CriticalSectionRawMutex, RefCell<Pka<'static, PkaPeriph>>>> = StaticCell::new();
    let pka = PKA_INST.init(Mutex::new(RefCell::new(Pka::new_blocking(p.PKA, Irqs))));

    info!("Hardware peripherals initialized (RNG, AES, PKA)");

    // Spawn the BLE runner task (required for proper BLE operation)
    spawner.spawn(ble_runner_task().expect("Failed to spawn BLE runner"));

    // Create BLE Event Channel
    static EVENT_BUFFER: StaticCell<[ChannelPacket; 8]> = StaticCell::new();
    static EVENT_CHANNEL: StaticCell<zerocopy_channel::Channel<'static, CriticalSectionRawMutex, ChannelPacket>> =
        StaticCell::new();

    let event_channel = EVENT_CHANNEL.init(zerocopy_channel::Channel::new(
        EVENT_BUFFER.init([ChannelPacket::default(); 8]),
    ));

    // Initialize BLE stack
    let controller = Controller::new(event_channel, rng, Some(aes), Some(pka), Irqs)
        .await
        .expect("BLE initialization failed");

    let mut ble = Ble::new(controller).await.unwrap();
    info!("BLE stack initialized");

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
                            {
                                let mut scanner = ble.scanner();
                                scanner.stop().expect("Failed to stop scanning");
                            }
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

                // Log other interesting events
                match &event {
                    Event::Vendor(VendorEvent::AttExchangeMtuResponse(AttExchangeMtuResponse {
                        conn_handle,
                        server_rx_mtu,
                    })) => {
                        info!("MTU Exchange: conn 0x{:04X}, MTU={}", conn_handle.0, server_rx_mtu);
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

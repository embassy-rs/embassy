//! BLE Scanner Example
//!
//! This example demonstrates BLE scanning (observer role):
//! - Scans for nearby BLE devices
//! - Parses advertising data (name, UUIDs, manufacturer data)
//! - Displays discovered devices with RSSI
//!
//! Hardware: STM32WBA65 or compatible
//!
//! To test:
//! 1. Flash this example to your STM32WBA6 board
//! 2. Have nearby BLE devices advertising (phones, beacons, etc.)
//! 3. Observe discovered devices in the logs

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::aes::{self, Aes};
use embassy_stm32::peripherals::{AES, PKA, RNG};
use embassy_stm32::pka::{self, Pka};
use embassy_stm32::rng::{self, Rng};
use embassy_stm32::{Config, bind_interrupts, rcc};
use embassy_stm32_wpan::bluetooth::gap_init::GapRole;
use embassy_stm32_wpan::bluetooth::gap::{ParsedAdvData, ScanParams, ScanType};
use embassy_stm32_wpan::bluetooth::HCI;
use embassy_stm32_wpan::{HighInterruptHandler, LowInterruptHandler, Platform, new_platform};
use stm32wb_hci::Event;
use {defmt_rtt as _, panic_probe as _};

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
    config.rcc = rcc::Config::new_wpan();

    let p = embassy_stm32::init(config);

    info!("Embassy STM32WBA6 BLE Scanner Example");

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

    // Initialize BLE stack in Observer role (required for ACI_GAP_START_OBSERVATION_PROC)
    let mut ble = HCI::new_with_role(platform, runtime, Irqs, GapRole::Observer)
        .await
        .expect("BLE initialization failed");

    info!("BLE stack initialized");

    // Configure scan parameters
    // Using active scanning to get scan response data (device names)
    let scan_params = ScanParams::new()
        .with_scan_type(ScanType::Active)
        .with_interval(0x0050) // 50ms
        .with_window(0x0030) // 30ms
        .with_filter_duplicates(true);

    // Start scanning
    let mut scanner = ble.scanner();
    scanner.start(scan_params).expect("Failed to start scanning");

    info!("=== BLE Scanning Started ===");
    info!("Looking for nearby devices...");
    info!("");

    let mut device_count = 0u32;

    // Main event loop - process advertising reports
    loop {
        let event = ble.read_event().await;

        // Check for advertising reports
        if let Event::LeAdvertisingReport(reports) = &event {
            for report in reports.iter() {
                device_count += 1;

                // Parse the advertising data
                let parsed = ParsedAdvData::parse(&report.data);

                info!("--- Device #{} ---", device_count);

                // Display device address
                info!("  Address: {}", report.address);

                // Display RSSI
                info!("  RSSI: {} dBm", report.rssi);

                // Display event type
                info!("  Type: {}", report.event_type);

                // Display parsed name if available
                if let Some(name) = parsed.name {
                    info!("  Name: \"{}\"", name);
                }

                // Display flags if available
                if let Some(flags) = parsed.flags {
                    info!("  Flags: 0x{:02X} ({})", flags, flags_str(flags));
                }

                // Display TX power if available
                if let Some(tx_power) = parsed.tx_power {
                    info!("  TX Power: {} dBm", tx_power);
                }

                // Display 16-bit service UUIDs
                if !parsed.service_uuids_16.is_empty() {
                    for uuid in parsed.service_uuids_16.iter() {
                        info!("  Service UUID: 0x{:04X} ({})", uuid, service_uuid_str(*uuid));
                    }
                }

                // Display 128-bit service UUIDs
                if !parsed.service_uuids_128.is_empty() {
                    for uuid in parsed.service_uuids_128.iter() {
                        info!(
                            "  Service UUID (128): {:02X}{:02X}{:02X}{:02X}-{:02X}{:02X}-{:02X}{:02X}-{:02X}{:02X}-{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}",
                            uuid[15],
                            uuid[14],
                            uuid[13],
                            uuid[12],
                            uuid[11],
                            uuid[10],
                            uuid[9],
                            uuid[8],
                            uuid[7],
                            uuid[6],
                            uuid[5],
                            uuid[4],
                            uuid[3],
                            uuid[2],
                            uuid[1],
                            uuid[0]
                        );
                    }
                }

                // Display manufacturer data
                if let Some((company_id, data)) = parsed.manufacturer_data {
                    info!(
                        "  Manufacturer: 0x{:04X} ({}) - {} bytes",
                        company_id,
                        company_id_str(company_id),
                        data.len()
                    );
                }

                info!("");
            }
        }
    }
}

/// Convert flags byte to description
fn flags_str(flags: u8) -> &'static str {
    match flags {
        0x06 => "LE General Discoverable",
        0x04 => "BR/EDR Not Supported",
        0x02 => "LE General Discoverable Only",
        0x01 => "LE Limited Discoverable",
        0x05 => "LE Limited + BR/EDR Not Supported",
        _ => "Other",
    }
}

/// Convert common 16-bit service UUID to name
fn service_uuid_str(uuid: u16) -> &'static str {
    match uuid {
        0x1800 => "Generic Access",
        0x1801 => "Generic Attribute",
        0x180A => "Device Information",
        0x180D => "Heart Rate",
        0x180F => "Battery",
        0x1810 => "Blood Pressure",
        0x1816 => "Cycling Speed and Cadence",
        0x181A => "Environmental Sensing",
        0x181C => "User Data",
        0xFE9F => "Google Fast Pair",
        0xFD6F => "Apple Exposure Notification",
        _ => "Unknown",
    }
}

/// Convert common company IDs to names
fn company_id_str(company_id: u16) -> &'static str {
    match company_id {
        0x004C => "Apple",
        0x0006 => "Microsoft",
        0x00E0 => "Google",
        0x0075 => "Samsung",
        0x0059 => "Nordic Semiconductor",
        0x0030 => "STMicroelectronics",
        0x00D2 => "Huawei",
        0x038F => "Xiaomi",
        _ => "Unknown",
    }
}

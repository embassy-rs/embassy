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

use core::cell::RefCell;

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::peripherals::RNG;
use embassy_stm32::rcc::{
    AHB5Prescaler, AHBPrescaler, APBPrescaler, PllDiv, PllMul, PllPreDiv, PllSource, Sysclk, VoltageScale, mux,
};
use embassy_stm32::rng::{self, Rng};
use embassy_stm32::{Config, bind_interrupts};
use embassy_stm32_wpan::gap::{ParsedAdvData, ScanParams, ScanType};
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
    info!("Embassy STM32WBA6 BLE Scanner Example");

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
        if let EventParams::LeAdvertisingReport { reports } = &event.params {
            for report in reports.iter() {
                device_count += 1;

                // Parse the advertising data
                let parsed = ParsedAdvData::parse(&report.data);

                info!("--- Device #{} ---", device_count);

                // Display device address
                info!(
                    "  Address: {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X} ({})",
                    report.address.0[5],
                    report.address.0[4],
                    report.address.0[3],
                    report.address.0[2],
                    report.address.0[1],
                    report.address.0[0],
                    address_type_str(report.address_type)
                );

                // Display RSSI
                info!("  RSSI: {} dBm", report.rssi);

                // Display event type
                info!("  Type: {}", adv_event_type_str(report.event_type));

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

/// Convert address type to string
fn address_type_str(addr_type: embassy_stm32_wpan::hci::types::AddressType) -> &'static str {
    match addr_type {
        embassy_stm32_wpan::hci::types::AddressType::Public => "Public",
        embassy_stm32_wpan::hci::types::AddressType::Random => "Random",
        embassy_stm32_wpan::hci::types::AddressType::PublicIdentity => "Public Identity",
        embassy_stm32_wpan::hci::types::AddressType::RandomIdentity => "Random Identity",
    }
}

/// Convert advertising event type to string
fn adv_event_type_str(event_type: u8) -> &'static str {
    match event_type {
        0x00 => "Connectable Undirected",
        0x01 => "Connectable Directed",
        0x02 => "Scannable Undirected",
        0x03 => "Non-Connectable Undirected",
        0x04 => "Scan Response",
        _ => "Unknown",
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

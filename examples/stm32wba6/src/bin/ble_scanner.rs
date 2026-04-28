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
use embassy_stm32::aes::{self, Aes};
use embassy_stm32::mode::Blocking;
use embassy_stm32::peripherals::{AES, PKA, RNG};
use embassy_stm32::pka::{self, Pka};
use embassy_stm32::rcc::{
    AHB5Prescaler, AHBPrescaler, APBPrescaler, Hse, HsePrescaler, LsConfig, LseConfig, LseDrive, LseMode, PllDiv,
    PllMul, PllPreDiv, PllSource, RtcClockSource, Sysclk, VoltageScale, mux,
};
use embassy_stm32::rng::{self, Rng};
use embassy_stm32::time::Hertz;
use embassy_stm32::{Config, bind_interrupts};
use embassy_stm32_wpan::bluetooth::ble::Ble;
use embassy_stm32_wpan::bluetooth::gap::{ParsedAdvData, ScanParams, ScanType};
use embassy_stm32_wpan::{ChannelPacket, Controller, HighInterruptHandler, LowInterruptHandler, ble_runner};
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::zerocopy_channel;
use static_cell::StaticCell;
use stm32wb_hci::Event;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    RNG => rng::InterruptHandler<RNG>;
    AES => aes::InterruptHandler<AES>;
    PKA => pka::InterruptHandler<PKA>;
    RADIO => HighInterruptHandler;
    HASH => LowInterruptHandler;
});

/// BLE runner task - drives the BLE stack sequencer
#[embassy_executor::task]
async fn ble_runner_task() {
    ble_runner().await
}

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

    // Apply HSE trimming for accurate radio frequency (matching ST's Config_HSE)
    // and configure radio sleep timer to use LSE
    {
        use embassy_stm32::pac::RCC;
        use embassy_stm32::pac::rcc::vals::Radiostsel;
        RCC.ecscr1().modify(|w| w.set_hsetrim(0x0C));
        RCC.bdcr().modify(|w| w.set_radiostsel(Radiostsel::Lse));
    }

    info!("Embassy STM32WBA6 BLE Scanner Example");

    // Initialize hardware peripherals required by BLE stack
    static RNG_INST: StaticCell<Mutex<CriticalSectionRawMutex, RefCell<Rng<'static, RNG>>>> = StaticCell::new();
    let rng = RNG_INST.init(Mutex::new(RefCell::new(Rng::new(p.RNG, Irqs))));

    static AES_INST: StaticCell<Mutex<CriticalSectionRawMutex, RefCell<Aes<'static, AES, Blocking>>>> =
        StaticCell::new();
    let aes = AES_INST.init(Mutex::new(RefCell::new(Aes::new_blocking(p.AES, Irqs))));

    static PKA_INST: StaticCell<Mutex<CriticalSectionRawMutex, RefCell<Pka<'static, PKA>>>> = StaticCell::new();
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

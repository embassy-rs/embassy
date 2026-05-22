//! BLE Beacon Example
//!
//! Demonstrates four beacon advertisement formats, rotating every 5 seconds:
//!
//! 1. **iBeacon** (Apple): manufacturer-specific data with Apple company ID (0x004C)
//! 2. **Eddystone-UID**: 10-byte namespace + 6-byte instance identifier
//! 3. **Eddystone-URL**: encoded URL (https://www.st.com)
//! 4. **Eddystone-TLM**: telemetry frame with uptime and advertisement count
//!
//! All beacons are non-connectable, undirected. No GATT server is needed.
//!
//! Based on ST's BLE_Beacon example for NUCLEO-WBA65RI.
//!
//! ## Testing
//! 1. Flash to NUCLEO-WBA65RI
//! 2. Open a BLE scanner app (nRF Connect, LightBlue, Beacon Scanner)
//! 3. Watch the advertisement payload rotate every 5 seconds

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_futures::select::{Either, select};
use embassy_stm32::aes::{self, Aes};
use embassy_stm32::peripherals::{AES, PKA, RNG};
use embassy_stm32::pka::{self, Pka};
use embassy_stm32::rng::{self, Rng};
use embassy_stm32::{Config, bind_interrupts, rcc};
use embassy_stm32_wpan::bluetooth::HCI;
use embassy_stm32_wpan::bluetooth::gap::{AdvData, AdvParams, AdvType};
use embassy_stm32_wpan::{HighInterruptHandler, LowInterruptHandler, Platform, new_platform};
use embassy_time::{Duration, Ticker};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    RNG => rng::InterruptHandler<RNG>;
    AES => aes::InterruptHandler<AES>;
    PKA => pka::InterruptHandler<PKA>;
    RADIO => HighInterruptHandler;
    HASH => LowInterruptHandler;
});

#[embassy_executor::task]
async fn rng_runner_task(platform: &'static Platform) {
    platform.run_rng().await
}

#[embassy_executor::task]
async fn ble_runner_task(platform: &'static Platform) {
    platform.run_ble().await
}

// ── iBeacon ──────────────────────────────────────────────────────────────────
// Apple company ID (little-endian)
const APPLE_COMPANY_ID: u16 = 0x004C;
// 16-byte proximity UUID (change for your deployment)
const IBEACON_UUID: [u8; 16] = [
    0xE2, 0xC5, 0x6D, 0xB5, 0xDF, 0xFB, 0x48, 0xD2, 0xB0, 0x60, 0xD0, 0xF5, 0xA7, 0x10, 0x96, 0xE0,
];
const IBEACON_MAJOR: [u8; 2] = [0x00, 0x01]; // major = 1
const IBEACON_MINOR: [u8; 2] = [0x00, 0x01]; // minor = 1
const IBEACON_TX_POWER: i8 = -59; // measured RSSI at 1 m

fn build_ibeacon() -> AdvData {
    // Manufacturer-specific payload: [type=0x02, length=0x15, uuid[16], major[2], minor[2], power]
    let mut payload = [0u8; 22];
    payload[0] = 0x02; // iBeacon type
    payload[1] = 0x15; // remaining length
    payload[2..18].copy_from_slice(&IBEACON_UUID);
    payload[18..20].copy_from_slice(&IBEACON_MAJOR);
    payload[20..22].copy_from_slice(&IBEACON_MINOR);
    // tx_power is appended as the last byte of the manufacturer data via add_manufacturer_data
    // but add_manufacturer_data only takes company_id + data, so we embed it in payload
    // Final layout from add_manufacturer_data(0x004C, payload): company_id[2] + payload[22]
    // We need: 0x4C 0x00 0x02 0x15 uuid[16] major[2] minor[2] power[1] = 24 bytes → AD len = 25
    // Rebuild to include tx_power in the data slice:
    let mut data = [0u8; 23];
    data[..22].copy_from_slice(&payload);
    data[22] = IBEACON_TX_POWER as u8;

    let mut adv = AdvData::new();
    adv.add_flags(0x04).unwrap(); // BR/EDR not supported (non-discoverable)
    adv.add_manufacturer_data(APPLE_COMPANY_ID, &data).unwrap();
    adv
}

// ── Eddystone-UID ─────────────────────────────────────────────────────────────
// Eddystone service UUID
const EDDYSTONE_UUID: u16 = 0xFEAA;
// 10-byte namespace (e.g. hash of your FQDN)
const EDDYSTONE_NAMESPACE: [u8; 10] = [0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09];
// 6-byte instance (identifies this specific beacon)
const EDDYSTONE_INSTANCE: [u8; 6] = [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF];
const EDDYSTONE_TX_POWER: i8 = -20; // measured RSSI at 0 m

fn build_eddystone_uid() -> AdvData {
    // Service data: [frame_type=0x00, tx_power, namespace[10], instance[6], rsvd[2]]
    let mut data = [0u8; 20];
    data[0] = 0x00; // UID frame type
    data[1] = EDDYSTONE_TX_POWER as u8;
    data[2..12].copy_from_slice(&EDDYSTONE_NAMESPACE);
    data[12..18].copy_from_slice(&EDDYSTONE_INSTANCE);
    // bytes 18..20 reserved, already zero

    let mut adv = AdvData::new();
    adv.add_flags(0x06).unwrap(); // General discoverable, no BR/EDR
    adv.add_service_uuid_16(EDDYSTONE_UUID).unwrap();
    adv.add_service_data(EDDYSTONE_UUID, &data).unwrap();
    adv
}

// ── Eddystone-URL ─────────────────────────────────────────────────────────────
// URL scheme prefix: 0x03 = "https://"
// Encoded URL suffixes: 0x00=".com/", 0x01=".org/", 0x02=".edu/", 0x03=".net/", 0x04=".info/"
// Encodes "https://www.st.com" as: scheme=0x03, "www", ".st", ".com"
// Using "https://" (0x03) + "www.st" + 0x00 (".com") = 8 bytes URL data + 2 header = 10 total
const URL_SCHEME_HTTPS: u8 = 0x03; // "https://"

fn build_eddystone_url() -> AdvData {
    // "https://www.st.com" → scheme=0x03, body=b"www.st" + 0x00
    let url_body: &[u8] = b"www.st\x00"; // 0x00 = ".com" expansion
    let url_len = 2 + url_body.len(); // frame_type + tx_power + scheme + body
    let mut data = heapless::Vec::<u8, 16>::new();
    data.push(0x10).unwrap(); // URL frame type
    data.push(EDDYSTONE_TX_POWER as u8).unwrap();
    data.push(URL_SCHEME_HTTPS).unwrap();
    data.extend_from_slice(url_body).unwrap();
    let _ = url_len; // used only for clarity

    let mut adv = AdvData::new();
    adv.add_flags(0x06).unwrap();
    adv.add_service_uuid_16(EDDYSTONE_UUID).unwrap();
    adv.add_service_data(EDDYSTONE_UUID, &data).unwrap();
    adv
}

// ── Eddystone-TLM ─────────────────────────────────────────────────────────────
fn build_eddystone_tlm(adv_count: u32, uptime_100ms: u32) -> AdvData {
    // Telemetry frame: version=0x00, battery=0 (not supported), temp, adv_count, time
    // Temperature: 8.8 fixed point (integer part | fractional/256)
    // Encode 23.5°C as [23, 128] (128/256 = 0.5)
    let temp_int: i8 = 23;
    let temp_frac: u8 = 128; // 0.5 °C

    let mut data = [0u8; 14];
    data[0] = 0x20; // TLM frame type
    data[1] = 0x00; // TLM version 0
    data[2] = 0x00; // battery voltage MSB (0 = not supported)
    data[3] = 0x00; // battery voltage LSB
    data[4] = temp_int as u8;
    data[5] = temp_frac;
    data[6..10].copy_from_slice(&adv_count.to_be_bytes());
    data[10..14].copy_from_slice(&uptime_100ms.to_be_bytes());

    let mut adv = AdvData::new();
    adv.add_flags(0x06).unwrap();
    adv.add_service_uuid_16(EDDYSTONE_UUID).unwrap();
    adv.add_service_data(EDDYSTONE_UUID, &data).unwrap();
    adv
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut config = Config::default();
    config.rcc = rcc::Config::new_wpan();
    let p = embassy_stm32::init(config);

    info!("Embassy STM32WBA6 BLE Beacon Example");

    let (platform, runtime) = new_platform!(
        Rng::new(p.RNG, Irqs),
        Aes::new_blocking(p.AES, Irqs),
        Pka::new_blocking(p.PKA, Irqs),
        8
    );

    spawner.spawn(rng_runner_task(platform).expect("Failed to spawn rng runner"));
    spawner.spawn(ble_runner_task(platform).expect("Failed to spawn BLE runner"));

    let mut ble = HCI::new(platform, runtime, Irqs).await.expect("BLE init failed");

    // Non-connectable, undirected advertising at ~320 ms interval
    let adv_params = AdvParams {
        interval_min: 0x0200, // 320 ms (512 * 0.625 ms)
        interval_max: 0x0200,
        adv_type: AdvType::NonConnectableUndirected,
        ..AdvParams::default()
    };

    // Start with minimal data; update_adv_data sets the real payload below
    let mut initial_adv = AdvData::new();
    initial_adv.add_flags(0x04).unwrap();
    ble.start_advertising(adv_params, initial_adv, None)
        .await
        .expect("Failed to start advertising");

    info!("Non-connectable advertising started");
    info!("Rotating beacon types every 5 seconds");

    let beacon_names = ["iBeacon", "Eddystone-UID", "Eddystone-URL", "Eddystone-TLM"];
    let mut beacon_idx = 0usize;
    let mut adv_count: u32 = 0;
    let mut uptime_ticks: u32 = 0; // units: 100 ms

    // Set the first beacon type immediately
    let first = build_ibeacon();
    ble.update_adv_data(first).expect("Failed to set beacon data");
    info!("Beacon type: {}", beacon_names[0]);

    let mut ticker = Ticker::every(Duration::from_secs(5));

    loop {
        match select(ble.read_event(), ticker.next()).await {
            Either::First(_event) => {
                // Non-connectable beacons generate no connection events; ignore.
            }
            Either::Second(_) => {
                beacon_idx = (beacon_idx + 1) % 4;
                adv_count = adv_count.wrapping_add(1);
                uptime_ticks = uptime_ticks.wrapping_add(50); // 5 s = 50 × 100 ms ticks

                let adv_data = match beacon_idx {
                    0 => build_ibeacon(),
                    1 => build_eddystone_uid(),
                    2 => build_eddystone_url(),
                    3 => build_eddystone_tlm(adv_count, uptime_ticks),
                    _ => core::unreachable!(),
                };
                ble.update_adv_data(adv_data).expect("Failed to update beacon data");
                info!("Beacon type: {}", beacon_names[beacon_idx]);
            }
        }
    }
}

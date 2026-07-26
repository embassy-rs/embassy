//! BLE Heart Rate Profile Example
//!
//! Implements the Bluetooth SIG Heart Rate Profile (HRP) as a GATT server.
//! The simulated sensor advertises as "HRS_xx" and sends heart rate
//! measurements as GATT notifications every second.
//!
//! ## GATT structure
//! - **Heart Rate Service** (UUID 0x180D)
//!   - Heart Rate Measurement (0x2A37) — NOTIFY
//!     - Flags byte: 8-bit value, no EE, no RR intervals
//!     - Heart Rate Value (uint8)
//!   - Body Sensor Location (0x2A38) — READ
//!     - 0x02 = Wrist
//!   - HR Control Point (0x2A39) — WRITE (reset energy expended)
//!
//! Based on ST's BLE_HeartRate example for NUCLEO-WBA65RI.
//!
//! ## Testing
//! 1. Flash to NUCLEO-WBA65RI
//! 2. Connect with nRF Connect or a health/fitness app
//! 3. Open the Heart Rate service and enable notifications
//! 4. Observe simulated BPM values updating every second

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
use embassy_stm32_wpan::bluetooth::gap::{AdvData, AdvParams, AdvType, GapEvent};
use embassy_stm32_wpan::bluetooth::gatt::{
    CharProperties, CharacteristicHandle, GattEventMask, SecurityPermissions, ServiceHandle, ServiceType, Uuid,
    is_cccd_handle, is_value_handle,
};
use embassy_stm32_wpan::{HighInterruptHandler, LowInterruptHandler, Platform, new_platform};
use embassy_time::{Duration, Ticker};
use stm32wb_hci::Event;
use stm32wb_hci::vendor::event::{AttExchangeMtuResponse, VendorEvent};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    RNG => rng::InterruptHandler<RNG>;
    AES => aes::InterruptHandler<AES>;
    PKA => pka::InterruptHandler<PKA>;
    RADIO => HighInterruptHandler;
    HASH => LowInterruptHandler;
});

#[embassy_executor::task]
async fn ble_runner_task(platform: &'static Platform) {
    platform.run_ble().await
}

// ── GATT UUIDs (Bluetooth SIG assigned) ──────────────────────────────────────
const HRS_SERVICE_UUID: u16 = 0x180D; // Heart Rate Service
const HRM_CHAR_UUID: u16 = 0x2A37; // Heart Rate Measurement
const BSL_CHAR_UUID: u16 = 0x2A38; // Body Sensor Location
const HRCP_CHAR_UUID: u16 = 0x2A39; // HR Control Point

// Body Sensor Location value: 0x02 = Wrist
const BODY_SENSOR_LOCATION_WRIST: u8 = 0x02;

// HRCP command: 0x01 = Reset Energy Expended
const HRCP_RESET_ENERGY_EXPENDED: u8 = 0x01;

struct HrsState {
    service_handle: ServiceHandle,
    hrm_char_handle: CharacteristicHandle,
    hrcp_char_handle: CharacteristicHandle,
    notifications_enabled: bool,
    conn_handle: Option<u16>,
    // Simulated sensor state
    heart_rate: u8,
    hr_direction: i8, // +1 or -1
}

/// Encode a Heart Rate Measurement value.
///
/// Format (minimal, flags = 0x00):
/// - Byte 0: Flags (0x00 = 8-bit HR, no sensor contact, no EE, no RR)
/// - Byte 1: Heart Rate Value (uint8, BPM)
fn encode_hrm(bpm: u8) -> [u8; 2] {
    [0x00, bpm]
}

/// Simulate a realistic heart rate that wanders between 60 and 100 BPM.
fn next_heart_rate(current: u8, direction: &mut i8) -> u8 {
    let next = current as i16 + *direction as i16;
    if next >= 100 {
        *direction = -1;
        99
    } else if next <= 60 {
        *direction = 1;
        61
    } else {
        next as u8
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut config = Config::default();
    config.rcc = rcc::Config::new_wpan();
    let p = embassy_stm32::init(config);

    info!("Embassy STM32WBA6 BLE Heart Rate Profile Example");

    let (platform, runtime) = new_platform!(
        Rng::new(p.RNG, Irqs),
        Pka::new(p.PKA, Irqs),
        Aes::new_blocking(p.AES, Irqs),
        8
    );

    spawner.spawn(ble_runner_task(platform).expect("Failed to spawn BLE runner"));

    let mut ble = HCI::new(platform, runtime, Irqs).await.expect("BLE init failed");

    embassy_futures::yield_now().await;

    // ── Build GATT server ────────────────────────────────────────────────────
    let mut gatt = ble.gatt_server();

    // Heart Rate Service: 1 service + 3 chars + 1 CCCD + 1 write permit = 8 attribute records
    let service_handle = gatt
        .add_service(Uuid::from_u16(HRS_SERVICE_UUID), ServiceType::Primary, 8)
        .expect("Failed to add HRS service");

    // Heart Rate Measurement: NOTIFY, variable length (max 5 bytes for minimal format)
    let hrm_char_handle = gatt
        .add_characteristic(
            service_handle,
            Uuid::from_u16(HRM_CHAR_UUID),
            5,
            CharProperties::NOTIFY,
            SecurityPermissions::NONE,
            GattEventMask::empty(),
            0,
            true,
        )
        .expect("Failed to add HRM characteristic");

    // Body Sensor Location: READ only, 1 byte constant
    let bsl_char_handle = gatt
        .add_characteristic(
            service_handle,
            Uuid::from_u16(BSL_CHAR_UUID),
            1,
            CharProperties::READ,
            SecurityPermissions::NONE,
            GattEventMask::empty(),
            0,
            false,
        )
        .expect("Failed to add BSL characteristic");

    // Set body sensor location to Wrist
    gatt.update_characteristic_value(service_handle, bsl_char_handle, 0, &[BODY_SENSOR_LOCATION_WRIST])
        .expect("Failed to set BSL value");

    // HR Control Point: WRITE (requires write permission for validation)
    let hrcp_char_handle = gatt
        .add_characteristic(
            service_handle,
            Uuid::from_u16(HRCP_CHAR_UUID),
            1,
            CharProperties::WRITE,
            SecurityPermissions::NONE,
            GattEventMask::ATTRIBUTE_MODIFIED,
            0,
            false,
        )
        .expect("Failed to add HRCP characteristic");

    info!("Heart Rate Service created (handle 0x{:04X})", service_handle.0);
    info!("  HRM  char: 0x{:04X}", hrm_char_handle.0);
    info!("  BSL  char: 0x{:04X}", bsl_char_handle.0);
    info!("  HRCP char: 0x{:04X}", hrcp_char_handle.0);

    let mut state = HrsState {
        service_handle,
        hrm_char_handle,
        hrcp_char_handle,
        notifications_enabled: false,
        conn_handle: None,
        heart_rate: 72,
        hr_direction: 1,
    };

    // ── Advertising ──────────────────────────────────────────────────────────
    let mut adv_data = AdvData::new();
    adv_data.add_flags(0x06).unwrap();
    adv_data.add_name("HRS_WBA6").unwrap();
    adv_data.add_service_uuid_16(HRS_SERVICE_UUID).unwrap();

    let adv_params = AdvParams {
        interval_min: 0x0050, // 50 ms
        interval_max: 0x0064, // 62.5 ms
        adv_type: AdvType::ConnectableUndirected,
        ..AdvParams::default()
    };

    ble.start_advertising(adv_params.clone(), adv_data.clone(), None)
        .await
        .expect("Failed to start advertising");

    info!("Advertising as 'HRS_WBA6'");

    // ── Main loop: send HR notifications every 1 s ───────────────────────────
    let mut ticker = Ticker::every(Duration::from_secs(1));

    loop {
        match select(ble.read_event(), ticker.next()).await {
            Either::First(event) => {
                // ── GAP events ────────────────────────────────────────────────
                if let Some(gap_event) = ble.process_event(&event) {
                    match gap_event {
                        GapEvent::Connected(conn) => {
                            info!("Connected: 0x{:04X}", conn.handle.0);
                            state.conn_handle = Some(conn.handle.0);
                            state.notifications_enabled = false;
                        }
                        GapEvent::Disconnected { handle, reason } => {
                            info!(
                                "Disconnected: 0x{:04X}, reason 0x{:02X} ({})",
                                handle.0,
                                reason.as_u8(),
                                Display2Format(&reason)
                            );
                            state.conn_handle = None;
                            state.notifications_enabled = false;
                            ble.start_advertising(adv_params.clone(), adv_data.clone(), None)
                                .await
                                .expect("Failed to restart advertising");
                        }
                        _ => {}
                    }
                }

                // ── GATT events ───────────────────────────────────────────────
                match &event {
                    Event::Vendor(VendorEvent::GattAttributeModified(attr)) => {
                        // CCCD write → enable/disable notifications
                        if is_cccd_handle(state.hrm_char_handle.0, attr.attr_handle.0) {
                            let enabled = attr.data().first().copied().unwrap_or(0) & 0x01 != 0;
                            state.notifications_enabled = enabled;
                            info!("HR notifications {}", if enabled { "ENABLED" } else { "DISABLED" });
                        }
                        // HRCP write → validate and handle reset command
                        else if is_value_handle(state.hrcp_char_handle.0, attr.attr_handle.0) {
                            if attr.data().first().copied() == Some(HRCP_RESET_ENERGY_EXPENDED) {
                                info!("Energy expended reset");
                            } else {
                                info!("Unknown HRCP command: {:?}", attr.data());
                            }
                        }
                    }
                    Event::Vendor(VendorEvent::AttExchangeMtuResponse(AttExchangeMtuResponse {
                        conn_handle,
                        server_rx_mtu,
                    })) => {
                        if let Some(conn) = ble.get_connection_mut(*conn_handle) {
                            conn.update_mtu(*server_rx_mtu as u16);
                        }
                    }
                    _ => {}
                }
            }

            Either::Second(_) => {
                // ── 1-second tick: update and send HRM notification ───────────
                state.heart_rate = next_heart_rate(state.heart_rate, &mut state.hr_direction);
                let hrm = encode_hrm(state.heart_rate);
                debug!("HR: {} BPM", state.heart_rate);

                if state.notifications_enabled {
                    if let Some(conn) = state.conn_handle {
                        if let Err(e) = gatt.notify(conn, state.service_handle, state.hrm_char_handle, &hrm) {
                            error!("HRM notify failed: {:?}", e);
                        }
                    }
                }
            }
        }
    }
}

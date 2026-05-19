//! BLE Health Thermometer Profile Example
//!
//! Implements the Bluetooth SIG Health Thermometer Profile (HTP) as a GATT server.
//! Demonstrates **INDICATE** (confirmed) vs **NOTIFY** (unconfirmed) delivery:
//!
//! - Temperature Measurement uses INDICATE — the client must confirm receipt.
//! - Intermediate Temperature uses NOTIFY — fire-and-forget for live streaming.
//!
//! ## GATT structure
//! - **Health Thermometer Service** (UUID 0x1809)
//!   - Temperature Measurement (0x2A1C) — INDICATE
//!     - Flags (1 byte): 0x00 = Celsius, no timestamp, no type
//!     - Temperature (4 bytes): IEEE-11073 FLOAT (mantissa × 10^exponent)
//!   - Temperature Type (0x2A1D) — READ
//!     - 0x02 = Body (axillary)
//!   - Intermediate Temperature (0x2A1E) — NOTIFY
//!     - Same encoding; sent every second for live readings
//!
//! Based on ST's BLE_HealthThermometer example for NUCLEO-WBA65RI.
//!
//! ## Testing
//! 1. Flash to NUCLEO-WBA65RI
//! 2. Connect with nRF Connect or a health app
//! 3. Enable indications on Temperature Measurement (0x2A1C)
//! 4. Enable notifications on Intermediate Temperature (0x2A1E)
//! 5. Observe temperature values updating

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_futures::select::{Either, select};
use embassy_stm32::aes::{self, Aes};
use embassy_stm32::peripherals::{AES, PKA, RNG};
use embassy_stm32::pka::{self, Pka};
use embassy_stm32::rcc;
use embassy_stm32::rng::{self, Rng};
use embassy_stm32::{Config, bind_interrupts};
use embassy_stm32_wpan::bluetooth::HCI;
use embassy_stm32_wpan::bluetooth::gap::{AdvData, AdvParams, AdvType, GapEvent};
use embassy_stm32_wpan::bluetooth::gatt::{
    CccdValue, CharProperties, CharacteristicHandle, GattEventMask, SecurityPermissions, ServiceHandle, ServiceType,
    Uuid, is_cccd_handle,
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
async fn rng_runner_task(platform: &'static Platform) {
    platform.run_rng().await
}

#[embassy_executor::task]
async fn ble_runner_task(platform: &'static Platform) {
    platform.run_ble().await
}

// ── GATT UUIDs (Bluetooth SIG assigned) ──────────────────────────────────────
const HTS_SERVICE_UUID: u16 = 0x1809; // Health Thermometer Service
const TEMM_CHAR_UUID: u16 = 0x2A1C; // Temperature Measurement
const TEMP_TYPE_CHAR_UUID: u16 = 0x2A1D; // Temperature Type
const INTERM_TEMP_CHAR_UUID: u16 = 0x2A1E; // Intermediate Temperature

// Temperature Type: 0x02 = Body (axillary)
const TEMP_TYPE_BODY: u8 = 0x02;

struct HtsState {
    service_handle: ServiceHandle,
    temm_char_handle: CharacteristicHandle,
    interm_char_handle: CharacteristicHandle,
    indications_enabled: bool,
    interm_notifications_enabled: bool,
    conn_handle: Option<u16>,
    indication_pending: bool,
    // Simulated temperature: 36.0 – 37.5 °C, step 0.1
    temp_tenths: i16, // temperature × 10 (e.g. 368 = 36.8 °C)
    temp_direction: i16,
}

/// Encode temperature into a 5-byte Temperature Measurement value.
///
/// Format:
/// - Byte 0: Flags (0x00 = Celsius, no timestamp, no type)
/// - Bytes 1-4: IEEE-11073 FLOAT, little-endian
///   [mantissa_lo, mantissa_mid, mantissa_hi, exponent]
///   value = mantissa × 10^exponent
///
/// For 36.8 °C: exponent = -1 (0xFF), mantissa = 368 (0x0170)
/// → [0x00, 0x70, 0x01, 0x00, 0xFF]
fn encode_temperature(temp_tenths: i16) -> [u8; 5] {
    // exponent = -1 means value = mantissa × 10^-1 = mantissa / 10
    let exponent: i8 = -1;
    let mantissa: i32 = temp_tenths as i32;

    let mantissa_bytes = mantissa.to_le_bytes();
    [
        0x00,              // flags: Celsius, no extras
        mantissa_bytes[0], // mantissa LSB
        mantissa_bytes[1], // mantissa mid
        mantissa_bytes[2], // mantissa MSB (mantissa fits in 24 bits for our range)
        exponent as u8,    // exponent (0xFF = -1)
    ]
}

fn next_temperature(current: i16, direction: &mut i16) -> i16 {
    let next = current + *direction;
    if next >= 375 {
        *direction = -1;
        374
    } else if next <= 360 {
        *direction = 1;
        361
    } else {
        next
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut config = Config::default();
    config.rcc = rcc::Config::new_wpan();
    let p = embassy_stm32::init(config);

    info!("Embassy STM32WBA6 BLE Health Thermometer Example");

    let (platform, runtime) = new_platform!(
        Rng::new(p.RNG, Irqs),
        Aes::new_blocking(p.AES, Irqs),
        Pka::new_blocking(p.PKA, Irqs),
        8
    );

    spawner.spawn(rng_runner_task(platform).expect("Failed to spawn rng runner"));
    spawner.spawn(ble_runner_task(platform).expect("Failed to spawn BLE runner"));

    let mut ble = HCI::new(platform, runtime, Irqs).await.expect("BLE init failed");

    embassy_futures::yield_now().await;

    // ── Build GATT server ────────────────────────────────────────────────────
    let mut gatt = ble.gatt_server();

    // HTS: 1 service + 3 chars + 2 CCCDs = 8 attribute records
    let service_handle = gatt
        .add_service(Uuid::from_u16(HTS_SERVICE_UUID), ServiceType::Primary, 8)
        .expect("Failed to add HTS service");

    // Temperature Measurement: INDICATE, variable length up to 13 bytes (full format)
    let temm_char_handle = gatt
        .add_characteristic(
            service_handle,
            Uuid::from_u16(TEMM_CHAR_UUID),
            13,
            CharProperties::INDICATE,
            SecurityPermissions::NONE,
            GattEventMask::empty(),
            0,
            true,
        )
        .expect("Failed to add Temperature Measurement characteristic");

    // Temperature Type: READ only, 1 byte constant
    let temp_type_char_handle = gatt
        .add_characteristic(
            service_handle,
            Uuid::from_u16(TEMP_TYPE_CHAR_UUID),
            1,
            CharProperties::READ,
            SecurityPermissions::NONE,
            GattEventMask::empty(),
            0,
            false,
        )
        .expect("Failed to add Temperature Type characteristic");

    gatt.update_characteristic_value(service_handle, temp_type_char_handle, 0, &[TEMP_TYPE_BODY])
        .expect("Failed to set Temperature Type value");

    // Intermediate Temperature: NOTIFY, same encoding
    let interm_char_handle = gatt
        .add_characteristic(
            service_handle,
            Uuid::from_u16(INTERM_TEMP_CHAR_UUID),
            13,
            CharProperties::NOTIFY,
            SecurityPermissions::NONE,
            GattEventMask::empty(),
            0,
            true,
        )
        .expect("Failed to add Intermediate Temperature characteristic");

    info!("Health Thermometer Service created (handle 0x{:04X})", service_handle.0);
    info!("  TEMM (indicate) char: 0x{:04X}", temm_char_handle.0);
    info!("  Type (read)     char: 0x{:04X}", temp_type_char_handle.0);
    info!("  INT  (notify)   char: 0x{:04X}", interm_char_handle.0);

    let mut state = HtsState {
        service_handle,
        temm_char_handle,
        interm_char_handle,
        indications_enabled: false,
        interm_notifications_enabled: false,
        conn_handle: None,
        indication_pending: false,
        temp_tenths: 368, // start at 36.8 °C
        temp_direction: 1,
    };

    // ── Advertising ──────────────────────────────────────────────────────────
    let mut adv_data = AdvData::new();
    adv_data.add_flags(0x06).unwrap();
    adv_data.add_name("HT_WBA6").unwrap();
    adv_data.add_service_uuid_16(HTS_SERVICE_UUID).unwrap();

    let adv_params = AdvParams {
        interval_min: 0x0050,
        interval_max: 0x0064,
        adv_type: AdvType::ConnectableUndirected,
        ..AdvParams::default()
    };

    ble.start_advertising(adv_params.clone(), adv_data.clone(), None)
        .await
        .expect("Failed to start advertising");

    info!("Advertising as 'HT_WBA6'");
    info!("Enable indications on TEMM (0x2A1C) and notifications on INT (0x2A1E)");

    // ── Main loop: send temperature updates every 1 s ────────────────────────
    // Intermediate temperature (notify) updates every 1 s.
    // Final temperature measurement (indicate) updates every 5 s.
    let mut ticker = Ticker::every(Duration::from_secs(1));
    let mut tick_count: u32 = 0;

    loop {
        match select(ble.read_event(), ticker.next()).await {
            Either::First(event) => {
                // ── GAP events ────────────────────────────────────────────────
                if let Some(gap_event) = ble.process_event(&event) {
                    match gap_event {
                        GapEvent::Connected(conn) => {
                            info!("Connected: 0x{:04X}", conn.handle.0);
                            state.conn_handle = Some(conn.handle.0);
                            state.indications_enabled = false;
                            state.interm_notifications_enabled = false;
                            state.indication_pending = false;
                        }
                        GapEvent::Disconnected { handle, reason } => {
                            info!("Disconnected: 0x{:04X}, reason 0x{:02X}", handle.0, reason);
                            state.conn_handle = None;
                            state.indications_enabled = false;
                            state.interm_notifications_enabled = false;
                            state.indication_pending = false;
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
                        // CCCD write for Temperature Measurement (INDICATE)
                        if is_cccd_handle(state.temm_char_handle.0, attr.attr_handle.0) {
                            let cccd = CccdValue::from_bytes(attr.data());
                            state.indications_enabled = cccd.indications;
                            info!("TEMM indications {}", if cccd.indications { "ENABLED" } else { "DISABLED" });
                        }
                        // CCCD write for Intermediate Temperature (NOTIFY)
                        else if is_cccd_handle(state.interm_char_handle.0, attr.attr_handle.0) {
                            let cccd = CccdValue::from_bytes(attr.data());
                            state.interm_notifications_enabled = cccd.notifications;
                            info!(
                                "INT notifications {}",
                                if cccd.notifications { "ENABLED" } else { "DISABLED" }
                            );
                        }
                    }

                    // Indication confirmed — safe to send the next one
                    Event::Vendor(VendorEvent::GattServerConfirmation(_conn_handle)) => {
                        debug!("Indication confirmed");
                        state.indication_pending = false;
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
                // ── 1-second tick ─────────────────────────────────────────────
                tick_count = tick_count.wrapping_add(1);
                state.temp_tenths = next_temperature(state.temp_tenths, &mut state.temp_direction);

                let temp_value = encode_temperature(state.temp_tenths);
                debug!("Temperature: {}.{} °C", state.temp_tenths / 10, state.temp_tenths % 10);

                if let Some(conn) = state.conn_handle {
                    // Intermediate Temperature: notify every second
                    if state.interm_notifications_enabled {
                        if let Err(e) =
                            gatt.notify(conn, state.service_handle, state.interm_char_handle, &temp_value)
                        {
                            error!("INT notify failed: {:?}", e);
                        }
                    }

                    // Temperature Measurement: indicate every 5 seconds
                    // Only send if no indication is already pending confirmation.
                    if tick_count % 5 == 0 && state.indications_enabled && !state.indication_pending {
                        match gatt.indicate(conn, state.service_handle, state.temm_char_handle, &temp_value) {
                            Ok(()) => {
                                info!(
                                    "TEMM indication sent: {}.{} °C",
                                    state.temp_tenths / 10,
                                    state.temp_tenths % 10
                                );
                                state.indication_pending = true;
                            }
                            Err(e) => error!("TEMM indicate failed: {:?}", e),
                        }
                    }
                }
            }
        }
    }
}

//! BLE Direction Finding Tag Example
//!
//! Implements the ST proprietary Direction Finding Tag service as a GATT server.
//! The tag acts as a BLE peripheral that:
//! - Advertises and accepts a connection from a locator (central)
//! - Exposes a simple LED/button control service (the application layer of the DF demo)
//! - Enables Constant Tone Extension (CTE) in connected mode for AoA
//!   angle-of-arrival measurement
//!
//! ## GATT structure
//! - **Direction Finding Tag Service** (UUID 0000FE40-CC7A-482A-984A-7F2ED5B3E58F)
//!   - LED_C (0000FE41-8E22-4541-9D4C-21EDAE82ED19) — READ + WRITE_WITHOUT_RESPONSE
//!     Value: 0x00=LED off, 0x01=LED on (locator writes to control the tag LED)
//!   - SWITCH_C (0000FE42-8E22-4541-9D4C-21EDAE82ED19) — NOTIFY
//!     Value: 0x00/0x01 (tag sends button-state change notifications to the locator)
//!
//! ## Direction Finding (CTE)
//! The angle-of-arrival measurement relies on Constant Tone Extension (BT 5.1+).
//! On connection, the tag calls `le_set_connection_cte_transmit_parameters` (AoA type)
//! and `le_set_connection_cte_transmit_enable`, so it will respond with CTE when the
//! locator sends `HCI_LE_Connection_CTE_Request_Enable`.  IQ sample events
//! (`LeConnectionIqReport`) are returned via `read_event_ext()`.
//!
//! Based on ST's BLE_DirectionFinding_Peripheral_Tag example for NUCLEO-WBA55CGA.
//!
//! ## Testing
//! 1. Flash to NUCLEO-WBA65RI
//! 2. Connect with nRF Connect or the ST BLE_DirectionFinding_Central_Locator app
//! 3. Enable notifications on SWITCH_C (0000FE42-...) to receive button events
//! 4. Write 0x01/0x00 to LED_C (0000FE41-...) to toggle the simulated LED
//! 5. Observe the simulated button press notifications every 5 seconds

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
use embassy_stm32_wpan::bluetooth::cte::CteEvent;
use embassy_stm32_wpan::bluetooth::gap::{AdvData, AdvParams, AdvType, GapEvent};
use embassy_stm32_wpan::bluetooth::gatt::{
    CccdValue, CharProperties, CharacteristicHandle, GattEventMask, SecurityPermissions, ServiceHandle, ServiceType,
    Uuid, is_cccd_handle, is_value_handle,
};
use embassy_stm32_wpan::bluetooth::{BleEvent, HCI};
use embassy_stm32_wpan::{HighInterruptHandler, LowInterruptHandler, Platform, new_platform};
use embassy_time::{Duration, Ticker};
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

// ── ST Direction Finding Tag service — 128-bit UUIDs (little-endian) ─────────
// Service:  0000FE40-CC7A-482A-984A-7F2ED5B3E58F
const DF_SERVICE_UUID: [u8; 16] = [
    0x8F, 0xE5, 0xB3, 0xD5, 0x2E, 0x7F, 0x4A, 0x98, 0x2A, 0x48, 0x7A, 0xCC, 0x40, 0xFE, 0x00, 0x00,
];
// LED_C (READ + WRITE_WITHOUT_RESPONSE): 0000FE41-8E22-4541-9D4C-21EDAE82ED19
const LED_CHAR_UUID: [u8; 16] = [
    0x19, 0xED, 0x82, 0xAE, 0xED, 0x21, 0x4C, 0x9D, 0x41, 0x45, 0x22, 0x8E, 0x41, 0xFE, 0x00, 0x00,
];
// SWITCH_C (NOTIFY): 0000FE42-8E22-4541-9D4C-21EDAE82ED19
const SWITCH_CHAR_UUID: [u8; 16] = [
    0x19, 0xED, 0x82, 0xAE, 0xED, 0x21, 0x4C, 0x9D, 0x41, 0x45, 0x22, 0x8E, 0x42, 0xFE, 0x00, 0x00,
];

const LED_OFF: u8 = 0x00;
const LED_ON: u8 = 0x01;

struct DfTagState {
    service_handle: ServiceHandle,
    led_char_handle: CharacteristicHandle,
    switch_char_handle: CharacteristicHandle,
    conn_handle: Option<u16>,
    switch_notifications_enabled: bool,
    led_value: u8,
    switch_value: u8,
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut config = Config::default();
    config.rcc = rcc::Config::new_wpan();
    let p = embassy_stm32::init(config);

    info!("Embassy STM32WBA6 BLE Direction Finding Tag Example");

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

    // DF Tag service: 1 service + 2 chars + 1 CCCD + 1 write permit = 7 records
    let service_handle = gatt
        .add_service(Uuid::from_u128_le(DF_SERVICE_UUID), ServiceType::Primary, 7)
        .expect("Failed to add DF service");

    // LED_C: locator writes 0x00/0x01 to control the tag's LED
    let led_char_handle = gatt
        .add_characteristic(
            service_handle,
            Uuid::from_u128_le(LED_CHAR_UUID),
            1,
            CharProperties::READ | CharProperties::WRITE_WITHOUT_RESPONSE,
            SecurityPermissions::NONE,
            GattEventMask::ATTRIBUTE_MODIFIED,
            0,
            false,
        )
        .expect("Failed to add LED_C characteristic");

    // Set initial LED value (off)
    gatt.update_characteristic_value(service_handle, led_char_handle, 0, &[LED_OFF])
        .expect("Failed to set LED_C initial value");

    // SWITCH_C: tag notifies the locator of button-state changes
    let switch_char_handle = gatt
        .add_characteristic(
            service_handle,
            Uuid::from_u128_le(SWITCH_CHAR_UUID),
            1,
            CharProperties::NOTIFY,
            SecurityPermissions::NONE,
            GattEventMask::empty(),
            0,
            false,
        )
        .expect("Failed to add SWITCH_C characteristic");

    info!(
        "Direction Finding Tag Service ready (handle 0x{:04X})",
        service_handle.0
    );
    info!("  LED_C    (read+write)  char: 0x{:04X}", led_char_handle.0);
    info!("  SWITCH_C (notify)      char: 0x{:04X}", switch_char_handle.0);

    let mut state = DfTagState {
        service_handle,
        led_char_handle,
        switch_char_handle,
        conn_handle: None,
        switch_notifications_enabled: false,
        led_value: LED_OFF,
        switch_value: 0,
    };

    // ── Advertising ──────────────────────────────────────────────────────────
    // Main advertisement: device name
    let mut adv_data = AdvData::new();
    adv_data.add_flags(0x06).unwrap();
    adv_data.add_name("DF_WBA6").unwrap();

    // Scan response: DF service UUID (lets locators identify us during active scan)
    let mut scan_rsp = AdvData::new();
    scan_rsp.add_service_uuid_128(&DF_SERVICE_UUID).unwrap();

    let adv_params = AdvParams {
        interval_min: 0x0050,
        interval_max: 0x0064,
        adv_type: AdvType::ConnectableUndirected,
        ..AdvParams::default()
    };

    ble.start_advertising(adv_params.clone(), adv_data.clone(), Some(scan_rsp.clone()))
        .await
        .expect("Failed to start advertising");

    info!("Advertising as 'DF_WBA6'");
    info!("Enable SWITCH_C notifications to receive simulated button events");
    info!("Write LED_C (0x01/0x00) to toggle the simulated LED");

    // Simulate a button toggle every 5 seconds
    let mut ticker = Ticker::every(Duration::from_secs(5));

    // AoA CTE type (bit 0). For a single-antenna tag transmitting AoA CTE,
    // antenna_ids must still have at least 2 entries — use [0, 0] as a
    // placeholder when no switching is needed (locator does the switching).
    const CTE_TYPES_AOA: u8 = 0x01;
    const ANTENNA_IDS: [u8; 2] = [0x00, 0x00];

    // ── Main loop ────────────────────────────────────────────────────────────
    loop {
        match select(ble.read_event_ext(), ticker.next()).await {
            Either::First(ble_event) => {
                let event = match &ble_event {
                    BleEvent::Hci(e) => Some(e),
                    BleEvent::Cte(cte_event) => {
                        match cte_event {
                            CteEvent::ConnectionIqReport(r) => {
                                info!(
                                    "CTE IQ Report: conn=0x{:04X} ch={} rssi={} samples={}",
                                    r.conn_handle, r.data_channel_index, r.rssi, r.sample_count
                                );
                            }
                            CteEvent::CteRequestFailed(f) => {
                                warn!(
                                    "CTE Request Failed: conn=0x{:04X} status=0x{:02X}",
                                    f.conn_handle, f.status
                                );
                            }
                        }
                        None
                    }
                };

                if let Some(event) = event {
                    // ── GAP events ────────────────────────────────────────────────
                    if let Some(gap_event) = ble.process_event(event) {
                        match gap_event {
                            GapEvent::Connected(conn) => {
                                info!("Connected: 0x{:04X}", conn.handle.0);
                                state.conn_handle = Some(conn.handle.0);
                                state.switch_notifications_enabled = false;

                                // Enable CTE transmit: configure AoA type, then enable response
                                match ble.le_set_connection_cte_transmit_parameters(
                                    conn.handle,
                                    CTE_TYPES_AOA,
                                    &ANTENNA_IDS,
                                ) {
                                    Ok(()) => match ble.le_connection_cte_response_enable(conn.handle, true) {
                                        Ok(()) => info!("CTE TX enabled (AoA)"),
                                        Err(e) => warn!("CTE TX enable failed: {:?}", e),
                                    },
                                    Err(e) => warn!("CTE TX params failed: {:?}", e),
                                }
                            }
                            GapEvent::Disconnected { handle, reason } => {
                                info!("Disconnected: 0x{:04X}, reason 0x{:02X}", handle.0, reason);
                                state.conn_handle = None;
                                state.switch_notifications_enabled = false;
                                ble.start_advertising(adv_params.clone(), adv_data.clone(), Some(scan_rsp.clone()))
                                    .await
                                    .expect("Failed to restart advertising");
                            }
                            _ => {}
                        }
                    }

                    // ── GATT events ───────────────────────────────────────────────
                    use stm32wb_hci::Event;
                    match event {
                        Event::Vendor(VendorEvent::GattAttributeModified(attr)) => {
                            if is_cccd_handle(state.switch_char_handle.0, attr.attr_handle.0) {
                                let cccd = CccdValue::from_bytes(attr.data());
                                state.switch_notifications_enabled = cccd.notifications;
                                info!(
                                    "SWITCH_C notifications {}",
                                    if cccd.notifications { "ENABLED" } else { "DISABLED" }
                                );
                            } else if is_value_handle(state.led_char_handle.0, attr.attr_handle.0) {
                                let new_val = attr.data().first().copied().unwrap_or(0);
                                state.led_value = new_val;
                                gatt.update_characteristic_value(
                                    state.service_handle,
                                    state.led_char_handle,
                                    0,
                                    &[new_val],
                                )
                                .ok();
                                info!(
                                    "LED_C write: {} ({})",
                                    new_val,
                                    if new_val == LED_ON { "ON" } else { "OFF" }
                                );
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
            }

            Either::Second(_) => {
                // ── 5-second tick: simulate a button press ────────────────────
                state.switch_value ^= 0x01; // toggle 0 ↔ 1

                if let Some(conn) = state.conn_handle {
                    if state.switch_notifications_enabled {
                        match gatt.notify(
                            conn,
                            state.service_handle,
                            state.switch_char_handle,
                            &[state.switch_value],
                        ) {
                            Ok(()) => info!(
                                "SWITCH_C: button {}",
                                if state.switch_value == 1 { "PRESSED" } else { "RELEASED" }
                            ),
                            Err(e) => error!("SWITCH_C notify failed: {:?}", e),
                        }
                    }
                }
            }
        }
    }
}

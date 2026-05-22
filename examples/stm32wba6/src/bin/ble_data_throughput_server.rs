//! BLE Data Throughput Server Example
//!
//! Implements the ST proprietary Data Transfer Service (DTS) as a GATT server peripheral.
//! Demonstrates high-throughput BLE communication using 244-byte notification packets.
//!
//! ## GATT structure
//! - **Data Transfer Service** (128-bit UUID: 0000FE80-CC7A-482A-984A-7F2ED5B3E58F)
//!   - TX Characteristic (0000FE81-8E22-4541-9D4C-21EDAE82ED19) — NOTIFY
//!     Server streams 244-byte packets to the client as fast as the stack allows.
//!     Packet format: [seq_num (4 bytes LE)] [counter fill (240 bytes)]
//!   - RX Characteristic (0000FE82-8E22-4541-9D4C-21EDAE82ED19) — WRITE_WITHOUT_RESPONSE
//!     Client sends data to the server (reverse direction throughput).
//!   - Throughput Characteristic (0000FE83-8E22-4541-9D4C-21EDAE82ED19) — NOTIFY
//!     Periodic stats: [tx_bytes (4 LE)] [rx_bytes (4 LE)] [tx_kbps (4 LE)] [rx_kbps (4 LE)]
//!
//! ## Throughput optimizations
//! - Requests LE 2M PHY on connect for ~2× throughput over 1M PHY
//! - Requests maximum data length (251 bytes) to minimize per-packet overhead
//! - Sends notifications in a tight burst; only waits for events when the TX buffer is full
//!
//! Based on ST's BLE_DataThroughput_Server example for NUCLEO-WBA55CGA.
//!
//! ## Testing
//! 1. Flash to NUCLEO-WBA65RI
//! 2. Connect with nRF Connect or ST's BLE_DataThroughput_Client (another WBA board)
//! 3. Enable notifications on TX char (FE81) to start the burst
//! 4. Optionally enable notifications on THROUGH char (FE83) for live stats
//! 5. Write data to RX char (FE82) to test the reverse direction
//! 6. Observe throughput figures in the defmt serial log

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::aes::{self, Aes};
use embassy_stm32::peripherals::{AES, PKA, RNG};
use embassy_stm32::pka::{self, Pka};
use embassy_stm32::rng::{self, Rng};
use embassy_stm32::{Config, bind_interrupts, rcc};
use embassy_stm32_wpan::bluetooth::HCI;
use embassy_stm32_wpan::bluetooth::gap::{AdvData, AdvParams, AdvType, GapEvent};
use embassy_stm32_wpan::bluetooth::gatt::{
    CccdValue, CharProperties, CharacteristicHandle, GattEventMask, SecurityPermissions, ServiceHandle, ServiceType,
    Uuid, is_cccd_handle, is_value_handle,
};
use embassy_stm32_wpan::{HighInterruptHandler, LowInterruptHandler, Platform, new_platform};
use embassy_time::Instant;
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

// ── ST proprietary Data Transfer Service — 128-bit UUIDs (little-endian) ────
// Service:  0000FE80-CC7A-482A-984A-7F2ED5B3E58F
const DTS_SERVICE_UUID: [u8; 16] = [
    0x8F, 0xE5, 0xB3, 0xD5, 0x2E, 0x7F, 0x4A, 0x98, 0x2A, 0x48, 0x7A, 0xCC, 0x80, 0xFE, 0x00, 0x00,
];
// TX (server→client NOTIFY): 0000FE81-8E22-4541-9D4C-21EDAE82ED19
const DTS_TX_CHAR_UUID: [u8; 16] = [
    0x19, 0xED, 0x82, 0xAE, 0xED, 0x21, 0x4C, 0x9D, 0x41, 0x45, 0x22, 0x8E, 0x81, 0xFE, 0x00, 0x00,
];
// RX (client→server WRITE_WITHOUT_RESPONSE): 0000FE82-8E22-4541-9D4C-21EDAE82ED19
const DTS_RX_CHAR_UUID: [u8; 16] = [
    0x19, 0xED, 0x82, 0xAE, 0xED, 0x21, 0x4C, 0x9D, 0x41, 0x45, 0x22, 0x8E, 0x82, 0xFE, 0x00, 0x00,
];
// Throughput stats (server→client NOTIFY): 0000FE83-8E22-4541-9D4C-21EDAE82ED19
const DTS_THROUGH_CHAR_UUID: [u8; 16] = [
    0x19, 0xED, 0x82, 0xAE, 0xED, 0x21, 0x4C, 0x9D, 0x41, 0x45, 0x22, 0x8E, 0x83, 0xFE, 0x00, 0x00,
];

const PACKET_SIZE: usize = 244;

struct DtState {
    service_handle: ServiceHandle,
    tx_char_handle: CharacteristicHandle,
    rx_char_handle: CharacteristicHandle,
    through_char_handle: CharacteristicHandle,
    conn_handle: Option<u16>,
    tx_notifications_enabled: bool,
    through_notifications_enabled: bool,
    seq_num: u32,
    tx_bytes: u32,
    rx_bytes: u32,
    window_start: Option<Instant>,
    // Reusable packet buffer; fill bytes [4..] are constant (never change between packets)
    pkt_buf: [u8; PACKET_SIZE],
}

/// 16-byte throughput report sent on the THROUGH characteristic.
/// Fields: [tx_bytes (4 LE)] [rx_bytes (4 LE)] [tx_kbps (4 LE)] [rx_kbps (4 LE)]
fn encode_through(tx_bytes: u32, rx_bytes: u32, elapsed_ms: u32) -> [u8; 16] {
    let tx_kbps = if elapsed_ms > 0 { tx_bytes * 8 / elapsed_ms } else { 0 };
    let rx_kbps = if elapsed_ms > 0 { rx_bytes * 8 / elapsed_ms } else { 0 };
    let mut r = [0u8; 16];
    r[0..4].copy_from_slice(&tx_bytes.to_le_bytes());
    r[4..8].copy_from_slice(&rx_bytes.to_le_bytes());
    r[8..12].copy_from_slice(&tx_kbps.to_le_bytes());
    r[12..16].copy_from_slice(&rx_kbps.to_le_bytes());
    r
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut config = Config::default();
    config.rcc = rcc::Config::new_wpan();
    let p = embassy_stm32::init(config);

    info!("Embassy STM32WBA6 BLE Data Throughput Server Example");

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

    // DTS: 1 service + 3 chars + 2 CCCDs + 1 write-without-resp permit = 10 records
    let service_handle = gatt
        .add_service(Uuid::from_u128_le(DTS_SERVICE_UUID), ServiceType::Primary, 10)
        .expect("Failed to add DTS service");

    // TX: server → client notifications, 244-byte packets
    let tx_char_handle = gatt
        .add_characteristic(
            service_handle,
            Uuid::from_u128_le(DTS_TX_CHAR_UUID),
            PACKET_SIZE as u16,
            CharProperties::NOTIFY,
            SecurityPermissions::NONE,
            GattEventMask::empty(),
            0,
            true,
        )
        .expect("Failed to add TX characteristic");

    // RX: client → server writes (no response), triggers throughput measurement
    let rx_char_handle = gatt
        .add_characteristic(
            service_handle,
            Uuid::from_u128_le(DTS_RX_CHAR_UUID),
            PACKET_SIZE as u16,
            CharProperties::WRITE_WITHOUT_RESPONSE,
            SecurityPermissions::NONE,
            GattEventMask::ATTRIBUTE_MODIFIED,
            0,
            true,
        )
        .expect("Failed to add RX characteristic");

    // THROUGH: server → client throughput stats, 16 bytes
    let through_char_handle = gatt
        .add_characteristic(
            service_handle,
            Uuid::from_u128_le(DTS_THROUGH_CHAR_UUID),
            16,
            CharProperties::NOTIFY,
            SecurityPermissions::NONE,
            GattEventMask::empty(),
            0,
            false,
        )
        .expect("Failed to add THROUGH characteristic");

    info!("Data Transfer Service ready (handle 0x{:04X})", service_handle.0);
    info!("  TX   (notify)              char: 0x{:04X}", tx_char_handle.0);
    info!("  RX   (write-without-resp)  char: 0x{:04X}", rx_char_handle.0);
    info!("  THRU (notify, stats)       char: 0x{:04X}", through_char_handle.0);

    let mut pkt_buf = [0u8; PACKET_SIZE];
    for (i, b) in pkt_buf[4..].iter_mut().enumerate() {
        *b = (i & 0xFF) as u8;
    }

    let mut state = DtState {
        service_handle,
        tx_char_handle,
        rx_char_handle,
        through_char_handle,
        conn_handle: None,
        tx_notifications_enabled: false,
        through_notifications_enabled: false,
        seq_num: 0,
        tx_bytes: 0,
        rx_bytes: 0,
        window_start: None,
        pkt_buf,
    };

    // ── Advertising ──────────────────────────────────────────────────────────
    let mut adv_data = AdvData::new();
    adv_data.add_flags(0x06).unwrap();
    adv_data.add_name("DT_WBA6").unwrap();

    let adv_params = AdvParams {
        interval_min: 0x0050,
        interval_max: 0x0064,
        adv_type: AdvType::ConnectableUndirected,
        ..AdvParams::default()
    };

    ble.start_advertising(adv_params.clone(), adv_data.clone(), None)
        .await
        .expect("Failed to start advertising");

    info!("Advertising as 'DT_WBA6'");
    info!("Enable notifications on TX char to start throughput burst");

    // ── Main loop ────────────────────────────────────────────────────────────
    // When TX notifications are enabled, notifications are sent in a tight burst.
    // `gatt.notify()` fails (returns Err) when the stack TX buffer is full; in that
    // case we fall through to `read_event()` and wait for the stack to drain.
    loop {
        // Burst-send while connected and the client has subscribed
        if state.tx_notifications_enabled {
            if let Some(conn) = state.conn_handle {
                state.pkt_buf[..4].copy_from_slice(&state.seq_num.to_le_bytes());
                match gatt.notify(conn, state.service_handle, state.tx_char_handle, &state.pkt_buf) {
                    Ok(()) => {
                        state.seq_num = state.seq_num.wrapping_add(1);
                        state.tx_bytes += PACKET_SIZE as u32;

                        let now = Instant::now();
                        let start = *state.window_start.get_or_insert(now);
                        let elapsed_ms = (now - start).as_millis() as u32;

                        if elapsed_ms >= 1000 {
                            let tx_kbps = state.tx_bytes * 8 / elapsed_ms;
                            let rx_kbps = state.rx_bytes * 8 / elapsed_ms;
                            info!(
                                "TX: {} B ({} kbps)  |  RX: {} B ({} kbps)",
                                state.tx_bytes * 1000 / elapsed_ms,
                                tx_kbps,
                                state.rx_bytes * 1000 / elapsed_ms,
                                rx_kbps,
                            );
                            if state.through_notifications_enabled {
                                let report = encode_through(state.tx_bytes, state.rx_bytes, elapsed_ms);
                                let _ = gatt.notify(conn, state.service_handle, state.through_char_handle, &report);
                            }
                            state.tx_bytes = 0;
                            state.rx_bytes = 0;
                            state.window_start = Some(now);
                        }

                        continue;
                    }
                    Err(_) => {
                        // TX buffer full — drop to event wait below
                    }
                }
            }
        }

        // Wait for the next HCI event
        let event = ble.read_event().await;

        // ── GAP events ────────────────────────────────────────────────────────
        if let Some(gap_event) = ble.process_event(&event) {
            match gap_event {
                GapEvent::Connected(conn) => {
                    info!("Connected: 0x{:04X}", conn.handle.0);
                    let handle = conn.handle.0;
                    state.conn_handle = Some(handle);
                    state.tx_notifications_enabled = false;
                    state.through_notifications_enabled = false;
                    state.seq_num = 0;
                    state.tx_bytes = 0;
                    state.rx_bytes = 0;
                    state.window_start = None;

                    // Request 2M PHY for ~2× throughput improvement
                    if let Err(e) = ble.command_sender().le_set_phy(handle, 0x00, 0x02, 0x02, 0) {
                        warn!("2M PHY request failed: {:?}", e);
                    }
                    // Request maximum PDU size to minimize header overhead
                    if let Err(e) = ble.command_sender().le_set_data_length(handle, 251, 2120) {
                        warn!("Data length request failed: {:?}", e);
                    }
                }
                GapEvent::Disconnected { handle, reason } => {
                    info!("Disconnected: 0x{:04X}, reason 0x{:02X}", handle.0, reason);
                    state.conn_handle = None;
                    state.tx_notifications_enabled = false;
                    state.through_notifications_enabled = false;
                    ble.start_advertising(adv_params.clone(), adv_data.clone(), None)
                        .await
                        .expect("Failed to restart advertising");
                }
                GapEvent::PhyUpdated { handle, tx_phy, rx_phy } => {
                    info!("PHY updated on 0x{:04X}: TX={:?} RX={:?}", handle.0, tx_phy, rx_phy);
                }
                GapEvent::DataLengthChanged {
                    handle,
                    max_tx_octets,
                    max_rx_octets,
                    ..
                } => {
                    info!(
                        "Data length on 0x{:04X}: TX={} RX={}",
                        handle.0, max_tx_octets, max_rx_octets
                    );
                }
                _ => {}
            }
        }

        // ── GATT events ───────────────────────────────────────────────────────
        match &event {
            Event::Vendor(VendorEvent::GattAttributeModified(attr)) => {
                if is_cccd_handle(state.tx_char_handle.0, attr.attr_handle.0) {
                    let cccd = CccdValue::from_bytes(attr.data());
                    state.tx_notifications_enabled = cccd.notifications;
                    info!(
                        "TX notifications {}",
                        if cccd.notifications { "ENABLED" } else { "DISABLED" }
                    );
                    if cccd.notifications {
                        state.window_start = None; // reset measurement window
                    }
                } else if is_cccd_handle(state.through_char_handle.0, attr.attr_handle.0) {
                    let cccd = CccdValue::from_bytes(attr.data());
                    state.through_notifications_enabled = cccd.notifications;
                    info!(
                        "THROUGH notifications {}",
                        if cccd.notifications { "ENABLED" } else { "DISABLED" }
                    );
                } else if is_value_handle(state.rx_char_handle.0, attr.attr_handle.0) {
                    state.rx_bytes += attr.data().len() as u32;
                    debug!("RX {} bytes (window total {} B)", attr.data().len(), state.rx_bytes);
                }
            }
            Event::Vendor(VendorEvent::AttExchangeMtuResponse(AttExchangeMtuResponse {
                conn_handle,
                server_rx_mtu,
            })) => {
                if let Some(conn) = ble.get_connection_mut(*conn_handle) {
                    conn.update_mtu(*server_rx_mtu as u16);
                }
                info!("MTU exchanged: {} bytes", server_rx_mtu);
            }
            _ => {}
        }
    }
}

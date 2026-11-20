//! STM32WBA6 BLE "stack extras" example.
//!
//! Exercises the STM32WBA wrapper APIs that are easy to miss:
//!
//! 1. **Full custom advertising payload** — manufacturer data, service data and
//!    TX power in connectable advertising (`advertiser::configure` now applies
//!    the exact `AdvData` bytes instead of only name + 16-bit UUIDs).
//! 2. **Directed advertising** — `AdvParams::peer_addr` +
//!    `AdvType::ConnectableDirected*` routes to `aci_gap_set_direct_connectable`.
//! 3. **RSSI + radio activity mask** — `HCI::read_rssi` and
//!    `HCI::set_radio_activity_mask`.
//! 4. **GATT characteristic descriptors** — `GattServer::add_descriptor` /
//!    `update_descriptor_value`, plus `GattServer::store_db` to persist the DB.
//! 5. **Passkey-entry keypresses** — `SecurityManager::passkey_input`.
//!
//! Hardware: STM32WBA65 or compatible.

#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::{Config, bind_interrupts, rcc};
use embassy_stm32_wpan::bluetooth::gap::types::OwnAddressType;
use embassy_stm32_wpan::bluetooth::gap::{AdvData, AdvParams, AdvType, BdAddr, BdAddrType, GapEvent};
use embassy_stm32_wpan::bluetooth::gatt::{
    AccessPermissions, CharProperties, GattEventMask, SecurityPermissions, ServiceType, Uuid,
};
use embassy_stm32_wpan::bluetooth::security::{IoCapability, PasskeyInputType, SecurityEvent, SecurityParams};
use embassy_stm32_wpan::bluetooth::{EventBuffer, HCI};
use embassy_stm32_wpan::{HighInterruptHandler, LowInterruptHandler, Platform, new_platform};
use panic_probe as _;
use stm32wb_hci::aci::flags::RadioActivityMask;
use stm32wb_hci::adv_data::AdvFlags;

bind_interrupts!(struct Irqs {
    RADIO => HighInterruptHandler;
    HASH => LowInterruptHandler;
});

/// Flip to `true` to advertise directly at the peer below instead of
/// undirected. High-duty-cycle directed advertising stops after 1.28 s if no
/// connection is established.
const DIRECTED_DEMO: bool = false;

/// Characteristic Presentation Format descriptor (Bluetooth SIG 0x2904).
const CPF_DESCRIPTOR_UUID: u16 = 0x2904;

#[embassy_executor::task]
async fn ble_runner_task(platform: &'static Platform) {
    platform.run_ble().await
}

/// Build an advertising payload that uses fields beyond name + 16-bit UUID.
fn make_adv_data() -> AdvData {
    let mut d = AdvData::new();
    d = d
        .flags(AdvFlags::LE_GENERAL_DISCOVERABLE | AdvFlags::BR_EDR_NOT_SUPPORTED)
        .unwrap();
    d = d.complete_local_name(b"Extras").unwrap();
    d = d.complete_uuid16_list(&[0xBEEF]).unwrap();
    d = d.tx_power_level(0).unwrap();
    d = d.manufacturer_specific_data(0x004C, b"hi").unwrap();
    d = d.service_data_uuid16(0x1234, &[0xAA]).unwrap();
    d
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut config = Config::default();
    config.rcc = rcc::Config::new_wpan();
    let _p = embassy_stm32::init(config);

    let (platform, runtime) = new_platform!(8);
    spawner.spawn(ble_runner_task(platform).expect("spawn ble runner"));

    let mut ble = HCI::new(platform, runtime, Irqs).await.expect("BLE init failed");

    let mut security = ble.security_manager();
    security
        .set_authentication_requirements(
            ble.controller(),
            SecurityParams::new()
                .with_bonding(true)
                .with_mitm_protection(true)
                .with_io_capability(IoCapability::DisplayYesNo),
        )
        .await
        .expect("set security params");

    // -- 1. Radio activity mask (only report the activities we care about) ----
    ble.set_radio_activity_mask(RadioActivityMask::ADVERTISING | RadioActivityMask::PERIPHERAL_CONNECTION)
        .await
        .expect("set radio activity mask");

    // -- 2. GATT service with a custom descriptor, persisted to NVM ----------
    let mut gatt = ble.gatt_server();
    let svc = gatt
        .add_service(ble.controller(), Uuid::Uuid16(0xBEEF), ServiceType::Primary, 8)
        .await
        .expect("add service");
    let ch = gatt
        .add_characteristic(
            ble.controller(),
            svc,
            Uuid::Uuid16(0xBEF0),
            20,
            CharProperties::READ | CharProperties::NOTIFY,
            SecurityPermissions::empty(),
            GattEventMask::empty(),
            0,
            true,
        )
        .await
        .expect("add characteristic");

    // Characteristic Presentation Format: format(1) exponent(1) unit(2)
    // namespace(1) description(2).
    let cpf: [u8; 7] = [0x04, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00];
    let desc = gatt
        .add_descriptor(
            ble.controller(),
            svc,
            ch,
            Uuid::Uuid16(CPF_DESCRIPTOR_UUID),
            cpf.len() as u8,
            &cpf,
            SecurityPermissions::empty(),
            AccessPermissions::READ,
            GattEventMask::empty(),
            0,
            false,
        )
        .await
        .expect("add descriptor");
    gatt.update_descriptor_value(ble.controller(), svc, ch, desc, 0, &cpf)
        .await
        .expect("set descriptor value");
    gatt.store_db(ble.controller()).await.expect("store GATT database");

    // -- 3. Undirected advertising with the full custom payload --------------
    let adv_params = AdvParams {
        interval_min: 0x0050,
        interval_max: 0x0050,
        adv_type: AdvType::ConnectableUndirected,
        ..AdvParams::default()
    };
    ble.start_advertising(adv_params, make_adv_data(), None)
        .await
        .expect("start advertising");
    info!("advertising as 'Extras' (manufacturer + service data)");

    // -- 4. Directed advertising (opt-in) ------------------------------------
    if DIRECTED_DEMO {
        let directed = AdvParams {
            interval_min: 0x0006,
            interval_max: 0x0006,
            adv_type: AdvType::ConnectableDirectedHighDuty,
            own_addr_type: OwnAddressType::Public,
            peer_addr: Some(BdAddrType::Public(BdAddr::new([0x11, 0x22, 0x33, 0x44, 0x55, 0x66]))),
            ..AdvParams::default()
        };
        ble.start_advertising(directed, AdvData::new(), None)
            .await
            .expect("start directed advertising");
        info!("directed advertising enabled");
    }

    // -- Event loop ----------------------------------------------------------
    let mut event_buf = EventBuffer::new();
    loop {
        let event = ble.read_event(&mut event_buf).await;

        if let Some(gap) = ble.process_event(&event).await {
            if let GapEvent::Connected(conn) = gap {
                info!("connected: handle=0x{:04X}", conn.handle.raw());
                match ble.read_rssi().await {
                    Ok(resp) => match resp.rssi.known() {
                        Some(rssi) => info!("last packet RSSI: {} dBm", rssi.get()),
                        None => warn!("RSSI not available"),
                    },
                    Err(_) => warn!("RSSI read failed"),
                }
            }
        }

        if let Some(sec) = ble.process_security_event(&event) {
            if let SecurityEvent::PasskeyRequest { conn_handle } = sec {
                // Relay keypresses to the peer, then send the entered passkey.
                let _ = security
                    .passkey_input(ble.controller(), conn_handle, PasskeyInputType::EntryStarted)
                    .await;
                let _ = security
                    .passkey_input(ble.controller(), conn_handle, PasskeyInputType::DigitEntered)
                    .await;
                let _ = security
                    .passkey_input(ble.controller(), conn_handle, PasskeyInputType::EntryCompleted)
                    .await;
                match security.pass_key_response(ble.controller(), conn_handle, 123_456).await {
                    Ok(()) => info!("passkey sent to stack"),
                    Err(_) => warn!("pass_key_response failed"),
                }
            }
        }
    }
}

//! BLE Rhai Interpreter Demo
//!
//! Receives Rhai expressions over BLE NUS (Nordic UART Service), evaluates
//! them and sends the result back as a BLE notification.
//!
//! ## Usage
//! 1. Flash to STM32WBA55 board
//! 2. Connect with nRF Connect (or similar BLE app)
//! 3. Enable notifications on the TX characteristic
//! 4. Write a Rhai expression to the RX characteristic, e.g. `40 + 2`
//! 5. Result appears as a notification: `42`
//!
//! ## Build
//! cargo build --release --bin ble_rhai --features scripting

#![no_std]
#![no_main]

extern crate alloc;
use alloc::format;

use embedded_alloc::LlffHeap as Heap;

use defmt::*;
use embassy_executor::Spawner;
use embassy_futures::select::{Either, select};
use embassy_stm32::aes::{self, Aes};
use embassy_stm32::peripherals::{AES as AesPeriph, PKA as PkaPeriph};
use embassy_stm32::pka::{self, Pka};
use embassy_stm32::rcc;
use embassy_stm32::rng::{self, Rng};
use embassy_stm32::{Config, bind_interrupts, peripherals};
use embassy_stm32_wpan::bluetooth::HCI;
use embassy_stm32_wpan::bluetooth::gap::{AdvData, AdvParams, AdvType, GapEvent};
use embassy_stm32_wpan::bluetooth::gap::types::OwnAddressType;
use embassy_stm32_wpan::bluetooth::gatt::{
    CccdValue, CharProperties, GattEventMask, SecurityPermissions, ServiceType, Uuid,
    is_cccd_handle, is_value_handle,
};
use embassy_stm32_wpan::{HighInterruptHandler, LowInterruptHandler, Platform, new_platform};
use rhai::{Dynamic, Engine, packages::BasicMathPackage, packages::CorePackage, packages::Package};
use stm32wb_hci::Event;
use stm32wb_hci::vendor::event::{AttExchangeMtuResponse, VendorEvent};
use {defmt_rtt as _, panic_probe as _};

#[global_allocator]
static HEAP: Heap = Heap::empty();

const HEAP_SIZE: usize = 48 * 1024;

bind_interrupts!(struct Irqs {
    RNG => rng::InterruptHandler<peripherals::RNG>;
    AES => aes::InterruptHandler<AesPeriph>;
    PKA => pka::InterruptHandler<PkaPeriph>;
    RADIO => HighInterruptHandler;
    HASH => LowInterruptHandler;
});

// Nordic UART Service (NUS) UUIDs - compatible with nRF Connect and similar apps
// Service UUID: 6E400001-B5A3-F393-E0A9-E50E24DCCA9E
const NUS_SERVICE_UUID: [u8; 16] = [
    0x9E, 0xCA, 0xDC, 0x24, 0x0E, 0xE5, 0xA9, 0xE0, 0x93, 0xF3, 0xA3, 0xB5, 0x01, 0x00, 0x40, 0x6E,
];

// RX Characteristic UUID: 6E400002-B5A3-F393-E0A9-E50E24DCCA9E (Client writes to this)
const NUS_RX_CHAR_UUID: [u8; 16] = [
    0x9E, 0xCA, 0xDC, 0x24, 0x0E, 0xE5, 0xA9, 0xE0, 0x93, 0xF3, 0xA3, 0xB5, 0x02, 0x00, 0x40, 0x6E,
];

// TX Characteristic UUID: 6E400003-B5A3-F393-E0A9-E50E24DCCA9E (Server notifies on this)
const NUS_TX_CHAR_UUID: [u8; 16] = [
    0x9E, 0xCA, 0xDC, 0x24, 0x0E, 0xE5, 0xA9, 0xE0, 0x93, 0xF3, 0xA3, 0xB5, 0x03, 0x00, 0x40, 0x6E,
];

const MAX_DATA_LEN: usize = 244;
const INPUT_BUF_SIZE: usize = 512;

#[embassy_executor::task]
async fn rng_runner_task(platform: &'static Platform) {
    platform.run_rng().await
}

#[embassy_executor::task]
async fn ble_runner_task(platform: &'static Platform) {
    platform.run_ble().await
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    {
        use core::mem::MaybeUninit;
        static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe { HEAP.init(core::ptr::addr_of!(HEAP_MEM) as usize, HEAP_SIZE) }
    }

    let mut config = Config::default();
    config.rcc = rcc::Config::new_wpan();
    let p = embassy_stm32::init(config);

    // Set up Rhai engine with math + core packages
    let mut engine = Engine::new_raw();
    BasicMathPackage::new().register_into_engine(&mut engine);
    CorePackage::new().register_into_engine(&mut engine);

    info!("BLE Rhai interpreter starting");

    let (platform, runtime) = new_platform!(
        Rng::new(p.RNG, Irqs),
        Aes::new_blocking(p.AES, Irqs),
        Pka::new_blocking(p.PKA, Irqs),
        8
    );

    spawner.spawn(rng_runner_task(platform).expect("spawn rng"));
    spawner.spawn(ble_runner_task(platform).expect("spawn ble"));

    let mut ble = HCI::new(platform, runtime, Irqs)
        .await
        .expect("BLE init failed");
    embassy_futures::yield_now().await;

    let mut gatt = ble.gatt_server();

    let service_handle = gatt
        .add_service(Uuid::from_u128_le(NUS_SERVICE_UUID), ServiceType::Primary, 10)
        .expect("add NUS service");

    let rx_char_handle = gatt
        .add_characteristic(
            service_handle,
            Uuid::from_u128_le(NUS_RX_CHAR_UUID),
            MAX_DATA_LEN as u16,
            CharProperties::WRITE | CharProperties::WRITE_WITHOUT_RESPONSE,
            SecurityPermissions::NONE,
            GattEventMask::ATTRIBUTE_MODIFIED,
            0,
            true,
        )
        .expect("add RX char");

    let tx_char_handle = gatt
        .add_characteristic(
            service_handle,
            Uuid::from_u128_le(NUS_TX_CHAR_UUID),
            MAX_DATA_LEN as u16,
            CharProperties::NOTIFY,
            SecurityPermissions::NONE,
            GattEventMask::empty(),
            0,
            true,
        )
        .expect("add TX char");

    let mut adv_data = AdvData::new();
    adv_data.add_flags(0x06).unwrap();
    adv_data.add_name("RhaiShell").unwrap();

    let mut scan_rsp = AdvData::new();
    scan_rsp.add_service_uuid_128(&NUS_SERVICE_UUID).unwrap();

    let adv_params = AdvParams {
        interval_min: 0x0050,
        interval_max: 0x0064,
        adv_type: AdvType::ConnectableUndirected,
        own_addr_type: OwnAddressType::Random,
        ..AdvParams::default()
    };

    ble.start_advertising(adv_params.clone(), adv_data.clone(), Some(scan_rsp.clone()))
        .await
        .expect("start advertising");

    info!("Advertising as 'RhaiShell' — connect and send Rhai expressions");

    let mut input_buf: heapless::Vec<u8, INPUT_BUF_SIZE> = heapless::Vec::new();
    let mut tx_notifications = false;
    let mut conn_handle: Option<u16> = None;

    loop {
        // If we have buffered data, race BLE events against a 500 ms idle timeout.
        // Every incoming packet resets the timer (we restart the select from the top
        // of the loop), so the eval only fires when nothing arrives for 500 ms.
        let maybe_event = if !input_buf.is_empty() {
            match select(ble.read_event(), embassy_time::Timer::after_millis(500)).await {
                Either::First(ev) => Some(ev),
                Either::Second(_) => None, // 500 ms idle → evaluate
            }
        } else {
            Some(ble.read_event().await)
        };

        // Idle timeout fired: evaluate and send result without disconnecting
        if maybe_event.is_none() {
            if let Ok(script) = core::str::from_utf8(&input_buf) {
                info!("eval (timeout): {} bytes\n{}", input_buf.len(), script);
                let reply = match engine.eval::<Dynamic>(script) {
                    Ok(result) => {
                        let type_name = result.type_name();
                        let is_string = result.is_string();
                        info!(
                            "eval ok: type={} is_string={}",
                            type_name,
                            if is_string { "yes" } else { "no" }
                        );
                        format!("{}\r\n", result)
                    }
                    Err(e) => {
                        let err_str = format!("{}", e);
                                    warn!("eval err: {}", err_str.as_str());
                        format!("err: {}\r\n", e)
                    }
                };
                if let Some(conn) = conn_handle {
                    for chunk in reply.as_bytes().chunks(MAX_DATA_LEN) {
                        let _ = gatt.notify(conn, service_handle, tx_char_handle, chunk);
                    }
                    // Re-prompt so the user can send another expression
                    let _ = gatt.notify(conn, service_handle, tx_char_handle, b"> ");
                }
            }
            input_buf.clear();
            continue;
        }

        let event = maybe_event.unwrap();

        if let Some(gap_event) = ble.process_event(&event) {
            match gap_event {
                GapEvent::Connected(conn) => {
                    info!("Connected: 0x{:04X}", conn.handle.0);
                    conn_handle = Some(conn.handle.0);
                    tx_notifications = false;
                    input_buf.clear();
                }
                GapEvent::Disconnected { handle, reason } => {
                    info!("Disconnected: 0x{:04X} reason=0x{:02X}", handle.0, reason);
                    // Evaluate any remaining buffered data on disconnect too
                    if !input_buf.is_empty() {
                        if let Ok(script) = core::str::from_utf8(&input_buf) {
                            info!("eval (disconnect): {} bytes\n{}", input_buf.len(), script);
                            let reply = match engine.eval::<Dynamic>(script) {
                                Ok(result) => {
                                    let type_name = result.type_name();
                                    let is_string = result.is_string();
                                    info!(
                                        "eval ok: type={} is_string={}",
                                        type_name,
                                        if is_string { "yes" } else { "no" }
                                    );
                                    format!("{}\r\n", result)
                                }
                                Err(e) => {
                                    let err_str = format!("{}", e);
                                    warn!("eval err: {}", err_str.as_str());
                                    format!("err: {}\r\n", e)
                                }
                            };
                            if let Some(conn) = conn_handle {
                                for chunk in reply.as_bytes().chunks(MAX_DATA_LEN) {
                                    let _ = gatt.notify(conn, service_handle, tx_char_handle, chunk);
                                }
                            }
                        }
                        input_buf.clear();
                    }
                    conn_handle = None;
                    tx_notifications = false;
                    ble.start_advertising(adv_params.clone(), adv_data.clone(), Some(scan_rsp.clone()))
                        .await
                        .expect("restart advertising");
                }
                _ => {}
            }
        }

        match &event {
            Event::Vendor(VendorEvent::GattAttributeModified(attr)) => {
                if is_cccd_handle(tx_char_handle.0, attr.attr_handle.0) {
                    tx_notifications = CccdValue::from_bytes(attr.data()).notifications;
                    if tx_notifications {
                        if let Some(conn) = conn_handle {
                            let _ = gatt.notify(conn, service_handle, tx_char_handle, b"> ");
                        }
                    }
                    info!("TX notifications {}", if tx_notifications { "on" } else { "off" });
                } else if is_value_handle(rx_char_handle.0, attr.attr_handle.0) {
                    for &b in attr.data() {
                        if input_buf.len() < INPUT_BUF_SIZE {
                            let _ = input_buf.push(b);
                        }
                    }
                    debug!("buffered {} bytes total", input_buf.len());
                }
            }

            Event::Vendor(VendorEvent::AttExchangeMtuResponse(AttExchangeMtuResponse {
                conn_handle: ch,
                server_rx_mtu,
            })) => {
                if let Some(conn) = ble.get_connection_mut(*ch) {
                    conn.update_mtu(*server_rx_mtu as u16);
                }
            }

            _ => {}
        }
    }
}

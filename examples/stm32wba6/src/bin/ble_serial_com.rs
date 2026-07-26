//! BLE Serial Communication Peripheral Example
//!
//! This example implements a BLE-to-UART bridge using a Nordic UART Service (NUS)
//! compatible GATT service. It allows bidirectional serial communication over BLE.
//!
//! Based on ST's BLE_SerialCom_Peripheral example but using GATT instead of L2CAP CoC.
//!
//! ## Features
//! - Nordic UART Service (NUS) compatible UUIDs (works with nRF Connect app)
//! - USART1 for terminal I/O (PB12=TX, PA8=RX on NUCLEO-WBA65RI)
//! - Bidirectional data bridging between UART and BLE
//! - 115200 baud, 8N1
//!
//! ## Hardware
//! - STM32WBA65RI (NUCLEO-WBA65RI)
//! - USART1: PB12 (TX), PA8 (RX) - connects to ST-Link VCP
//!
//! ## Testing
//! 1. Flash this example to your NUCLEO-WBA65RI board
//! 2. Connect a terminal to the board's serial port (115200 baud)
//! 3. Connect with nRF Connect or similar BLE app
//! 4. Enable notifications on the TX characteristic
//! 5. Type in the terminal - text appears on BLE
//! 6. Write to RX characteristic - text appears on terminal

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::aes::{self, Aes};
use embassy_stm32::peripherals::{AES, PKA, RNG, USART1};
use embassy_stm32::pka::{self, Pka};
use embassy_stm32::rng::{self, Rng};
use embassy_stm32::usart::{self, BufferedUart, BufferedUartRx, BufferedUartTx, Config as UartConfig};
use embassy_stm32::{Config, bind_interrupts, rcc};
use embassy_stm32_wpan::bluetooth::HCI;
use embassy_stm32_wpan::bluetooth::gap::{AdvData, AdvParams, AdvType, GapEvent};
use embassy_stm32_wpan::bluetooth::gatt::{
    CHAR_VALUE_HANDLE_OFFSET, CccdValue, CharProperties, CharacteristicHandle, GattEventMask, SecurityPermissions,
    ServiceHandle, ServiceType, Uuid, is_cccd_handle, is_value_handle,
};
use embassy_stm32_wpan::{HighInterruptHandler, LowInterruptHandler, Platform, new_platform};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embedded_io_async::{Read, Write};
use static_cell::StaticCell;
use stm32wb_hci::Event;
use stm32wb_hci::vendor::event::{AttExchangeMtuResponse, VendorEvent};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    RNG => rng::InterruptHandler<RNG>;
    AES => aes::InterruptHandler<AES>;
    PKA => pka::InterruptHandler<PKA>;
    USART1 => usart::BufferedInterruptHandler<USART1>;
    RADIO => HighInterruptHandler;
    HASH => LowInterruptHandler;
});

#[embassy_executor::task]
async fn ble_runner_task(platform: &'static Platform) {
    platform.run_ble().await
}

// Nordic UART Service (NUS) UUIDs — compatible with nRF Connect and similar apps.
// Service UUID: 6E400001-B5A3-F393-E0A9-E50E24DCCA9E
const NUS_SERVICE_UUID: [u8; 16] = [
    0x9E, 0xCA, 0xDC, 0x24, 0x0E, 0xE5, 0xA9, 0xE0, 0x93, 0xF3, 0xA3, 0xB5, 0x01, 0x00, 0x40, 0x6E,
];
// RX UUID: 6E400002-... (client writes to this)
const NUS_RX_CHAR_UUID: [u8; 16] = [
    0x9E, 0xCA, 0xDC, 0x24, 0x0E, 0xE5, 0xA9, 0xE0, 0x93, 0xF3, 0xA3, 0xB5, 0x02, 0x00, 0x40, 0x6E,
];
// TX UUID: 6E400003-... (server notifies on this)
const NUS_TX_CHAR_UUID: [u8; 16] = [
    0x9E, 0xCA, 0xDC, 0x24, 0x0E, 0xE5, 0xA9, 0xE0, 0x93, 0xF3, 0xA3, 0xB5, 0x03, 0x00, 0x40, 0x6E,
];

const MAX_DATA_LEN: usize = 244;

static UART_TO_BLE: Channel<CriticalSectionRawMutex, heapless::Vec<u8, MAX_DATA_LEN>, 4> = Channel::new();
static BLE_TO_UART: Channel<CriticalSectionRawMutex, heapless::Vec<u8, MAX_DATA_LEN>, 4> = Channel::new();

struct SerialComState {
    service_handle: ServiceHandle,
    rx_char_handle: CharacteristicHandle,
    tx_char_handle: CharacteristicHandle,
    tx_notifications_enabled: bool,
    current_conn_handle: Option<u16>,
}

#[embassy_executor::task]
async fn uart_reader_task(mut uart_rx: BufferedUartRx<'static>) {
    let mut buf = [0u8; MAX_DATA_LEN];
    loop {
        match uart_rx.read(&mut buf).await {
            Ok(n) if n > 0 => {
                let mut data: heapless::Vec<u8, MAX_DATA_LEN> = heapless::Vec::new();
                let _ = data.extend_from_slice(&buf[..n]);
                if UART_TO_BLE.try_send(data).is_err() {
                    warn!("UART->BLE channel full, dropping data");
                }
            }
            Ok(_) => {}
            Err(e) => error!("UART read error: {:?}", e),
        }
    }
}

#[embassy_executor::task]
async fn uart_writer_task(mut uart_tx: BufferedUartTx<'static>) {
    loop {
        let data = BLE_TO_UART.receive().await;
        let mut written = 0;
        while written < data.len() {
            match uart_tx.write(&data[written..]).await {
                Ok(n) => written += n,
                Err(e) => {
                    error!("UART write error: {:?}", e);
                    break;
                }
            }
        }
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut config = Config::default();
    config.rcc = rcc::Config::new_wpan();
    let p = embassy_stm32::init(config);

    info!("Embassy STM32WBA6 BLE Serial Communication Example");

    let (platform, runtime) = new_platform!(
        Rng::new(p.RNG, Irqs),
        Pka::new(p.PKA, Irqs),
        Aes::new_blocking(p.AES, Irqs),
        8
    );

    spawner.spawn(ble_runner_task(platform).expect("Failed to spawn BLE runner"));

    // USART1: PB12 = TX, PA8 = RX (ST-Link VCP on NUCLEO-WBA65RI)
    let mut uart_config = UartConfig::default();
    uart_config.baudrate = 115200;

    static TX_BUF: StaticCell<[u8; 256]> = StaticCell::new();
    static RX_BUF: StaticCell<[u8; 256]> = StaticCell::new();
    let tx_buf = TX_BUF.init([0u8; 256]);
    let rx_buf = RX_BUF.init([0u8; 256]);

    let uart = BufferedUart::new(p.USART1, p.PA8, p.PB12, tx_buf, rx_buf, Irqs, uart_config)
        .expect("Failed to initialize USART1");
    let (uart_tx, uart_rx) = uart.split();

    let mut ble = HCI::new(platform, runtime, Irqs).await.expect("BLE init failed");

    embassy_futures::yield_now().await;

    spawner.spawn(uart_reader_task(uart_rx).expect("Failed to spawn uart_reader"));
    spawner.spawn(uart_writer_task(uart_tx).expect("Failed to spawn uart_writer"));

    let mut gatt = ble.gatt_server();

    let service_handle = gatt
        .add_service(Uuid::from_u128_le(NUS_SERVICE_UUID), ServiceType::Primary, 10)
        .expect("Failed to add NUS service");

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
        .expect("Failed to add RX characteristic");

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
        .expect("Failed to add TX characteristic");

    info!("NUS service ready");
    info!("TX value handle: 0x{:04X}", tx_char_handle.0 + CHAR_VALUE_HANDLE_OFFSET);

    let mut state = SerialComState {
        service_handle,
        rx_char_handle,
        tx_char_handle,
        tx_notifications_enabled: false,
        current_conn_handle: None,
    };

    let mut adv_data = AdvData::new();
    adv_data.add_flags(0x06).unwrap();
    adv_data.add_name("Serial_Com").unwrap();

    let mut scan_rsp = AdvData::new();
    scan_rsp.add_service_uuid_128(&NUS_SERVICE_UUID).unwrap();

    let adv_params = AdvParams {
        interval_min: 0x0050,
        interval_max: 0x0064,
        adv_type: AdvType::ConnectableUndirected,
        ..AdvParams::default()
    };

    ble.start_advertising(adv_params.clone(), adv_data.clone(), Some(scan_rsp.clone()))
        .await
        .expect("Failed to start advertising");

    info!("BLE Serial Communication ready — device name: Serial_Com");
    info!("Connect with nRF Connect, enable TX notifications, write to RX");

    loop {
        if state.tx_notifications_enabled {
            if let Ok(data) = UART_TO_BLE.try_receive() {
                if let Some(conn) = state.current_conn_handle {
                    if let Err(e) = gatt.notify(conn, state.service_handle, state.tx_char_handle, &data) {
                        error!("Notify failed: {:?}", e);
                    }
                }
            }
        }

        let event = ble.read_event().await;

        if let Some(gap_event) = ble.process_event(&event) {
            match gap_event {
                GapEvent::Connected(conn) => {
                    info!("Connected: 0x{:04X}", conn.handle.0);
                    state.current_conn_handle = Some(conn.handle.0);
                    state.tx_notifications_enabled = false;
                }
                GapEvent::Disconnected { handle, reason } => {
                    info!(
                        "Disconnected: 0x{:04X}, reason 0x{:02X} ({})",
                        handle.0,
                        reason.as_u8(),
                        Display2Format(&reason)
                    );
                    state.current_conn_handle = None;
                    state.tx_notifications_enabled = false;
                    ble.start_advertising(adv_params.clone(), adv_data.clone(), Some(scan_rsp.clone()))
                        .await
                        .expect("Failed to restart advertising");
                }
                _ => {}
            }
        }

        match &event {
            Event::Vendor(VendorEvent::GattAttributeModified(attr)) => {
                if is_cccd_handle(state.tx_char_handle.0, attr.attr_handle.0) {
                    let cccd = CccdValue::from_bytes(attr.data());
                    state.tx_notifications_enabled = cccd.notifications;
                    info!(
                        "TX notifications {}",
                        if cccd.notifications { "ENABLED" } else { "DISABLED" }
                    );
                } else if is_value_handle(state.rx_char_handle.0, attr.attr_handle.0) {
                    let mut uart_data: heapless::Vec<u8, MAX_DATA_LEN> = heapless::Vec::new();
                    let _ = uart_data.extend_from_slice(attr.data());
                    if BLE_TO_UART.try_send(uart_data).is_err() {
                        warn!("BLE->UART channel full, dropping data");
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
}

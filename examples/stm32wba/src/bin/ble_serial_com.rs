//! BLE Serial Communication Peripheral Example
//!
//! This example implements a BLE-to-UART bridge using a Nordic UART Service (NUS)
//! compatible GATT service. It allows bidirectional serial communication over BLE.
//!
//! Based on ST's BLE_SerialCom_Peripheral example but using GATT instead of L2CAP CoC.
//!
//! ## Features
//! - Nordic UART Service (NUS) compatible UUIDs (works with nRF Connect app)
//! - USART1 for terminal I/O (PB12=TX, PA8=RX on NUCLEO-WBA boards)
//! - Bidirectional data bridging between UART and BLE
//! - 115200 baud, 8N1
//!
//! ## Hardware
//! - STM32WBA52 or STM32WBA65 (NUCLEO board recommended)
//! - USART1: PB12 (TX), PA8 (RX) - connects to ST-Link VCP on NUCLEO boards
//!
//! ## Testing
//! 1. Flash this example to your STM32WBA board
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
use embassy_stm32::peripherals::{AES as AesPeriph, PKA as PkaPeriph};
use embassy_stm32::pka::{self, Pka};
use embassy_stm32::rcc::{self};
use embassy_stm32::rng::{self, Rng};
use embassy_stm32::usart::{self, BufferedUart, BufferedUartRx, BufferedUartTx, Config as UartConfig};
use embassy_stm32::{Config, bind_interrupts, peripherals};
use embassy_stm32_wpan::bluetooth::HCI;
use embassy_stm32_wpan::bluetooth::gap::{AdvData, AdvParams, AdvType, GapEvent};
use embassy_stm32_wpan::bluetooth::gap::types::OwnAddressType;
use embassy_stm32_wpan::bluetooth::gap_init::{AddressType, GapInitParams};
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

// Interrupt bindings
bind_interrupts!(struct Irqs {
    RNG => rng::InterruptHandler<peripherals::RNG>;
    AES => aes::InterruptHandler<AesPeriph>;
    PKA => pka::InterruptHandler<PkaPeriph>;
    USART1 => usart::BufferedInterruptHandler<peripherals::USART1>;
    RADIO => HighInterruptHandler;
    HASH => LowInterruptHandler;
});

/// RNG runner task
#[embassy_executor::task]
async fn rng_runner_task(platform: &'static Platform) {
    platform.run_rng().await
}

/// BLE runner task - drives the BLE stack sequencer
#[embassy_executor::task]
async fn ble_runner_task(platform: &'static Platform) {
    platform.run_ble().await
}

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

/// Maximum data length for BLE transfers (ATT MTU - 3 for ATT header)
const MAX_DATA_LEN: usize = 244;

// ---- Test configuration ----
const ADDR_TYPE: OwnAddressType = OwnAddressType::Random;

/// Channel for UART -> BLE data flow
static UART_TO_BLE: Channel<CriticalSectionRawMutex, heapless::Vec<u8, MAX_DATA_LEN>, 4> = Channel::new();

/// Channel for BLE -> UART data flow
static BLE_TO_UART: Channel<CriticalSectionRawMutex, heapless::Vec<u8, MAX_DATA_LEN>, 4> = Channel::new();

/// Application state for the serial communication service
struct SerialComState {
    service_handle: ServiceHandle,
    rx_char_handle: CharacteristicHandle,
    tx_char_handle: CharacteristicHandle,
    tx_notifications_enabled: bool,
    current_conn_handle: Option<u16>,
}

/// UART reader task - reads from UART and sends to BLE channel
#[embassy_executor::task]
async fn uart_reader_task(mut uart_rx: BufferedUartRx<'static>) {
    info!("UART reader task started");
    let mut buf = [0u8; MAX_DATA_LEN];

    loop {
        // Read available data from UART
        match uart_rx.read(&mut buf).await {
            Ok(n) if n > 0 => {
                let mut data: heapless::Vec<u8, MAX_DATA_LEN> = heapless::Vec::new();
                let _ = data.extend_from_slice(&buf[..n]);
                // Send to BLE channel (non-blocking, will drop if full)
                if UART_TO_BLE.try_send(data).is_err() {
                    warn!("UART->BLE channel full, dropping data");
                }
            }
            Ok(_) => {
                // No data, continue
            }
            Err(e) => {
                error!("UART read error: {:?}", e);
            }
        }
    }
}

/// UART writer task - receives from BLE channel and writes to UART
#[embassy_executor::task]
async fn uart_writer_task(mut uart_tx: BufferedUartTx<'static>) {
    info!("UART writer task started");

    loop {
        let data = BLE_TO_UART.receive().await;
        // Write all data to UART
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

    info!("Embassy STM32WBA BLE Serial Communication Example");
    info!("Based on ST BLE_SerialCom_Peripheral");

    // Initialize hardware peripherals required by BLE stack
    let (platform, runtime) = new_platform!(
        Rng::new(p.RNG, Irqs),
        Aes::new_blocking(p.AES, Irqs),
        Pka::new_blocking(p.PKA, Irqs),
        8
    );

    info!("Hardware peripherals initialized (RNG, AES, PKA)");

    // Spawn the RNG runner task
    spawner.spawn(rng_runner_task(platform).expect("Failed to spawn rng runner"));

    // Spawn the BLE runner task (required for proper BLE operation)
    spawner.spawn(ble_runner_task(platform).expect("Failed to spawn BLE runner"));

    // Initialize USART1 for terminal communication
    // NUCLEO-WBA boards: PB12 = TX, PA8 = RX (ST-Link VCP)
    let mut uart_config = UartConfig::default();
    uart_config.baudrate = 115200;

    static TX_BUF: StaticCell<[u8; 256]> = StaticCell::new();
    static RX_BUF: StaticCell<[u8; 256]> = StaticCell::new();
    let tx_buf = TX_BUF.init([0u8; 256]);
    let rx_buf = RX_BUF.init([0u8; 256]);

    let uart = BufferedUart::new(p.USART1, p.PA8, p.PB12, tx_buf, rx_buf, Irqs, uart_config)
        .expect("Failed to initialize USART1");

    let (uart_tx, uart_rx) = uart.split();
    info!("USART1 initialized (115200 baud, PB12=TX, PA8=RX)");

    // Initialize BLE stack
    let mut ble = match ADDR_TYPE {
        OwnAddressType::Public => {
            let gap_params = GapInitParams {
                bd_addr: [0x01, 0x00, 0x00, 0xE1, 0x80, 0x00],
                address_type: AddressType::Public,
                ..GapInitParams::default()
            };
            HCI::new_with_gap_params(platform, runtime, Irqs, gap_params).await
        }
        _ => HCI::new(platform, runtime, Irqs).await,
    }
    .expect("BLE initialization failed");

    info!("BLE stack initialized");

    // Give the BLE runner a chance to start processing
    // This is needed because BLE operations require BleStack_Process to run
    embassy_futures::yield_now().await;
    info!("BLE runner started");

    // Spawn UART tasks
    spawner.spawn(uart_reader_task(uart_rx).expect("Failed to create UART reader task"));
    spawner.spawn(uart_writer_task(uart_tx).expect("Failed to create UART writer task"));

    // Initialize GATT server with Nordic UART Service
    let mut gatt = ble.gatt_server();

    // Add NUS Service (128-bit UUID)
    let service_uuid = Uuid::from_u128_le(NUS_SERVICE_UUID);
    let service_handle = gatt
        .add_service(service_uuid, ServiceType::Primary, 10)
        .expect("Failed to add NUS service");
    info!("NUS Service created: handle 0x{:04X}", service_handle.0);

    // Add RX Characteristic (client writes to this)
    // Write without response for faster throughput
    let rx_char_uuid = Uuid::from_u128_le(NUS_RX_CHAR_UUID);
    let rx_char_handle = gatt
        .add_characteristic(
            service_handle,
            rx_char_uuid,
            MAX_DATA_LEN as u16,
            CharProperties::WRITE | CharProperties::WRITE_WITHOUT_RESPONSE,
            SecurityPermissions::NONE,
            GattEventMask::ATTRIBUTE_MODIFIED,
            0,
            true, // Variable length
        )
        .expect("Failed to add RX characteristic");
    info!("RX Characteristic: handle 0x{:04X}", rx_char_handle.0);

    // Add TX Characteristic (server notifies on this)
    let tx_char_uuid = Uuid::from_u128_le(NUS_TX_CHAR_UUID);
    let tx_char_handle = gatt
        .add_characteristic(
            service_handle,
            tx_char_uuid,
            MAX_DATA_LEN as u16,
            CharProperties::NOTIFY,
            SecurityPermissions::NONE,
            GattEventMask::empty(), // No events needed, we only send
            0,
            true, // Variable length
        )
        .expect("Failed to add TX characteristic");
    info!("TX Characteristic: handle 0x{:04X}", tx_char_handle.0);
    info!("  Value handle: 0x{:04X}", tx_char_handle.0 + CHAR_VALUE_HANDLE_OFFSET);

    // Application state
    let mut state = SerialComState {
        service_handle,
        rx_char_handle,
        tx_char_handle,
        tx_notifications_enabled: false,
        current_conn_handle: None,
    };

    // Create advertising data
    let mut adv_data = AdvData::new();
    adv_data.add_flags(0x06).expect("Failed to add flags"); // General discoverable, no BR/EDR
    adv_data.add_name("Serial_Com").expect("Failed to add name");

    // Create scan response with full service UUID
    let mut scan_rsp = AdvData::new();
    scan_rsp
        .add_service_uuid_128(&NUS_SERVICE_UUID)
        .expect("Failed to add service UUID");

    let adv_params = AdvParams {
        interval_min: 0x0050, // 50ms (80 * 0.625ms)
        interval_max: 0x0064, // 62.5ms (100 * 0.625ms)
        adv_type: AdvType::ConnectableUndirected,
        own_addr_type: ADDR_TYPE,
        ..AdvParams::default()
    };

    // Start advertising
    {
        ble.start_advertising(adv_params.clone(), adv_data.clone(), Some(scan_rsp.clone()))
            .await
            .expect("Failed to start advertising");
    }

    info!("===========================================");
    info!("BLE Serial Communication Peripheral Ready");
    info!("Device name: Serial_Com");
    info!("===========================================");
    info!("Connect with nRF Connect or similar app");
    info!("Enable notifications on TX characteristic");
    info!("Write to RX characteristic to send to UART");
    info!("===========================================");

    // Main event loop
    loop {
        // Check for UART data to send via BLE (non-blocking)
        if state.tx_notifications_enabled {
            if let Ok(data) = UART_TO_BLE.try_receive() {
                if let Some(conn) = state.current_conn_handle {
                    match gatt.notify(conn, state.service_handle, state.tx_char_handle, &data) {
                        Ok(()) => {
                            debug!("Sent {} bytes via BLE notification", data.len());
                        }
                        Err(e) => {
                            error!("Failed to send notification: {:?}", e);
                        }
                    }
                }
            }
        }

        // Wait for BLE event
        let event = ble.read_event().await;

        // Process GAP events (connections)
        if let Some(gap_event) = ble.process_event(&event) {
            match gap_event {
                GapEvent::Connected(conn) => {
                    info!("Connected: handle 0x{:04X}", conn.handle.0);
                    state.current_conn_handle = Some(conn.handle.0);
                    state.tx_notifications_enabled = false; // Reset on new connection
                }
                GapEvent::Disconnected { handle, reason } => {
                    info!("Disconnected: handle 0x{:04X}, reason 0x{:02X}", handle.0, reason);
                    state.current_conn_handle = None;
                    state.tx_notifications_enabled = false;

                    // Restart advertising
                    ble.start_advertising(adv_params.clone(), adv_data.clone(), Some(scan_rsp.clone()))
                        .await
                        .expect("Failed to start advertising");
                    info!("Advertising restarted");
                }
                _ => {}
            }
        }

        // Process GATT events
        match &event {
            Event::Vendor(VendorEvent::GattAttributeModified(attribute)) => {
                // Check if this is a CCCD write (notification enable/disable) for TX char
                if is_cccd_handle(state.tx_char_handle.0, attribute.attr_handle.0) {
                    let cccd = CccdValue::from_bytes(attribute.data());
                    state.tx_notifications_enabled = cccd.notifications;
                    info!(
                        "TX notifications {}",
                        if cccd.notifications { "ENABLED" } else { "DISABLED" }
                    );
                }
                // Check if this is a write to RX characteristic (data from BLE client)
                else if is_value_handle(state.rx_char_handle.0, attribute.attr_handle.0) {
                    debug!(
                        "Received {} bytes via BLE from conn 0x{:04X}",
                        attribute.data().len(),
                        attribute.conn_handle.0
                    );

                    // Forward to UART
                    let mut uart_data: heapless::Vec<u8, MAX_DATA_LEN> = heapless::Vec::new();
                    let _ = uart_data.extend_from_slice(attribute.data());

                    if BLE_TO_UART.try_send(uart_data).is_err() {
                        warn!("BLE->UART channel full, dropping data");
                    }
                }
            }

            Event::Vendor(VendorEvent::AttExchangeMtuResponse(AttExchangeMtuResponse {
                conn_handle,
                server_rx_mtu,
            })) => {
                info!("MTU exchanged: conn 0x{:04X}, MTU={}", conn_handle.0, server_rx_mtu);
                if let Some(conn) = ble.get_connection_mut(*conn_handle) {
                    conn.update_mtu(*server_rx_mtu as u16);
                }
            }

            Event::Vendor(VendorEvent::GattNotificationComplete(attr_handle)) => {
                debug!("Notification complete: attr 0x{:04X}", attr_handle);
            }

            _ => {
                // Ignore other events
            }
        }
    }
}

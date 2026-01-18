//! BLE GATT Server Example with Notifications
//!
//! This example demonstrates full GATT server functionality:
//! - Creates a custom service with read/write/notify characteristics
//! - Handles characteristic writes from clients
//! - Sends notifications when values change
//! - Tracks CCCD (notification enable/disable) state
//!
//! Hardware: STM32WBA52 or compatible
//!
//! To test:
//! 1. Flash this example to your STM32WBA board
//! 2. Connect with nRF Connect or similar app
//! 3. Enable notifications on the characteristic
//! 4. Write values to the characteristic
//! 5. Observe notifications being sent back

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::rcc::{
    AHB5Prescaler, AHBPrescaler, APBPrescaler, PllDiv, PllMul, PllPreDiv, PllSource, Sysclk, VoltageScale, mux,
};
use embassy_stm32::rng::{self, Rng};
use embassy_stm32::{Config, bind_interrupts};
use embassy_stm32_wpan::gap::{AdvData, AdvParams, AdvType, GapEvent};
use embassy_stm32_wpan::gatt::{
    CHAR_VALUE_HANDLE_OFFSET, CccdValue, CharProperties, CharacteristicHandle, GattEventMask, GattServer,
    SecurityPermissions, ServiceHandle, ServiceType, Uuid, is_cccd_handle, is_value_handle,
};
use embassy_stm32_wpan::hci::event::EventParams;
use embassy_stm32_wpan::{Ble, ble_runner, set_rng_instance};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    RNG => rng::InterruptHandler<embassy_stm32::peripherals::RNG>;
});

/// BLE runner task - drives the BLE stack sequencer
#[embassy_executor::task]
async fn ble_runner_task() {
    ble_runner().await
}

/// Custom service UUID (use your own for production)
const CUSTOM_SERVICE_UUID: u16 = 0xABCD;
/// Read/Write/Notify characteristic UUID
const DATA_CHAR_UUID: u16 = 0xABCE;

/// Application state for tracking GATT handles and CCCD state
struct AppState {
    service_handle: ServiceHandle,
    data_char_handle: CharacteristicHandle,
    notifications_enabled: bool,
    current_conn_handle: Option<u16>,
    counter: u8,
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut config = Config::default();

    // Configure PLL1 (required on WBA)
    config.rcc.pll1 = Some(embassy_stm32::rcc::Pll {
        source: PllSource::HSI,
        prediv: PllPreDiv::DIV1,
        mul: PllMul::MUL30,
        divr: Some(PllDiv::DIV5),
        divq: None,
        divp: Some(PllDiv::DIV30),
        frac: Some(0),
    });

    config.rcc.ahb_pre = AHBPrescaler::DIV1;
    config.rcc.apb1_pre = APBPrescaler::DIV1;
    config.rcc.apb2_pre = APBPrescaler::DIV1;
    config.rcc.apb7_pre = APBPrescaler::DIV1;
    config.rcc.ahb5_pre = AHB5Prescaler::DIV4;
    config.rcc.voltage_scale = VoltageScale::RANGE1;
    config.rcc.sys = Sysclk::PLL1_R;
    config.rcc.mux.rngsel = mux::Rngsel::HSI;

    let p = embassy_stm32::init(config);
    info!("Embassy STM32WBA GATT Server Example");

    // Initialize RNG (required by BLE stack)
    let mut rng = Rng::new(p.RNG, Irqs);
    set_rng_instance(&mut rng as *mut _ as *mut ());

    // Initialize BLE stack
    let mut ble = Ble::new();
    ble.init().expect("BLE initialization failed");
    info!("BLE stack initialized");

    // Spawn the BLE runner task (required for proper BLE operation)
    spawner.spawn(ble_runner_task().expect("Failed to create BLE runner task"));

    // Initialize GATT server
    let mut gatt = GattServer::new();
    gatt.init().expect("GATT initialization failed");

    // Create custom service
    let service_uuid = Uuid::from_u16(CUSTOM_SERVICE_UUID);
    let service_handle = gatt
        .add_service(service_uuid, ServiceType::Primary, 6)
        .expect("Failed to add service");
    info!("Service created: handle 0x{:04X}", service_handle.0);

    // Add data characteristic with read/write/notify
    let char_uuid = Uuid::from_u16(DATA_CHAR_UUID);
    let char_properties = CharProperties::READ | CharProperties::WRITE | CharProperties::NOTIFY;
    let data_char_handle = gatt
        .add_characteristic(
            service_handle,
            char_uuid,
            32, // Max 32 bytes
            char_properties,
            SecurityPermissions::NONE,
            GattEventMask::ATTRIBUTE_MODIFIED,
            0,
            true, // Variable length
        )
        .expect("Failed to add characteristic");
    info!("Characteristic created: handle 0x{:04X}", data_char_handle.0);
    info!(
        "  Value handle: 0x{:04X}",
        data_char_handle.0 + CHAR_VALUE_HANDLE_OFFSET
    );
    info!("  CCCD handle: 0x{:04X}", data_char_handle.0 + 2);

    // Set initial value
    let initial_value = b"Hello!";
    gatt.update_characteristic_value(service_handle, data_char_handle, 0, initial_value)
        .expect("Failed to set initial value");

    // Application state
    let mut state = AppState {
        service_handle,
        data_char_handle,
        notifications_enabled: false,
        current_conn_handle: None,
        counter: 0,
    };

    // Create advertising data
    let mut adv_data = AdvData::new();
    adv_data.add_flags(0x06).expect("Failed to add flags");
    adv_data.add_name("Embassy-GATT").expect("Failed to add name");
    adv_data
        .add_service_uuid_16(CUSTOM_SERVICE_UUID)
        .expect("Failed to add service UUID");

    let adv_params = AdvParams {
        interval_min: 0x0050,
        interval_max: 0x0050,
        adv_type: AdvType::ConnectableUndirected,
        ..AdvParams::default()
    };

    // Start advertising
    {
        let mut advertiser = ble.advertiser();
        advertiser
            .start(adv_params.clone(), adv_data.clone(), None)
            .expect("Failed to start advertising");
    }

    info!("GATT Server started as 'Embassy-GATT'");
    info!("Waiting for connections...");

    // Main event loop
    loop {
        let event = ble.read_event().await;

        // Process GAP events (connections)
        if let Some(gap_event) = ble.process_event(&event) {
            match gap_event {
                GapEvent::Connected(conn) => {
                    info!("Connected: handle 0x{:04X}", conn.handle.0);
                    state.current_conn_handle = Some(conn.handle.0);
                    state.notifications_enabled = false; // Reset on new connection
                }
                GapEvent::Disconnected { handle, reason } => {
                    info!("Disconnected: handle 0x{:04X}, reason 0x{:02X}", handle.0, reason);
                    state.current_conn_handle = None;
                    state.notifications_enabled = false;

                    // Restart advertising
                    let mut advertiser = ble.advertiser();
                    let _ = advertiser.start(adv_params.clone(), adv_data.clone(), None);
                    info!("Advertising restarted");
                }
                _ => {}
            }
        }

        // Process GATT events
        match &event.params {
            EventParams::GattAttributeModified {
                conn_handle,
                attr_handle,
                data,
                ..
            } => {
                info!(
                    "Attribute modified: conn 0x{:04X}, attr 0x{:04X}, {} bytes",
                    conn_handle.0,
                    attr_handle,
                    data.len()
                );

                // Check if this is a CCCD write (notification enable/disable)
                if is_cccd_handle(state.data_char_handle.0, *attr_handle) {
                    let cccd = CccdValue::from_bytes(data);
                    state.notifications_enabled = cccd.notifications;
                    info!(
                        "CCCD updated: notifications={}, indications={}",
                        cccd.notifications, cccd.indications
                    );

                    if state.notifications_enabled {
                        info!("Notifications ENABLED - will send updates");
                    } else {
                        info!("Notifications DISABLED");
                    }
                }
                // Check if this is a characteristic value write
                else if is_value_handle(state.data_char_handle.0, *attr_handle) {
                    info!("Characteristic value written: {:?}", data.as_slice());

                    // Echo the data back as a notification if enabled
                    if state.notifications_enabled {
                        if let Some(conn) = state.current_conn_handle {
                            // Increment counter and append to response
                            state.counter = state.counter.wrapping_add(1);
                            let mut response: heapless::Vec<u8, 33> = heapless::Vec::new();
                            let _ = response.extend_from_slice(data);
                            let _ = response.push(state.counter);

                            match gatt.notify(conn, state.service_handle, state.data_char_handle, &response) {
                                Ok(()) => {
                                    info!("Notification sent: {} bytes", response.len());
                                }
                                Err(e) => {
                                    error!("Failed to send notification: {:?}", e);
                                }
                            }
                        }
                    }
                }
            }

            EventParams::GattNotificationComplete {
                conn_handle,
                attr_handle,
            } => {
                info!(
                    "Notification complete: conn 0x{:04X}, attr 0x{:04X}",
                    conn_handle.0, attr_handle
                );
            }

            EventParams::AttExchangeMtuResponse {
                conn_handle,
                server_mtu,
            } => {
                info!("MTU exchanged: conn 0x{:04X}, MTU={}", conn_handle.0, server_mtu);
                // Update connection MTU
                if let Some(conn) = ble.get_connection_mut(*conn_handle) {
                    conn.update_mtu(*server_mtu);
                }
            }

            _ => {
                // Log other events at debug level
                debug!("Event: {:?}", event);
            }
        }
    }
}

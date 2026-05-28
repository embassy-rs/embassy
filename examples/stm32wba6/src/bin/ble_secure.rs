//! BLE Secure Peripheral Example
//!
//! This example demonstrates BLE security features:
//! - Configures security parameters (bonding, MITM protection)
//! - Handles pairing requests and passkey entry
//! - Supports numeric comparison for Secure Connections
//! - Demonstrates bond management
//!
//! Hardware: STM32WBA65 or compatible
//!
//! To test:
//! 1. Flash this example to your STM32WBA6 board
//! 2. Connect with nRF Connect or similar app
//! 3. Initiate pairing from the app
//! 4. Observe pairing events in the logs
//! 5. For passkey entry: enter the displayed passkey on your phone
//! 6. For numeric comparison: confirm the displayed numbers match

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
use embassy_stm32_wpan::bluetooth::gap::types::OwnAddressType;
use embassy_stm32_wpan::bluetooth::gap::{AdvData, AdvParams, AdvType, GapEvent};
use embassy_stm32_wpan::bluetooth::gap_init::{AddressType, GapInitParams};
use embassy_stm32_wpan::bluetooth::gatt::{CharProperties, GattEventMask, SecurityPermissions, ServiceType, Uuid};
use embassy_stm32_wpan::bluetooth::security::{IoCapability, SecureConnectionsSupport, SecurityParams};
use embassy_stm32_wpan::{HighInterruptHandler, LowInterruptHandler, Platform, new_platform};
use stm32wb_hci::Event;
use stm32wb_hci::event::EncryptionChange;
use stm32wb_hci::vendor::event::{GapNumericComparisonValue, GapPairingComplete, GapPairingStatus, VendorEvent};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    RNG => rng::InterruptHandler<RNG>;
    AES => aes::InterruptHandler<AES>;
    PKA => pka::InterruptHandler<PKA>;
    RADIO => HighInterruptHandler;
    HASH => LowInterruptHandler;
});

/// Custom service UUID
const SECURE_SERVICE_UUID: u16 = 0xABCD;
const SECURE_CHAR_UUID: u16 = 0xABCE;

// ---- Test configuration ----
const ADDR_TYPE: OwnAddressType = OwnAddressType::Random;

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

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut config = Config::default();
    config.rcc = rcc::Config::new_wpan();

    let p = embassy_stm32::init(config);

    info!("Embassy STM32WBA6 BLE Secure Peripheral Example");

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

    // ===== Configure Security =====
    let mut security = ble.security_manager();

    // Configure security parameters:
    // - Enable bonding (store keys)
    // - Require MITM protection (passkey entry or numeric comparison)
    // - Support Secure Connections (LE Secure Connections pairing)
    let security_params = SecurityParams::new()
        .with_bonding(true)
        .with_mitm_protection(true)
        .with_secure_connections(SecureConnectionsSupport::Optional)
        .with_key_size_range(7, 16)
        // DisplayYesNo: device can show a passkey or numeric comparison value.
        // Required for MITM — NoInputNoOutput (the default) only allows "Just
        // Works" pairing which provides no MITM protection.
        .with_io_capability(IoCapability::DisplayYesNo);

    security
        .set_authentication_requirements(security_params)
        .expect("Failed to set security parameters");
    info!("Security configured: Bonding + MITM + SC");

    // Enable address resolution (for bonded devices using RPA)
    security
        .set_address_resolution_enable(true)
        .expect("Failed to enable address resolution");

    // Initialize GATT server
    let mut gatt = ble.gatt_server();

    // Create a service with a secure characteristic
    let service_uuid = Uuid::from_u16(SECURE_SERVICE_UUID);
    let service_handle = gatt
        .add_service(service_uuid, ServiceType::Primary, 4)
        .expect("Failed to add service");

    // Add characteristic that requires authenticated encryption
    let char_uuid = Uuid::from_u16(SECURE_CHAR_UUID);
    let char_handle = gatt
        .add_characteristic(
            service_handle,
            char_uuid,
            20,
            CharProperties::READ | CharProperties::WRITE,
            SecurityPermissions::AUTHEN_READ | SecurityPermissions::AUTHEN_WRITE,
            GattEventMask::ATTRIBUTE_MODIFIED,
            0,
            true,
        )
        .expect("Failed to add characteristic");

    // Set initial value
    gatt.update_characteristic_value(service_handle, char_handle, 0, b"Secure!")
        .expect("Failed to set value");

    info!("GATT service created with secure characteristic");

    // Advertising parameters (reused for restarts)
    let adv_params = AdvParams {
        interval_min: 0x0050,
        interval_max: 0x0050,
        adv_type: AdvType::ConnectableUndirected,
        own_addr_type: ADDR_TYPE,
        ..AdvParams::default()
    };

    // Helper to create advertising data
    fn create_adv_data() -> AdvData {
        let mut adv_data = AdvData::new();
        adv_data.add_flags(0x06).expect("Failed to add flags");
        adv_data.add_name("Embassy-Secure").expect("Failed to add name");
        adv_data
            .add_service_uuid_16(SECURE_SERVICE_UUID)
            .expect("Failed to add UUID");
        adv_data
    }

    // Start advertising
    {
        ble.start_advertising(adv_params.clone(), create_adv_data(), None)
            .await
            .expect("Failed to start advertising");
    }

    info!("=== BLE Secure Peripheral Started ===");
    info!("Device name: 'Embassy-Secure'");
    info!("Connect and pair to access secure characteristic");
    info!("");

    // Main event loop
    loop {
        let event = ble.read_event().await;

        // Process GAP events (connections)
        if let Some(gap_event) = ble.process_event(&event) {
            match gap_event {
                GapEvent::Connected(conn) => {
                    info!("=== CONNECTED ===");
                    info!("  Handle: 0x{:04X}", conn.handle.0);
                    info!("  Peer: {}", conn.peer_address);

                    // Immediately request pairing from the peripheral side so
                    // the central doesn't have to trigger it via an
                    // insufficient-security GATT error first.
                    if let Err(e) = security.request_pairing(conn.handle.0) {
                        warn!("request_pairing failed: {:?}", e);
                    } else {
                        info!("Pairing requested — waiting for central to respond...");
                    }
                }

                GapEvent::Disconnected { handle, reason } => {
                    info!("=== DISCONNECTED ===");
                    info!("  Handle: 0x{:04X}, Reason: 0x{:02X}", handle.0, reason);

                    // Restart advertising
                    ble.start_advertising(adv_params.clone(), create_adv_data(), None)
                        .await
                        .expect("Failed to start advertising");
                    info!("Advertising restarted");
                }

                _ => {}
            }
        }

        // Process security events
        match &event {
            Event::Vendor(VendorEvent::GapPairingComplete(GapPairingComplete { conn_handle, status })) => {
                info!("=== PAIRING COMPLETE ===");
                info!("  Connection: 0x{:04X}", conn_handle.0);

                match status {
                    GapPairingStatus::Success => {
                        info!("  Status: SUCCESS");
                        info!("  Device is now bonded and can access secure characteristics");
                    }
                    GapPairingStatus::Timeout(reason) => {
                        info!("  Status: TIMEOUT (reason: {:?})", reason);
                    }
                    GapPairingStatus::Failed(reason) => {
                        info!("  Status: FAILED (reason: {:?})", reason);
                    }
                    GapPairingStatus::EncryptionFailed(reason) => {
                        info!("  Status: ENCRYPTION FAILED (reason: {:?})", reason);
                    }
                }
            }
            Event::Vendor(VendorEvent::GapPassKeyRequest(conn_handle)) => {
                info!("=== PASSKEY REQUEST ===");
                info!("  Connection: 0x{:04X}", conn_handle.0);

                // Generate a random passkey (in production, display this to user)
                // For this example, we use a fixed passkey
                let passkey: u32 = 123456;
                info!("  Passkey: {:06}", passkey);
                info!("  Enter this passkey on your phone/device!");

                if let Err(e) = security.pass_key_response(conn_handle.0, passkey) {
                    error!("Failed to send passkey response: {:?}", e);
                }
            }

            Event::Vendor(VendorEvent::GapNumericComparisonValue(GapNumericComparisonValue {
                connection_handle,
                numeric_value,
            })) => {
                info!("=== NUMERIC COMPARISON ===");
                info!("  Connection: 0x{:04X}", connection_handle.0);
                info!("  Displayed value: {:06}", numeric_value);
                info!("  Confirm this matches the value on your phone!");

                // Auto-confirm for this example (in production, wait for user input)
                // Set to true to accept, false to reject
                let confirm = true;
                info!("  Auto-confirming: {}", if confirm { "YES" } else { "NO" });

                if let Err(e) = security.numeric_comparison_response(connection_handle.0, confirm) {
                    error!("Failed to send numeric comparison response: {:?}", e);
                }
            }
            Event::Vendor(VendorEvent::GapBondLost) => {
                info!("=== BOND LOST ===");
                //                info!("  Connection: 0x{:04X}", conn_handle.0);
                //                info!("  Previous bond invalid, allowing rebond...");
                //
                //                if let Err(e) = security.allow_rebond(conn_handle.as_u16()) {
                //                    error!("Failed to allow rebond: {:?}", e);
                //                }
            }

            // TODO: Not currently implemented

            //            EventParams::GapPairingRequest { conn_handle, is_bonded } => {
            //                info!("=== PAIRING REQUEST ===");
            //                info!("  Connection: 0x{:04X}", conn_handle.0);
            //                info!("  Previously bonded: {}", is_bonded);
            //                // The stack will handle the pairing process automatically
            //            }
            Event::Vendor(VendorEvent::GattAttributeModified(attribute)) => {
                info!("=== SECURE WRITE RECEIVED ===");
                info!(
                    "  Connection: 0x{:04X}, Attr: 0x{:04X}",
                    attribute.conn_handle, attribute.attr_handle
                );
                info!("  Data ({} bytes): {:?}", attribute.data().len(), attribute.data());
                info!("  (This write succeeded because device is paired!)");
            }

            Event::EncryptionChange(EncryptionChange {
                status,
                conn_handle,
                encryption,
            }) => {
                info!("=== ENCRYPTION CHANGE ===");
                info!("  Connection: 0x{:04X}", conn_handle.0);
                info!("  Status: {:?}", status);
                info!("  Encryption: {}", encryption);
            }

            _ => {
                // Log other events at debug level
                debug!("Event: {:?}", event);
            }
        }
    }
}

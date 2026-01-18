//! BLE Secure Peripheral Example
//!
//! This example demonstrates BLE security features:
//! - Configures security parameters (bonding, MITM protection)
//! - Handles pairing requests and passkey entry
//! - Supports numeric comparison for Secure Connections
//! - Demonstrates bond management
//!
//! Hardware: STM32WBA52 or compatible
//!
//! To test:
//! 1. Flash this example to your STM32WBA board
//! 2. Connect with nRF Connect or similar app
//! 3. Initiate pairing from the app
//! 4. Observe pairing events in the logs
//! 5. For passkey entry: enter the displayed passkey on your phone
//! 6. For numeric comparison: confirm the displayed numbers match

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
use embassy_stm32_wpan::gatt::{CharProperties, GattEventMask, GattServer, SecurityPermissions, ServiceType, Uuid};
use embassy_stm32_wpan::hci::event::EventParams;
use embassy_stm32_wpan::security::{
    PairingFailureReason, PairingStatus, SecureConnectionsSupport, SecurityManager, SecurityParams,
};
use embassy_stm32_wpan::{Ble, ble_runner, set_rng_instance};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    RNG => rng::InterruptHandler<embassy_stm32::peripherals::RNG>;
});

/// Custom service UUID
const SECURE_SERVICE_UUID: u16 = 0xABCD;
/// Characteristic that requires encryption
const SECURE_CHAR_UUID: u16 = 0xABCE;

/// BLE runner task - drives the BLE stack sequencer
#[embassy_executor::task]
async fn ble_runner_task() {
    ble_runner().await
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
    info!("Embassy STM32WBA BLE Secure Peripheral Example");

    // Initialize RNG (required by BLE stack)
    let mut rng = Rng::new(p.RNG, Irqs);
    set_rng_instance(&mut rng as *mut _ as *mut ());

    // Initialize BLE stack
    let mut ble = Ble::new();
    ble.init().expect("BLE initialization failed");
    info!("BLE stack initialized");

    // Spawn the BLE runner task (required for proper BLE operation)
    spawner.spawn(ble_runner_task().expect("Failed to create BLE runner task"));

    // ===== Configure Security =====
    let mut security = SecurityManager::new();

    // Configure security parameters:
    // - Enable bonding (store keys)
    // - Require MITM protection (passkey entry or numeric comparison)
    // - Support Secure Connections (LE Secure Connections pairing)
    let security_params = SecurityParams::new()
        .with_bonding(true)
        .with_mitm_protection(true)
        .with_secure_connections(SecureConnectionsSupport::Optional)
        .with_key_size_range(7, 16);

    security
        .set_authentication_requirements(security_params)
        .expect("Failed to set security parameters");
    info!("Security configured: Bonding + MITM + SC");

    // Enable address resolution (for bonded devices using RPA)
    security
        .set_address_resolution_enable(true)
        .expect("Failed to enable address resolution");

    // Initialize GATT server
    let mut gatt = GattServer::new();
    gatt.init().expect("GATT initialization failed");

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
        let mut advertiser = ble.advertiser();
        advertiser
            .start(adv_params.clone(), create_adv_data(), None)
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
                    info!(
                        "  Peer: {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                        conn.peer_address.0[5],
                        conn.peer_address.0[4],
                        conn.peer_address.0[3],
                        conn.peer_address.0[2],
                        conn.peer_address.0[1],
                        conn.peer_address.0[0]
                    );

                    info!("Waiting for pairing request...");
                    info!("(Try to read the secure characteristic to trigger pairing)");
                }

                GapEvent::Disconnected { handle, reason } => {
                    info!("=== DISCONNECTED ===");
                    info!("  Handle: 0x{:04X}, Reason: 0x{:02X}", handle.0, reason);

                    // Restart advertising
                    let mut advertiser = ble.advertiser();
                    let _ = advertiser.start(adv_params.clone(), create_adv_data(), None);
                    info!("Advertising restarted");
                }

                _ => {}
            }
        }

        // Process security events
        match &event.params {
            EventParams::GapPairingComplete {
                conn_handle,
                status,
                reason,
            } => {
                let pairing_status = PairingStatus::from_u8(*status);
                info!("=== PAIRING COMPLETE ===");
                info!("  Connection: 0x{:04X}", conn_handle.0);

                match pairing_status {
                    PairingStatus::Success => {
                        info!("  Status: SUCCESS");
                        info!("  Device is now bonded and can access secure characteristics");
                    }
                    PairingStatus::Timeout => {
                        info!("  Status: TIMEOUT");
                        info!("  Pairing timed out - please try again");
                    }
                    PairingStatus::Failed => {
                        let failure_reason = PairingFailureReason::from_u8(*reason);
                        info!("  Status: FAILED");
                        info!("  Reason: 0x{:02X} ({})", reason, failure_reason.description());
                    }
                }
            }

            EventParams::GapPasskeyRequest { conn_handle } => {
                info!("=== PASSKEY REQUEST ===");
                info!("  Connection: 0x{:04X}", conn_handle.0);

                // Generate a random passkey (in production, display this to user)
                // For this example, we use a fixed passkey
                let passkey: u32 = 123456;
                info!("  Passkey: {:06}", passkey);
                info!("  Enter this passkey on your phone/device!");

                if let Err(e) = security.pass_key_response(conn_handle.as_u16(), passkey) {
                    error!("Failed to send passkey response: {:?}", e);
                }
            }

            EventParams::GapNumericComparisonRequest {
                conn_handle,
                numeric_value,
            } => {
                info!("=== NUMERIC COMPARISON ===");
                info!("  Connection: 0x{:04X}", conn_handle.0);
                info!("  Displayed value: {:06}", numeric_value);
                info!("  Confirm this matches the value on your phone!");

                // Auto-confirm for this example (in production, wait for user input)
                // Set to true to accept, false to reject
                let confirm = true;
                info!("  Auto-confirming: {}", if confirm { "YES" } else { "NO" });

                if let Err(e) = security.numeric_comparison_response(conn_handle.as_u16(), confirm) {
                    error!("Failed to send numeric comparison response: {:?}", e);
                }
            }

            EventParams::GapBondLost { conn_handle } => {
                info!("=== BOND LOST ===");
                info!("  Connection: 0x{:04X}", conn_handle.0);
                info!("  Previous bond invalid, allowing rebond...");

                if let Err(e) = security.allow_rebond(conn_handle.as_u16()) {
                    error!("Failed to allow rebond: {:?}", e);
                }
            }

            EventParams::GapPairingRequest { conn_handle, is_bonded } => {
                info!("=== PAIRING REQUEST ===");
                info!("  Connection: 0x{:04X}", conn_handle.0);
                info!("  Previously bonded: {}", is_bonded);
                // The stack will handle the pairing process automatically
            }

            EventParams::GattAttributeModified {
                conn_handle,
                attr_handle,
                data,
                ..
            } => {
                info!("=== SECURE WRITE RECEIVED ===");
                info!("  Connection: 0x{:04X}, Attr: 0x{:04X}", conn_handle.0, attr_handle);
                info!("  Data ({} bytes): {:?}", data.len(), data.as_slice());
                info!("  (This write succeeded because device is paired!)");
            }

            EventParams::EncryptionChange {
                status,
                handle,
                enabled,
                ..
            } => {
                info!("=== ENCRYPTION CHANGE ===");
                info!("  Connection: 0x{:04X}", handle.0);
                info!("  Status: {:?}", status);
                info!("  Encrypted: {}", enabled);
            }

            _ => {
                // Log other events at debug level
                debug!("Event: {:?}", event);
            }
        }
    }
}

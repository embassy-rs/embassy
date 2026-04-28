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

use core::cell::RefCell;

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::aes::{self, Aes};
use embassy_stm32::mode::Blocking;
use embassy_stm32::peripherals::{AES as AesPeriph, PKA as PkaPeriph, RNG};
use embassy_stm32::pka::{self, Pka};
use embassy_stm32::rcc::{
    AHB5Prescaler, AHBPrescaler, APBPrescaler, Hse, HsePrescaler, LsConfig, LseConfig, LseDrive, LseMode, PllDiv,
    PllMul, PllPreDiv, PllSource, RtcClockSource, Sysclk, VoltageScale, mux,
};
use embassy_stm32::rng::{self, Rng};
use embassy_stm32::time::Hertz;
use embassy_stm32::{Config, bind_interrupts};
use embassy_stm32_wpan::bluetooth::ble::Ble;
use embassy_stm32_wpan::bluetooth::gap::{AdvData, AdvParams, AdvType, GapEvent};
use embassy_stm32_wpan::bluetooth::gatt::{
    CharProperties, GattEventMask, GattServer, SecurityPermissions, ServiceType, Uuid,
};
use embassy_stm32_wpan::bluetooth::security::{SecureConnectionsSupport, SecurityManager, SecurityParams};
use embassy_stm32_wpan::{ChannelPacket, Controller, HighInterruptHandler, LowInterruptHandler, ble_runner};
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::zerocopy_channel;
use static_cell::StaticCell;
use stm32wb_hci::Event;
use stm32wb_hci::event::EncryptionChange;
use stm32wb_hci::vendor::event::{GapNumericComparisonValue, GapPairingComplete, GapPairingStatus, VendorEvent};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    RNG => rng::InterruptHandler<embassy_stm32::peripherals::RNG>;
    AES => aes::InterruptHandler<AesPeriph>;
    PKA => pka::InterruptHandler<PkaPeriph>;
    RADIO => HighInterruptHandler;
    HASH => LowInterruptHandler;
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

    // Enable HSE (32 MHz external crystal) - REQUIRED for BLE radio
    config.rcc.hse = Some(Hse {
        prescaler: HsePrescaler::Div1,
    });

    // Enable LSE (32.768 kHz external crystal) - REQUIRED for BLE radio sleep timer
    config.rcc.ls = LsConfig {
        rtc: RtcClockSource::Lse,
        lsi: false,
        lse: Some(LseConfig {
            frequency: Hertz(32_768),
            mode: LseMode::Oscillator(LseDrive::MediumLow),
            peripherals_clocked: true,
        }),
    };

    // Configure PLL1 (required on WBA)
    config.rcc.pll1 = Some(embassy_stm32::rcc::Pll {
        source: PllSource::Hsi,
        prediv: PllPreDiv::Div1,
        mul: PllMul::Mul30,
        divr: Some(PllDiv::Div5),
        divq: None,
        divp: Some(PllDiv::Div30),
        frac: Some(0),
    });

    config.rcc.ahb_pre = AHBPrescaler::Div1;
    config.rcc.apb1_pre = APBPrescaler::Div1;
    config.rcc.apb2_pre = APBPrescaler::Div1;
    config.rcc.apb7_pre = APBPrescaler::Div1;
    config.rcc.ahb5_pre = AHB5Prescaler::Div4;
    config.rcc.voltage_scale = VoltageScale::Range1;
    config.rcc.sys = Sysclk::Pll1R;
    config.rcc.mux.rngsel = mux::Rngsel::Hsi;

    let p = embassy_stm32::init(config);

    // Configure radio sleep timer to use LSE
    {
        use embassy_stm32::pac::RCC;
        use embassy_stm32::pac::rcc::vals::Radiostsel;
        RCC.bdcr().modify(|w| w.set_radiostsel(Radiostsel::Lse));
    }

    info!("Embassy STM32WBA BLE Secure Peripheral Example");

    // Initialize hardware peripherals required by BLE stack
    static RNG_INST: StaticCell<Mutex<CriticalSectionRawMutex, RefCell<Rng<'static, RNG>>>> = StaticCell::new();
    let rng = RNG_INST.init(Mutex::new(RefCell::new(Rng::new(p.RNG, Irqs))));

    static AES_INST: StaticCell<Mutex<CriticalSectionRawMutex, RefCell<Aes<'static, AesPeriph, Blocking>>>> =
        StaticCell::new();
    let aes = AES_INST.init(Mutex::new(RefCell::new(Aes::new_blocking(p.AES, Irqs))));

    static PKA_INST: StaticCell<Mutex<CriticalSectionRawMutex, RefCell<Pka<'static, PkaPeriph>>>> = StaticCell::new();
    let pka = PKA_INST.init(Mutex::new(RefCell::new(Pka::new_blocking(p.PKA, Irqs))));

    info!("Hardware peripherals initialized (RNG, AES, PKA)");

    // Spawn the BLE runner task (required for proper BLE operation)
    spawner.spawn(ble_runner_task().expect("Failed to spawn BLE runner"));

    // Create BLE Event Channel
    static EVENT_BUFFER: StaticCell<[ChannelPacket; 8]> = StaticCell::new();
    static EVENT_CHANNEL: StaticCell<zerocopy_channel::Channel<'static, CriticalSectionRawMutex, ChannelPacket>> =
        StaticCell::new();

    let event_channel = EVENT_CHANNEL.init(zerocopy_channel::Channel::new(
        EVENT_BUFFER.init([ChannelPacket::default(); 8]),
    ));

    // Initialize BLE stack
    let controller = Controller::new(event_channel, rng, Some(aes), Some(pka), Irqs)
        .await
        .expect("BLE initialization failed");

    let mut ble = Ble::new(controller).await.unwrap();
    info!("BLE stack initialized");

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

                    info!("Waiting for pairing request...");
                    info!("(Try to read the secure characteristic to trigger pairing)");
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
            Event::Vendor(VendorEvent::GapPairingComplete(GapPairingComplete {
                conn_handle,
                status,
                reason,
            })) => {
                info!("=== PAIRING COMPLETE ===");
                info!("  Connection: 0x{:04X}", conn_handle.0);

                match status {
                    GapPairingStatus::Success => {
                        info!("  Status: SUCCESS");
                        info!("  Device is now bonded and can access secure characteristics");
                    }
                    GapPairingStatus::Timeout => {
                        info!("  Status: TIMEOUT");
                        info!("  Pairing timed out - please try again");
                    }
                    GapPairingStatus::Failed => {
                        info!("  Status: FAILED");
                        info!("  Reason: 0x{:02X} ({})", reason, reason);
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

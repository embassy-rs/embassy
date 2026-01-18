//! BLE Advertiser Example
//!
//! This example demonstrates the Phase 1 BLE stack implementation:
//! - Initializes the BLE stack
//! - Creates a simple GATT service with a characteristic
//! - Starts BLE advertising
//! - The device will appear as "Embassy-WBA6" in BLE scanner apps
//!
//! Hardware: STM32WBA65 or compatible
//!
//! To test:
//! 1. Flash this example to your STM32WBA6 board
//! 2. Use a BLE scanner app (nRF Connect, LightBlue, etc.)
//! 3. Look for "Embassy-WBA6" in the scan results
//! 4. Connect to see the GATT service

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::rcc::{
    AHB5Prescaler, AHBPrescaler, APBPrescaler, PllDiv, PllMul, PllPreDiv, PllSource, Sysclk, VoltageScale, mux,
};
use embassy_stm32::rng::{self, Rng};
use embassy_stm32::{Config, bind_interrupts};
use embassy_stm32_wpan::gap::{AdvData, AdvParams, AdvType};
use embassy_stm32_wpan::gatt::{CharProperties, GattEventMask, GattServer, SecurityPermissions, ServiceType, Uuid};
use embassy_stm32_wpan::{Ble, set_rng_instance};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    RNG => rng::InterruptHandler<embassy_stm32::peripherals::RNG>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();

    // Configure PLL1 (required on WBA)
    config.rcc.pll1 = Some(embassy_stm32::rcc::Pll {
        source: PllSource::HSI,
        prediv: PllPreDiv::DIV1,  // PLLM = 1 → HSI / 1 = 16 MHz
        mul: PllMul::MUL30,       // PLLN = 30 → 16 MHz * 30 = 480 MHz VCO
        divr: Some(PllDiv::DIV5), // PLLR = 5 → 96 MHz (Sysclk)
        divq: None,
        divp: Some(PllDiv::DIV30), // PLLP = 30 → 16 MHz (required for SAI)
        frac: Some(0),
    });

    config.rcc.ahb_pre = AHBPrescaler::DIV1;
    config.rcc.apb1_pre = APBPrescaler::DIV1;
    config.rcc.apb2_pre = APBPrescaler::DIV1;
    config.rcc.apb7_pre = APBPrescaler::DIV1;
    config.rcc.ahb5_pre = AHB5Prescaler::DIV4;
    config.rcc.voltage_scale = VoltageScale::RANGE1;
    config.rcc.sys = Sysclk::PLL1_R;

    // Configure RNG clock source to HSI (required for WBA)
    config.rcc.mux.rngsel = mux::Rngsel::HSI;

    let p = embassy_stm32::init(config);
    info!("Embassy STM32WBA6 BLE Advertiser Example");

    // Initialize RNG (required by BLE stack)
    let mut rng = Rng::new(p.RNG, Irqs);
    set_rng_instance(&mut rng as *mut _ as *mut ());
    info!("RNG initialized");

    // Initialize BLE stack
    let mut ble = Ble::new();
    ble.init().expect("BLE initialization failed");
    info!("BLE stack initialized");

    // Initialize GATT server
    let mut gatt = GattServer::new();
    gatt.init().expect("GATT initialization failed");
    info!("GATT server initialized");

    // Create a custom service (UUID: 0x1234)
    let service_uuid = Uuid::from_u16(0x1234);
    let service_handle = gatt
        .add_service(service_uuid, ServiceType::Primary, 5)
        .expect("Failed to add service");
    info!("Created service with handle: 0x{:04X}", service_handle.0);

    // Add a read/write characteristic (UUID: 0x5678)
    let char_uuid = Uuid::from_u16(0x5678);
    let char_properties = CharProperties::READ | CharProperties::WRITE | CharProperties::NOTIFY;
    let char_handle = gatt
        .add_characteristic(
            service_handle,
            char_uuid,
            20, // Max 20 bytes
            char_properties,
            SecurityPermissions::NONE,
            GattEventMask::ATTRIBUTE_MODIFIED,
            0,    // No encryption
            true, // Variable length
        )
        .expect("Failed to add characteristic");
    info!("Created characteristic with handle: 0x{:04X}", char_handle.0);

    // Set initial characteristic value
    let initial_value = b"Hello BLE!";
    gatt.update_characteristic_value(service_handle, char_handle, 0, initial_value)
        .expect("Failed to set characteristic value");
    info!("Set initial characteristic value");

    // Create advertising data
    let mut adv_data = AdvData::new();
    adv_data
        .add_flags(0x06) // General discoverable, BR/EDR not supported
        .expect("Failed to add flags");
    adv_data.add_name("Embassy-WBA6").expect("Failed to add name");
    adv_data
        .add_service_uuid_16(0x1234) // Advertise our custom service
        .expect("Failed to add service UUID");

    info!("Advertising data created ({} bytes)", adv_data.len());

    // Configure advertising parameters
    let adv_params = AdvParams {
        interval_min: 0x0080, // 80 ms
        interval_max: 0x0080, // 80 ms
        adv_type: AdvType::ConnectableUndirected,
        ..AdvParams::default()
    };

    // Start advertising
    let mut advertiser = ble.advertiser();
    advertiser
        .start(adv_params, adv_data, None)
        .expect("Failed to start advertising");

    info!("BLE advertising started!");
    info!("Device is visible as 'Embassy-WBA6'");
    info!("Use a BLE scanner app to discover and connect");

    // Main loop - handle BLE events
    loop {
        let event = ble.read_event().await;
        info!("BLE Event: {:?}", event);

        // In a real application, you would handle connection events,
        // characteristic writes, etc. here
    }
}

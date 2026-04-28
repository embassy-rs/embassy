//! BLE Advertiser Example
//!
//! This example demonstrates the Phase 1 BLE stack implementation:
//! - Initializes the BLE stack
//! - Creates a simple GATT service with a characteristic
//! - Starts BLE advertising
//! - The device will appear as "Embassy-WBA" in BLE scanner apps
//!
//! Hardware: STM32WBA52 or compatible
//!
//! To test:
//! 1. Flash this example to your STM32WBA board
//! 2. Use a BLE scanner app (nRF Connect, LightBlue, etc.)
//! 3. Look for "Embassy-WBA" in the scan results
//! 4. Connect to see the GATT service

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
use embassy_stm32_wpan::bluetooth::gap::{AdvData, AdvParams, AdvType};
use embassy_stm32_wpan::bluetooth::gatt::{
    CharProperties, GattEventMask, GattServer, SecurityPermissions, ServiceType, Uuid,
};
use embassy_stm32_wpan::{ChannelPacket, Controller, HighInterruptHandler, LowInterruptHandler, ble_runner};
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::zerocopy_channel;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    RNG => rng::InterruptHandler<embassy_stm32::peripherals::RNG>;
    AES => aes::InterruptHandler<AesPeriph>;
    PKA => pka::InterruptHandler<PkaPeriph>;
    RADIO => HighInterruptHandler;
    HASH => LowInterruptHandler;
});

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

    info!("Embassy STM32WBA BLE Advertiser Example");

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

    // Initialize GATT server
    let mut gatt = GattServer::new();
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
    adv_data.add_name("Embassy-WBA").expect("Failed to add name");
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
    ble.start_advertising(adv_params, adv_data, None)
        .await
        .expect("Failed to start advertising");

    info!("BLE advertising started!");
    info!("Device is visible as 'Embassy-WBA'");
    info!("Use a BLE scanner app to discover and connect");

    // Main loop - handle BLE events
    loop {
        let event = ble.read_event().await;
        info!("BLE Event: {:?}", event);

        // In a real application, you would handle connection events,
        // characteristic writes, etc. here
    }
}

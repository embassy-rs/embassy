//! BLE Peripheral with Connection Handling Example
//!
//! This example demonstrates BLE connection management:
//! - Advertises as a connectable peripheral
//! - Handles connection and disconnection events
//! - Tracks active connections
//! - Allows disconnection via button press (if available)
//!
//! Hardware: STM32WBA65 or compatible
//!
//! To test:
//! 1. Flash this example to your STM32WBA6 board
//! 2. Use a BLE scanner app (nRF Connect, LightBlue, etc.)
//! 3. Connect to "Embassy-Peripheral"
//! 4. Observe connection/disconnection events in logs

#![no_std]
#![no_main]

use core::cell::RefCell;

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::aes::{self, Aes};
use embassy_stm32::mode::Blocking;
use embassy_stm32::peripherals::{AES, PKA, RNG};
use embassy_stm32::pka::{self, Pka};
use embassy_stm32::rcc::{
    AHB5Prescaler, AHBPrescaler, APBPrescaler, Hse, HsePrescaler, LsConfig, LseConfig, LseDrive, LseMode, PllDiv,
    PllMul, PllPreDiv, PllSource, RtcClockSource, Sysclk, VoltageScale, mux,
};
use embassy_stm32::rng::{self, Rng};
use embassy_stm32::time::Hertz;
use embassy_stm32::{Config, bind_interrupts};
use embassy_stm32_wpan::bluetooth::HCI;
use embassy_stm32_wpan::bluetooth::gap::{AdvData, AdvParams, AdvType, GapEvent};
use embassy_stm32_wpan::bluetooth::gatt::{CharProperties, GattEventMask, SecurityPermissions, ServiceType, Uuid};
use embassy_stm32_wpan::{HighInterruptHandler, LowInterruptHandler, ble_runner, new_controller_state};
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use static_cell::StaticCell;
use stm32wb_hci::event::ConnectionRole;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    RNG => rng::InterruptHandler<RNG>;
    AES => aes::InterruptHandler<AES>;
    PKA => pka::InterruptHandler<PKA>;
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

    // Configure PLL1 from HSE for system clock
    // HSE = 32MHz (fixed for WBA), using prescaler DIV1 gives 32MHz to PLL
    config.rcc.pll1 = Some(embassy_stm32::rcc::Pll {
        source: PllSource::Hse,   // Use HSE as PLL source
        prediv: PllPreDiv::Div2,  // 32MHz / 2 = 16MHz to PLL input (must be 4-16MHz)
        mul: PllMul::Mul12,       // 16MHz * 12 = 192MHz VCO
        divr: Some(PllDiv::Div2), // 192MHz / 2 = 96MHz system clock
        divq: None,
        divp: Some(PllDiv::Div12), // 192MHz / 12 = 16MHz for peripherals
        frac: Some(0),
    });

    config.rcc.ahb_pre = AHBPrescaler::Div1;
    config.rcc.apb1_pre = APBPrescaler::Div1;
    config.rcc.apb2_pre = APBPrescaler::Div1;
    config.rcc.apb7_pre = APBPrescaler::Div1;
    config.rcc.ahb5_pre = AHB5Prescaler::Div4;
    config.rcc.voltage_scale = VoltageScale::Range1;
    config.rcc.sys = Sysclk::Pll1R;
    config.rcc.mux.rngsel = mux::Rngsel::Hsi; // RNG can still use HSI

    let p = embassy_stm32::init(config);
    info!("Embassy STM32WBA6 BLE Peripheral Connection Example");

    // Apply HSE trimming for accurate radio frequency (matching ST's Config_HSE)
    // and configure radio sleep timer to use LSE
    {
        use embassy_stm32::pac::RCC;
        use embassy_stm32::pac::rcc::vals::Radiostsel;
        RCC.ecscr1().modify(|w| w.set_hsetrim(0x0C));
        RCC.bdcr().modify(|w| w.set_radiostsel(Radiostsel::Lse));
    }

    // Initialize hardware peripherals required by BLE stack
    static RNG_INST: StaticCell<Mutex<CriticalSectionRawMutex, RefCell<Rng<'static, RNG>>>> = StaticCell::new();
    let rng = RNG_INST.init(Mutex::new(RefCell::new(Rng::new(p.RNG, Irqs))));

    static AES_INST: StaticCell<Mutex<CriticalSectionRawMutex, RefCell<Aes<'static, AES, Blocking>>>> =
        StaticCell::new();
    let aes = AES_INST.init(Mutex::new(RefCell::new(Aes::new_blocking(p.AES, Irqs))));

    static PKA_INST: StaticCell<Mutex<CriticalSectionRawMutex, RefCell<Pka<'static, PKA>>>> = StaticCell::new();
    let pka = PKA_INST.init(Mutex::new(RefCell::new(Pka::new_blocking(p.PKA, Irqs))));

    info!("Hardware peripherals initialized (RNG, AES, PKA)");

    // Spawn the BLE runner task (required for proper BLE operation)
    spawner.spawn(ble_runner_task().expect("Failed to spawn BLE runner"));

    // Initialize BLE stack
    let mut ble = HCI::new(new_controller_state!(8), rng, aes, pka, Irqs)
        .await
        .expect("BLE initialization failed");

    info!("BLE stack initialized");

    // Initialize GATT server with a simple service
    let mut gatt = ble.gatt_server();

    // Create a simple service for demonstration
    let service_uuid = Uuid::from_u16(0x180F); // Battery Service UUID
    let service_handle = gatt
        .add_service(service_uuid, ServiceType::Primary, 4)
        .expect("Failed to add service");

    // Add battery level characteristic
    let char_uuid = Uuid::from_u16(0x2A19); // Battery Level UUID
    let char_handle = gatt
        .add_characteristic(
            service_handle,
            char_uuid,
            1, // 1 byte for battery level
            CharProperties::READ | CharProperties::NOTIFY,
            SecurityPermissions::NONE,
            GattEventMask::ATTRIBUTE_MODIFIED,
            0,
            false,
        )
        .expect("Failed to add characteristic");

    // Set initial battery level (100%)
    gatt.update_characteristic_value(service_handle, char_handle, 0, &[100])
        .expect("Failed to set battery level");

    info!("GATT service created (Battery Service)");

    // Create advertising data
    let mut adv_data = AdvData::new();
    adv_data.add_flags(0x06).expect("Failed to add flags");
    adv_data.add_name("Embassy-Peripheral").expect("Failed to add name");
    adv_data
        .add_service_uuid_16(0x180F)
        .expect("Failed to add service UUID");

    // Configure advertising parameters for connectable advertising
    let adv_params = AdvParams {
        interval_min: 0x0050, // 50 ms
        interval_max: 0x0050,
        adv_type: AdvType::ConnectableUndirected,
        ..AdvParams::default()
    };

    // Start advertising
    {
        ble.start_advertising(adv_params.clone(), adv_data.clone(), None)
            .await
            .expect("Failed to start advertising");
    }

    info!("BLE advertising started as 'Embassy-Peripheral'");
    info!("Waiting for connections...");

    // Main event loop
    loop {
        let event = ble.read_event().await;

        // Process the event and update connection state
        if let Some(gap_event) = ble.process_event(&event) {
            match gap_event {
                GapEvent::Connected(conn) => {
                    info!("=== CONNECTION ESTABLISHED ===");
                    info!("  Handle: 0x{:04X}", conn.handle.0);
                    info!(
                        "  Role: {}",
                        match conn.role {
                            ConnectionRole::Central => "Central",
                            ConnectionRole::Peripheral => "Peripheral",
                        }
                    );
                    info!("  Peer Address: {}", conn.peer_address);
                    info!("  Interval: {} ", conn.interval.interval());
                    info!("  Latency: {}", conn.interval.conn_latency());
                    info!("  Timeout: {}", conn.interval.supervision_timeout());
                    info!("  Active connections: {}", ble.connections().count());

                    // Note: Advertising typically stops automatically on connection
                    // If you want to support multiple connections, restart advertising here
                }

                GapEvent::Disconnected { handle, reason } => {
                    info!("=== DISCONNECTION ===");
                    info!("  Handle: 0x{:04X}", handle.0);
                    info!("  Reason: 0x{:02X} ({})", reason, disconnect_reason_str(reason));
                    info!("  Active connections: {}", ble.connections().count());

                    // Restart advertising after disconnection.
                    // Advertising parameters are still configured, just re-enable.
                    info!("Restarting advertising...");
                    match ble.start_advertising(adv_params.clone(), adv_data.clone(), None).await {
                        Ok(()) => info!("Advertising restarted"),
                        Err(e) => error!("Failed to restart advertising: {:?}", e),
                    }
                }

                GapEvent::ConnectionParamsUpdated { handle, interval } => {
                    info!("=== CONNECTION PARAMS UPDATED ===");
                    info!("  Handle: 0x{:04X}", handle.0);
                    info!("  New Interval: {}", interval.interval());
                    info!("  New Latency: {}", interval.conn_latency());
                    info!("  New Timeout: {}", interval.supervision_timeout());
                }

                GapEvent::PhyUpdated { handle, tx_phy, rx_phy } => {
                    info!("=== PHY UPDATED ===");
                    info!("  Handle: 0x{:04X}", handle.0);
                    info!("  TX PHY: {:?}", tx_phy);
                    info!("  RX PHY: {:?}", rx_phy);
                }

                GapEvent::DataLengthChanged {
                    handle,
                    max_tx_octets,
                    max_rx_octets,
                    ..
                } => {
                    info!("=== DATA LENGTH CHANGED ===");
                    info!("  Handle: 0x{:04X}", handle.0);
                    info!("  Max TX: {} bytes", max_tx_octets);
                    info!("  Max RX: {} bytes", max_rx_octets);
                }
            }
        } else {
            // Log other events for debugging
            debug!("Other BLE Event: {:?}", event);
        }
    }
}

/// Convert disconnect reason code to human-readable string
fn disconnect_reason_str(reason: u8) -> &'static str {
    match reason {
        0x08 => "Connection Timeout",
        0x13 => "Remote User Terminated",
        0x14 => "Remote Low Resources",
        0x15 => "Remote Power Off",
        0x16 => "Local Host Terminated",
        0x1A => "Unsupported Remote Feature",
        0x3B => "Unacceptable Connection Parameters",
        0x3D => "MIC Failure",
        0x3E => "Connection Failed to Establish",
        _ => "Unknown",
    }
}

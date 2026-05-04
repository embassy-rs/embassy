//! BLE Direct Test Mode (DTM) while BLE stack is running — STM32WBA65RI
//!
//! Demonstrates how to enter DTM mode from a live BLE stack:
//! - Starts up with normal BLE advertising as "Embassy-DTM"
//! - Press USER button (B1) to trigger a DTM test
//! - DTM runs for DTM_TEST_DURATION_SECS, then BLE advertising resumes
//! - Button can be pressed again to run another test
//!
//! This contrasts with ble_dtm.rs which uses new_dtm() for a dedicated
//! DTM-only device. This example shows prepare_for_dtm() / dtm_end() / deinit()
//! for devices that need both normal BLE and RF testing capability.
//!
//! Hardware: STM32WBA65RI (Nucleo-WBA65RI)

#![no_std]
#![no_main]

use core::cell::RefCell;

use defmt::*;
use embassy_executor::Spawner;
use embassy_futures::select::{Either, select};
use embassy_stm32::aes::{self, Aes};
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::Pull;
use embassy_stm32::mode::Blocking;
use embassy_stm32::peripherals::{AES as AesPeriph, PKA as PkaPeriph, RNG};
use embassy_stm32::pka::{self, Pka};
use embassy_stm32::rng::{self, Rng};
use embassy_stm32::{Config, bind_interrupts, exti, interrupt};
use embassy_stm32_wpan::bluetooth::gap::{AdvData, AdvParams, AdvType, GapEvent};
use embassy_stm32_wpan::bluetooth::gatt::{CharProperties, GattEventMask, SecurityPermissions, ServiceType, Uuid};
use embassy_stm32_wpan::bluetooth::hci::types::DtmPacketPayload;
use embassy_stm32_wpan::bluetooth::{HCI, Normal, Test};
use embassy_stm32_wpan::{
    ControllerState, HighInterruptHandler, LowInterruptHandler, ble_runner, new_controller_state,
};
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_time::Timer;
use static_cell::StaticCell;
use stm32wb_hci::Event;
use stm32wb_hci::event::ConnectionRole;
use {defmt_rtt as _, panic_probe as _};

// ---- DTM test configuration ----
#[allow(dead_code)]
enum DtmMode {
    Tx,
    Rx,
}
const DTM_MODE: DtmMode = DtmMode::Rx;
const DTM_CHANNEL: u8 = 19; // 2440 MHz
const DTM_DATA_LENGTH: u8 = 37; // bytes per packet
const DTM_TEST_DURATION_SECS: u64 = 10;
// --------------------------------

bind_interrupts!(struct Irqs {
    RNG    => rng::InterruptHandler<RNG>;
    AES    => aes::InterruptHandler<AesPeriph>;
    PKA    => pka::InterruptHandler<PkaPeriph>;
    EXTI13 => exti::InterruptHandler<interrupt::typelevel::EXTI13>;
    RADIO  => HighInterruptHandler;
    HASH   => LowInterruptHandler;
});

#[embassy_executor::task]
async fn ble_runner_task() {
    ble_runner().await
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        use embassy_stm32::time::Hertz;

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
        // HSE = 32MHz (fixed for WBA), prediv /2 gives 16MHz to PLL input (must be 4-16MHz)
        // VCO = 16MHz * 12 = 192MHz, PLLR = 192 / 2 = 96MHz system clock
        config.rcc.pll1 = Some(Pll {
            source: PllSource::Hse,
            prediv: PllPreDiv::Div2,  // 32MHz / 2 = 16MHz to PLL input
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
        config.rcc.ahb5_pre = AHB5Prescaler::Div4; // Radio bus: 96MHz / 4 = 24MHz
        config.rcc.voltage_scale = VoltageScale::Range1;
        config.rcc.sys = Sysclk::Pll1R;
        config.rcc.mux.rngsel = mux::Rngsel::Hsi; // RNG clock from HSI (16 MHz)
    }

    let p = embassy_stm32::init(config);
    // Configure radio sleep timer to use LSE
    {
        use embassy_stm32::pac::RCC;
        use embassy_stm32::pac::rcc::vals::Radiostsel;
        // WBA65 requires HSE trimming for accurate radio frequency
        RCC.ecscr1().modify(|w| w.set_hsetrim(0x0C));
        RCC.bdcr().modify(|w| w.set_radiostsel(Radiostsel::Lse));
    }

    info!("Embassy STM32WBA65 BLE DTM Runtime Example");

    let mut button = ExtiInput::new(p.PC13, p.EXTI13, Pull::Up, Irqs);

    static RNG_INST: StaticCell<Mutex<CriticalSectionRawMutex, RefCell<Rng<'static, RNG>>>> = StaticCell::new();
    let rng = RNG_INST.init(Mutex::new(RefCell::new(Rng::new(p.RNG, Irqs))));

    static AES_INST: StaticCell<Mutex<CriticalSectionRawMutex, RefCell<Aes<'static, AesPeriph, Blocking>>>> =
        StaticCell::new();
    let aes = AES_INST.init(Mutex::new(RefCell::new(Aes::new_blocking(p.AES, Irqs))));

    static PKA_INST: StaticCell<Mutex<CriticalSectionRawMutex, RefCell<Pka<'static, PkaPeriph>>>> = StaticCell::new();
    let pka = PKA_INST.init(Mutex::new(RefCell::new(Pka::new_blocking(p.PKA, Irqs))));

    // Spawn the BLE runner task (required for proper BLE operation)
    spawner.spawn(ble_runner_task().expect("Failed to spawn BLE runner"));

    // Initialize BLE stack — state is stored and passed through each reinit
    let mut state: &'static mut ControllerState = new_controller_state!(8);
    let mut ble = HCI::new(state, rng, aes, pka, Irqs)
        .await
        .expect("BLE initialization failed");

    info!("BLE stack initialized");

    let mut gatt = ble.gatt_server();

    let service_handle = gatt
        .add_service(Uuid::from_u16(0x180F), ServiceType::Primary, 4)
        .expect("Failed to add service");

    let char_handle = gatt
        .add_characteristic(
            service_handle,
            Uuid::from_u16(0x2A19),
            1,
            CharProperties::READ | CharProperties::NOTIFY,
            SecurityPermissions::NONE,
            GattEventMask::ATTRIBUTE_MODIFIED,
            0,
            false,
        )
        .expect("Failed to add characteristic");

    gatt.update_characteristic_value(service_handle, char_handle, 0, &[100])
        .expect("Failed to set battery level");

    info!("GATT service created (Battery Service)");

    let mut adv_data = AdvData::new();
    adv_data.add_flags(0x06).expect("Failed to add flags");
    adv_data.add_name("Embassy-DTM").expect("Failed to add name");
    adv_data
        .add_service_uuid_16(0x180F)
        .expect("Failed to add service UUID");

    let adv_params = AdvParams {
        interval_min: 0x0050,
        interval_max: 0x0050,
        adv_type: AdvType::ConnectableUndirected,
        ..AdvParams::default()
    };

    start_advertising(&mut ble, &adv_params, &adv_data).await;
    info!("Advertising as 'Embassy-DTM' — press B1 to trigger DTM test");

    // DTM packet interval is 625 µs per Vol 6, Part F, Section 4.1.6.
    let expected: u32 = DTM_TEST_DURATION_SECS as u32 * 1_000_000 / 625;

    loop {
        match select(ble.read_event(), button.wait_for_falling_edge()).await {
            Either::First(event) => {
                handle_ble_event(&mut ble, &event, &adv_params, &adv_data).await;
            }
            Either::Second(_) => {
                info!("Button pressed — entering DTM mode");

                // deinit terminates connections and advertising via hci_reset(),
                // leaving the LL in a clean idle state. State is returned so it
                // can be passed directly to the next HCI instance.
                state = ble.deinit().expect("deinit failed");

                // Initialize a minimal DTM-only instance (no GAP/GATT needed for DTM)
                let mut dtm_ble = HCI::new_dtm(state, rng, Irqs).await.expect("DTM initialization failed");

                run_dtm_test(&mut dtm_ble, expected).await;

                // Deinit the DTM instance — resets radio hardware so PhyStartClbr
                // succeeds when advertising is configured after full BLE reinit.
                info!("DTM done — reinitializing BLE stack");
                state = dtm_ble.deinit().expect("deinit after DTM failed");

                // Reinitialize full BLE stack with the same state
                ble = HCI::new(state, rng, aes, pka, Irqs).await.expect("BLE reinit failed");

                // Rebuild GATT services (cleared by hci_reset inside deinit)
                let mut gatt = ble.gatt_server();
                let service_handle = gatt
                    .add_service(Uuid::from_u16(0x180F), ServiceType::Primary, 4)
                    .expect("Failed to add service");
                let char_handle = gatt
                    .add_characteristic(
                        service_handle,
                        Uuid::from_u16(0x2A19),
                        1,
                        CharProperties::READ | CharProperties::NOTIFY,
                        SecurityPermissions::NONE,
                        GattEventMask::ATTRIBUTE_MODIFIED,
                        0,
                        false,
                    )
                    .expect("Failed to add characteristic");
                gatt.update_characteristic_value(service_handle, char_handle, 0, &[100])
                    .expect("Failed to set battery level");

                start_advertising(&mut ble, &adv_params, &adv_data).await;
                info!("BLE reinitialized — advertising resumed, press B1 for another test");
            }
        }
    }
}

async fn handle_ble_event(ble: &mut HCI<Normal>, event: &Event, adv_params: &AdvParams, adv_data: &AdvData) {
    if let Some(gap_event) = ble.process_event(event) {
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
                info!("  Interval: {}", conn.interval.interval());
                info!("  Latency: {}", conn.interval.conn_latency());
                info!("  Timeout: {}", conn.interval.supervision_timeout(),);
                info!("  Active connections: {}", ble.connections().count());
            }

            GapEvent::Disconnected { handle, reason } => {
                info!("=== DISCONNECTION ===");
                info!("  Handle: 0x{:04X}", handle.0);
                info!("  Reason: 0x{:02X} ({})", reason, disconnect_reason_str(reason));
                info!("  Active connections: {}", ble.connections().count());

                info!("Restarting advertising...");
                start_advertising(ble, adv_params, adv_data).await;
                info!("Advertising restarted");
            }

            GapEvent::ConnectionParamsUpdated { handle, interval } => {
                info!("=== CONNECTION PARAMS UPDATED ===");
                info!("  Handle: 0x{:04X}", handle.0);
                info!("  New Interval: {} ", interval.interval());
                info!("  New Latency: {}", interval.conn_latency());
                info!("  New Timeout: {}", interval.supervision_timeout(),);
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
        debug!("Other BLE Event: {:?}", event);
    }
}

async fn start_advertising(ble: &mut HCI<Normal>, params: &AdvParams, data: &AdvData) {
    ble.start_advertising(params.clone(), data.clone(), None)
        .await
        .expect("start advertising failed");
}

async fn run_dtm_test(ble: &mut HCI<Test>, expected: u32) {
    let freq_mhz = 2402 + 2 * DTM_CHANNEL as u32;

    match DTM_MODE {
        DtmMode::Tx => {
            info!(
                "DTM TX: channel {} ({}MHz), {} byte payload, PRBS9, {}s",
                DTM_CHANNEL, freq_mhz, DTM_DATA_LENGTH, DTM_TEST_DURATION_SECS
            );
            info!("Expected ~{} packets (~1600 packets/s at 625us interval)", expected);

            ble.dtm_transmit(DTM_CHANNEL, DTM_DATA_LENGTH, DtmPacketPayload::Prbs9)
                .expect("DTM TX start failed");

            Timer::after_secs(DTM_TEST_DURATION_SECS).await;

            match ble.dtm_end() {
                Ok(_) => info!("DTM TX ended after {}s", DTM_TEST_DURATION_SECS),
                Err(e) => error!("dtm_end failed: {:?}", e),
            }
        }

        DtmMode::Rx => {
            info!(
                "DTM RX: channel {} ({}MHz), {}s",
                DTM_CHANNEL, freq_mhz, DTM_TEST_DURATION_SECS
            );
            info!("Expected ~{} packets (~1600 packets/s at 625us interval)", expected);

            ble.dtm_receive(DTM_CHANNEL).expect("DTM RX start failed");

            Timer::after_secs(DTM_TEST_DURATION_SECS).await;

            match ble.dtm_end() {
                Ok(received) => {
                    let received = received as u32;
                    let lost = expected.saturating_sub(received);
                    let per_pct = lost * 100 / expected;
                    let per_frac = (lost * 10000 / expected) % 100;
                    info!("--- DTM RX Results ---");
                    info!("  Expected : {} packets", expected);
                    info!("  Received : {} packets", received);
                    info!("  Lost     : {} packets", lost);
                    info!("  PER      : {}.{:02}%", per_pct, per_frac);
                }
                Err(e) => error!("dtm_end failed: {:?}", e),
            }
        }
    }
}

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

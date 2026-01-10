#![no_std]
#![no_main]

//! STM32H7 Primary Core (CM4) Intercore Communication Example
//!
//! This example demonstrates reliable communication between the Cortex-M7 and
//! Cortex-M4 cores using a shared memory region
//!
//! The CM4 core handles:
//! - MPU configuration to make shared memory non-cacheable
//! - Clock initialization
//! - Toggling LED state in shared memory
//!
//! Usage:
//! 1. Flash the CM0+ (secondary) core binary first
//! 2. Then flash this CM4 (primary) core binary
//! 3. The system will start with CM4 toggling LED state and CM0+ responding by
//!    physically toggling the LED

use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicBool, Ordering};

use defmt::*;
#[cfg(feature = "defmt-rtt")]
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::ipcc::{Config as IPCCConfig, Ipcc, ReceiveInterruptHandler, TransmitInterruptHandler};
use embassy_stm32::{Config, SharedData, bind_interrupts};
use embassy_time::{Duration, Timer};
use panic_probe as _;

bind_interrupts!(struct Irqs{
    IPCC_C1_RX => ReceiveInterruptHandler;
    IPCC_C1_TX => TransmitInterruptHandler;
});

#[unsafe(link_section = ".shared_data.0")]
static SHARED_DATA: MaybeUninit<SharedData> = MaybeUninit::uninit();
#[unsafe(link_section = ".shared_data.1")]
#[unsafe(no_mangle)] // make sure the symbol is not optimized out!
static LED_STATE: AtomicBool = AtomicBool::new(false);

#[embassy_executor::task]
async fn blink_heartbeat(mut led: Output<'static>) {
    loop {
        info!("CM4 heartbeat");
        led.set_level(Level::High);
        Timer::after_millis(100).await;
        led.set_level(Level::Low);
        Timer::after_millis(5900).await;
    }
}
#[embassy_executor::main(executor = "embassy_stm32::Executor", entry = "cortex_m_rt::entry")]
// #[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    // Initialize the CM4 core
    let mut config = Config::default();
    config.min_stop_pause = Duration::from_millis(50);
    // Set the LPTIM1 clock source to LSI & enable for secondary core time driver
    config.rcc.ls.lsi = true;
    config.rcc.mux.lptim1sel = embassy_stm32::pac::rcc::vals::Lptimsel::LSI;
    let _p = embassy_stm32::init_primary(config, &SHARED_DATA);
    #[cfg(feature = "defmt-serial")]
    {
        use embassy_stm32::mode::Blocking;
        use embassy_stm32::usart::Uart;
        use static_cell::StaticCell;
        let config = embassy_stm32::usart::Config::default();
        let uart = Uart::new_blocking(_p.LPUART1, _p.PA3, _p.PA2, config).expect("failed to configure UART!");
        static SERIAL: StaticCell<Uart<'static, Blocking>> = StaticCell::new();
        defmt_serial::defmt_serial(SERIAL.init(uart));
    }
    info!("CM4 core initialized");
    let ipcc = Ipcc::new(_p.IPCC, Irqs, IPCCConfig::default());
    let [ch1, _ch2, _ch3, _ch4, _ch5, _ch6] = ipcc.split();
    let (mut tx, mut _rx) = ch1;

    let blue_led = Output::new(_p.PB15, Level::High, Speed::Low);
    _spawner.spawn(blink_heartbeat(blue_led).unwrap());

    loop {
        info!("CM4: Sending message!!!");
        tx.send(|| {
            let new_led_state = !LED_STATE.load(Ordering::SeqCst);
            info!("CM4: Send! New LED state: {}", new_led_state);
            LED_STATE.store(new_led_state, Ordering::SeqCst);
        })
        .await;
        Timer::after_secs(10).await;
    }
}

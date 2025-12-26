#![no_std]
#![no_main]

use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicBool, Ordering};

use defmt::*;
#[cfg(feature = "defmt-rtt")]
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::ipcc::{Config as IPCCConfig, InterruptHandler, Ipcc};
use embassy_stm32::{SharedData, bind_interrupts};
use embassy_time::Timer;
use panic_probe as _;

bind_interrupts!(struct Irqs{
    IPCC_C2_RX_C2_TX => InterruptHandler;
});

#[unsafe(link_section = ".shared_data")]
static SHARED_DATA: MaybeUninit<SharedData> = MaybeUninit::uninit();
#[unsafe(link_section = ".shared_data")]
static LED_STATE: AtomicBool = AtomicBool::new(false);

#[embassy_executor::task]
async fn blink_heartbeat(mut led: Output<'static>) {
    loop {
        info!("CM0+ heartbeat");
        led.set_level(Level::High);
        Timer::after_millis(100).await;
        led.set_level(Level::Low);
        Timer::after_millis(900).await;
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    // Initialize the secondary core
    let p = embassy_stm32::init_secondary(&SHARED_DATA);
    #[cfg(feature = "defmt-serial")]
    {
        use embassy_stm32::mode::Blocking;
        use embassy_stm32::usart::Uart;
        use static_cell::StaticCell;
        let config = embassy_stm32::usart::Config::default();
        let uart = Uart::new_blocking(p.LPUART1, p.PA3, p.PA2, config).expect("failed to configure UART!");
        static SERIAL: StaticCell<Uart<'static, Blocking>> = StaticCell::new();
        defmt_serial::defmt_serial(SERIAL.init(uart));
    }
    info!("CM0+ core initialized!");

    let ipcc = Ipcc::new(p.IPCC, Irqs, IPCCConfig::default());
    let [ch1, _ch2, _ch3, _ch4, _ch5, _ch6] = ipcc.split();
    let (mut _tx, mut rx) = ch1;

    // Set up LED
    let mut blue_led = Output::new(p.PB15, Level::Low, Speed::Low); // LD3 (heartbeat)
    let red_led = Output::new(p.PB11, Level::High, Speed::Low);
    _spawner.spawn(blink_heartbeat(red_led).unwrap());

    loop {
        let state = rx.receive(|| Some(LED_STATE.load(Ordering::Relaxed))).await;
        info!("CM0+ Recieve: {}", state);
        blue_led.set_level(if state { Level::High } else { Level::Low });
    }
}

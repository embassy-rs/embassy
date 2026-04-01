#![no_std]
#![no_main]

mod dfu;
mod flash_ops;

use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::exti::{self, ExtiInput};
use embassy_stm32::gpio::{Input, Level, Output, Pull, Speed};
use embassy_stm32::mode::Async;
use embassy_stm32::rcc::XspiClkSrc;
use embassy_stm32::{bind_interrupts, interrupt, usart};
use embassy_time::{Duration, Timer, with_timeout};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(
    pub struct Irqs {
        EXTI0 => exti::InterruptHandler<interrupt::typelevel::EXTI0>;
        EXTI13 => exti::InterruptHandler<interrupt::typelevel::EXTI13>;
    }
);

#[embassy_executor::task]
async fn user_button_task(mut button: ExtiInput<'static, Async>, mut green_led: Output<'static>) {
    loop {
        button.wait_for_rising_edge().await;
        info!("Button pressed — green LED on");
        #[cfg(feature = "dk")]
        green_led.set_high();
        #[cfg(feature = "nucleo")]
        green_led.set_low();

        button.wait_for_falling_edge().await;
        info!("Button released — green LED off");
        #[cfg(feature = "dk")]
        green_led.set_low();
        #[cfg(feature = "nucleo")]
        green_led.set_high();
    }
}

/// Wait for PE0 to be held for 3 seconds, then call mark_booted().
#[embassy_executor::task]
async fn mark_booted_task(mut button: ExtiInput<'static, Async>) {
    loop {
        button.wait_for_rising_edge().await;
        info!("PE0 pressed, hold 3s to confirm boot...");

        match with_timeout(Duration::from_secs(3), button.wait_for_falling_edge()).await {
            Ok(()) => {
                info!("PE0 released early, not confirming");
            }
            Err(_) => {
                info!("Confirming boot...");
                flash_ops::mark_booted();
                info!("Boot confirmed! (BOOT_MAGIC written)");
                button.wait_for_falling_edge().await;
                return;
            }
        }
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("App starting");

    let mut config = embassy_stm32::Config::default();
    config.rcc.mux.xspi2sel = XspiClkSrc::PER;
    config.rcc.vddio3_1v8 = true;
    let p = embassy_stm32::init(config);

    info!("HAL initialized");

    // Check PE0 (tamper button) — DFU if held, otherwise pass to mark_booted_task
    let pe0 = Input::new(p.PE0, Pull::Down);

    if pe0.is_high() {
        drop(pe0);
        info!("PE0 held at boot — entering DFU mode");
        info!("Connect UART (USART1: PE5=TX, PE6=RX, 115200 8E1)");
        info!("Start XMODEM sender on host first, then reset board with PE0 held");

        let mut red_led = Output::new(p.PG10, Level::High, Speed::Low);
        #[cfg(feature = "dk")]
        let mut green_led = Output::new(p.PO1, Level::Low, Speed::Low);
        #[cfg(feature = "nucleo")]
        let mut green_led = Output::new(p.PG0, Level::High, Speed::Low);

        let mut uart_config = usart::Config::default();
        uart_config.parity = usart::Parity::ParityEven;
        let uart = usart::Uart::new_blocking(p.USART1, p.PE6, p.PE5, uart_config).unwrap();
        let (tx, rx) = uart.split();

        match dfu::receive_firmware(rx, tx) {
            Ok(size) => {
                info!("DFU complete: {} bytes. Reset to apply.", size);
                red_led.set_high();
                #[cfg(feature = "dk")]
                green_led.set_high();
                #[cfg(feature = "nucleo")]
                green_led.set_low();
            }
            Err(()) => {
                info!("DFU failed. Reset to retry.");
                loop {
                    red_led.toggle();
                    cortex_m::asm::delay(8_000_000);
                }
            }
        }

        loop {
            cortex_m::asm::wfe();
        }
    }

    // Normal app mode
    let mut red_led = Output::new(p.PG10, Level::Low, Speed::Low);
    #[cfg(feature = "dk")]
    let green_led = Output::new(p.PO1, Level::Low, Speed::Low);
    #[cfg(feature = "nucleo")]
    let green_led = Output::new(p.PG0, Level::High, Speed::Low);

    let user_button = ExtiInput::new(p.PC13, p.EXTI13, Pull::Down, Irqs);
    spawner.spawn(user_button_task(user_button, green_led).unwrap());

    let pe0_exti = ExtiInput::from_input(pe0, p.EXTI0, Irqs);
    spawner.spawn(mark_booted_task(pe0_exti).unwrap());

    info!("Blinking red LED. Hold PE0 3s to confirm boot.");
    loop {
        red_led.set_high();
        Timer::after_millis(500).await;
        red_led.set_low();
        Timer::after_millis(500).await;
    }
}

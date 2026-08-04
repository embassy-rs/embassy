#![no_std]
#![no_main]

//! Low-power example for STM32F401.
//!
//! This example automatically enters STOP mode during idle periods longer than 300ms
//! to save power. On the STM32F401, the expected current consumption in STOP mode is
//! ~42uA. This is the expected behavior. Enabling the `FPDS` bit in `PWR.CR1` can
//! further reduce the current to ~12uA at the cost of increased wakeup latency.
//!
//! # Debugging
//! Enabling `enable_debug_during_sleep` consumes a significant amount of current
//! (measured at ~1mA on STM32F401RC). Furthermore, even with this option enabled,
//! RTT output is unreliable because RAM clocks are likely stopped during sleep. Because
//! of this, this project relies on an LED connected to `PC5` to indicate system status.
//! It is highly recommended to use USART for logging instead.
//!
//! # GPIO Configuration
//! After power-on, most STM32F401 IOs default to floating input mode. Any unconnected
//! pins will float, causing their internal Schmitt triggers to oscillate and draw
//! significant leakage current. Setting these unused pins to Analog In mode disables
//! the digital input/output circuitry entirely to prevent this leakage. (This issue is
//! specific to older series, newer families like the STM32U5 default to Analog In mode
//! upon reset.)
//!
//! If the board is connected to other external peripherals, configuring communication IOs
//! as Analog IN may cause similar floating input leakage on the peripheral side. Depending
//! on the communication interface and the peripheral's requirements, you may need to
//! configure these pins as pulled-up/pulled-down inputs or explicit high/low outputs.
//!
//! For more information, please refer to:
//! [AN4365: Using STM32F4 MCU power modes with best dynamic efficiency](https://www.st.com/resource/en/application_note/an4365-using-stm32f4-mcu-power-modes-with-best-dynamic-efficiency-stmicroelectronics.pdf)

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::select::{Either, select};
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{Level, Output, Pull, Speed};
use embassy_stm32::{bind_interrupts, interrupt, pac};
use embassy_time::{Duration, Timer, block_for};
use panic_probe as _;

bind_interrupts!(struct Irqs {
    EXTI0 => embassy_stm32::exti::InterruptHandler<interrupt::typelevel::EXTI0>;
});

#[embassy_executor::main(entry = "cortex_m_rt::entry", executor = "embassy_stm32::executor::Executor")]
async fn main(_spawner: Spawner) {
    let mut config = embassy_stm32::Config::default();
    config.enable_debug_during_sleep = false;
    config.min_stop_pause = Duration::from_millis(300);
    // Initialize RTC so the time driver can hand off timekeeping during STOP mode
    config.rtc = Default::default();

    let p = embassy_stm32::init(config);

    info!("System initialized.");

    // Block for 3 seconds to keep the MCU awake after power-on.
    // This provides a window for the debugger to attach and reprogram the flash.
    // Without this, entering STOP mode immediately disconnects SWD,
    // requiring you to manually press the BOOT button to re-flash.
    info!("Waiting 3 seconds to allow debugger connection...");
    block_for(Duration::from_secs(3));

    // Set all unused IOs to Analog In mode to minimize power consumption.
    pac::GPIOA.moder().modify(|w| {
        for i in 0..16 {
            if i != 13 && i != 14 {
                // Exclude SWDIO/SWCLK here to handle them separately
                w.set_moder(i, pac::gpio::vals::Moder::Analog);
            }
        }
    });

    // Uncomment or comment the following block to toggle SWD pins (PA13, PA14) Analog mode.
    // Note: Setting these to Analog will minimize power consumption but will detach the SWD debugger.
    pac::GPIOA.moder().modify(|w| {
        w.set_moder(13, pac::gpio::vals::Moder::Analog);
        w.set_moder(14, pac::gpio::vals::Moder::Analog);
    });

    pac::GPIOB.moder().modify(|w| {
        for i in 0..16 {
            if i != 0 {
                // Exclude PB0 used for EXTI
                w.set_moder(i, pac::gpio::vals::Moder::Analog);
            }
        }
    });

    pac::GPIOC.moder().modify(|w| {
        for i in 0..16 {
            if i != 5 {
                // Exclude PC5 used for LED
                w.set_moder(i, pac::gpio::vals::Moder::Analog);
            }
        }
    });
    pac::GPIOD.moder().write_value(pac::gpio::regs::Moder(0xFFFF_FFFF));
    pac::GPIOH.moder().write_value(pac::gpio::regs::Moder(0xFFFF_FFFF));

    info!("Unused pins set to Analog mode.");

    // Configure PB0 as EXTI input with pull-down
    let mut button = ExtiInput::new(p.PB0, p.EXTI0, Pull::Up, Irqs);

    let mut led = Output::new(p.PC5, Level::Low, Speed::Low);

    info!("Starting main low-power loop. WFI will automatically enter STOP mode.");

    loop {
        let exti_fut = button.wait_for_falling_edge();
        let timeout_fut = Timer::after(Duration::from_secs(5));

        match select(exti_fut, timeout_fut).await {
            Either::First(_) => {
                info!("Woke up by EXTI (PB0 falling edge).");
            }
            Either::Second(_) => {
                info!("Woke up by 5s Timer timeout.");
            }
        }

        led.set_high();
        Timer::after(Duration::from_millis(300)).await;
        led.set_low();
    }
}

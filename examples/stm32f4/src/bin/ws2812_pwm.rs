// Configure TIM3 in PWM mode, and start DMA Transfer(s) to send color data into ws2812.
// We assume the DIN pin of ws2812 connect to GPIO PB4, and ws2812 is properly powered.
//
// The idea is that the data rate of ws2812 is 800 kHz, and it use different duty ratio to represent bit 0 and bit 1.
// Thus we can set TIM overflow at 800 kHz, and change duty ratio of TIM to meet the bit representation of ws2812.
//
// you may also want to take a look at `ws2812_spi.rs` file, which make use of SPI instead.
//
// Warning:
// DO NOT stare at ws2812 directy (especially after each MCU Reset), its (max) brightness could easily make your eyes feel burn.

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_stm32::gpio::OutputType;
use embassy_stm32::time::khz;
use embassy_stm32::timer::low_level::CountingMode;
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};
use embassy_stm32::timer::Channel;
use embassy_time::{Duration, Ticker, Timer};
use {defmt_rtt as _, panic_probe as _};

/// This example is written for the nucleo-stm32f429zi, with a stm32f429zi chip.
///
/// If you are using a different board or chip, make sure you update the following:
///
/// * [ ] Update .cargo/config.toml with the correct `probe-rs run --chip STM32F429ZITx`chip name.
/// * [ ] Update Cargo.toml to have the correct `embassy-stm32` feature, it is
///       currently `stm32f429zi`.
/// * [ ] If your board has a special clock or power configuration, make sure that it is
///       set up appropriately.
/// * [ ] If your board has different pin mapping, update any pin numbers or peripherals
///       to match your schematic
///
/// If you are unsure, please drop by the Embassy Matrix chat for support, and let us know:
///
/// * Which example you are trying to run
/// * Which chip and board you are using
///
/// Embassy Chat: https://matrix.to/#/#embassy-rs:matrix.org
#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut device_config = embassy_stm32::Config::default();

    // set SYSCLK/HCLK/PCLK2 to 20 MHz, thus each tick is 0.05 us,
    // and ws2812 timings are integer multiples of 0.05 us
    {
        use embassy_stm32::rcc::*;
        use embassy_stm32::time::*;
        device_config.enable_debug_during_sleep = true;
        device_config.rcc.hse = Some(Hse {
            freq: mhz(12),
            mode: HseMode::Oscillator,
        });
        device_config.rcc.pll_src = PllSource::HSE;
        device_config.rcc.pll = Some(Pll {
            prediv: PllPreDiv::DIV6,
            mul: PllMul::MUL80,
            divp: Some(PllPDiv::DIV8),
            divq: None,
            divr: None,
        });
        device_config.rcc.sys = Sysclk::PLL1_P;
    }

    let mut dp = embassy_stm32::init(device_config);

    let mut ws2812_pwm = SimplePwm::new(
        dp.TIM3,
        Some(PwmPin::new_ch1(dp.PB4, OutputType::PushPull)),
        None,
        None,
        None,
        khz(800), // data rate of ws2812
        CountingMode::EdgeAlignedUp,
    );

    // construct ws2812 non-return-to-zero (NRZ) code bit by bit
    // ws2812 only need 24 bits for each LED, but we add one bit more to keep PWM output low

    let max_duty = ws2812_pwm.get_max_duty() as u16;
    let n0 = 8 * max_duty / 25; // ws2812 Bit 0 high level timing
    let n1 = 2 * n0; // ws2812 Bit 1 high level timing

    let turn_off = [
        n0, n0, n0, n0, n0, n0, n0, n0, // Green
        n0, n0, n0, n0, n0, n0, n0, n0, // Red
        n0, n0, n0, n0, n0, n0, n0, n0, // Blue
        0,  // keep PWM output low after a transfer
    ];

    let dim_white = [
        n0, n0, n0, n0, n0, n0, n1, n0, // Green
        n0, n0, n0, n0, n0, n0, n1, n0, // Red
        n0, n0, n0, n0, n0, n0, n1, n0, // Blue
        0,  // keep PWM output low after a transfer
    ];

    let color_list = &[&turn_off, &dim_white];

    let pwm_channel = Channel::Ch1;

    // make sure PWM output keep low on first start
    ws2812_pwm.set_duty(pwm_channel, 0);

    // flip color at 2 Hz
    let mut ticker = Ticker::every(Duration::from_millis(500));

    loop {
        for &color in color_list {
            // with &mut, we can easily reuse same DMA channel multiple times
            ws2812_pwm.waveform_up(&mut dp.DMA1_CH2, pwm_channel, color).await;
            // ws2812 need at least 50 us low level input to confirm the input data and change it's state
            Timer::after_micros(50).await;
            // wait until ticker tick
            ticker.next().await;
        }
    }
}

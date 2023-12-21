// Configure TIM3 in PWM mode, and start DMA Transfer(s) to send color data into ws2812.
// We assume the DIN pin of ws2812 connect to GPIO PB4, and ws2812 is properly powered.
//
// The idea is that the data rate of ws2812 is 800 kHz, and it use different duty ratio to represent bit 0 and bit 1.
// Thus we can set TIM overflow at 800 kHz, and let TIM Update Event trigger a DMA transfer, then let DMA change CCR value,
// such that pwm duty ratio meet the bit representation of ws2812.
//
// You may want to modify TIM CCR with Cortex core directly,
// but according to my test, Cortex core will need to run far more than 100 MHz to catch up with TIM.
// Thus we need to use a DMA.
//
// This demo is a combination of HAL, PAC, and manually invoke `dma::Transfer`.
// If you need a simpler way to control ws2812, you may want to take a look at `ws2812_spi.rs` file, which make use of SPI.
//
// Warning:
// DO NOT stare at ws2812 directy (especially after each MCU Reset), its (max) brightness could easily make your eyes feel burn.

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_stm32::gpio::OutputType;
use embassy_stm32::pac;
use embassy_stm32::time::khz;
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};
use embassy_stm32::timer::{Channel, CountingMode};
use embassy_time::{Duration, Ticker, Timer};
use {defmt_rtt as _, panic_probe as _};

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

    let max_duty = ws2812_pwm.get_max_duty();
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

    {
        use embassy_stm32::dma::{Burst, FifoThreshold, Transfer, TransferOptions};

        // configure FIFO and MBURST of DMA, to minimize DMA occupation on AHB/APB
        let mut dma_transfer_option = TransferOptions::default();
        dma_transfer_option.fifo_threshold = Some(FifoThreshold::Full);
        dma_transfer_option.mburst = Burst::Incr8;

        // flip color at 2 Hz
        let mut ticker = Ticker::every(Duration::from_millis(500));

        loop {
            for &color in color_list {
                // start PWM output
                ws2812_pwm.enable(pwm_channel);

                // PAC level hacking, enable timer-update-event trigger DMA
                pac::TIM3.dier().modify(|v| v.set_ude(true));

                unsafe {
                    Transfer::new_write(
                        // with &mut, we can easily reuse same DMA channel multiple times
                        &mut dp.DMA1_CH2,
                        5,
                        color,
                        pac::TIM3.ccr(pwm_channel.index()).as_ptr() as *mut _,
                        dma_transfer_option,
                    )
                    .await;

                    // Turn off timer-update-event trigger DMA as soon as possible.
                    // Then clean the FIFO Error Flag if set.
                    pac::TIM3.dier().modify(|v| v.set_ude(false));
                    if pac::DMA1.isr(0).read().feif(2) {
                        pac::DMA1.ifcr(0).write(|v| v.set_feif(2, true));
                    }

                    // ws2812 need at least 50 us low level input to confirm the input data and change it's state
                    Timer::after_micros(50).await;
                }

                // stop PWM output for saving some energy
                ws2812_pwm.disable(pwm_channel);

                // wait until ticker tick
                ticker.next().await;
            }
        }
    }
}

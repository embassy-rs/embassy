// Mimic PWM with SPI, to control ws2812
// We assume the DIN pin of ws2812 connect to GPIO PB5, and ws2812 is properly powered.
//
// The idea is that the data rate of ws2812 is 800 kHz, and it use different duty ratio to represent bit 0 and bit 1.
// Thus we can adjust SPI to send each *round* of data at 800 kHz, and in each *round*, we can adjust each *bit* to mimic 2 different PWM waveform.
// such that the output waveform meet the bit representation of ws2812.
//
// If you want to save SPI for other purpose, you may want to take a look at `ws2812_pwm_dma.rs` file, which make use of TIM and DMA.
//
// Warning:
// DO NOT stare at ws2812 directy (especially after each MCU Reset), its (max) brightness could easily make your eyes feel burn.

#![no_std]
#![no_main]

use embassy_stm32::time::khz;
use embassy_stm32::{dma, spi};
use embassy_time::{Duration, Ticker, Timer};
use {defmt_rtt as _, panic_probe as _};

// we use 16 bit data frame format of SPI, to let timing as accurate as possible.
// thanks to loose tolerance of ws2812 timing, you can also use 8 bit data frame format, thus you will need to adjust the bit representation.
const N0: u16 = 0b1111100000000000u16; // ws2812 Bit 0 high level timing
const N1: u16 = 0b1111111111000000u16; // ws2812 Bit 1 high level timing

// ws2812 only need 24 bits for each LED,
// but we add one bit more to keep SPI output low at the end

static TURN_OFF: [u16; 25] = [
    N0, N0, N0, N0, N0, N0, N0, N0, // Green
    N0, N0, N0, N0, N0, N0, N0, N0, // Red
    N0, N0, N0, N0, N0, N0, N0, N0, // Blue
    0,  // keep SPI output low after last bit
];

static DIM_WHITE: [u16; 25] = [
    N0, N0, N0, N0, N0, N0, N1, N0, // Green
    N0, N0, N0, N0, N0, N0, N1, N0, // Red
    N0, N0, N0, N0, N0, N0, N1, N0, // Blue
    0,  // keep SPI output low after last bit
];

static COLOR_LIST: &[&[u16]] = &[&TURN_OFF, &DIM_WHITE];

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    let mut device_config = embassy_stm32::Config::default();

    // Since we use 16 bit SPI, and we need each round 800 kHz,
    // thus SPI output speed should be 800 kHz * 16 = 12.8 MHz, and APB clock should be 2 * 12.8 MHz = 25.6 MHz.
    //
    // As for my setup, with 12 MHz HSE, I got 25.5 MHz SYSCLK, which is slightly slower, but it's ok for ws2812.
    {
        use embassy_stm32::rcc::{Hse, HseMode, Pll, PllMul, PllPDiv, PllPreDiv, PllSource, Sysclk};
        use embassy_stm32::time::mhz;
        device_config.enable_debug_during_sleep = true;
        device_config.rcc.hse = Some(Hse {
            freq: mhz(12),
            mode: HseMode::Oscillator,
        });
        device_config.rcc.pll_src = PllSource::HSE;
        device_config.rcc.pll = Some(Pll {
            prediv: PllPreDiv::DIV6,
            mul: PllMul::MUL102,
            divp: Some(PllPDiv::DIV8),
            divq: None,
            divr: None,
        });
        device_config.rcc.sys = Sysclk::PLL1_P;
    }

    let dp = embassy_stm32::init(device_config);

    // Set SPI output speed.
    // It's ok to blindly set frequency to 12800 kHz, the hal crate will take care of the SPI CR1 BR field.
    // And in my case, the real bit rate will be 25.5 MHz / 2 = 12_750 kHz
    let mut spi_config = spi::Config::default();
    spi_config.frequency = khz(12_800);

    // Since we only output waveform, then the Rx and Sck and RxDma it is not considered
    let mut ws2812_spi = spi::Spi::new_txonly_nosck(dp.SPI1, dp.PB5, dp.DMA2_CH3, dma::NoDma, spi_config);

    // flip color at 2 Hz
    let mut ticker = Ticker::every(Duration::from_millis(500));

    loop {
        for &color in COLOR_LIST {
            ws2812_spi.write(color).await.unwrap();
            // ws2812 need at least 50 us low level input to confirm the input data and change it's state
            Timer::after_micros(50).await;
            // wait until ticker tick
            ticker.next().await;
        }
    }
}

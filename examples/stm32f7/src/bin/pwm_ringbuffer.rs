#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_stm32::gpio::OutputType;
use embassy_stm32::time::mhz;
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

// If you are trying this and your USB device doesn't connect, the most
// common issues are the RCC config and vbus_detection
//
// See https://embassy.dev/book/#_the_usb_examples_are_not_working_on_my_board_is_there_anything_else_i_need_to_configure
// for more information.
#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("PWM Ring Buffer Example");

    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        use embassy_stm32::time::Hertz;
        config.rcc.hse = Some(Hse {
            freq: Hertz(8_000_000),
            mode: HseMode::Bypass,
        });
        config.rcc.pll_src = PllSource::HSE;
        config.rcc.pll = Some(Pll {
            prediv: PllPreDiv::DIV4,
            mul: PllMul::MUL200,
            divp: Some(PllPDiv::DIV2), // 8mhz / 4 * 200 / 2 = 200Mhz
            divq: Some(PllQDiv::DIV4), // 8mhz / 4 * 200 / 4 = 100Mhz
            divr: None,
        });
        config.rcc.ahb_pre = AHBPrescaler::DIV1;
        config.rcc.apb1_pre = APBPrescaler::DIV4;
        config.rcc.apb2_pre = APBPrescaler::DIV2;
        config.rcc.sys = Sysclk::PLL1_P;
    }
    let p = embassy_stm32::init(config);

    // Initialize PWM on TIM1
    let ch1_pin = PwmPin::new(p.PE9, OutputType::PushPull);
    let ch2_pin = PwmPin::new(p.PE11, OutputType::PushPull);
    let mut pwm = SimplePwm::new(
        p.TIM1,
        Some(ch1_pin),
        Some(ch2_pin),
        None,
        None,
        mhz(1),
        Default::default(),
    );

    // Use channel 1 for static PWM at 50%
    let mut ch1 = pwm.ch1();
    ch1.enable();
    ch1.set_duty_cycle_fraction(1, 2);
    info!("Channel 1 (PE9/D6): Static 50% duty cycle");

    // Get max duty from channel 1 before converting channel 2
    let max_duty = ch1.max_duty_cycle();
    info!("PWM max duty: {}", max_duty);

    // Create a DMA ring buffer for channel 2
    const BUFFER_SIZE: usize = 128;
    static mut DMA_BUFFER: [u16; BUFFER_SIZE] = [0u16; BUFFER_SIZE];
    let dma_buffer = unsafe { &mut *core::ptr::addr_of_mut!(DMA_BUFFER) };

    // Pre-fill buffer with initial sine wave using lookup table approach
    for i in 0..BUFFER_SIZE {
        // Simple sine approximation using triangle wave
        let phase = (i * 256) / BUFFER_SIZE;
        let sine_approx = if phase < 128 {
            phase as u16 * 2
        } else {
            (255 - phase) as u16 * 2
        };
        dma_buffer[i] = (sine_approx as u32 * max_duty as u32 / 256) as u16;
    }

    // Convert channel 2 to ring-buffered PWM
    let mut ring_pwm = pwm.ch1().into_ring_buffered_channel(p.DMA2_CH5, dma_buffer);

    info!("Ring buffer capacity: {}", ring_pwm.capacity());

    // Pre-write some initial data to the buffer before starting
    info!("Pre-writing initial waveform data...");

    ring_pwm.write(&[0; BUFFER_SIZE]).unwrap();

    // Enable the PWM channel output
    ring_pwm.enable();

    // Start the DMA ring buffer
    ring_pwm.start();
    info!("Channel 2 (PE11/D5): Ring buffered sine wave started");

    // Give DMA time to start consuming
    Timer::after_millis(10).await;

    // Continuously update the waveform
    let mut phase: f32 = 0.0;
    let mut amplitude: f32 = 1.0;
    let mut amplitude_direction = -0.05;

    loop {
        // Generate new waveform data with varying amplitude
        let mut new_data = [0u16; 32];
        for i in 0..new_data.len() {
            // Triangle wave approximation for sine
            let pos = ((i as u32 + phase as u32) * 4) % 256;
            let sine_approx = if pos < 128 {
                pos as u16 * 2
            } else {
                (255 - pos) as u16 * 2
            };
            let scaled = (sine_approx as u32 * (amplitude * 256.0) as u32) / (256 * 256);
            new_data[i] = ((scaled * max_duty as u32) / 256) as u16;
        }

        // Write new data to the ring buffer
        match ring_pwm.write_exact(&new_data).await {
            Ok(_remaining) => {}
            Err(e) => {
                info!("Write error: {:?}", e);
            }
        }

        // Update phase for animation effect
        phase += 2.0;
        if phase >= 64.0 {
            phase = 0.0;
        }

        // Vary amplitude for breathing effect
        amplitude += amplitude_direction;
        if amplitude <= 0.2 || amplitude >= 1.0 {
            amplitude_direction = -amplitude_direction;
        }

        // Log buffer status periodically
        if (phase as u32) % 10 == 0 {
            match ring_pwm.len() {
                Ok(len) => info!("Ring buffer fill: {}/{}", len, ring_pwm.capacity()),
                Err(_) => info!("Error reading buffer length"),
            }
        }
    }
}

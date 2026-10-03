// required-features: pwm, input
#![no_std]
#![no_main]
#[path = "../common.rs"]
mod common;

use common::*;
use defmt::assert;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_stm32::gpio::{Input, OutputType, Pull};
use embassy_stm32::time::Hertz;
use embassy_stm32::timer::Channel;
use embassy_stm32::timer::low_level::CountingMode;
use embassy_stm32::timer::ringbuffered::RingBufferedPwmChannel;
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};
use embassy_time::{Duration, Instant, Ticker, Timer};
use panic_probe as _;
use rand_chacha::ChaCha8Rng;
use rand_chacha::rand_core::{Rng, SeedableRng};

/// count of random length writes
const COUNT: u32 = 10;
/// max data length in writes
const MAX_LEN: usize = 8;
/// PWM frequency
const FREQUENCY: u32 = 20;
/// ring buffer size
const DMA_BUF_SIZE: usize = 16;
/// required success ratio
const SUCCESS_RATIO: f32 = 0.9;

#[cfg_attr(
    feature = "stop",
    embassy_executor::main(executor = "embassy_stm32::executor::Executor", entry = "cortex_m_rt::entry")
)]
#[cfg_attr(not(feature = "stop"), embassy_executor::main)]
async fn main(_spawner: Spawner) {
    let p = init();
    info!("Hello World!");

    let tim = peri!(p, TIM);
    let tim_dma = peri!(p, TIM_DMA);
    let pwm_pin = peri!(p, TIM_PWM_PIN);
    let input_pin = peri!(p, INPUT_PIN);
    let irq = irqs!(TIM);

    let input = Input::new(input_pin, Pull::None);

    let ch1 = PwmPin::new(pwm_pin, OutputType::PushPull);
    let mut pwm = SimplePwm::new(
        tim,
        Some(ch1),
        None,
        None,
        None,
        Hertz::hz(FREQUENCY),
        CountingMode::EdgeAlignedUp,
    );

    let duration = Duration::from_hz(FREQUENCY as u64);
    let go = Instant::now() + 2 * duration;

    let max_duty = pwm.max_duty_cycle() as u16;
    info!("max duty: {}", max_duty);

    let mut ch1 = pwm.channel(Channel::Ch1);
    ch1.enable();

    static mut BUF: [u16; DMA_BUF_SIZE] = [0; _];
    let mut ch1: RingBufferedPwmChannel<'_, _, _> =
        ch1.into_ring_buffered_channel(tim_dma, unsafe { &mut BUF[..] }, irq);

    // For different signals will be generated
    // and then sampled to verify correctness:
    // 1. ___________________________ 0/0 duty
    // 2. ---______---______---______ 1/3 duty
    // 3. ------___------___------___ 2/3 duty
    // 4. --------------------------- 3/3 duty
    //     ^  ^  ^  ^  ^  ^  ^  ^  ^  sampling points

    let len_seed = 1337;
    let data_seed = 2137;

    let tx = async |ch: &mut RingBufferedPwmChannel<'_, _, _>, data_buf: &[u8]| {
        let mut pwm_buf = [0; 8];
        for (i, data) in data_buf.iter().enumerate() {
            pwm_buf[i] = (*data as u32 % 4 * max_duty as u32 / 3) as u16;
        }
        ch.write_exact(&pwm_buf[..data_buf.len()]).await.unwrap();
    };

    let tx_f = async {
        let mut len_rng = ChaCha8Rng::seed_from_u64(len_seed);
        let mut data_rng = ChaCha8Rng::seed_from_u64(data_seed);

        let mut data_buf = [0; 8];
        data_rng.fill_bytes(&mut data_buf);
        tx(&mut ch1, &data_buf).await;

        // start at go - half duration (to start ring buffer before update event)
        Timer::at(go - duration / 2).await;
        ch1.start();

        for _ in 0..COUNT {
            let len = 1 + len_rng.next_u32() as usize % MAX_LEN;
            let data_buf = &mut data_buf[..len];
            data_rng.fill_bytes(data_buf);
            tx(&mut ch1, data_buf).await;
        }
    };

    let mut successes = 0;
    let mut failures = 0;

    let mut rx = async |ticker: &mut Ticker, data_buf: &[u8]| {
        for data in data_buf {
            ticker.next().await;
            let a = input.is_high();
            ticker.next().await;
            let b = input.is_high();
            ticker.next().await;
            let c = input.is_high();

            let got = match (a, b, c) {
                (false, false, false) => 0,
                (true, false, false) => 1,
                (true, true, false) => 2,
                (true, true, true) => 3,
                _ => u8::MAX,
            };

            if got == *data % 4 {
                successes += 1;
            } else {
                failures += 1;
            }
        }
    };

    let rx_f = async {
        let mut len_rng = ChaCha8Rng::seed_from_u64(len_seed);
        let mut data_rng = ChaCha8Rng::seed_from_u64(data_seed);

        let mut ticker = Ticker::every(duration / 3);
        // start at go + offset to the middle between transitions
        ticker.reset_at(go - duration / 3 + duration / 6);

        let mut data_buf = [0; MAX_LEN];
        data_rng.fill_bytes(&mut data_buf);
        rx(&mut ticker, &data_buf).await;

        for _ in 0..COUNT {
            let len = 1 + len_rng.next_u32() as usize % MAX_LEN;
            let data_buf = &mut data_buf[..len];
            data_rng.fill_bytes(data_buf);
            rx(&mut ticker, data_buf).await;
        }
    };

    join(tx_f, rx_f).await;

    let success_ratio = successes as f32 / (successes + failures) as f32;
    info!("success ratio: {}", success_ratio);
    assert!(success_ratio > SUCCESS_RATIO);

    info!("Test OK");
    cortex_m::asm::bkpt();
}

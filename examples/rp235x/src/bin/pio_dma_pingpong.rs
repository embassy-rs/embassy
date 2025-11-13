//! This example shows how to feed alternating buffers to the PIO without downtime.

#![no_std]
#![no_main]
use defmt::info;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::PIO0;
use embassy_rp::pio::program::pio_asm;
use embassy_rp::pio::{Config, Direction, InterruptHandler, Pio};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

/// The desired samples/second to output
const SAMPLE_RATE: u32 = 16_000;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut p = embassy_rp::init(Default::default());
    let mut pio = Pio::new(p.PIO0, Irqs);

    const PIO_OUTPUT_RATE: u32 = 2; // pio program efficiency (clocks per output)
    let clock_freq = embassy_rp::clocks::clk_sys_freq();
    let divider = clock_freq / PIO_OUTPUT_RATE / SAMPLE_RATE;
    info!("PIO base divider: {}", divider);

    let pio_program = pio_asm!(
        ".origin 0"
        ".wrap_target"
        "PULL"
        "OUT PINS, 8"
        ".wrap"
    );

    let pio_pins = [
        &pio.common.make_pio_pin(p.PIN_5),
        &pio.common.make_pio_pin(p.PIN_6),
        &pio.common.make_pio_pin(p.PIN_7),
        &pio.common.make_pio_pin(p.PIN_8),
        &pio.common.make_pio_pin(p.PIN_9),
        &pio.common.make_pio_pin(p.PIN_10),
        &pio.common.make_pio_pin(p.PIN_11),
        &pio.common.make_pio_pin(p.PIN_12),
    ];

    let mut cfg = Config::default();
    cfg.use_program(&pio.common.load_program(&pio_program.program), &[]);
    cfg.clock_divider = (divider as u16).into();
    cfg.set_out_pins(&pio_pins);

    pio.sm0.set_pin_dirs(Direction::Out, &pio_pins);
    pio.sm0.set_config(&cfg);
    pio.sm0.set_enable(true);

    let tx = pio.sm0.tx();

    let mut buffer_1 = [0x0u8; 128];
    let mut buffer_2 = [0x0u8; 128];

    let mut sample_index = 0usize;
    tx.dma_push_ping_pong(
        p.DMA_CH0.reborrow(),
        p.DMA_CH1.reborrow(),
        &mut buffer_1,
        &mut buffer_2,
        |buf| {
            info!("In start of fill callback, index={}", sample_index);
            if sample_index > 100_000 {
                buf.iter_mut().for_each(|b| *b = 0);
                return core::ops::ControlFlow::Break(());
            }

            for b in buf.iter_mut() {
                // generate a 440hz sine wave
                let time = sample_index as f32 / SAMPLE_RATE as f32;
                let wave = fast_sin(time * 440. * core::f32::consts::PI * 2.);

                // convert [-1, 1] to [0, 255]
                *b = ((wave + 1.) / 2. * 256.) as u8;

                sample_index += 1;
            }

            core::ops::ControlFlow::Continue(())
        },
    )
    .await;

    // push a zero to reset the pin state
    tx.dma_push(p.DMA_CH0, &[0u8; 1], false).await;
}

/// Based on https://bmtechjournal.wordpress.com/2020/05/27/super-fast-quadratic-sinusoid-approximation/
fn fast_sin(x: f32) -> f32 {
    use num_traits::float::FloatCore as _;

    let fake_sin_2 = |x: f32| 2.0 * x * (1.0 - (2.0 * x).abs());
    let range_limiter_2 = |x: f32| x - x.floor() - 0.5;

    -4.0 * fake_sin_2(range_limiter_2(x / (2.0 * core::f32::consts::PI)))
}

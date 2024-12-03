//! This example shows how to use the PIO module in the RP2040 to read a quadrature rotary encoder.

#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_rp::gpio::Pull;
use embassy_rp::peripherals::PIO0;
use embassy_rp::{bind_interrupts, pio};
use fixed::traits::ToFixed;
use pio::{Common, Config, FifoJoin, Instance, InterruptHandler, Pio, PioPin, ShiftDirection, StateMachine};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

pub struct PioEncoder<'d, T: Instance, const SM: usize> {
    sm: StateMachine<'d, T, SM>,
}

impl<'d, T: Instance, const SM: usize> PioEncoder<'d, T, SM> {
    pub fn new(
        pio: &mut Common<'d, T>,
        mut sm: StateMachine<'d, T, SM>,
        pin_a: impl PioPin,
        pin_b: impl PioPin,
    ) -> Self {
        let mut pin_a = pio.make_pio_pin(pin_a);
        let mut pin_b = pio.make_pio_pin(pin_b);
        pin_a.set_pull(Pull::Up);
        pin_b.set_pull(Pull::Up);
        sm.set_pin_dirs(pio::Direction::In, &[&pin_a, &pin_b]);

        let prg = pio_proc::pio_asm!("wait 1 pin 1", "wait 0 pin 1", "in pins, 2", "push",);

        let mut cfg = Config::default();
        cfg.set_in_pins(&[&pin_a, &pin_b]);
        cfg.fifo_join = FifoJoin::RxOnly;
        cfg.shift_in.direction = ShiftDirection::Left;
        cfg.clock_divider = 10_000.to_fixed();
        cfg.use_program(&pio.load_program(&prg.program), &[]);
        sm.set_config(&cfg);
        sm.set_enable(true);
        Self { sm }
    }

    pub async fn read(&mut self) -> Direction {
        loop {
            match self.sm.rx().wait_pull().await {
                0 => return Direction::CounterClockwise,
                1 => return Direction::Clockwise,
                _ => {}
            }
        }
    }
}

pub enum Direction {
    Clockwise,
    CounterClockwise,
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let Pio { mut common, sm0, .. } = Pio::new(p.PIO0, Irqs);

    let mut encoder = PioEncoder::new(&mut common, sm0, p.PIN_4, p.PIN_5);

    let mut count = 0;
    loop {
        info!("Count: {}", count);
        count += match encoder.read().await {
            Direction::Clockwise => 1,
            Direction::CounterClockwise => -1,
        };
    }
}

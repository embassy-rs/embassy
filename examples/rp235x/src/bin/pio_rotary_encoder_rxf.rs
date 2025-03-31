//! This example shows how to use the PIO module in the RP235x to read a quadrature rotary encoder.
//! It differs from the other example in that it uses the RX FIFO as a status register

#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_rp::gpio::Pull;
use embassy_rp::peripherals::PIO0;
use embassy_rp::pio::program::pio_asm;
use embassy_rp::{bind_interrupts, pio, Peri};
use embassy_time::Timer;
use fixed::traits::ToFixed;
use pio::{Common, Config, FifoJoin, Instance, InterruptHandler, Pio, PioPin, ShiftDirection, StateMachine};
use {defmt_rtt as _, panic_probe as _};

// Program metadata for `picotool info`
#[link_section = ".bi_entries"]
#[used]
pub static PICOTOOL_ENTRIES: [embassy_rp::binary_info::EntryAddr; 4] = [
    embassy_rp::binary_info::rp_program_name!(c"example_pio_rotary_encoder_rxf"),
    embassy_rp::binary_info::rp_cargo_version!(),
    embassy_rp::binary_info::rp_program_description!(c"Rotary encoder (RXF)"),
    embassy_rp::binary_info::rp_program_build_attribute!(),
];

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
        pin_a: Peri<'d, impl PioPin>,
        pin_b: Peri<'d, impl PioPin>,
    ) -> Self {
        let mut pin_a = pio.make_pio_pin(pin_a);
        let mut pin_b = pio.make_pio_pin(pin_b);
        pin_a.set_pull(Pull::Up);
        pin_b.set_pull(Pull::Up);

        sm.set_pin_dirs(pio::Direction::In, &[&pin_a, &pin_b]);

        let prg = pio_asm!(
            "start:"
            // encoder count is stored in X
            "mov isr, x"
            // and then moved to the RX FIFO register
            "mov rxfifo[0], isr"

            // wait for encoder transition
            "wait 1 pin 1"
            "wait 0 pin 1"

            "set y, 0"
            "mov y, pins[1]"

            // update X depending on pin 1
            "jmp !y decr"

            // this is just a clever way of doing x++
            "mov x, ~x"
            "jmp x--, incr"
            "incr:"
            "mov x, ~x"
            "jmp start"

            // and this is x--
            "decr:"
            "jmp x--, start"
        );

        let mut cfg = Config::default();
        cfg.set_in_pins(&[&pin_a, &pin_b]);
        cfg.fifo_join = FifoJoin::RxAsStatus;
        cfg.shift_in.direction = ShiftDirection::Left;
        cfg.clock_divider = 10_000.to_fixed();
        cfg.use_program(&pio.load_program(&prg.program), &[]);
        sm.set_config(&cfg);

        sm.set_enable(true);
        Self { sm }
    }

    pub async fn read(&mut self) -> u32 {
        self.sm.get_rxf_entry(0)
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

    loop {
        info!("Count: {}", encoder.read().await);
        Timer::after_millis(1000).await;
    }
}

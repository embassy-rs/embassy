//! This example shows powerful PIO module in the RP235x chip to communicate with a HD44780 display.
//! See (https://www.sparkfun.com/datasheets/LCD/HD44780.pdf)

#![no_std]
#![no_main]

use core::fmt::Write;

use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::PIO0;
use embassy_rp::pio::{InterruptHandler, Pio};
use embassy_rp::pio_programs::hd44780::{PioHD44780, PioHD44780CommandSequenceProgram, PioHD44780CommandWordProgram};
use embassy_rp::pwm::{self, Pwm};
use embassy_time::{Instant, Timer};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(pub struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // this test assumes a 2x16 HD44780 display attached as follow:
    //   rs  = PIN0
    //   rw  = PIN1
    //   e   = PIN2
    //   db4 = PIN3
    //   db5 = PIN4
    //   db6 = PIN5
    //   db7 = PIN6
    // additionally a pwm signal for a bias voltage charge pump is provided on pin 15,
    // allowing direct connection of the display to the RP235x without level shifters.
    let p = embassy_rp::init(Default::default());

    let _pwm = Pwm::new_output_b(p.PWM_SLICE7, p.PIN_15, {
        let mut c = pwm::Config::default();
        c.divider = 125.into();
        c.top = 100;
        c.compare_b = 50;
        c
    });

    let Pio {
        mut common, sm0, irq0, ..
    } = Pio::new(p.PIO0, Irqs);

    let word_prg = PioHD44780CommandWordProgram::new(&mut common);
    let seq_prg = PioHD44780CommandSequenceProgram::new(&mut common);

    let mut hd = PioHD44780::new(
        &mut common,
        sm0,
        irq0,
        p.DMA_CH3,
        p.PIN_0,
        p.PIN_1,
        p.PIN_2,
        p.PIN_3,
        p.PIN_4,
        p.PIN_5,
        p.PIN_6,
        &word_prg,
        &seq_prg,
    )
    .await;

    loop {
        struct Buf<const N: usize>([u8; N], usize);
        impl<const N: usize> Write for Buf<N> {
            fn write_str(&mut self, s: &str) -> Result<(), core::fmt::Error> {
                for b in s.as_bytes() {
                    if self.1 >= N {
                        return Err(core::fmt::Error);
                    }
                    self.0[self.1] = *b;
                    self.1 += 1;
                }
                Ok(())
            }
        }
        let mut buf = Buf([0; 16], 0);
        write!(buf, "up {}s", Instant::now().as_micros() as f32 / 1e6).unwrap();
        hd.add_line(&buf.0[0..buf.1]).await;
        Timer::after_secs(1).await;
    }
}

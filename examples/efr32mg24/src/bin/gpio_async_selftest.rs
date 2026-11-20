//! GPIO edge-wait self-test on the Seeed Studio XIAO MG24. Needs no wiring.
//!
//! A Series 2 output keeps its input on, so PA07 sees its own edges.
#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::select::{Either, select};
use embassy_silabs::gpio::Flex;
use embassy_silabs::pac;
use embassy_time::{Instant, Timer};
use panic_probe as _;

const ROUNDS: u32 = 10;
/// PA07: port A, pin 7.
const PORT: usize = 0;
const PIN: u32 = 7;

fn drive(high: bool) {
    let bit = pac::gpio::regs::PortDout(1 << PIN);
    if high {
        pac::GPIO.p_dout_set(PORT).write_value(bit);
    } else {
        pac::GPIO.p_dout_clr(PORT).write_value(bit);
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_silabs::init(embassy_silabs::Config::default());
    info!("gpio_async_selftest: start");

    let mut flex = Flex::new(p.PA07);
    flex.set_low();
    flex.set_as_output();

    let mut failures = 0u32;
    for rising in [true, false] {
        let edge = if rising { "rising" } else { "falling" };
        for round in 0..ROUNDS {
            flex.set_level((!rising).into());
            Timer::after_millis(10).await;
            let written = core::cell::Cell::new(None::<Instant>);
            let wait = async {
                if rising {
                    flex.wait_for_rising_edge().await
                } else {
                    flex.wait_for_falling_edge().await
                }
            };
            let stimulus = async {
                Timer::after_millis(100).await;
                written.set(Some(Instant::now()));
                drive(rising);
                Timer::after_millis(100).await;
            };
            let ok = match select(wait, stimulus).await {
                Either::First(()) => match written.get() {
                    Some(t) => {
                        info!(
                            "{} edge {}: woke {} us after the write",
                            edge,
                            round,
                            t.elapsed().as_micros()
                        );
                        true
                    }
                    None => {
                        warn!("{} edge {}: woke before the write", edge, round);
                        false
                    }
                },
                Either::Second(()) => {
                    warn!("{} edge {}: no wake 100 ms after the write", edge, round);
                    false
                }
            };
            if !ok {
                failures += 1;
            }
        }
    }
    flex.set_low();

    if failures == 0 {
        info!("PASS: {} rising and {} falling edges", ROUNDS, ROUNDS);
    } else {
        error!("FAIL: {} of {} rounds", failures, 2 * ROUNDS);
    }
    loop {
        Timer::after_secs(1).await;
    }
}

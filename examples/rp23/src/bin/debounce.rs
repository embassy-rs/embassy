//! This example shows the ease of debouncing a button with async rust.
//! Hook up a button or switch between pin 9 and ground.

#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_rp::gpio::{Input, Level, Pull};
use embassy_time::{with_deadline, Duration, Instant, Timer};
use {defmt_rtt as _, panic_probe as _};

pub struct Debouncer<'a> {
    input: Input<'a>,
    debounce: Duration,
}

impl<'a> Debouncer<'a> {
    pub fn new(input: Input<'a>, debounce: Duration) -> Self {
        Self { input, debounce }
    }

    pub async fn debounce(&mut self) -> Level {
        loop {
            let l1 = self.input.get_level();

            self.input.wait_for_any_edge().await;

            Timer::after(self.debounce).await;

            let l2 = self.input.get_level();
            if l1 != l2 {
                break l2;
            }
        }
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let mut btn = Debouncer::new(Input::new(p.PIN_9, Pull::Up), Duration::from_millis(20));

    info!("Debounce Demo");

    loop {
        // button pressed
        btn.debounce().await;
        let start = Instant::now();
        info!("Button Press");

        match with_deadline(start + Duration::from_secs(1), btn.debounce()).await {
            // Button Released < 1s
            Ok(_) => {
                info!("Button pressed for: {}ms", start.elapsed().as_millis());
                continue;
            }
            // button held for > 1s
            Err(_) => {
                info!("Button Held");
            }
        }

        match with_deadline(start + Duration::from_secs(5), btn.debounce()).await {
            // Button released <5s
            Ok(_) => {
                info!("Button pressed for: {}ms", start.elapsed().as_millis());
                continue;
            }
            // button held for > >5s
            Err(_) => {
                info!("Button Long Held");
            }
        }

        // wait for button release before handling another press
        btn.debounce().await;
        info!("Button pressed for: {}ms", start.elapsed().as_millis());
    }
}

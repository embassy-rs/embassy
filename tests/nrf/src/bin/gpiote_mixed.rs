//! HAL GPIO/GPIOTE interrupt handlers and a user handler bound on the same GPIOTE interrupt.
//!
//! PIN_A is an async `Input`, then an async `InputChannel`. PIN_B drives the wire, and its own
//! SENSE is serviced by `PinBHandler`. Both pins see the same edges, so each handler must leave the
//! other's LATCH bit and event alone.
#![no_std]
#![no_main]

#[path = "../common.rs"]
mod common;

use core::sync::atomic::{AtomicU8, AtomicU32, Ordering};

use defmt::{assert, assert_eq, info, unwrap};
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_nrf::gpio::{self, Flex, Input, Level, OutputDrive, Pin as _, Pull};
use embassy_nrf::gpiote::{self, InputChannel, InputChannelPolarity};
use embassy_nrf::interrupt::typelevel::{Handler, Interrupt};
use embassy_nrf::pac::gpio::vals::Sense;
use embassy_nrf::peripherals;
use embassy_time::{Duration, Timer, with_timeout};

static PIN_B_NUM: AtomicU8 = AtomicU8::new(0);
static PIN_B_EVENTS: AtomicU32 = AtomicU32::new(0);

/// User handler for PIN_B's SENSE.
struct PinBHandler;

impl<I: Interrupt> Handler<I> for PinBHandler {
    unsafe fn on_interrupt() {
        #[cfg(not(feature = "nrf54l15"))]
        peri_pac!(GPIOTE).events_port(0).write_value(0);
        #[cfg(feature = "nrf54l15")]
        peri_pac!(GPIOTE).events_port(0).secure().write_value(0);

        let port = peri_pac!(PIN_B_PORT);
        let pin = PIN_B_NUM.load(Ordering::Relaxed) as usize;
        if port.latch().read().0 & (1 << pin) != 0 {
            port.pin_cnf(pin).modify(|w| w.set_sense(Sense::Disabled));
            port.latch().write(|w| w.0 = 1 << pin);
            PIN_B_EVENTS.fetch_add(1, Ordering::Relaxed);
        }
    }
}

fn arm_pin_b(sense: Sense) {
    let pin = PIN_B_NUM.load(Ordering::Relaxed) as usize;
    peri_pac!(PIN_B_PORT).pin_cnf(pin).modify(|w| w.set_sense(sense));
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut p = embassy_nrf::init(Default::default());
    let irqs = irqs!(PIN_A_MIXED);

    PIN_B_NUM.store(peri!(p, PIN_B).pin(), Ordering::Relaxed);
    let mut b = Flex::new(peri!(p, PIN_B).reborrow());
    b.set_low();
    b.set_as_input_output(Pull::None, OutputDrive::Standard);

    // Async pin and user-handled pin, on the same edges.
    {
        let mut a = Input::new_async(peri!(p, PIN_A).reborrow(), irqs, Pull::None);
        Timer::after_millis(1).await;
        assert!(a.is_low());

        for i in 1..=4 {
            let rising = i % 2 == 1;
            arm_pin_b(if rising { Sense::High } else { Sense::Low });
            let wait = async {
                if rising {
                    a.wait_for_high().await
                } else {
                    a.wait_for_low().await
                }
            };
            let drive = async {
                Timer::after_millis(10).await;
                b.set_level(Level::from(rising));
            };
            unwrap!(with_timeout(Duration::from_millis(100), join(wait, drive)).await);
            Timer::after_millis(1).await;
            assert_eq!(PIN_B_EVENTS.load(Ordering::Relaxed), i);
        }
        info!("async Input + user handler OK");

        // User-handled pin alone, while the async pin isn't waiting.
        arm_pin_b(Sense::High);
        b.set_high();
        Timer::after_millis(1).await;
        assert_eq!(PIN_B_EVENTS.load(Ordering::Relaxed), 5);
        b.set_low();
        Timer::after_millis(1).await;
        info!("user handler alone OK");
    }

    // Async GPIOTE channel and user-handled pin, on the same edge.
    {
        let mut ch = InputChannel::new_async(
            peri!(p, GPIOTE_CH).reborrow(),
            peri!(p, PIN_A).reborrow(),
            irqs,
            Pull::None,
            InputChannelPolarity::LoToHi,
        );
        arm_pin_b(Sense::High);
        let wait = ch.wait();
        let drive = async {
            Timer::after_millis(10).await;
            b.set_high();
        };
        unwrap!(with_timeout(Duration::from_millis(100), join(wait, drive)).await);
        Timer::after_millis(1).await;
        assert_eq!(PIN_B_EVENTS.load(Ordering::Relaxed), 6);
        info!("async InputChannel + user handler OK");
    }

    info!("Test OK");
    cortex_m::asm::bkpt();
}

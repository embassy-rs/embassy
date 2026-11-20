//! Async GPIO and a manual interrupt handler on the same GPIOTE interrupt.
//!
//! Button 1 is an async `Input`, serviced by `gpio::InterruptHandler`. Button 2 is a blocking
//! `Input` whose PORT event is serviced by `Button2Handler`, which counts presses in the interrupt.
#![no_std]
#![no_main]

use core::sync::atomic::{AtomicU32, Ordering};

use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_nrf::gpio::{self, Input, Pull};
use embassy_nrf::interrupt::typelevel::{self, Handler, Interrupt};
use embassy_nrf::pac::gpio::vals::{Detectmode, Sense};
use embassy_nrf::{bind_interrupts, pac, peripherals};
use panic_probe as _;

bind_interrupts!(struct Irqs {
    GPIOTE => gpio::InterruptHandler<peripherals::P0_11>, Button2Handler;
});

const BUTTON2_PIN: usize = 12;
static BUTTON2_PRESSES: AtomicU32 = AtomicU32::new(0);

struct Button2Handler;

impl Handler<typelevel::GPIOTE> for Button2Handler {
    unsafe fn on_interrupt() {
        // Clear the event first, then only this pin's LATCH bit. Other pins share the PORT event and
        // LATCH register, and a LATCH bit left set blocks new PORT events for the whole port.
        pac::GPIOTE.events_port(0).write_value(0);

        let mask = 1 << BUTTON2_PIN;
        if pac::P0.latch().read().0 & mask == 0 {
            return;
        }

        // Sense the opposite level next, to see both press and release. This also brings the pin's
        // DETECT signal low, which a LATCH bit needs to be cleared.
        let cnf = pac::P0.pin_cnf(BUTTON2_PIN);
        if cnf.read().sense() == Sense::Low {
            BUTTON2_PRESSES.fetch_add(1, Ordering::Relaxed);
            cnf.modify(|w| w.set_sense(Sense::High));
        } else {
            cnf.modify(|w| w.set_sense(Sense::Low));
        }
        pac::P0.latch().write(|w| w.0 = mask);
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    info!("Starting!");

    let mut button1 = Input::new_async(p.P0_11, Irqs, Pull::Up);
    let _button2 = Input::new(p.P0_12, Pull::Up);

    // Set up button 2 for the manual handler: latched detection, SENSE on the pin, the PORT
    // interrupt, and the interrupt in the NVIC. The async `Input` above already did all of these
    // except SENSE, but a program without async pins on this port must do them itself.
    pac::P0.detectmode().write(|w| w.set_detectmode(Detectmode::Ldetect));
    pac::P0.pin_cnf(BUTTON2_PIN).modify(|w| w.set_sense(Sense::Low));
    pac::GPIOTE.intenset(0).write(|w| w.set_port(true));
    unsafe { typelevel::GPIOTE::enable() };

    loop {
        button1.wait_for_low().await;
        info!(
            "Button 1 pressed. Button 2 was pressed {} times.",
            BUTTON2_PRESSES.load(Ordering::Relaxed)
        );
        button1.wait_for_high().await;
    }
}

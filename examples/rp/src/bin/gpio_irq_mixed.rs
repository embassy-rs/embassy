//! This example shows an async pin and a pin serviced by your own interrupt handler sharing the
//! `IO_IRQ_BANK0` interrupt.
//!
//! PIN_16 is an async input: the task waits for it to go low. PIN_17 is a blocking input whose
//! falling edges are counted by `CountHandler`. Connect buttons to ground on both pins.

#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_rp::gpio::{self, Input, Pull};
use embassy_rp::interrupt::typelevel::{Handler, IO_IRQ_BANK0};
use embassy_rp::interrupt::{self, InterruptExt};
use embassy_rp::peripherals::PIN_16;
use embassy_rp::{bind_interrupts, pac};
use embassy_time::Timer;
use panic_probe as _;
use portable_atomic::{AtomicU32, Ordering};

const COUNTED_PIN: usize = 17;

static EDGES: AtomicU32 = AtomicU32::new(0);

/// Counts falling edges on `COUNTED_PIN`.
struct CountHandler;

impl Handler<IO_IRQ_BANK0> for CountHandler {
    unsafe fn on_interrupt() {
        // This only runs on core 0, so look at core 0's interrupt registers.
        let ints = pac::IO_BANK0.int_proc(0).ints(COUNTED_PIN / 8).read();
        if ints.edge_low(COUNTED_PIN % 8) {
            // Edge events are latched in INTR; writing 1 clears this pin's event only.
            pac::IO_BANK0
                .intr(COUNTED_PIN / 8)
                .write(|w| w.set_edge_low(COUNTED_PIN % 8, true));
            EDGES.fetch_add(1, Ordering::Relaxed);
        }
    }
}

bind_interrupts!(struct Irqs {
    IO_IRQ_BANK0 => gpio::InterruptHandler<PIN_16>, CountHandler;
});

#[embassy_executor::main(executor = "embassy_rp::executor::Executor", entry = "cortex_m_rt::entry")]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let mut button = Input::new_async(p.PIN_16, Irqs, Pull::Up);

    // The HAL doesn't touch interrupts of blocking pins: enable the edge interrupt for PIN_17 on
    // core 0, and the interrupt in the NVIC.
    let _counted = Input::new(p.PIN_17, Pull::Up);
    pac::IO_BANK0
        .intr(COUNTED_PIN / 8)
        .write(|w| w.set_edge_low(COUNTED_PIN % 8, true));
    // INTE is shared with PIN_16, whose handler clears its bits, so keep the handler from running
    // in the middle of this read-modify-write.
    critical_section::with(|_| {
        pac::IO_BANK0
            .int_proc(0)
            .inte(COUNTED_PIN / 8)
            .modify(|w| w.set_edge_low(COUNTED_PIN % 8, true));
    });
    unsafe { interrupt::IO_IRQ_BANK0.enable() };

    loop {
        button.wait_for_low().await;
        info!(
            "button pressed, PIN_17 saw {} falling edges",
            EDGES.load(Ordering::Relaxed)
        );
        Timer::after_millis(200).await;
        button.wait_for_high().await;
    }
}

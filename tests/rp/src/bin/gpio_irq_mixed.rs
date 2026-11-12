//! An async pin and a pin serviced by a user interrupt handler, sharing `IO_IRQ_BANK0` and the same
//! INTE register. The HAL handler must not take the user pin's events, and the user's edges must not
//! complete the async wait.
#![no_std]
#![no_main]
#[cfg(feature = "rp2040")]
teleprobe_meta::target!(b"rpi-pico");
#[cfg(feature = "rp235xb")]
teleprobe_meta::target!(b"pimoroni-pico-plus-2");

use defmt::{assert, assert_eq, *};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_rp::gpio::{self, Input, Level, Output, Pull};
use embassy_rp::interrupt::typelevel::{Handler, IO_IRQ_BANK0};
use embassy_rp::peripherals::PIN_1;
use embassy_rp::{bind_interrupts, pac};
use embassy_time::{Instant, Timer};
use panic_probe as _;
use portable_atomic::{AtomicU32, Ordering};

// On the CI device, PIN_0-PIN_1 and PIN_3-PIN_4 are connected. PIN_1 and PIN_4 share INTE0.
const USER_PIN: usize = 4;

static USER_EDGES: AtomicU32 = AtomicU32::new(0);

struct UserHandler;

impl Handler<IO_IRQ_BANK0> for UserHandler {
    unsafe fn on_interrupt() {
        let ints = pac::IO_BANK0.int_proc(0).ints(USER_PIN / 8).read();
        if ints.edge_high(USER_PIN % 8) {
            pac::IO_BANK0
                .intr(USER_PIN / 8)
                .write(|w| w.set_edge_high(USER_PIN % 8, true));
            USER_EDGES.fetch_add(1, Ordering::Relaxed);
        }
    }
}

bind_interrupts!(struct Irqs {
    IO_IRQ_BANK0 => gpio::InterruptHandler<PIN_1>, UserHandler;
});

async fn pulse(out: &mut Output<'_>) {
    out.set_high();
    Timer::after_millis(1).await;
    out.set_low();
    Timer::after_millis(1).await;
}

fn user_inte_set() -> bool {
    pac::IO_BANK0
        .int_proc(0)
        .inte(USER_PIN / 8)
        .read()
        .edge_high(USER_PIN % 8)
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    info!("embassy-rp gpio_irq_mixed test");

    let mut async_out = Output::new(p.PIN_0, Level::Low);
    let mut async_in = Input::new_async(p.PIN_1, Irqs, Pull::None);
    let mut user_out = Output::new(p.PIN_3, Level::Low);
    let _user_in = Input::new(p.PIN_4, Pull::None);

    // Arm the user pin on core 0. `Input::new_async` above already enabled the interrupt in the NVIC.
    pac::IO_BANK0
        .intr(USER_PIN / 8)
        .write(|w| w.set_edge_high(USER_PIN % 8, true));
    cortex_m::interrupt::free(|_| {
        pac::IO_BANK0
            .int_proc(0)
            .inte(USER_PIN / 8)
            .modify(|w| w.set_edge_high(USER_PIN % 8, true));
    });

    info!("user edges, no async wait");
    for _ in 0..3 {
        pulse(&mut user_out).await;
    }
    assert_eq!(USER_EDGES.load(Ordering::Relaxed), 3);
    assert!(user_inte_set());

    info!("user edges during an async wait");
    let wait = async {
        async_in.wait_for_rising_edge().await;
        Instant::now()
    };
    let stimulus = async {
        for _ in 0..3 {
            pulse(&mut user_out).await;
        }
        assert_eq!(USER_EDGES.load(Ordering::Relaxed), 6);
        assert!(user_inte_set());

        Timer::after_millis(10).await;
        let raised = Instant::now();
        async_out.set_high();
        raised
    };
    let (woken, raised) = join(wait, stimulus).await;
    assert!(woken >= raised, "async wait completed before its pin rose");

    info!("user edges after the async wait");
    for _ in 0..3 {
        pulse(&mut user_out).await;
    }
    assert_eq!(USER_EDGES.load(Ordering::Relaxed), 9);
    assert!(user_inte_set());

    info!("Test OK");
    cortex_m::asm::bkpt();
}

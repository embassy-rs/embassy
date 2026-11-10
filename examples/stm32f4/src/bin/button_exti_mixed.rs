//! Async and manually handled EXTI lines sharing one interrupt vector.
//!
//! The USER button (PC13, EXTI13) is awaited asynchronously. A second button on PB10 (EXTI10,
//! wired to GND, internal pull-up) is handled by our own interrupt handler. Both lines are on the
//! `EXTI15_10` vector, which is bound to both handlers. Each handler only services its own line.
#![no_std]
#![no_main]

use core::cell::RefCell;
use core::sync::atomic::{AtomicU32, Ordering};

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::exti::{self, ExtiInput, TriggerEdge};
use embassy_stm32::gpio::Pull;
use embassy_stm32::interrupt::typelevel::{EXTI15_10, Handler, Interrupt};
use embassy_stm32::{bind_interrupts, peripherals};
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use panic_probe as _;

static BUTTON2: Mutex<CriticalSectionRawMutex, RefCell<Option<ExtiInput<'static>>>> = Mutex::new(RefCell::new(None));
static BUTTON2_PRESSES: AtomicU32 = AtomicU32::new(0);

struct Button2Handler;

impl Handler<EXTI15_10> for Button2Handler {
    unsafe fn on_interrupt() {
        BUTTON2.lock(|b| {
            if let Some(b) = b.borrow_mut().as_mut()
                && b.is_pending()
            {
                b.clear_pending();
                BUTTON2_PRESSES.fetch_add(1, Ordering::Relaxed);
            }
        });
    }
}

bind_interrupts!(struct Irqs {
    EXTI15_10 => exti::InterruptHandler<peripherals::EXTI13>, Button2Handler;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    // Create it inside the lock so the handler can't run before it's stored.
    BUTTON2.lock(|b| b.replace(Some(ExtiInput::new(p.PB10, p.EXTI10, Pull::Up, TriggerEdge::Falling))));
    // The HAL only enables EXTI vectors for `ExtiInput<Async>`. Here the async USER button below
    // would enable EXTI15_10 too, but a manually handled line must not rely on that.
    unsafe { EXTI15_10::enable() };

    let mut button = ExtiInput::new_async(p.PC13, p.EXTI13, Pull::Down, Irqs);

    info!("Press the USER button...");

    loop {
        button.wait_for_rising_edge().await;
        info!(
            "USER button pressed! PB10 button pressed {} times so far",
            BUTTON2_PRESSES.load(Ordering::Relaxed)
        );
        button.wait_for_falling_edge().await;
    }
}

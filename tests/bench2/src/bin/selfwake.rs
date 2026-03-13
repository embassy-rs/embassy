#![no_std]
#![no_main]

#[cfg(feature = "nrf52832")]
teleprobe_meta::target!(b"nrf52832-dk");
#[cfg(feature = "rp2040")]
teleprobe_meta::target!(b"w5100s-evb-pico");

use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

use cortex_m_rt::{entry, exception};
use defmt::{info, unwrap};
use embassy_executor::Executor;
use embassy_executor::raw::TaskStorage;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

static mut COUNTER: u32 = 0;

#[exception]
fn SysTick() -> ! {
    let c = unsafe { COUNTER };
    info!("Test OK, count={=u32}, cycles={=u32}", c, 0x00ffffff * 100 / c);
    cortex_m::asm::bkpt();
    loop {}
}

struct Task1 {}
impl Future for Task1 {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        unsafe { COUNTER += 1 };
        cx.waker().wake_by_ref();
        Poll::Pending
    }
}

static EXECUTOR: StaticCell<Executor> = StaticCell::new();
static TASK1: TaskStorage<Task1> = TaskStorage::new();

#[entry]
fn main() -> ! {
    info!("Hello World!");

    #[cfg(feature = "_nrf")]
    let _p = embassy_nrf::init(Default::default());
    #[cfg(feature = "_rp")]
    let _p = embassy_rp::init(Default::default());
    let executor = EXECUTOR.init(Executor::new());
    executor.run(|spawner| {
        unwrap!(spawner.spawn(TASK1.spawn(|| Task1 {})));

        let mut systick: cortex_m::peripheral::SYST = unsafe { core::mem::transmute(()) };
        systick.disable_counter();
        systick.set_clock_source(cortex_m::peripheral::syst::SystClkSource::Core);
        systick.set_reload(0x00ffffff);
        systick.enable_interrupt();
        systick.enable_counter();
    });
}

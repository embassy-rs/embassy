//! This example shows how to measure the idle time of the CPU when using the simple thread mode executor.
//! For this, we copied the thread mode executor (embassy-executor/src/arch/cortex_m.rs), and adjusted it to keep track of the idled duration in some static variable.
//!
//! Note that the idle time might be larger than the actual idle time since interrupt handlers run before the idle-duration-timer is stopped for each executor-loop.
//!
//! As an example, we have one task that idles for ~2.3 seconds, and then blocks for ~1 second.
//!
//! Sample output below.
//!
//!
//! ```not_rust
//! [AWAIT]
//! In the last 1.000311 sec, core idled for 0.999716 sec (0.9994052%).
//! In the last 0.999938 sec, core idled for 0.999056 sec (0.999118%).
//! [BUSY WAIT]
//! [AWAIT]
//! In the last 1.300312 sec, core idled for 0.298950 sec (0.22990637%).
//! In the last 0.699701 sec, core idled for 0.698874 sec (0.99881804%).
//! In the last 0.999974 sec, core idled for 0.999208 sec (0.99923396%).
//! [BUSY WAIT]
//! [AWAIT]
//! In the last 1.600831 sec, core idled for 0.599517 sec (0.3745036%).
//! In the last 0.399187 sec, core idled for 0.398391 sec (0.9980059%).
//! In the last 0.999981 sec, core idled for 0.999209 sec (0.999228%).
//! [BUSY WAIT]
//! [AWAIT]
//! In the last 1.901323 sec, core idled for 0.900035 sec (0.47337303%).
//! In the last 0.098696 sec, core idled for 0.097889 sec (0.9918234%).
//! In the last 0.999983 sec, core idled for 0.999222 sec (0.99923897%).
//! In the last 0.999991 sec, core idled for 0.999239 sec (0.99924797%).
//! [BUSY WAIT]
//! [AWAIT]
//! In the last 1.201868 sec, core idled for 0.200560 sec (0.16687357%).
//! In the last 0.798152 sec, core idled for 0.797356 sec (0.9990027%).
//! In the last 0.999993 sec, core idled for 0.999228 sec (0.999235%).
//! [BUSY WAIT]
//! [AWAIT]
//! In the last 1.502379 sec, core idled for 0.501061 sec (0.3335117%).
//!
//! ````

#![no_std]
#![no_main]

use core::cell::Cell;
use core::ops::Sub;

use defmt::*;
use embassy_time::{Duration, Instant, Ticker, Timer};
use embedded_hal_1::delay::DelayNs;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

mod thread_idle_logging {
    pub(super) const THREAD_PENDER: usize = usize::MAX;

    use core::arch::asm;
    use core::cell::Cell;
    use core::marker::PhantomData;

    use embassy_executor::{raw, Spawner};
    use embassy_time::{Duration, Instant};

    /// Thread mode executor, using WFE/SEV.
    ///
    /// copied from embassy-executor/src/arch/cortex_m.rs
    pub struct Executor {
        inner: raw::Executor,
        total_idled_duration: &'static Cell<Duration>,
        not_send: PhantomData<*mut ()>,
    }

    impl Executor {
        /// Create a new Executor.
        /// Needs a reference to the cell storing the accumulated idle duration
        pub fn new(total_idled_duration: &'static Cell<Duration>) -> Self {
            Self {
                inner: raw::Executor::new(THREAD_PENDER as *mut ()),
                not_send: PhantomData,
                total_idled_duration,
            }
        }

        /// Run the executor.
        /// Copied from embassy-executor/src/arch/cortex_m.rs, but adjusted to accumulate the elapsed duation of the idle.
        pub fn run(&'static mut self, init: impl FnOnce(Spawner)) -> ! {
            init(self.inner.spawner());
            loop {
                unsafe {
                    self.inner.poll();
                    let now_pre_sleep = Instant::now();
                    asm!("wfe");
                    self.total_idled_duration
                        .replace(self.total_idled_duration.get() + (Instant::now().duration_since(now_pre_sleep)));
                };
            }
        }
    }
}

use thread_idle_logging::Executor;

#[cortex_m_rt::entry]
fn main() -> ! {
    embassy_rp::init(Default::default());

    static EXECUTOR: StaticCell<Executor> = StaticCell::new();
    static TOTAL_IDLE_DURATION: StaticCell<Cell<Duration>> = StaticCell::new();

    let total_idle_duration = TOTAL_IDLE_DURATION.init(Cell::new(Duration::from_ticks(0)));

    let executor = EXECUTOR.init(Executor::new(total_idle_duration));
    executor.run(|spawner| {
        unwrap!(spawner.spawn(cpu_info(total_idle_duration)));
        unwrap!(spawner.spawn(task()));
    });
}

#[embassy_executor::task]
async fn task() {
    loop {
        println!("[AWAIT]");
        Timer::after_millis(2300).await;
        println!("[BUSY WAIT]");
        embassy_time::Delay.delay_ms(1000); // ~1 second
    }
}

#[embassy_executor::task]
async fn cpu_info(idle_duration: &'static Cell<Duration>) {
    let mut ticker = Ticker::every(Duration::from_secs(1));
    let mut now_pretick = Instant::now();
    let mut total_idle_pretick = idle_duration.get();
    loop {
        ticker.next().await;
        let now_posttick = Instant::now();
        let total_idle_posttick = idle_duration.get();
        let delta_t = now_posttick.duration_since(now_pretick);
        let delta_idle_dur = total_idle_posttick.sub(total_idle_pretick);
        let idle_percentage = delta_idle_dur.as_ticks() as f32 / delta_t.as_ticks() as f32;
        println!(
            "In the last {=u64:us} sec, core idled for {=u64:us} sec ({=f32:09}%).",
            delta_t.as_ticks(),
            delta_idle_dur.as_ticks(),
            idle_percentage
        );

        now_pretick = now_posttick;
        total_idle_pretick = total_idle_posttick;
    }
}

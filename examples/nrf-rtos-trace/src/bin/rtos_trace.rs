#![no_std]
#![no_main]

use core::future::poll_fn;
use core::task::Poll;

use embassy_executor::Spawner;
use embassy_time::{Instant, Timer};
#[cfg(feature = "log")]
use log::*;
use panic_probe as _;
// N.B. systemview_target cannot be used at the same time as defmt_rtt.
use rtos_trace;
use systemview_target::SystemView;

static LOGGER: systemview_target::SystemView = systemview_target::SystemView::new();
rtos_trace::global_trace! {SystemView}

struct TraceInfo();

impl rtos_trace::RtosTraceApplicationCallbacks for TraceInfo {
    fn system_description() {}
    fn sysclock() -> u32 {
        64000000
    }
}
rtos_trace::global_application_callbacks! {TraceInfo}

#[embassy_executor::task]
async fn run1() {
    loop {
        #[cfg(feature = "log")]
        info!("DING DONG");
        #[cfg(not(feature = "log"))]
        rtos_trace::trace::marker(13);
        Timer::after_ticks(16000).await;
    }
}

#[embassy_executor::task]
async fn run2() {
    loop {
        Timer::at(Instant::from_ticks(0)).await;
    }
}

#[embassy_executor::task]
async fn run3() {
    poll_fn(|cx| {
        cx.waker().wake_by_ref();
        Poll::<()>::Pending
    })
    .await;
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let _p = embassy_nrf::init(Default::default());
    LOGGER.init();
    #[cfg(feature = "log")]
    {
        ::log::set_logger(&LOGGER).ok();
        ::log::set_max_level(::log::LevelFilter::Trace);
    }

    spawner.spawn(run1()).unwrap();
    spawner.spawn(run2()).unwrap();
    spawner.spawn(run3()).unwrap();
}

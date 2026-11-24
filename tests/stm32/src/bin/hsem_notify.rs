// required-features: hsem

//! HSEM notify/listen regression test for issue #7609.
//!
//! `blocking_listen` and `listen` used to clear the notification flag *before*
//! waiting for it, so a notification sent before the call was lost and the waiter
//! blocked forever. The dual-core init handshake deadlocks on this: `init_secondary`
//! waits on a notification that `init_primary` may have sent long before, e.g. when
//! the secondary core is started or restarted under a debugger while the primary
//! core keeps running.
//!
//! This exercises the same `blocking_listen`/`is_notified` primitives that
//! `init_secondary` uses. The handshake itself across both cores is covered by
//! the `hsem_secondary` test; `init_primary` is covered by `common::init()` on
//! this board.
#![no_std]
#![no_main]

#[path = "../common.rs"]
mod common;

use common::*;
use embassy_executor::Spawner;
use embassy_stm32::bind_interrupts;
use embassy_stm32::hsem::{HardwareSemaphore, HardwareSemaphoreChannel, HardwareSemaphoreInterruptHandler};
use embassy_stm32::peripherals::HSEM;
use embassy_time::{Duration, Timer, with_timeout};

teleprobe_meta::timeout!(30);

#[cfg(feature = "stm32h755zi")]
bind_interrupts!(struct Irqs {
    HSEM1 => HardwareSemaphoreInterruptHandler<HSEM>;
});

#[cfg(not(feature = "stm32h755zi"))]
bind_interrupts!(struct Irqs {
    HSEM => HardwareSemaphoreInterruptHandler<HSEM>;
});

#[embassy_executor::task]
async fn notify_later(mut ch: HardwareSemaphoreChannel<'static, HSEM>) {
    Timer::after_millis(100).await;
    info!("notifier: notifying");
    ch.blocking_notify();
}

#[embassy_executor::main(executor = "common::Executor", entry = "cortex_m_rt::entry")]
async fn main(spawner: Spawner) {
    let p: embassy_stm32::Peripherals = init();
    let hsem = HardwareSemaphore::new(p.HSEM, Irqs);

    // Channel 5 is not used by the low-power CPU2 coordination channels (WB uses
    // 3/4, WL uses 3). `split` returns channels with a caller-chosen lifetime, so
    // a copy can be moved into the notifier task.
    let mut ch: HardwareSemaphoreChannel<'static, HSEM> =
        hsem.split().into_iter().find(|ch| ch.channel() == 5).unwrap();
    ch.clear_notification();

    // `is_notified` is a level: observing it does not consume the notification.
    // The init handshake relies on this so the secondary core can be restarted
    // under a debugger without losing the notification of `init_primary`.
    info!("level: notify, observe twice, clear");
    ch.blocking_notify();
    defmt::assert!(ch.is_notified());
    defmt::assert!(ch.is_notified());
    ch.clear_notification();
    defmt::assert!(!ch.is_notified());

    // A notification sent *before* `listen()` must be consumed by it, not dropped.
    // (This also covers arming the notification interrupt while the status flag
    // is already set, which used to livelock the interrupt handler.)
    info!("async: notification sent before listen is not lost");
    ch.blocking_notify();
    if with_timeout(Duration::from_secs(5), ch.listen()).await.is_err() {
        defmt::panic!("listen() did not consume the pending notification");
    }
    defmt::assert!(!ch.is_notified());

    // A notification sent while `listen()` is waiting must wake it (interrupt
    // handler path).
    info!("async: notification sent while listen is waiting");
    let notifier = ch;
    spawner.spawn(notify_later(notifier).unwrap());
    if with_timeout(Duration::from_secs(5), ch.listen()).await.is_err() {
        defmt::panic!("listen() was not woken by a later notification");
    }
    defmt::assert!(!ch.is_notified());

    // Same for the blocking variant that `init_secondary` waits on. On regression
    // this hangs and the teleprobe job times out.
    info!("blocking: notification sent before blocking_listen is not lost");
    ch.blocking_notify();
    defmt::assert!(ch.is_notified());
    ch.blocking_listen();
    defmt::assert!(!ch.is_notified());

    info!("Test OK");
    cortex_m::asm::bkpt();
}

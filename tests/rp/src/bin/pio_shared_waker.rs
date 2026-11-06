#![no_std]
#![no_main]
#[cfg(feature = "rp2040")]
teleprobe_meta::target!(b"rpi-pico");
#[cfg(feature = "rp235xb")]
teleprobe_meta::target!(b"pimoroni-pico-plus-2");

// Each PIO block must have its own wakers. PIO0 raises its IRQ flag 0, and the main
// task waits for it. A task waits on PIO1's own IRQ flag 0, which never fires, so
// after its first poll nothing should ever poll it again.

use core::future::{Future, poll_fn};
use core::pin::pin;

use defmt::{assert_eq, info, unwrap};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::{PIO0, PIO1};
use embassy_rp::pio::program::pio_asm;
use embassy_rp::pio::{Config, InterruptHandler, Irq, Pio};
use embassy_time::{Duration, with_timeout};
use panic_probe as _;
use portable_atomic::{AtomicU32, Ordering};

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
    PIO1_IRQ_0 => InterruptHandler<PIO1>;
});

/// Polls of the PIO1 future.
static PIO1_POLLS: AtomicU32 = AtomicU32::new(0);

/// Awaits `fut`, counting each time it is polled.
async fn count_polls(fut: impl Future) {
    let mut fut = pin!(fut);
    poll_fn(|cx| {
        PIO1_POLLS.fetch_add(1, Ordering::Relaxed);
        fut.as_mut().poll(cx).map(|_| ())
    })
    .await
}

#[embassy_executor::task]
async fn pio1_irq(mut irq: Irq<'static, PIO1, 0>) {
    count_polls(irq.wait()).await;
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let Pio {
        mut common,
        mut irq0,
        mut sm0,
        ..
    } = Pio::new(p.PIO0, Irqs);
    let pio1 = Pio::new(p.PIO1, Irqs);
    spawner.spawn(unwrap!(pio1_irq(pio1.irq0)));

    // Raises IRQ flag 0 and waits for it to be cleared, then repeats about 1026 SM
    // cycles later: about 8 ms on RP2040 (125 MHz / 1000) and 7 ms on RP235x (150 MHz / 1000).
    let prg = pio_asm!(
        ".wrap_target",
        "set x, 31",
        "lp:",
        "jmp x-- lp [31]",
        "irq wait 0",
        ".wrap",
    );
    let mut cfg = Config::default();
    cfg.use_program(&common.load_program(&prg.program), &[]);
    cfg.clock_divider_x256 = 1000 << 8;
    sm0.set_config(&cfg);
    sm0.set_enable(true);

    // The main task waits on PIO0's IRQ flag 0, the same slot index as the PIO1 task.
    let pio0_events = async {
        for _ in 0..100 {
            irq0.wait().await;
        }
    };
    unwrap!(with_timeout(Duration::from_secs(5), pio0_events).await);

    // One poll to start waiting, and none after.
    assert_eq!(PIO1_POLLS.load(Ordering::Relaxed), 1, "PIO1 task woken by PIO0");

    info!("Test OK");
    cortex_m::asm::bkpt();
}

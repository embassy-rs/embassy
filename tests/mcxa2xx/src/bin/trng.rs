#![no_std]
#![no_main]

teleprobe_meta::target!(b"frdm-mcx-a266");

use core::future::Future;
use core::pin::pin;
use core::task::{Context, Poll, Waker};

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_mcxa as hal;
use embassy_time::{Duration, Timer, with_timeout};
use hal::bind_interrupts;
use hal::config::Config;
use hal::peripherals::TRNG0;
use hal::trng::{self, InterruptHandler, Trng};
use panic_probe as _;
use rand_core::Rng;
use rand_core::block::Generator;

bind_interrupts!(
    struct Irqs {
        TRNG0 => InterruptHandler<TRNG0>;
    }
);

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let config = Config::default();
    let mut p = hal::init(config);

    // Note: this test might fail once every ~2^28 or so.

    // Blocking
    let config = trng::Config::default();
    let mut trng = Trng::new_blocking(p.TRNG0.reborrow(), config);

    let rand1 = trng.blocking_next_u32();
    let rand2 = trng.blocking_next_u32();
    assert_ne!(rand1, rand2);

    let rand1 = trng.blocking_next_u64();
    let rand2 = trng.blocking_next_u64();
    assert_ne!(rand1, rand2);

    let rand1 = trng.next_u32();
    let rand2 = trng.next_u32();
    assert_ne!(rand1, rand2);

    let rand1 = trng.next_u64();
    let rand2 = trng.next_u64();
    assert_ne!(rand1, rand2);
    drop(trng);

    // Async
    let mut trng = Trng::new_async(p.TRNG0.reborrow(), Irqs, config);

    let rand1 = trng.async_next_u32().await.unwrap();
    let rand2 = trng.async_next_u32().await.unwrap();
    assert_ne!(rand1, rand2);

    let rand1 = trng.async_next_u64().await.unwrap();
    let rand2 = trng.async_next_u64().await.unwrap();
    assert_ne!(rand1, rand2);

    let mut rand1 = [0u8; 10];
    let mut rand2 = [0u8; 10];
    trng.async_fill_bytes(&mut rand1).await.unwrap();
    trng.async_fill_bytes(&mut rand2).await.unwrap();

    let mut rand1 = [0u32; 8];
    let mut rand2 = [0u32; 8];
    trng.async_next_block(&mut rand1).await.unwrap();
    trng.async_next_block(&mut rand2).await.unwrap();
    assert_ne!(rand1, rand2);

    exercise_mixed_reads(&mut trng).await;
    drop(trng);

    defmt::info!("Test OK");
    cortex_m::asm::bkpt();
}

async fn exercise_mixed_reads(trng: &mut Trng<'_, trng::Async>) {
    for kind in 0..5 {
        let mut block = [0u32; 8];
        trng.async_next_block(&mut block).await.unwrap();

        // Let the IRQ report entropy before a blocking call consumes it.
        with_timeout(Duration::from_secs(2), async {
            while !hal::pac::TRNG0.mctl().read().ent_val() {
                Timer::after_millis(1).await;
            }
        })
        .await
        .unwrap();

        match kind {
            0 => {
                let _ = trng.next_u32();
            }
            1 => {
                let _ = trng.next_u64();
            }
            2 => trng.fill_bytes(&mut [0u8; 33]),
            3 => trng.generate(&mut block),
            _ => trng.blocking_next_block(&mut block),
        }
        assert!(!hal::pac::TRNG0.mctl().read().ent_val());

        let receive = async {
            match kind {
                0 => {
                    trng.async_next_u32().await.unwrap();
                }
                1 => {
                    trng.async_next_u64().await.unwrap();
                }
                2 => trng.async_fill_bytes(&mut [0u8; 32]).await.unwrap(),
                _ => trng.async_next_block(&mut block).await.unwrap(),
            }
        };
        let mut receive = pin!(receive);
        let mut cx = Context::from_waker(Waker::noop());
        assert!(
            matches!(receive.as_mut().poll(&mut cx), Poll::Pending),
            "stale entropy IRQ completed an async read"
        );
        with_timeout(Duration::from_secs(2), receive).await.unwrap();
    }

    let mut bytes = [0u8; 33];
    with_timeout(Duration::from_secs(2), trng.async_fill_bytes(&mut bytes))
        .await
        .unwrap()
        .unwrap();
    assert!(bytes[..32].iter().any(|&byte| byte != 0));
    defmt::info!("Mixed blocking/async TRNG reads passed");
}

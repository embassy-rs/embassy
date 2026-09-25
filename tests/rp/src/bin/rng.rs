#![no_std]
#![no_main]
#[cfg(feature = "rp2040")]
teleprobe_meta::target!(b"rpi-pico");
#[cfg(feature = "rp235xb")]
teleprobe_meta::target!(b"pimoroni-pico-plus-2");

use defmt::{assert, assert_ne, *};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_rp::clocks::RoscRng;
use panic_probe as _;

#[cfg(feature = "rp235xb")]
embassy_rp::bind_interrupts!(struct Irqs {
    TRNG_IRQ => embassy_rp::trng::InterruptHandler<embassy_rp::peripherals::TRNG>;
});

/// Crude sanity check: the buffer must not be constant, and the number of set
/// bits must be roughly half. Catches "all zeros", "all ones" and stuck bits.
fn check_entropy(name: &str, buf: &[u8]) {
    let ones: u32 = buf.iter().map(|b| b.count_ones()).sum();
    let bits = (buf.len() * 8) as u32;
    info!("{}: {} bytes, {} of {} bits set", name, buf.len(), ones, bits);
    assert!(buf.iter().any(|&b| b != buf[0]), "{} output is constant", name);
    assert!(ones > bits * 40 / 100, "{} too few ones", name);
    assert!(ones < bits * 60 / 100, "{} too many ones", name);
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    #[allow(unused_variables)]
    let p = embassy_rp::init(Default::default());
    info!("Hello World!");

    // ROSC random bit, available on all chips.
    {
        let mut rng = RoscRng;
        let mut a = [0u8; 256];
        let mut b = [0u8; 256];
        rng.fill_bytes(&mut a);
        rng.fill_bytes(&mut b);
        check_entropy("rosc", &a);
        check_entropy("rosc", &b);
        assert_ne!(a, b);
        assert_ne!(rng.next_u32(), rng.next_u32());
        assert_ne!(rng.next_u64(), rng.next_u64());
    }

    #[cfg(feature = "rp235xb")]
    {
        use embassy_rp::trng::{Config, Trng};
        use embassy_time::{Duration, Instant, with_timeout};

        let mut trng = Trng::new(p.TRNG, Irqs, Config::default());

        // Blocking API. Loop many times: TRNG health-check failures are
        // statistical, so a single call passing proves little.
        let mut buf = [0u8; 48];
        let start = Instant::now();
        for _ in 0..200 {
            trng.blocking_fill_bytes(&mut buf);
        }
        check_entropy("trng blocking", &buf);
        let a = trng.blocking_next_u32();
        let b = trng.blocking_next_u32();
        assert_ne!(a, b);
        let a = trng.blocking_next_u64();
        let b = trng.blocking_next_u64();
        assert_ne!(a, b);
        info!("blocking: 200 x 48 bytes in {} ms", start.elapsed().as_millis());

        // Async API. While the future is pending the executor sleeps the
        // core, which is a different electrical environment for the ROSC
        // than the blocking busy-wait, so this must be exercised separately.
        let mut big = [0u8; 1024];
        let start = Instant::now();
        for i in 0..100 {
            with_timeout(Duration::from_secs(5), trng.fill_bytes(&mut big))
                .await
                .unwrap_or_else(|_| defmt::panic!("async fill {} timed out", i));
        }
        check_entropy("trng async", &big);
        info!("async: 100 x 1024 bytes in {} ms", start.elapsed().as_millis());
    }

    info!("Test OK");
    cortex_m::asm::bkpt();
}

// required-features: crypto
//! The hardware random number generator of the crypto accelerator, and its power management:
//! the accelerator stays powered for as long as any driver of it is alive.
#![no_std]
#![no_main]

#[path = "../common.rs"]
mod common;

use defmt::{assert, assert_ne, info};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_nrf::crypto::symmetric::{AesEcb, Direction, Symmetric};
use embassy_nrf::mode::Blocking;
use panic_probe as _;

type Rng<'d> = embassy_nrf::crypto::rng::Rng<'d, Blocking>;

/// Number of distinct byte values in a buffer.
fn distinct(buf: &[u8]) -> usize {
    let mut seen = [false; 256];
    for &b in buf {
        seen[b as usize] = true;
    }
    seen.iter().filter(|&&s| s).count()
}

fn check(rng: &mut Rng<'_>) {
    // Every length, including the ones that do not line up with the generator's word size.
    for len in [1usize, 2, 3, 4, 5, 7, 8, 15, 16, 17, 31, 33, 64, 255, 256] {
        let mut a = [0u8; 256];
        let mut b = [0u8; 256];
        rng.blocking_fill_bytes(&mut a[..len]);
        rng.blocking_fill_bytes(&mut b[..len]);
        // Two draws of the same length repeating exactly would take an absurd stroke of luck,
        // so this catches a generator that has stopped producing.
        if len >= 8 {
            assert_ne!(a[..len], b[..len]);
        }
        // Writing past the end of the requested slice would be a bug.
        assert!(a[len..].iter().all(|&x| x == 0));
    }

    // A rough spread check: a stuck or constant generator fails this by a mile.
    let mut buf = [0u8; 1024];
    rng.blocking_fill_bytes(&mut buf);
    let d = distinct(&buf);
    info!("distinct byte values in 1024: {}", d);
    assert!(d > 200);

    assert_ne!(rng.blocking_next_u32(), rng.blocking_next_u32());
    assert_ne!(rng.blocking_next_u64(), rng.blocking_next_u64());
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());

    info!("rng alone");
    let mut rng = Rng::new_blocking(peri!(p, RNG));
    check(&mut rng);

    // The accelerator is shared: powering it is reference counted, so a second driver must
    // not disturb the first, and dropping either must leave the other working.
    info!("rng alongside aes");
    let mut crypto = Symmetric::new_blocking(p.CRYPTO_SYMMETRIC);
    let key = [0x42u8; 16];
    let mut block = [0u8; 16];
    let mut out = [0u8; 16];
    let cipher = defmt::unwrap!(AesEcb::new(&key));

    let mut ctx = crypto.aes_start(cipher, Direction::Encrypt);
    defmt::unwrap!(crypto.aes_blocking_payload(&mut ctx, &block, &mut out, true));
    let first = out;
    check(&mut rng);

    // Same input, same key: the engine must still be configured the way it was.
    let mut ctx = crypto.aes_start(cipher, Direction::Encrypt);
    defmt::unwrap!(crypto.aes_blocking_payload(&mut ctx, &block, &mut out, true));
    assert!(out == first);

    info!("rng after dropping aes");
    drop(crypto);
    check(&mut rng);

    info!("aes after dropping rng");
    drop(rng);
    let mut crypto = Symmetric::new_blocking(unsafe { embassy_nrf::peripherals::CRYPTO_SYMMETRIC::steal() });
    block[0] = 0;
    let mut ctx = crypto.aes_start(cipher, Direction::Encrypt);
    defmt::unwrap!(crypto.aes_blocking_payload(&mut ctx, &block, &mut out, true));
    assert!(out == first);

    info!("Test OK");
    cortex_m::asm::bkpt();
}

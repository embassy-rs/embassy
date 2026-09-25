//! Blocking SHA-256 example using the LPC55S69 HASHCRYPT peripheral.
//!
//! This example demonstrates how to use the HASHCRYPT hardware accelerator
//! through the `CpuSha256` driver.
//!
//! The example covers:
//! - Creating a new hashing context.
//! - Hashing a complete message.
//! - Verifying the result against a known SHA-256 test vector.
//! - Reusing the same hardware context with `reset()`.
//! - Streaming data using multiple `update()` calls.
//! - Feeding arbitrary chunk sizes.
//!
//! The resulting SHA-256 digests are printed over the debug console.

#![no_std]
#![no_main]

use defmt::{info, unwrap};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_nxp::sha::CpuSha256;
use panic_probe as _;

/// SHA-256 test vector for the message "abc".
///
/// Expected digest:
/// ba7816bf8f01cfea414140de5dae2223
/// b00361a396177a9cb410ff61f20015ad
const SHA256_ABC: [u32; 8] = [
    0xba7816bf, 0x8f01cfea, 0x414140de, 0x5dae2223, 0xb00361a3, 0x96177a9c, 0xb410ff61, 0xf20015ad,
];

/// Print a SHA-256 digest returned by the HASHCRYPT peripheral.
///
/// The hardware returns the digest as eight 32-bit words.
fn print_digest(title: &str, digest: [u32; 8]) {
    info!("----------------------------------------");
    info!("{}", title);

    for word in digest {
        info!("{=u32:08x}", word);
    }

    info!("----------------------------------------");
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nxp::init(Default::default());
    info!("LPC55S69 HASHCRYPT SHA-256 example");

    //----------------------------------------------------------------------
    // Create a SHA-256 driver.
    //
    // The driver takes ownership of the HASHCRYPT peripheral and
    // initializes the hardware.
    //----------------------------------------------------------------------
    let mut sha = CpuSha256::new(p.HASHCRYPT);

    //----------------------------------------------------------------------
    // Example 1
    //
    // Hash a complete message using a single update() call.
    //----------------------------------------------------------------------
    info!("Example 1");

    unwrap!(sha.update(b"abc"));

    let digest = unwrap!(sha.finish());

    print_digest("SHA-256(\"abc\")", digest);

    // Verify the hardware result against the standard SHA-256 test vector.
    assert_eq!(digest, SHA256_ABC);
    info!("SHA-256(\"abc\") verification passed.");

    //----------------------------------------------------------------------
    // Prepare the driver for another independent hash operation.
    //
    // reset() clears the software state and starts a new HASHCRYPT session.
    //----------------------------------------------------------------------
    sha.reset();

    //----------------------------------------------------------------------
    // Example 2
    //
    // Hash the same message incrementally.
    //
    // Multiple update() calls are equivalent to one large update().
    //----------------------------------------------------------------------
    info!("Example 2");

    unwrap!(sha.update(b"a"));
    unwrap!(sha.update(b"b"));
    unwrap!(sha.update(b"c"));

    let digest = unwrap!(sha.finish());

    print_digest("SHA-256 streamed \"abc\"", digest);

    // Verify that streaming input produces the same digest.
    assert_eq!(digest, SHA256_ABC);
    info!("SHA-256 streamed \"abc\" verification passed.");

    //----------------------------------------------------------------------
    // Start another independent hash.
    //----------------------------------------------------------------------
    sha.reset();

    //----------------------------------------------------------------------
    // Example 3
    //
    // Feed a larger message in several chunks.
    //
    // The driver automatically buffers incomplete blocks and sends
    // complete 512-bit blocks to the HASHCRYPT peripheral.
    //----------------------------------------------------------------------
    info!("Example 3");

    unwrap!(sha.update(b"The quick "));
    unwrap!(sha.update(b"brown fox "));
    unwrap!(sha.update(b"jumps over "));
    unwrap!(sha.update(b"the lazy dog"));

    let digest = unwrap!(sha.finish());

    print_digest("SHA-256(\"The quick brown fox jumps over the lazy dog\")", digest);

    //----------------------------------------------------------------------
    // Reuse the hardware once more.
    //----------------------------------------------------------------------
    sha.reset();

    //----------------------------------------------------------------------
    // Example 4
    //
    // update() accepts data of any size.
    //
    // Here we intentionally split the input into small pieces to exercise
    // the driver's internal buffering logic.
    //----------------------------------------------------------------------
    info!("Example 4");

    let message = b"Embassy makes embedded Rust development enjoyable and ergonomic.";

    for chunk in message.chunks(5) {
        unwrap!(sha.update(chunk));
    }

    let digest = unwrap!(sha.finish());

    print_digest("SHA-256(chunked message)", digest);

    info!("All SHA-256 examples completed.");

    loop {
        cortex_m::asm::wfi();
    }
}

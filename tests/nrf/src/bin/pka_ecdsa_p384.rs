// required-features: crypto
#![no_std]
#![no_main]

//! ECDSA verification through the direct API, against the Wycheproof suite. See `pka_common.rs`.

#[path = "../common.rs"]
mod common;
#[path = "../pka_common.rs"]
mod pka_common;
#[cfg(feature = "cracen")]
#[path = "../ba414ep_ucode.rs"]
mod ucode;

use defmt_rtt as _;
use embassy_crypto_test::vectors::P384_ECDSA;
use embassy_nrf::crypto::pka::curve::NIST_P384;
use panic_probe as _;

teleprobe_meta::timeout!(120);

pka_test!(|_p, pka| pka_common::ecdsa(&mut pka, &NIST_P384, &P384_ECDSA));

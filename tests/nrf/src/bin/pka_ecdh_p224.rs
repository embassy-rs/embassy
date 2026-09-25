// required-features: crypto
#![no_std]
#![no_main]

//! ECDH through the direct API, against the Wycheproof suite. See `pka_common.rs`.

#[path = "../common.rs"]
mod common;
#[path = "../pka_common.rs"]
mod pka_common;
#[cfg(feature = "cracen")]
#[path = "../ba414ep_ucode.rs"]
mod ucode;

use defmt_rtt as _;
use embassy_crypto_test::vectors::P224_ECDH;
use embassy_nrf::crypto::pka::curve::NIST_P224;
use panic_probe as _;

teleprobe_meta::timeout!(120);

pka_test!(|_p, pka| pka_common::ecdh(&mut pka, &NIST_P224, &P224_ECDH));

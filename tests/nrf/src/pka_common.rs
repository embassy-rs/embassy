//! Shared code of the PKA tests. The Wycheproof suites are large and the tests are linked into
//! RAM, so each curve's ECDSA and ECDH suites run in a binary of their own.
#![macro_use]
#![allow(dead_code)]

use defmt::{assert, assert_eq, info};
use embassy_crypto_test::vectors::{Dh, EcKat, Ecdsa, Expected, Suite};
use embassy_nrf::crypto::pka::{Curve, Error, MAX_CURVE_LEN, Pka, Point, PointMut, Signature, curve};
use embassy_nrf::mode::Blocking;

pub type P = Pka<'static, Blocking>;

/// The curves the driver has parameters for, with their shared known answers.
pub const KAT: &[(&Curve, &Suite<EcKat>)] = {
    use embassy_crypto_test::vectors::*;
    &[
        (&curve::NIST_P192, &P192_KAT),
        (&curve::NIST_P224, &P224_KAT),
        (&curve::NIST_P256, &P256_KAT),
        (&curve::NIST_P384, &P384_KAT),
        (&curve::NIST_P521, &P521_KAT),
        (&curve::SECP256K1, &SECP256K1_KAT),
    ]
};

/// A test binary: creates the engine driver and runs `$body` with it as `$pka`, with the
/// remaining peripherals as `$p`.
macro_rules! pka_test {
    (|$p:ident, $pka:ident| $body:expr) => {
        #[embassy_executor::main]
        async fn main(_spawner: embassy_executor::Spawner) {
            let $p = embassy_nrf::init(Default::default());
            #[cfg(not(feature = "cracen"))]
            let mut $pka = embassy_nrf::crypto::pka::Pka::new_blocking($p.CRYPTO_PKA);
            #[cfg(feature = "cracen")]
            let mut $pka = embassy_nrf::crypto::pka::Pka::new_blocking($p.CRYPTO_PKA, &ucode::BA414EP_UCODE);
            $body;
            defmt::info!("Test OK");
            cortex_m::asm::bkpt();
        }
    };
}

/// Splits an uncompressed SEC1 point of `n`-byte coordinates into `(x, y)`, or `None` if it is
/// not one.
pub fn sec1(point: &[u8], n: usize) -> Option<(&[u8], &[u8])> {
    if point.len() != 2 * n + 1 || point[0] != 4 {
        return None;
    }
    Some((&point[1..1 + n], &point[1 + n..]))
}

/// Left-pads a big-endian integer of any length to `out.len()` bytes, or `None` if it does not
/// fit.
pub fn fixed<'a>(bytes: &[u8], out: &'a mut [u8]) -> Option<&'a [u8]> {
    let start = bytes.iter().position(|&b| b != 0).unwrap_or(bytes.len());
    let bytes = &bytes[start..];
    if bytes.len() > out.len() {
        return None;
    }
    out.fill(0);
    let start = out.len() - bytes.len();
    out[start..].copy_from_slice(bytes);
    Some(out)
}

/// ECDSA verification against a Wycheproof suite.
pub fn ecdsa(pka: &mut P, curve: &Curve, suite: &Suite<Ecdsa>) {
    info!("{}", suite.name);
    let n = curve.size();
    let (mut passed, mut skipped) = (0, 0);
    for v in suite.cases {
        let Some((qx, qy)) = sec1(v.public, n) else {
            defmt::panic!("tc {}: public key is not an uncompressed point", v.tc_id)
        };
        // The signature is `r || s` of whatever length the case has; only the right length is a
        // signature at all.
        if v.sig.len() != 2 * n {
            assert!(v.result != Expected::Valid, "tc {}", v.tc_id);
            skipped += 1;
            continue;
        }
        let (r, s) = v.sig.split_at(n);
        let result = pka.blocking_ecdsa_verify(curve, Point { x: qx, y: qy }, Signature { r, s }, v.digest);
        match (v.result, result) {
            (Expected::Valid, Ok(())) | (Expected::Invalid, Err(Error::InvalidSignature)) => {}
            (Expected::Acceptable, Ok(()) | Err(Error::InvalidSignature)) => {}
            (expected, result) => defmt::panic!("tc {}: expected {:?}, got {:?}", v.tc_id, expected, result),
        }
        passed += 1;
    }
    info!("{} passed, {} skipped", passed, skipped);
}

/// ECDH against a Wycheproof suite: valid agreements reach the shared secret, and points off the
/// curve or malformed scalars are rejected.
pub fn ecdh(pka: &mut P, curve: &Curve, suite: &Suite<Dh>) {
    info!("{}", suite.name);
    let n = curve.size();
    let mut x = [0u8; MAX_CURVE_LEN];
    let mut y = [0u8; MAX_CURVE_LEN];
    let mut d = [0u8; MAX_CURVE_LEN];
    let (mut passed, mut skipped) = (0, 0);
    for v in suite.cases {
        // Only uncompressed points are an encoding this API reads; the compressed ("acceptable")
        // and truncated cases are not.
        let Some((qx, qy)) = sec1(v.public, n) else {
            assert!(v.result != Expected::Valid, "tc {}", v.tc_id);
            skipped += 1;
            continue;
        };
        // The private key is an integer of whatever length.
        let Some(private) = fixed(v.private, &mut d[..n]) else {
            assert!(v.result != Expected::Valid, "tc {}", v.tc_id);
            skipped += 1;
            continue;
        };
        let out = PointMut {
            x: &mut x[..n],
            y: &mut y[..n],
        };
        match (
            v.result,
            pka.blocking_ecc_mul(curve, private, Point { x: qx, y: qy }, out),
        ) {
            (Expected::Valid, Ok(())) => assert_eq!(&x[..n], v.shared, "tc {}", v.tc_id),
            (Expected::Invalid, Err(Error::InvalidPoint | Error::InvalidScalar)) => {}
            (Expected::Acceptable, _) => {}
            (expected, result) => defmt::panic!("tc {}: expected {:?}, got {:?}", v.tc_id, expected, result),
        }
        passed += 1;
    }
    info!("{} passed, {} skipped", passed, skipped);
}

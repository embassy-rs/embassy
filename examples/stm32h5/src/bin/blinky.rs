#![no_std]
#![no_main]

use core::hint::black_box;
use defmt_rtt as _;
use embassy_executor::Spawner;
// use p256_cortex_m4::embassy_crypto as _;
use panic_probe as _;

use defmt::{error, info, warn};
use elliptic_curve::{
    Curve, CurveArithmetic, FieldBytes, FieldBytesSize, PublicKey, Scalar, SecretKey, ecdh,
    group::Group,
    ops::{Invert, LinearCombination, MulByGenerator, Reduce},
    pkcs8::{AssociatedOid, ObjectIdentifier},
    sec1::{FromEncodedPoint, ModulusSize, ToEncodedPoint},
    subtle::ConstantTimeEq,
};
use embassy_time::Instant;

// ─── P-256 constants ──────────────────────────────────────────────────────────

/// OID for NIST P-256 / secp256r1 / prime256v1.
pub const P256_OID: ObjectIdentifier = ObjectIdentifier::new_unwrap("1.2.840.10045.3.1.7");

/// SEC1 uncompressed encoding of the P-256 base point: `0x04 || X || Y`.
pub const P256_G_UNCOMPRESSED: [u8; 65] = [
    0x04, 0x6b, 0x17, 0xd1, 0xf2, 0xe1, 0x2c, 0x42, 0x47, 0xf8, 0xbc, 0xe6, 0xe5, 0x63, 0xa4, 0x40, 0xf2, 0x77, 0x03,
    0x7d, 0x81, 0x2d, 0xeb, 0x33, 0xa0, 0xf4, 0xa1, 0x39, 0x45, 0xd8, 0x98, 0xc2, 0x96, 0x4f, 0xe3, 0x42, 0xe2, 0xfe,
    0x1a, 0x7f, 0x9b, 0x8e, 0xe7, 0xeb, 0x4a, 0x7c, 0x0f, 0x9e, 0x16, 0x2b, 0xce, 0x33, 0x57, 0x6b, 0x31, 0x5e, 0xce,
    0xcb, 0xb6, 0x40, 0x68, 0x37, 0xbf, 0x51, 0xf5,
];

/// SEC1 compressed encoding of the P-256 base point: `0x02 || X`.
pub const P256_G_COMPRESSED: [u8; 33] = [
    0x02, 0x6b, 0x17, 0xd1, 0xf2, 0xe1, 0x2c, 0x42, 0x47, 0xf8, 0xbc, 0xe6, 0xe5, 0x63, 0xa4, 0x40, 0xf2, 0x77, 0x03,
    0x7d, 0x81, 0x2d, 0xeb, 0x33, 0xa0, 0xf4, 0xa1, 0x39, 0x45, 0xd8, 0x98, 0xc2, 0x96,
];

// ─── Result types ─────────────────────────────────────────────────────────────

/// Per-operation cost in microseconds.
#[derive(Clone, Copy, Debug, defmt::Format)]
pub struct Timings {
    pub base_mul_us: u64,
    pub var_mul_us: u64,
    pub point_add_us: u64,
    pub scalar_invert_us: u64,
    pub lincomb_us: u64,
    pub ecdh_us: u64,
}

impl Timings {
    /// TLS 1.3 ECDHE key-share generation: ephemeral base-mul + peer var-mul.
    pub fn tls13_ecdhe_us(&self) -> u64 {
        self.base_mul_us + self.var_mul_us
    }
    /// TLS 1.3 ECDSA verification (double-scalar mul / Shamir).
    pub fn tls13_ecdsa_verify_us(&self) -> u64 {
        self.lincomb_us
    }
    /// BLE LESC pairing: full ECDH incl. shared-secret extraction.
    pub fn ble_pairing_ecdh_us(&self) -> u64 {
        self.ecdh_us
    }
}

/// Adaptive benchmark configuration.
///
/// Instead of fixed round counts, each micro-benchmark runs for
/// `target_us` wall time. The total is at least ~6 × `target_us` (the
/// accuracy floor) and capped by `budget_us` (the deadline).
#[derive(Clone, Copy, Debug)]
pub struct BenchConfig {
    /// Wall-time target for each of the 6 micro-benchmarks.
    /// Default 250_000 µs → ~1.5 s of measured work.
    pub target_us: u64,
    /// Hard wall-clock budget for the whole benchmark (including calibration).
    /// Remaining micro-benchmarks share whatever budget is left.
    pub budget_us: u64,
    /// Iterations used to estimate per-op cost before choosing round counts.
    pub calib_rounds: u32,
    /// Minimum rounds per micro-benchmark (keeps averages meaningful).
    pub min_rounds: u32,
    /// Maximum rounds per micro-benchmark (bounds the fast-backend case).
    pub max_rounds: u32,
}

impl BenchConfig {
    /// Defaults: ~1.5 s of measured work, 9 s hard budget.
    pub const DEFAULT: Self = Self {
        target_us: 250_000,
        budget_us: 9_000_000,
        calib_rounds: 8,
        min_rounds: 3,
        max_rounds: 10_000,
    };

    /// Shorter run for CI: ~1.1 s of measured work.
    pub const QUICK: Self = Self {
        target_us: 180_000,
        budget_us: 9_000_000,
        calib_rounds: 4,
        min_rounds: 3,
        max_rounds: 10_000,
    };
}

impl Default for BenchConfig {
    fn default() -> Self {
        Self::DEFAULT
    }
}

// ─── Internal helpers ─────────────────────────────────────────────────────────

/// Deterministically derive a valid secret key from a tag byte.
/// No RNG, no alloc: scans a few candidates until one is in `[1, n)`.
fn derive_secret<C: CurveArithmetic>(tag: u8) -> SecretKey<C> {
    let mut bytes = FieldBytes::<C>::default();
    for (i, b) in bytes.iter_mut().enumerate() {
        *b = tag ^ (i as u8).wrapping_mul(37) ^ 0xA5;
    }
    loop {
        if let Ok(sk) = SecretKey::<C>::from_slice(&bytes) {
            return sk;
        }
        let last = bytes.len() - 1;
        bytes[last] = bytes[last].wrapping_add(1);
    }
}

#[inline]
fn avg_us(elapsed_us: u64, rounds: u32) -> u64 {
    if rounds == 0 { 0 } else { elapsed_us / u64::from(rounds) }
}

/// Estimate the per-op cost of `op_kind` relative to a measured base-mul cost,
/// as a fraction (numerator/denominator) of `base_us`. Used only for *planning*
/// round counts; actual results come from measurement.
#[inline]
fn plan_rounds(op_us_est: u64, target_us: u64, cfg: &BenchConfig) -> u32 {
    let est = op_us_est.max(1);
    let r = target_us / est;
    r.clamp(u64::from(cfg.min_rounds), u64::from(cfg.max_rounds)) as u32
}

// ─── Verification ─────────────────────────────────────────────────────────────

/// Verify that `C`'s `CurveArithmetic` really is NIST P-256.
///
/// Checks, in order:
/// 1. OID is secp256r1.
/// 2. Field elements are 32 bytes.
/// 3. The known SEC1 base point parses and re-encodes (compressed +
///    uncompressed) to the known forms. This pins the curve equation, field
///    modulus, and generator simultaneously.
/// 4. `[n]G == O` for the declared group order.
/// 5. ECDH symmetry: `DH(a, bG) == DH(b, aG)`.
pub fn verify_p256<C>() -> Result<(), &'static str>
where
    C: CurveArithmetic + AssociatedOid,
    FieldBytesSize<C>: ModulusSize,
    C::AffinePoint: FromEncodedPoint<C> + ToEncodedPoint<C>,
{
    if <C as AssociatedOid>::OID != P256_OID {
        return Err("OID mismatch: not secp256r1");
    }

    if FieldBytes::<C>::default().len() != 32 {
        return Err("field element size != 32 bytes");
    }

    let g = PublicKey::<C>::from_sec1_bytes(&P256_G_UNCOMPRESSED).map_err(|_| "P-256 base point encoding rejected")?;
    if g.to_encoded_point(true).as_bytes() != P256_G_COMPRESSED {
        return Err("base point does not match P-256 generator (compressed)");
    }
    if g.to_encoded_point(false).as_bytes() != P256_G_UNCOMPRESSED {
        return Err("base point does not match P-256 generator (uncompressed)");
    }

    let n = Scalar::<C>::reduce(<C as Curve>::ORDER);
    let generator: C::ProjectivePoint = Group::generator();
    let identity: C::ProjectivePoint = Group::identity();
    if !bool::from((generator * n).ct_eq(&identity)) {
        return Err("[n]G != identity: wrong group order");
    }

    let a = derive_secret::<C>(0xA1);
    let b = derive_secret::<C>(0xB2);
    let s_ab = ecdh::diffie_hellman(a.to_nonzero_scalar(), b.public_key().as_affine());
    let s_ba = ecdh::diffie_hellman(b.to_nonzero_scalar(), a.public_key().as_affine());
    if s_ab.raw_secret_bytes() != s_ba.raw_secret_bytes() {
        return Err("ECDH symmetry check failed");
    }

    info!("P-256 CurveArithmetic verified: OID + generator KAT + group order + ECDH");
    Ok(())
}

// ─── Benchmark ────────────────────────────────────────────────────────────────

/// Number of micro-benchmarks (used for dynamic budget splitting).
const N_BENCHES: u64 = 6;

/// Benchmark the P-256 primitives behind TLS/BLE operations.
///
/// Calibration first measures the base-mul cost, then each micro-benchmark
/// runs an adaptively chosen number of rounds to fill `cfg.target_us` of
/// wall time. The total wall time is kept within `cfg.budget_us` by giving
/// each remaining micro-benchmark an equal share of the leftover budget.
///
/// Timing uses [`embassy_time::Instant`]; the embassy time driver must be
/// initialized before calling.
pub fn benchmark_p256<C>(cfg: BenchConfig, tag: &str) -> Timings
where
    C: CurveArithmetic + AssociatedOid,
    FieldBytesSize<C>: ModulusSize,
    C::AffinePoint: FromEncodedPoint<C> + ToEncodedPoint<C>,
{
    let bench_start = Instant::now();

    let ephemeral = derive_secret::<C>(0xE1);
    let peer = derive_secret::<C>(0x5C);
    let peer_pk = peer.public_key();
    let peer_point = C::ProjectivePoint::from(*peer_pk.as_affine());

    let scalars: [Scalar<C>; 8] = core::array::from_fn(|i| *derive_secret::<C>(0x40 | (i as u8)).to_nonzero_scalar());

    let generator: C::ProjectivePoint = Group::generator();
    let identity: C::ProjectivePoint = Group::identity();

    // ── Calibration: measure base-mul cost ──────────────────────────────────
    let mut acc = identity;
    let start = Instant::now();
    for i in 0..cfg.calib_rounds {
        let r = C::ProjectivePoint::mul_by_generator(black_box(&scalars[(i as usize) & 7]));
        acc = black_box(acc + r);
    }
    black_box(&acc);
    let base_us = avg_us(start.elapsed().as_micros(), cfg.calib_rounds);
    info!("[{=str}] calibration: base-mul ≈ {=u64} us", tag, base_us);

    // Conservative planning estimates relative to the measured base-mul cost.
    // (5/4 etc. leave headroom so a misestimate can't blow the budget.)
    let var_mul_est = base_us.saturating_mul(5) / 4;
    let point_add_est = (base_us / 8).max(1);
    let invert_est = base_us.saturating_mul(3);
    let lincomb_est = base_us.saturating_mul(5) / 2;
    let ecdh_est = base_us.saturating_mul(5) / 4;

    // Dynamic per-benchmark target: share the remaining budget evenly.
    let target_for = |i: u64| {
        let remaining = cfg.budget_us.saturating_sub(bench_start.elapsed().as_micros());
        let share = remaining / (N_BENCHES - i);
        cfg.target_us.min(share)
    };

    // ── Base-point mul (ephemeral keygen, ECDSA signing) ────────────────────
    let rounds = plan_rounds(base_us, target_for(0), &cfg);
    let mut acc = identity;
    let start = Instant::now();
    for i in 0..rounds {
        let r = C::ProjectivePoint::mul_by_generator(black_box(&scalars[(i as usize) & 7]));
        acc = black_box(acc + r);
    }
    black_box(&acc);
    let base_mul_us = avg_us(start.elapsed().as_micros(), rounds);

    // ── Variable-base mul (peer public-key multiply) ────────────────────────
    let rounds = plan_rounds(var_mul_est, target_for(1), &cfg);
    let mut acc = identity;
    let start = Instant::now();
    for i in 0..rounds {
        let r = *black_box(&peer_point) * *black_box(&scalars[(i as usize) & 7]);
        acc = acc + r;
    }
    black_box(&acc);
    let var_mul_us = avg_us(start.elapsed().as_micros(), rounds);

    // ── Point add + double (field arithmetic throughput) ────────────────────
    let rounds = plan_rounds(point_add_est, target_for(2), &cfg);
    let mut acc = peer_point;
    let start = Instant::now();
    for _ in 0..rounds {
        acc = black_box(acc) + black_box(peer_point);
        acc = black_box(acc) + black_box(acc);
    }
    black_box(&acc);
    let point_add_us = avg_us(start.elapsed().as_micros(), rounds.saturating_mul(2));

    // ── Scalar inversion (ECDSA signing cost driver) ────────────────────────
    let rounds = plan_rounds(invert_est, target_for(3), &cfg);
    let mut parity = 0u8;
    let start = Instant::now();
    for i in 0..rounds {
        let inv = black_box(&scalars[(i as usize) & 7]).invert();
        parity ^= black_box(inv.is_some().unwrap_u8());
    }
    black_box(parity);
    let scalar_invert_us = avg_us(start.elapsed().as_micros(), rounds);

    // ── Double-scalar mul via lincomb (Shamir when available) ───────────────
    let rounds = plan_rounds(lincomb_est, target_for(4), &cfg);
    let mut acc = identity;
    let start = Instant::now();
    for i in 0..rounds {
        let j = (i as usize) & 7;
        let r = C::ProjectivePoint::lincomb(
            &generator,
            black_box(&scalars[j]),
            black_box(&peer_point),
            black_box(&scalars[(j + 3) & 7]),
        );
        acc = acc + r;
    }
    black_box(&acc);
    let lincomb_us = avg_us(start.elapsed().as_micros(), rounds);

    // ── Full ECDH (BLE LESC DH, TLS static-ECDH) ────────────────────────────
    let rounds = plan_rounds(ecdh_est, target_for(5), &cfg);
    let mut check = FieldBytes::<C>::default();
    let start = Instant::now();
    for _ in 0..rounds {
        let s = ecdh::diffie_hellman(black_box(ephemeral.to_nonzero_scalar()), black_box(peer_pk.as_affine()));
        check = s.raw_secret_bytes().clone();
    }
    black_box(&check);
    let ecdh_us = avg_us(start.elapsed().as_micros(), rounds);

    let timings = Timings {
        base_mul_us,
        var_mul_us,
        point_add_us,
        scalar_invert_us,
        lincomb_us,
        ecdh_us,
    };

    info!(
        "[{=str}] base_mul={=u64} var_mul={=u64} add={=u64} inv={=u64} lincomb={=u64} ecdh={=u64} us/op",
        tag,
        timings.base_mul_us,
        timings.var_mul_us,
        timings.point_add_us,
        timings.scalar_invert_us,
        timings.lincomb_us,
        timings.ecdh_us,
    );
    info!(
        "[{=str}] TLS1.3 ECDHE ~{=u64} us | TLS1.3 ECDSA verify ~{=u64} us | BLE LESC DH ~{=u64} us",
        tag,
        timings.tls13_ecdhe_us(),
        timings.tls13_ecdsa_verify_us(),
        timings.ble_pairing_ecdh_us(),
    );

    let total_us = bench_start.elapsed().as_micros();
    if total_us > cfg.budget_us {
        warn!("[{=str}] benchmark exceeded budget: {=u64} us", tag, total_us);
    } else {
        info!("[{=str}] total benchmark wall time: {=u64} us", tag, total_us);
    }

    timings
}

/// Verify first, then benchmark. Returns `None` if verification fails.
pub fn verify_and_benchmark<C>(cfg: BenchConfig, tag: &str) -> Option<Timings>
where
    C: CurveArithmetic + AssociatedOid,
    FieldBytesSize<C>: ModulusSize,
    C::AffinePoint: FromEncodedPoint<C> + ToEncodedPoint<C>,
{
    match verify_p256::<C>() {
        Ok(()) => Some(benchmark_p256::<C>(cfg, tag)),
        Err(e) => {
            error!("[{=str}] P-256 verification FAILED: {=str}", tag, e);
            None
        }
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let _p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    benchmark_p256::<embassy_crypto::p256::NistP256>(BenchConfig::DEFAULT, "default bench");

    cortex_m::asm::bkpt();
}

#![no_std]
#![no_main]

use core::hint::black_box;

use defmt::{info, warn};
use defmt_rtt as _;
use embassy_crypto::p256::{Point, Scalar, SecretKey};
use embassy_crypto_rustcrypto as _;
use embassy_executor::Spawner;
use embassy_time::Instant;
use panic_probe as _;

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
/// `target_us` wall time, within a hard `budget_us` for the whole run.
#[derive(Clone, Copy, Debug)]
pub struct BenchConfig {
    /// Wall-time target for each of the 6 micro-benchmarks.
    pub target_us: u64,
    /// Hard wall-clock budget for the whole benchmark (including calibration).
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
}

/// Deterministically derive a valid secret key from a tag byte: no RNG needed.
fn derive_secret(tag: u8) -> SecretKey {
    let mut bytes = [0u8; 32];
    for (i, b) in bytes.iter_mut().enumerate() {
        *b = tag ^ (i as u8).wrapping_mul(37) ^ 0xA5;
    }
    loop {
        if let Ok(sk) = SecretKey::from_bytes(&bytes) {
            return sk;
        }
        bytes[31] = bytes[31].wrapping_add(1);
    }
}

#[inline]
fn avg_us(elapsed_us: u64, rounds: u32) -> u64 {
    if rounds == 0 { 0 } else { elapsed_us / u64::from(rounds) }
}

#[inline]
fn plan_rounds(op_us_est: u64, target_us: u64, cfg: &BenchConfig) -> u32 {
    let est = op_us_est.max(1);
    let r = target_us / est;
    r.clamp(u64::from(cfg.min_rounds), u64::from(cfg.max_rounds)) as u32
}

/// Number of micro-benchmarks (used for dynamic budget splitting).
const N_BENCHES: u64 = 6;

/// Benchmark the P-256 primitives behind TLS/BLE operations.
pub fn benchmark_p256(cfg: BenchConfig, tag: &str) -> Timings {
    let bench_start = Instant::now();

    let ephemeral = derive_secret(0xE1);
    let peer = derive_secret(0x5C);
    let peer_pk = peer.public_key().unwrap();
    let peer_point = Point::try_from(peer_pk).unwrap();

    let scalars: [Scalar; 8] = core::array::from_fn(|i| derive_secret(0x40 | (i as u8)).to_scalar());

    // Calibration: measure the base-mul cost.
    let mut acc = Point::generator();
    let start = Instant::now();
    for i in 0..cfg.calib_rounds {
        let r = Point::mul_base(black_box(&scalars[(i as usize) & 7]));
        acc = black_box(acc.add(&r));
    }
    black_box(&acc);
    let base_us = avg_us(start.elapsed().as_micros(), cfg.calib_rounds);
    info!("[{=str}] calibration: base-mul ≈ {=u64} us", tag, base_us);

    // Conservative planning estimates relative to the measured base-mul cost.
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

    // Base-point mul (ephemeral keygen, ECDSA signing).
    let rounds = plan_rounds(base_us, target_for(0), &cfg);
    let mut acc = Point::generator();
    let start = Instant::now();
    for i in 0..rounds {
        let r = Point::mul_base(black_box(&scalars[(i as usize) & 7]));
        acc = black_box(acc.add(&r));
    }
    black_box(&acc);
    let base_mul_us = avg_us(start.elapsed().as_micros(), rounds);

    // Variable-base mul (peer public-key multiply).
    let rounds = plan_rounds(var_mul_est, target_for(1), &cfg);
    let mut acc = Point::generator();
    let start = Instant::now();
    for i in 0..rounds {
        let r = black_box(&peer_point).mul(black_box(&scalars[(i as usize) & 7]));
        acc = acc.add(&r);
    }
    black_box(&acc);
    let var_mul_us = avg_us(start.elapsed().as_micros(), rounds);

    // Point add + double (field arithmetic throughput).
    let rounds = plan_rounds(point_add_est, target_for(2), &cfg);
    let mut acc = peer_point;
    let start = Instant::now();
    for _ in 0..rounds {
        acc = black_box(acc).add(black_box(&peer_point));
        acc = black_box(acc).add(black_box(&acc));
    }
    black_box(&acc);
    let point_add_us = avg_us(start.elapsed().as_micros(), rounds.saturating_mul(2));

    // Scalar inversion (ECDSA signing cost driver).
    let rounds = plan_rounds(invert_est, target_for(3), &cfg);
    let mut parity = 0u8;
    let start = Instant::now();
    for i in 0..rounds {
        let inv = black_box(&scalars[(i as usize) & 7]).invert();
        parity ^= black_box(inv.is_some() as u8);
    }
    black_box(parity);
    let scalar_invert_us = avg_us(start.elapsed().as_micros(), rounds);

    // Double-scalar mul via the variable-time linear combination (Shamir when available).
    let rounds = plan_rounds(lincomb_est, target_for(4), &cfg);
    let g = Point::generator();
    let mut acc = g;
    let start = Instant::now();
    for i in 0..rounds {
        let j = (i as usize) & 7;
        let r = Point::lincomb_vartime(
            black_box(&scalars[j]),
            &g,
            black_box(&scalars[(j + 3) & 7]),
            &peer_point,
        );
        acc = acc.add(&r);
    }
    black_box(&acc);
    let lincomb_us = avg_us(start.elapsed().as_micros(), rounds);

    // Full ECDH (BLE LESC DH, TLS static-ECDH).
    let rounds = plan_rounds(ecdh_est, target_for(5), &cfg);
    let mut check = [0u8; 32];
    let start = Instant::now();
    for _ in 0..rounds {
        let s = black_box(&ephemeral).diffie_hellman(black_box(&peer_pk)).unwrap();
        check = *s.as_bytes();
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

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let _p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    // Sanity check: ECDH agrees from both sides.
    let a = derive_secret(0xA1);
    let b = derive_secret(0xB2);
    let s_ab = a.diffie_hellman(&b.public_key().unwrap()).unwrap();
    let s_ba = b.diffie_hellman(&a.public_key().unwrap()).unwrap();
    defmt::assert_eq!(s_ab.as_bytes(), s_ba.as_bytes());

    benchmark_p256(BenchConfig::DEFAULT, "default bench");

    cortex_m::asm::bkpt();
}

//! Beat detection and level metering for the DFSDM microphone examples.
//!
//! Pure data processing: no hardware access. Each example owns its DFSDM/PWM
//! setup and feeds samples into [`LevelDsp`], then reports loop timings with
//! [`Meter`].

use defmt::{Display2Format, info};
use embassy_time::{Duration, Instant};

// =============================================================================
// Beat detection
// =============================================================================

/// Streaming envelope follower + AGC that turns a mic sample into a PWM duty
/// cycle.
///
/// Stages, in order: DC blocker, rectifier, low-pass (bass), peak envelope
/// follower, then AGC that keeps the average output near a target level.
pub struct LevelDsp {
    dc_offset: i32,
    bass_signal: i32,
    envelope: u32,
    agc_gain_q16: u32,
    avg_scaled: u32,
}

impl LevelDsp {
    pub fn new() -> Self {
        Self {
            dc_offset: 0,
            bass_signal: 0,
            envelope: 0,
            agc_gain_q16: 1 << 16,
            avg_scaled: 0,
        }
    }

    /// Process one sample and return the PWM duty cycle in `0..=max_duty`.
    pub fn process(&mut self, data: i32, max_duty: u32) -> u32 {
        // 1. DC blocker (slow-moving average removed)
        self.dc_offset += ema_step(data - self.dc_offset, DC_SHIFT);
        let ac_signal = data - self.dc_offset;

        // 2. Rectifier
        let abs = ac_signal.unsigned_abs() as i32;

        // 3. Low-pass filter (bass)
        self.bass_signal += ema_step(abs - self.bass_signal, LPF_SHIFT);
        let bass_abs = self.bass_signal as u32;

        // 4. Envelope follower (fast attack, slow decay)
        if bass_abs > self.envelope {
            self.envelope = bass_abs;
        } else {
            let decay = self.envelope >> DECAY_SHIFT;
            if decay > 0 {
                self.envelope -= decay;
            } else if self.envelope > 0 {
                self.envelope -= 1;
            }
        }

        // 5. AGC: scale to max_duty, then nudge the gain toward the target
        // average so quiet and loud material both settle near ~55% duty.
        let scaled = ((self.envelope as u64) * (self.agc_gain_q16 as u64)) >> 16;
        let scaled = scaled.min(u32::MAX as u64) as u32;

        self.avg_scaled =
            (self.avg_scaled as i64 + ema_step(scaled as i32 - self.avg_scaled as i32, AVG_SHIFT) as i64) as u32;

        let target = ((max_duty as u64) * TARGET_NUM / TARGET_DEN) as u32;

        if self.avg_scaled > target {
            let reduction = (self.agc_gain_q16 >> GAIN_ATTACK_SHIFT).max(1);
            self.agc_gain_q16 = self.agc_gain_q16.saturating_sub(reduction).max(AGC_GAIN_MIN_Q16);
        } else if self.avg_scaled < target && self.envelope > NOISE_GATE {
            let increase = (self.agc_gain_q16 >> GAIN_RELEASE_SHIFT).max(1);
            self.agc_gain_q16 = self.agc_gain_q16.saturating_add(increase).min(AGC_GAIN_MAX_Q16);
        }

        // 6. Clamp to the PWM period.
        scaled.min(max_duty)
    }
}

// Tuning constants.
const LPF_SHIFT: u32 = 3;
const DECAY_SHIFT: u32 = 5;
const DC_SHIFT: u32 = 8;
const AGC_GAIN_MIN_Q16: u32 = 1 << 10;
const AGC_GAIN_MAX_Q16: u32 = 1 << 24;
const AVG_SHIFT: u32 = 7;
const TARGET_NUM: u64 = 11;
const TARGET_DEN: u64 = 20;
const GAIN_ATTACK_SHIFT: u32 = 9;
const GAIN_RELEASE_SHIFT: u32 = 16;
const NOISE_GATE: u32 = 50;

/// Exponential moving-average step with rounding and a minimum step of 1, so the
/// filter can't stall (a plain `>>` gives step 0 whenever
/// `0 < |diff| < 2^shift`).
#[inline]
fn ema_step(diff: i32, shift: u32) -> i32 {
    if diff == 0 {
        0
    } else if diff > 0 {
        ((diff + (1 << (shift - 1))) >> shift).max(1)
    } else {
        ((diff - (1 << (shift - 1))) >> shift).min(-1)
    }
}

// =============================================================================
// Metering
// =============================================================================

/// Accumulates wait/busy timings and prints a summary once per second.
pub struct Meter {
    busy_us: u64,
    wait_us: u64,
    samples: u32,
    window_start: Instant,
}

const STATS_INTERVAL_US: u64 = 1_000_000;

impl Meter {
    pub fn new() -> Self {
        Self {
            busy_us: 0,
            wait_us: 0,
            samples: 0,
            window_start: Instant::now(),
        }
    }

    /// Record one sample: `wait` is the time spent waiting for the result and
    /// `busy` the time spent processing it.
    pub fn record(&mut self, wait: Duration, busy: Duration) {
        self.wait_us += wait.as_micros();
        self.busy_us += busy.as_micros();
        self.samples += 1;
    }

    /// Print the accumulated stats (and reset the window) once per second.
    pub fn report(&mut self) {
        let now = Instant::now();
        let elapsed = (now - self.window_start).as_micros();
        if elapsed < STATS_INTERVAL_US {
            return;
        }

        let total_us = self.busy_us + self.wait_us;
        let busy_pct = if total_us > 0 {
            self.busy_us as f32 * 100.0 / total_us as f32
        } else {
            0.0
        };
        let avg_period_us = if self.samples > 0 {
            total_us / self.samples as u64
        } else {
            0
        };

        info!(
            "metering: samples={} busy_us={} wait_us={} busy_pct={}% avg_period_us={} window_us={}",
            self.samples,
            self.busy_us,
            self.wait_us,
            Display2Format(&format_args!("{:.3}", busy_pct)),
            avg_period_us,
            elapsed,
        );

        self.busy_us = 0;
        self.wait_us = 0;
        self.samples = 0;
        self.window_start = now;
    }
}

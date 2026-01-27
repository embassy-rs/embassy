#![no_std]
#![no_main]

use biquad::{
    Biquad, DirectForm1, Coefficients, Type, ToHertz, Q_BUTTERWORTH_F32,
};

pub struct PpgFilter {
    hp: DirectForm1<f32>,
    lp: DirectForm1<f32>,
}

// •	0.5 Hz high-pass → removes baseline drift, motion, slow pressure changes
// •	5 Hz low-pass → passes heart rate up to ~300 BPM
// •	Butterworth → flat passband, no ripple

impl PpgFilter {
    pub fn new(sample_rate_hz: f32) -> Self {
        // High-pass: remove DC & slow drift
        let hp_coeffs = Coefficients::<f32>::from_params(
            Type::HighPass,
            sample_rate_hz.hz(),
            0.5.hz(),              // cutoff ~0.5 Hz
            Q_BUTTERWORTH_F32,
        ).unwrap();

        // Low-pass: remove high-frequency noise
        let lp_coeffs = Coefficients::<f32>::from_params(
            Type::LowPass,
            sample_rate_hz.hz(),
            5.0.hz(),              // cutoff ~5 Hz
            Q_BUTTERWORTH_F32,
        ).unwrap();

        Self {
            hp: DirectForm1::new(hp_coeffs),
            lp: DirectForm1::new(lp_coeffs),
        }
    }

    /// Feed one raw IR sample, get filtered AC signal
    pub fn process(&mut self, ir: u32) -> f32 {
        let x = ir as f32;
        let y_hp = self.hp.run(x);
        let y_lp = self.lp.run(y_hp);
        y_lp
    }
}


pub struct HrDetector {
    // For peak detection
    last_sample: f32,
    last_peak_time_ms: u32,

    // Output
    bpm: f32,
}

impl HrDetector {
    pub fn new() -> Self {
        Self {
            last_sample: 0.0,
            last_peak_time_ms: 0,
            bpm: 0.0,
        }
    }

    /// Process one filtered PPG sample.
    /// `sample` should be AC-only (after HP+LP filtering).
    /// `now_ms` must be monotonic milliseconds.
    pub fn update(&mut self, sample: f32, now_ms: u32) {
        let slope = sample - self.last_sample;
        self.last_sample = sample;

        // --- TUNABLE PARAMETERS ---
        const THRESHOLD: f32 = 1500.0;      // depends on LED current / gain
        const REFRACTORY_MS: u32 = 450;      // prevents double counting
        const MIN_RR_MS: u32 = 300;          // 200 BPM max
        const MAX_RR_MS: u32 = 2000;         // 30 BPM min
        // --------------------------

        // Peak = above threshold AND slope just turned negative
        if sample > THRESHOLD && slope < 0.0 {
            // Check refractory period to prevent double counting
            if self.last_peak_time_ms == 0 || now_ms.saturating_sub(self.last_peak_time_ms) >= REFRACTORY_MS {
                if self.last_peak_time_ms != 0 {
                    let rr = now_ms.saturating_sub(self.last_peak_time_ms);

                    if rr >= MIN_RR_MS && rr <= MAX_RR_MS {
                        // Simple direct calculation: BPM = 60,000 ms / RR_interval_ms
                        let new_bpm = 60_000.0 / rr as f32;
                        
                        // Simple smoothing to reduce jitter (optional - can remove if you want instant updates)
                        self.bpm = 0.8 * self.bpm + 0.2 * new_bpm;
                    }
                }

                self.last_peak_time_ms = now_ms;
            }
        }
    }

    /// Get current BPM estimate (even if not enough beats detected yet)
    pub fn current_bpm(&self) -> f32 {
        self.bpm
    }


    /// Reset the detector (e.g., when finger is removed)
    pub fn reset(&mut self) {
        self.last_sample = 0.0;
        self.last_peak_time_ms = 0;
        self.bpm = 0.0;
    }
}

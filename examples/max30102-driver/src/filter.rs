use biquad::{Biquad, Coefficients, DirectForm1, Q_BUTTERWORTH_F32, ToHertz, Type};

/// PPG signal filter
/// • 0.5 Hz high-pass → removes baseline drift, motion, slow pressure changes
/// • 5 Hz low-pass → passes heart rate up to ~300 BPM
/// • Butterworth → flat passband, no ripple
pub struct PpgFilter {
    hp: DirectForm1<f32>,
    lp: DirectForm1<f32>,
}

impl PpgFilter {
    pub fn new(sample_rate_hz: f32) -> Self {
        // High-pass: remove DC & slow drift
        let hp_coeffs = Coefficients::<f32>::from_params(
            Type::HighPass,
            sample_rate_hz.hz(),
            0.5.hz(), // cutoff ~0.5 Hz
            Q_BUTTERWORTH_F32,
        )
        .unwrap();

        // Low-pass: remove high-frequency noise
        let lp_coeffs = Coefficients::<f32>::from_params(
            Type::LowPass,
            sample_rate_hz.hz(),
            5.0.hz(), // cutoff ~5 Hz
            Q_BUTTERWORTH_F32,
        )
        .unwrap();

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

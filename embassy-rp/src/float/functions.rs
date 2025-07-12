// Credit: taken from `rp-hal` (also licensed Apache+MIT)
// https://github.com/rp-rs/rp-hal/blob/main/rp2040-hal/src/float/functions.rs

use crate::float::{Float, Int};
use crate::rom_data;

trait ROMFunctions {
    fn sqrt(self) -> Self;
    fn ln(self) -> Self;
    fn exp(self) -> Self;
    fn sin(self) -> Self;
    fn cos(self) -> Self;
    fn tan(self) -> Self;
    fn atan2(self, y: Self) -> Self;

    fn to_trig_range(self) -> Self;
}

impl ROMFunctions for f32 {
    fn sqrt(self) -> Self {
        rom_data::float_funcs::fsqrt(self)
    }

    fn ln(self) -> Self {
        rom_data::float_funcs::fln(self)
    }

    fn exp(self) -> Self {
        rom_data::float_funcs::fexp(self)
    }

    fn sin(self) -> Self {
        rom_data::float_funcs::fsin(self)
    }

    fn cos(self) -> Self {
        rom_data::float_funcs::fcos(self)
    }

    fn tan(self) -> Self {
        rom_data::float_funcs::ftan(self)
    }

    fn atan2(self, y: Self) -> Self {
        rom_data::float_funcs::fatan2(self, y)
    }

    fn to_trig_range(self) -> Self {
        // -128 < X < 128, logic from the Pico SDK
        let exponent = (self.repr() & Self::EXPONENT_MASK) >> Self::SIGNIFICAND_BITS;
        if exponent < 134 {
            self
        } else {
            self % (core::f32::consts::PI * 2.0)
        }
    }
}

impl ROMFunctions for f64 {
    fn sqrt(self) -> Self {
        rom_data::double_funcs::dsqrt(self)
    }

    fn ln(self) -> Self {
        rom_data::double_funcs::dln(self)
    }

    fn exp(self) -> Self {
        rom_data::double_funcs::dexp(self)
    }

    fn sin(self) -> Self {
        rom_data::double_funcs::dsin(self)
    }

    fn cos(self) -> Self {
        rom_data::double_funcs::dcos(self)
    }
    fn tan(self) -> Self {
        rom_data::double_funcs::dtan(self)
    }

    fn atan2(self, y: Self) -> Self {
        rom_data::double_funcs::datan2(self, y)
    }

    fn to_trig_range(self) -> Self {
        // -1024 < X < 1024, logic from the Pico SDK
        let exponent = (self.repr() & Self::EXPONENT_MASK) >> Self::SIGNIFICAND_BITS;
        if exponent < 1033 {
            self
        } else {
            self % (core::f64::consts::PI * 2.0)
        }
    }
}

fn is_negative_nonzero_or_nan<F: Float>(f: F) -> bool {
    let repr = f.repr();
    if (repr & F::SIGN_MASK) != F::Int::ZERO {
        // Negative, so anything other than exactly zero
        return (repr & (!F::SIGN_MASK)) != F::Int::ZERO;
    }
    // NaN
    (repr & (F::EXPONENT_MASK | F::SIGNIFICAND_MASK)) > F::EXPONENT_MASK
}

fn sqrt<F: Float + ROMFunctions>(f: F) -> F {
    if is_negative_nonzero_or_nan(f) {
        F::NAN
    } else {
        f.sqrt()
    }
}

fn ln<F: Float + ROMFunctions>(f: F) -> F {
    if is_negative_nonzero_or_nan(f) {
        F::NAN
    } else {
        f.ln()
    }
}

fn exp<F: Float + ROMFunctions>(f: F) -> F {
    if f.is_nan() {
        F::NAN
    } else {
        f.exp()
    }
}

fn sin<F: Float + ROMFunctions>(f: F) -> F {
    if f.is_not_finite() {
        F::NAN
    } else {
        f.to_trig_range().sin()
    }
}

fn cos<F: Float + ROMFunctions>(f: F) -> F {
    if f.is_not_finite() {
        F::NAN
    } else {
        f.to_trig_range().cos()
    }
}

fn tan<F: Float + ROMFunctions>(f: F) -> F {
    if f.is_not_finite() {
        F::NAN
    } else {
        f.to_trig_range().tan()
    }
}

fn atan2<F: Float + ROMFunctions>(x: F, y: F) -> F {
    if x.is_nan() || y.is_nan() {
        F::NAN
    } else {
        x.to_trig_range().atan2(y)
    }
}

// Name collisions
mod intrinsics {
    intrinsics! {
        extern "C" fn sqrtf(f: f32) -> f32 {
            super::sqrt(f)
        }

        #[bootrom_v2]
        extern "C" fn sqrt(f: f64) -> f64 {
            super::sqrt(f)
        }

        extern "C" fn logf(f: f32) -> f32 {
            super::ln(f)
        }

        #[bootrom_v2]
        extern "C" fn log(f: f64) -> f64 {
            super::ln(f)
        }

        extern "C" fn expf(f: f32) -> f32 {
            super::exp(f)
        }

        #[bootrom_v2]
        extern "C" fn exp(f: f64) -> f64 {
            super::exp(f)
        }

        #[slower_than_default]
        extern "C" fn sinf(f: f32) -> f32 {
            super::sin(f)
        }

        #[slower_than_default]
        #[bootrom_v2]
        extern "C" fn sin(f: f64) -> f64 {
            super::sin(f)
        }

        #[slower_than_default]
        extern "C" fn cosf(f: f32) -> f32 {
            super::cos(f)
        }

        #[slower_than_default]
        #[bootrom_v2]
        extern "C" fn cos(f: f64) -> f64 {
            super::cos(f)
        }

        #[slower_than_default]
        extern "C" fn tanf(f: f32) -> f32 {
            super::tan(f)
        }

        #[slower_than_default]
        #[bootrom_v2]
        extern "C" fn tan(f: f64) -> f64 {
            super::tan(f)
        }

        // Questionable gain
        #[bootrom_v2]
        extern "C" fn atan2f(a: f32, b: f32) -> f32 {
            super::atan2(a, b)
        }

        // Questionable gain
        #[bootrom_v2]
        extern "C" fn atan2(a: f64, b: f64) -> f64 {
            super::atan2(a, b)
        }
    }
}

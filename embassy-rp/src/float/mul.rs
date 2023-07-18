// Credit: taken from `rp-hal` (also licensed Apache+MIT)
// https://github.com/rp-rs/rp-hal/blob/main/rp2040-hal/src/float/mul.rs

use super::Float;
use crate::rom_data;

trait ROMMul {
    fn rom_mul(self, b: Self) -> Self;
}

impl ROMMul for f32 {
    fn rom_mul(self, b: Self) -> Self {
        rom_data::float_funcs::fmul(self, b)
    }
}

impl ROMMul for f64 {
    fn rom_mul(self, b: Self) -> Self {
        rom_data::double_funcs::dmul(self, b)
    }
}

fn mul<F: Float + ROMMul>(a: F, b: F) -> F {
    if a.is_not_finite() {
        if b.is_zero() {
            // [-]inf/NaN * 0 = NaN
            return F::NAN;
        }

        return if b.is_sign_negative() {
            // [+/-]inf/NaN * (-X) = [-/+]inf/NaN
            a.negate()
        } else {
            // [-]inf/NaN * X = [-]inf/NaN
            a
        };
    }

    if b.is_not_finite() {
        if a.is_zero() {
            // 0 * [-]inf/NaN = NaN
            return F::NAN;
        }

        return if b.is_sign_negative() {
            // (-X) * [+/-]inf/NaN = [-/+]inf/NaN
            b.negate()
        } else {
            // X * [-]inf/NaN = [-]inf/NaN
            b
        };
    }

    a.rom_mul(b)
}

intrinsics! {
    #[alias = __mulsf3vfp]
    #[aeabi = __aeabi_fmul]
    extern "C" fn __mulsf3(a: f32, b: f32) -> f32 {
        mul(a, b)
    }

    #[bootrom_v2]
    #[alias = __muldf3vfp]
    #[aeabi = __aeabi_dmul]
    extern "C" fn __muldf3(a: f64, b: f64) -> f64 {
        mul(a, b)
    }
}

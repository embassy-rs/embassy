// Credit: taken from `rp-hal` (also licensed Apache+MIT)
// https://github.com/rp-rs/rp-hal/blob/main/rp2040-hal/src/float/add_sub.rs

use super::{Float, Int};
use crate::rom_data;

trait ROMAdd {
    fn rom_add(self, b: Self) -> Self;
}

impl ROMAdd for f32 {
    fn rom_add(self, b: Self) -> Self {
        rom_data::float_funcs::fadd(self, b)
    }
}

impl ROMAdd for f64 {
    fn rom_add(self, b: Self) -> Self {
        rom_data::double_funcs::dadd(self, b)
    }
}

fn add<F: Float + ROMAdd>(a: F, b: F) -> F {
    if a.is_not_finite() {
        if b.is_not_finite() {
            let class_a = a.repr() & (F::SIGNIFICAND_MASK | F::SIGN_MASK);
            let class_b = b.repr() & (F::SIGNIFICAND_MASK | F::SIGN_MASK);

            if class_a == F::Int::ZERO && class_b == F::Int::ZERO {
                // inf + inf = inf
                return a;
            }
            if class_a == F::SIGN_MASK && class_b == F::SIGN_MASK {
                // -inf + (-inf) = -inf
                return a;
            }

            // Sign mismatch, or either is NaN already
            return F::NAN;
        }

        // [-]inf/NaN + X = [-]inf/NaN
        return a;
    }

    if b.is_not_finite() {
        // X + [-]inf/NaN = [-]inf/NaN
        return b;
    }

    a.rom_add(b)
}

intrinsics! {
    #[alias = __addsf3vfp]
    #[aeabi = __aeabi_fadd]
    extern "C" fn __addsf3(a: f32, b: f32) -> f32 {
        add(a, b)
    }

    #[bootrom_v2]
    #[alias = __adddf3vfp]
    #[aeabi = __aeabi_dadd]
    extern "C" fn __adddf3(a: f64, b: f64) -> f64 {
        add(a, b)
    }

    // The ROM just implements subtraction the same way, so just do it here
    // and save the work of implementing more complicated NaN/inf handling.

    #[alias = __subsf3vfp]
    #[aeabi = __aeabi_fsub]
    extern "C" fn __subsf3(a: f32, b: f32) -> f32 {
        add(a, -b)
    }

    #[bootrom_v2]
    #[alias = __subdf3vfp]
    #[aeabi = __aeabi_dsub]
    extern "C" fn __subdf3(a: f64, b: f64) -> f64 {
        add(a, -b)
    }

    extern "aapcs" fn __aeabi_frsub(a: f32, b: f32) -> f32 {
        add(b, -a)
    }

    #[bootrom_v2]
    extern "aapcs" fn __aeabi_drsub(a: f64, b: f64) -> f64 {
        add(b, -a)
    }
}

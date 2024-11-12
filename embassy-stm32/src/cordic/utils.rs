//! Common math utils
use super::errors::NumberOutOfRange;

macro_rules! floating_fixed_convert {
    ($f_to_q:ident, $q_to_f:ident, $unsigned_bin_typ:ty, $signed_bin_typ:ty, $float_ty:ty, $offset:literal, $min_positive:literal) => {
        /// convert float point to fixed point format
        pub fn $f_to_q(value: $float_ty) -> Result<$unsigned_bin_typ, NumberOutOfRange> {
            const MIN_POSITIVE: $float_ty = unsafe { core::mem::transmute($min_positive) };

            if value < -1.0 {
                return Err(NumberOutOfRange::BelowLowerBound)
            }

            if value > 1.0 {
                return Err(NumberOutOfRange::AboveUpperBound)
            }


            let value = if 1.0 - MIN_POSITIVE < value && value <= 1.0 {
                // make a exception for value between (1.0^{-x} , 1.0] float point,
                // convert it to max representable value of q1.x format
                (1.0 as $float_ty) - MIN_POSITIVE
            } else {
                value
            };

            // It's necessary to cast the float value to signed integer, before convert it to a unsigned value.
            // Since value from register is actually a "signed value", a "as" cast will keep original binary format but mark it as a unsigned value for register writing.
            // see https://doc.rust-lang.org/reference/expressions/operator-expr.html#numeric-cast
            Ok((value * ((1 as $unsigned_bin_typ << $offset) as $float_ty)) as $signed_bin_typ as $unsigned_bin_typ)
        }

        #[inline(always)]
        /// convert fixed point to float point format
        pub fn $q_to_f(value: $unsigned_bin_typ) -> $float_ty {
            // It's necessary to cast the unsigned integer to signed integer, before convert it to a float value.
            // Since value from register is actually a "signed value", a "as" cast will keep original binary format but mark it as a signed value.
            // see https://doc.rust-lang.org/reference/expressions/operator-expr.html#numeric-cast
            (value as $signed_bin_typ as $float_ty) / ((1 as $unsigned_bin_typ << $offset) as $float_ty)
        }
    };
}

floating_fixed_convert!(
    f64_to_q1_31,
    q1_31_to_f64,
    u32,
    i32,
    f64,
    31,
    0x3E00_0000_0000_0000u64 // binary form of 1f64^(-31)
);

floating_fixed_convert!(
    f32_to_q1_15,
    q1_15_to_f32,
    u16,
    i16,
    f32,
    15,
    0x3800_0000u32 // binary form of 1f32^(-15)
);

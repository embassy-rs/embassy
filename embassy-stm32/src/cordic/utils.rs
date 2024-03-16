//! Common match utils

macro_rules! floating_fixed_convert {
    ($f_to_q:ident, $q_to_f:ident, $unsigned_bin_typ:ty, $signed_bin_typ:ty, $float_ty:ty, $offset:literal, $min_positive:literal) => {
        /// convert float point to fixed point format
        pub(crate) fn $f_to_q(value: $float_ty) -> $unsigned_bin_typ {
            const MIN_POSITIVE: $float_ty = unsafe { core::mem::transmute($min_positive) };

            assert!(
                (-1.0 as $float_ty) <= value,
                "input value {} should be equal or greater than -1",
                value
            );


            let value = if value == 1.0 as $float_ty{
                // make a exception for user specifing exact 1.0 float point,
                // convert 1.0 to max representable value of q1.x format
                (1.0 as $float_ty) - MIN_POSITIVE
            } else {
                assert!(
                    value <= (1.0 as $float_ty) - MIN_POSITIVE,
                    "input value {} should be equal or less than 1-2^(-{})",
                    value, $offset
                );
                value
            };

            (value * ((1 as $unsigned_bin_typ << $offset) as $float_ty)) as $unsigned_bin_typ
        }

        #[inline(always)]
        /// convert fixed point to float point format
        pub(crate) fn $q_to_f(value: $unsigned_bin_typ) -> $float_ty {
            // It's needed to convert from unsigned to signed first, for correct result.
            -(value as $signed_bin_typ as $float_ty) / ((1 as $unsigned_bin_typ << $offset) as $float_ty)
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

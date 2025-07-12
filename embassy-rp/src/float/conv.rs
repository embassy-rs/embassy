// Credit: taken from `rp-hal` (also licensed Apache+MIT)
// https://github.com/rp-rs/rp-hal/blob/main/rp2040-hal/src/float/conv.rs

use super::Float;
use crate::rom_data;

// Some of these are also not connected in the Pico SDK.  This is probably
// because the ROM version actually does a fixed point conversion, just with
// the fractional width set to zero.

intrinsics! {
    // Not connected in the Pico SDK
    #[slower_than_default]
    #[aeabi = __aeabi_i2f]
    extern "C" fn __floatsisf(i: i32) -> f32 {
        rom_data::float_funcs::int_to_float(i)
    }

    // Not connected in the Pico SDK
    #[slower_than_default]
    #[aeabi = __aeabi_i2d]
    extern "C" fn __floatsidf(i: i32) -> f64 {
        rom_data::double_funcs::int_to_double(i)
    }

    // Questionable gain
    #[aeabi = __aeabi_l2f]
    extern "C" fn __floatdisf(i: i64) -> f32 {
        rom_data::float_funcs::int64_to_float(i)
    }

    #[bootrom_v2]
    #[aeabi = __aeabi_l2d]
    extern "C" fn __floatdidf(i: i64) -> f64 {
        rom_data::double_funcs::int64_to_double(i)
    }

    // Not connected in the Pico SDK
    #[slower_than_default]
    #[aeabi = __aeabi_ui2f]
    extern "C" fn __floatunsisf(i: u32) -> f32 {
        rom_data::float_funcs::uint_to_float(i)
    }

    // Questionable gain
    #[bootrom_v2]
    #[aeabi = __aeabi_ui2d]
    extern "C" fn __floatunsidf(i: u32) -> f64 {
        rom_data::double_funcs::uint_to_double(i)
    }

    // Questionable gain
    #[bootrom_v2]
    #[aeabi = __aeabi_ul2f]
    extern "C" fn __floatundisf(i: u64) -> f32 {
        rom_data::float_funcs::uint64_to_float(i)
    }

    #[bootrom_v2]
    #[aeabi = __aeabi_ul2d]
    extern "C" fn __floatundidf(i: u64) -> f64 {
        rom_data::double_funcs::uint64_to_double(i)
    }


    // The Pico SDK does some optimization here (e.x. fast paths for zero and
    // one), but we can just directly connect it.
    #[aeabi = __aeabi_f2iz]
    extern "C" fn __fixsfsi(f: f32) -> i32 {
        rom_data::float_funcs::float_to_int(f)
    }

    #[bootrom_v2]
    #[aeabi = __aeabi_f2lz]
    extern "C" fn __fixsfdi(f: f32) -> i64 {
        rom_data::float_funcs::float_to_int64(f)
    }

    // Not connected in the Pico SDK
    #[slower_than_default]
    #[bootrom_v2]
    #[aeabi = __aeabi_d2iz]
    extern "C" fn __fixdfsi(f: f64) -> i32 {
        rom_data::double_funcs::double_to_int(f)
    }

    // Like with the 32 bit version, there's optimization that we just
    // skip.
    #[bootrom_v2]
    #[aeabi = __aeabi_d2lz]
    extern "C" fn __fixdfdi(f: f64) -> i64 {
        rom_data::double_funcs::double_to_int64(f)
    }

    #[slower_than_default]
    #[aeabi = __aeabi_f2uiz]
    extern "C" fn __fixunssfsi(f: f32) -> u32 {
        rom_data::float_funcs::float_to_uint(f)
    }

    #[slower_than_default]
    #[bootrom_v2]
    #[aeabi = __aeabi_f2ulz]
    extern "C" fn __fixunssfdi(f: f32) -> u64 {
        rom_data::float_funcs::float_to_uint64(f)
    }

    #[slower_than_default]
    #[bootrom_v2]
    #[aeabi = __aeabi_d2uiz]
    extern "C" fn __fixunsdfsi(f: f64) -> u32 {
        rom_data::double_funcs::double_to_uint(f)
    }

    #[slower_than_default]
    #[bootrom_v2]
    #[aeabi = __aeabi_d2ulz]
    extern "C" fn __fixunsdfdi(f: f64) -> u64 {
        rom_data::double_funcs::double_to_uint64(f)
    }

    #[bootrom_v2]
    #[alias = __extendsfdf2vfp]
    #[aeabi = __aeabi_f2d]
    extern "C" fn  __extendsfdf2(f: f32) -> f64 {
        if f.is_not_finite() {
            return f64::from_repr(
                // Not finite
                f64::EXPONENT_MASK |
                // Preserve NaN or inf
                ((f.repr() & f32::SIGNIFICAND_MASK) as u64) |
                // Preserve sign
                ((f.repr() & f32::SIGN_MASK) as u64) << (f64::BITS-f32::BITS)
            );
        }
        rom_data::float_funcs::float_to_double(f)
    }

    #[bootrom_v2]
    #[alias = __truncdfsf2vfp]
    #[aeabi = __aeabi_d2f]
    extern "C" fn __truncdfsf2(f: f64) -> f32 {
        if f.is_not_finite() {
            let mut repr: u32 =
                // Not finite
                f32::EXPONENT_MASK |
                // Preserve sign
                ((f.repr() & f64::SIGN_MASK) >> (f64::BITS-f32::BITS)) as u32;
            // Set NaN
            if  (f.repr() & f64::SIGNIFICAND_MASK) != 0 {
                repr |= 1;
            }
            return f32::from_repr(repr);
        }
        rom_data::double_funcs::double_to_float(f)
    }
}

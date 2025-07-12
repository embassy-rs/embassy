// Credit: taken from `rp-hal` (also licensed Apache+MIT)
// https://github.com/rp-rs/rp-hal/blob/main/rp2040-hal/src/float/cmp.rs

use super::Float;
use crate::rom_data;

trait ROMCmp {
    fn rom_cmp(self, b: Self) -> i32;
}

impl ROMCmp for f32 {
    fn rom_cmp(self, b: Self) -> i32 {
        rom_data::float_funcs::fcmp(self, b)
    }
}

impl ROMCmp for f64 {
    fn rom_cmp(self, b: Self) -> i32 {
        rom_data::double_funcs::dcmp(self, b)
    }
}

fn le_abi<F: Float + ROMCmp>(a: F, b: F) -> i32 {
    if a.is_nan() || b.is_nan() {
        1
    } else {
        a.rom_cmp(b)
    }
}

fn ge_abi<F: Float + ROMCmp>(a: F, b: F) -> i32 {
    if a.is_nan() || b.is_nan() {
        -1
    } else {
        a.rom_cmp(b)
    }
}

intrinsics! {
    #[slower_than_default]
    #[bootrom_v2]
    #[alias = __eqsf2, __ltsf2, __nesf2]
    extern "C" fn __lesf2(a: f32, b: f32) -> i32 {
        le_abi(a, b)
    }

    #[slower_than_default]
    #[bootrom_v2]
    #[alias = __eqdf2, __ltdf2, __nedf2]
    extern "C" fn __ledf2(a: f64, b: f64) -> i32 {
        le_abi(a, b)
    }

    #[slower_than_default]
    #[bootrom_v2]
    #[alias = __gtsf2]
    extern "C" fn __gesf2(a: f32, b: f32) -> i32 {
        ge_abi(a, b)
    }

    #[slower_than_default]
    #[bootrom_v2]
    #[alias = __gtdf2]
    extern "C" fn __gedf2(a: f64, b: f64) -> i32 {
        ge_abi(a, b)
    }


    #[slower_than_default]
    #[bootrom_v2]
    extern "aapcs" fn __aeabi_fcmple(a: f32, b: f32) -> i32 {
        (le_abi(a, b) <= 0) as i32
    }

    #[slower_than_default]
    #[bootrom_v2]
    extern "aapcs" fn __aeabi_fcmpge(a: f32, b: f32) -> i32 {
        (ge_abi(a, b) >= 0) as i32
    }

    #[slower_than_default]
    #[bootrom_v2]
    extern "aapcs" fn __aeabi_fcmpeq(a: f32, b: f32) -> i32 {
        (le_abi(a, b) == 0) as i32
    }

    #[slower_than_default]
    #[bootrom_v2]
    extern "aapcs" fn __aeabi_fcmplt(a: f32, b: f32) -> i32 {
        (le_abi(a, b) < 0) as i32
    }

    #[slower_than_default]
    #[bootrom_v2]
    extern "aapcs" fn __aeabi_fcmpgt(a: f32, b: f32) -> i32 {
        (ge_abi(a, b) > 0) as i32
    }

    #[slower_than_default]
    #[bootrom_v2]
    extern "aapcs" fn __aeabi_dcmple(a: f64, b: f64) -> i32 {
        (le_abi(a, b) <= 0) as i32
    }

    #[slower_than_default]
    #[bootrom_v2]
    extern "aapcs" fn __aeabi_dcmpge(a: f64, b: f64) -> i32 {
        (ge_abi(a, b) >= 0) as i32
    }

    #[slower_than_default]
    #[bootrom_v2]
    extern "aapcs" fn __aeabi_dcmpeq(a: f64, b: f64) -> i32 {
        (le_abi(a, b) == 0) as i32
    }

    #[slower_than_default]
    #[bootrom_v2]
    extern "aapcs" fn __aeabi_dcmplt(a: f64, b: f64) -> i32 {
        (le_abi(a, b) < 0) as i32
    }

    #[slower_than_default]
    #[bootrom_v2]
    extern "aapcs" fn __aeabi_dcmpgt(a: f64, b: f64) -> i32 {
        (ge_abi(a, b) > 0) as i32
    }


    #[slower_than_default]
    #[bootrom_v2]
    extern "C" fn __gesf2vfp(a: f32, b: f32) -> i32 {
        (ge_abi(a, b) >= 0) as i32
    }

    #[slower_than_default]
    #[bootrom_v2]
    extern "C" fn __gedf2vfp(a: f64, b: f64) -> i32 {
        (ge_abi(a, b) >= 0) as i32
    }

    #[slower_than_default]
    #[bootrom_v2]
    extern "C" fn __gtsf2vfp(a: f32, b: f32) -> i32 {
        (ge_abi(a, b) > 0) as i32
    }

    #[slower_than_default]
    #[bootrom_v2]
    extern "C" fn __gtdf2vfp(a: f64, b: f64) -> i32 {
        (ge_abi(a, b) > 0) as i32
    }

    #[slower_than_default]
    #[bootrom_v2]
    extern "C" fn __ltsf2vfp(a: f32, b: f32) -> i32 {
        (le_abi(a, b) < 0) as i32
    }

    #[slower_than_default]
    #[bootrom_v2]
    extern "C" fn __ltdf2vfp(a: f64, b: f64) -> i32 {
        (le_abi(a, b) < 0) as i32
    }

    #[slower_than_default]
    #[bootrom_v2]
    extern "C" fn __lesf2vfp(a: f32, b: f32) -> i32 {
        (le_abi(a, b) <= 0) as i32
    }

    #[slower_than_default]
    #[bootrom_v2]
    extern "C" fn __ledf2vfp(a: f64, b: f64) -> i32 {
        (le_abi(a, b) <= 0) as i32
    }

    #[slower_than_default]
    #[bootrom_v2]
    extern "C" fn __nesf2vfp(a: f32, b: f32) -> i32 {
        (le_abi(a, b) != 0) as i32
    }

    #[slower_than_default]
    #[bootrom_v2]
    extern "C" fn __nedf2vfp(a: f64, b: f64) -> i32 {
        (le_abi(a, b) != 0) as i32
    }

    #[slower_than_default]
    #[bootrom_v2]
    extern "C" fn __eqsf2vfp(a: f32, b: f32) -> i32 {
        (le_abi(a, b) == 0) as i32
    }

    #[slower_than_default]
    #[bootrom_v2]
    extern "C" fn __eqdf2vfp(a: f64, b: f64) -> i32 {
        (le_abi(a, b) == 0) as i32
    }
}

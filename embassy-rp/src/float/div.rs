// Credit: taken from `rp-hal` (also licensed Apache+MIT)
// https://github.com/rp-rs/rp-hal/blob/main/rp2040-hal/src/float/conv.rs

use super::Float;
use crate::rom_data;

// Make sure this stays as a separate call, because when it's inlined the
// compiler will move the save of the registers used to contain the divider
// state into the function prologue.  That save and restore (push/pop) takes
// longer than the actual division, so doing it in the common case where
// they are not required wastes a lot of time.
#[inline(never)]
#[cold]
fn save_divider_and_call<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    let sio = rp_pac::SIO;

    // Since we can't save the signed-ness of the calculation, we have to make
    // sure that there's at least an 8 cycle delay before we read the result.
    // The Pico SDK ensures this by using a 6 cycle push and two 1 cycle reads.
    // Since we can't be sure the Rust implementation will optimize to the same,
    // just use an explicit wait.
    while !sio.div().csr().read().ready() {}

    // Read the quotient last, since that's what clears the dirty flag
    let dividend = sio.div().udividend().read();
    let divisor = sio.div().udivisor().read();
    let remainder = sio.div().remainder().read();
    let quotient = sio.div().quotient().read();

    // If we get interrupted here (before a write sets the DIRTY flag) its fine, since
    // we have the full state, so the interruptor doesn't have to restore it.  Once the
    // write happens and the DIRTY flag is set, the interruptor becomes responsible for
    // restoring our state.
    let result = f();

    // If we are interrupted here, then the interruptor will start an incorrect calculation
    // using a wrong divisor, but we'll restore the divisor and result ourselves correctly.
    // This sets DIRTY, so any interruptor will save the state.
    sio.div().udividend().write_value(dividend);
    // If we are interrupted here, the the interruptor may start the calculation using
    // incorrectly signed inputs, but we'll restore the result ourselves.
    // This sets DIRTY, so any interruptor will save the state.
    sio.div().udivisor().write_value(divisor);
    // If we are interrupted here, the interruptor will have restored everything but the
    // quotient may be wrongly signed.  If the calculation started by the above writes is
    // still ongoing it is stopped, so it won't replace the result we're restoring.
    // DIRTY and READY set, but only DIRTY matters to make the interruptor save the state.
    sio.div().remainder().write_value(remainder);
    // State fully restored after the quotient write.  This sets both DIRTY and READY, so
    // whatever we may have interrupted can read the result.
    sio.div().quotient().write_value(quotient);

    result
}

fn save_divider<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    let sio = rp_pac::SIO;
    if !sio.div().csr().read().dirty() {
        // Not dirty, so nothing is waiting for the calculation.  So we can just
        // issue it directly without a save/restore.
        f()
    } else {
        save_divider_and_call(f)
    }
}

trait ROMDiv {
    fn rom_div(self, b: Self) -> Self;
}

impl ROMDiv for f32 {
    fn rom_div(self, b: Self) -> Self {
        // ROM implementation uses the hardware divider, so we have to save it
        save_divider(|| rom_data::float_funcs::fdiv(self, b))
    }
}

impl ROMDiv for f64 {
    fn rom_div(self, b: Self) -> Self {
        // ROM implementation uses the hardware divider, so we have to save it
        save_divider(|| rom_data::double_funcs::ddiv(self, b))
    }
}

fn div<F: Float + ROMDiv>(a: F, b: F) -> F {
    if a.is_not_finite() {
        if b.is_not_finite() {
            // inf/NaN / inf/NaN = NaN
            return F::NAN;
        }

        if b.is_zero() {
            // inf/NaN / 0 = NaN
            return F::NAN;
        }

        return if b.is_sign_negative() {
            // [+/-]inf/NaN / (-X) = [-/+]inf/NaN
            a.negate()
        } else {
            // [-]inf/NaN / X = [-]inf/NaN
            a
        };
    }

    if b.is_nan() {
        // X / NaN = NaN
        return b;
    }

    // ROM handles X / 0 = [-]inf and X / [-]inf = [-]0, so we only
    // need to catch 0 / 0
    if b.is_zero() && a.is_zero() {
        return F::NAN;
    }

    a.rom_div(b)
}

intrinsics! {
    #[alias = __divsf3vfp]
    #[aeabi = __aeabi_fdiv]
    extern "C" fn __divsf3(a: f32, b: f32) -> f32 {
        div(a, b)
    }

    #[bootrom_v2]
    #[alias = __divdf3vfp]
    #[aeabi = __aeabi_ddiv]
    extern "C" fn __divdf3(a: f64, b: f64) -> f64 {
        div(a, b)
    }
}

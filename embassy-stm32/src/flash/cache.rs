//! Cache maintenance around flash program and erase operations.
//!
//! The caches between the CPU and the flash are not kept coherent with program and erase
//! operations, so they are disabled while an operation runs and flushed before they are
//! re-enabled:
//!
//! - Flash accelerator caches in `FLASH_ACR` (F2/F4, L4, G0/G4, C0, U0, WB, WL): the I/D-cache
//!   resets may only be used while the caches are disabled. On dual-bank F42x/F43x this is also
//!   the workaround for the data cache corruption erratum (ES0206 2.2.12).
//! - The ART accelerator on F7.
//! - The ICACHE peripheral (L5, U5, U3, WBA, H5, C5). The reference manuals recommend
//!   modifying the flash with the ICACHE disabled; disabling it invalidates it.
//! - The Cortex-M7 L1 caches (F7, H7).

#[allow(unused_imports)]
use crate::pac;

#[cfg(any(
    flash_v2_f2,
    flash_v2_f4,
    flash_v4_l4,
    flash_v4_l4p,
    flash_v4_g4c2,
    flash_v4_g4c3,
    flash_v4_g4c4,
    flash_v4_wb,
    flash_v4_wl,
))]
#[allow(unused_macros)]
macro_rules! acr_dcache {
    ($($t:tt)*) => { $($t)* };
}
#[cfg(not(any(
    flash_v2_f2,
    flash_v2_f4,
    flash_v4_l4,
    flash_v4_l4p,
    flash_v4_g4c2,
    flash_v4_g4c3,
    flash_v4_g4c4,
    flash_v4_wb,
    flash_v4_wl,
)))]
#[allow(unused_macros)]
macro_rules! acr_dcache {
    ($($t:tt)*) => {};
}

#[cfg(any(
    flash_v2_f2,
    flash_v2_f4,
    flash_v4_l4,
    flash_v4_l4p,
    flash_v4_g4c2,
    flash_v4_g4c3,
    flash_v4_g4c4,
    flash_v4_wb,
    flash_v4_wl,
    flash_v4_g0x0,
    flash_v4_g0x1,
    flash_v4_c0,
    flash_v4_u0,
))]
macro_rules! acr_icache {
    ($($t:tt)*) => { $($t)* };
}
#[cfg(not(any(
    flash_v2_f2,
    flash_v2_f4,
    flash_v4_l4,
    flash_v4_l4p,
    flash_v4_g4c2,
    flash_v4_g4c3,
    flash_v4_g4c4,
    flash_v4_wb,
    flash_v4_wl,
    flash_v4_g0x0,
    flash_v4_g0x1,
    flash_v4_c0,
    flash_v4_u0,
)))]
macro_rules! acr_icache {
    ($($t:tt)*) => {};
}

/// Cache state saved before an operation.
pub(super) struct State {
    #[allow(unused)]
    icache: bool,
    #[allow(unused)]
    dcache: bool,
}

/// Disables the caches that must not be active while the flash is modified.
pub(super) fn before_op() -> State {
    #[allow(unused_mut)]
    let mut state = State {
        icache: false,
        dcache: false,
    };

    acr_icache! {
        let acr = pac::FLASH.acr().read();
        state.icache = acr.icen();
        acr_dcache! {
            state.dcache = acr.dcen();
        }
        pac::FLASH.acr().modify(|w| {
            w.set_icen(false);
            acr_dcache! {
                w.set_dcen(false);
            }
        });
    }

    #[cfg(flash_v2_f7)]
    {
        state.icache = pac::FLASH.acr().read().arten();
        pac::FLASH.acr().modify(|w| w.set_arten(false));
    }

    #[cfg(all(icache, any(flash_v4_l5, flash_v4_u5, flash_v4_u3, flash_v4_wba, flash_v5)))]
    {
        let icache = pac::ICACHE;
        state.icache = icache.cr().read().en();
        if state.icache {
            // Disabling the ICACHE also invalidates it.
            icache.fcr().write(|w| w.set_cbsyendf(true));
            icache.cr().modify(|w| w.set_en(false));
            while icache.cr().read().en() {}
        }
    }

    state
}

/// Flushes and restores the caches after an operation on `range` (address, length).
pub(super) fn after_op(state: &State, range: (u32, u32)) {
    let _ = (state, range);

    acr_icache! {
        pac::FLASH.acr().modify(|w| {
            w.set_icrst(true);
            acr_dcache! {
                w.set_dcrst(true);
            }
        });
        pac::FLASH.acr().modify(|w| {
            w.set_icrst(false);
            acr_dcache! {
                w.set_dcrst(false);
            }
        });
        pac::FLASH.acr().modify(|w| {
            w.set_icen(state.icache);
            acr_dcache! {
                w.set_dcen(state.dcache);
            }
        });
    }

    #[cfg(flash_v2_f7)]
    {
        pac::FLASH.acr().modify(|w| w.set_artrst(true));
        pac::FLASH.acr().modify(|w| w.set_artrst(false));
        pac::FLASH.acr().modify(|w| w.set_arten(state.icache));
    }

    #[cfg(all(icache, any(flash_v4_l5, flash_v4_u5, flash_v4_u3, flash_v4_wba, flash_v5)))]
    if state.icache {
        let icache = pac::ICACHE;
        // Wait for the invalidation triggered by the disable to finish.
        while icache.sr().read().busyf() {}
        icache.fcr().write(|w| w.set_cbsyendf(true));
        icache.cr().modify(|w| w.set_en(true));
    }

    #[cfg(any(
        flash_v2_f7,
        all(flash_v5_h7, not(feature = "_core-cm4")),
        flash_v5_h7ab,
        flash_v5_h7rs
    ))]
    {
        use cortex_m::peripheral::SCB;

        let mut scb = unsafe { cortex_m::Peripherals::steal() }.SCB;
        if SCB::dcache_enabled() {
            scb.clean_invalidate_dcache_by_address(range.0 as usize, range.1 as usize);
        }
        if SCB::icache_enabled() {
            scb.invalidate_icache();
        }
    }
}

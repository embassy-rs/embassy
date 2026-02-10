//! # Performance counters
//!
//! This module contains simple performance counters, intended to aid debugging
//! for metrics like "number of interrupts served" per peripheral, etc.
//!
//! When the `perf` feature is active, then the performance counters are functional.
//! When the `perf` feature is NOT active, the "increment" and "clear" interfaces are
//! still available, but act as a no-op.

#[cfg_attr(not(feature = "perf"), allow(unused_imports))]
use core::sync::atomic::{AtomicU32, Ordering};

use paste::paste;
macro_rules! define_counters {
    ($($name:ident),*) => {
        #[cfg_attr(not(feature = "perf"), allow(dead_code))]
        static PERF_COUNTERS: Counters = Counters::new();

        impl Counters {
            const fn new() -> Self {
                Self {
                    $(
                        #[cfg(feature = "perf")]
                        $name: AtomicU32::new(0),
                    )*
                }
            }
        }

        paste! {
            /// Reset all perf counters to zero
            #[cfg(feature = "perf")]
            pub fn clear_all() {
                $(
                    [<clear_ $name>]();
                )*
            }

            /// Get a snapshot report of all perf counters
            #[cfg(feature = "perf")]
            pub fn get_report() -> Report {
                Report {
                    $(
                        $name: ([<get_ $name>])(),
                    )*
                }
            }

            /// Get a snapshot report of all perf counters, and also reset all counters to zero
            #[cfg(feature = "perf")]
            pub fn get_report_and_clear() -> Report {
                Report {
                    $(
                        $name: ([<get_and_clear_ $name>])(),
                    )*
                }
            }


            $(
                /// Increment perf counter by 1
                #[inline(always)]
                pub fn [<incr_ $name>]() {
                    #[cfg(feature = "perf")]
                    PERF_COUNTERS.$name.fetch_add(1, Ordering::Relaxed);
                }

                /// Reset perf counter to zero
                #[inline(always)]
                pub fn [<clear_ $name>]() {
                    #[cfg(feature = "perf")]
                    PERF_COUNTERS.$name.store(0, Ordering::Relaxed);
                }

                /// Get current perf counter snapshot
                ///
                /// If the `perf` feature is not enabled, this always returns zero
                #[inline(always)]
                pub fn [<get_ $name>]() -> u32 {
                    #[cfg(feature = "perf")]
                    let ret = PERF_COUNTERS.$name.load(Ordering::Relaxed);
                    #[cfg(not(feature = "perf"))]
                    let ret = 0;
                    ret
                }

                /// Get current perf counter snapshot and reset the perf counter to zero
                ///
                /// If the `perf` feature is not enabled, this always returns zero
                #[inline(always)]
                pub fn [<get_and_clear_ $name>]() -> u32 {
                    #[cfg(feature = "perf")]
                    let ret = PERF_COUNTERS.$name.swap(0, Ordering::Relaxed);
                    #[cfg(not(feature = "perf"))]
                    let ret = 0;
                    ret
                }
            )*

        }

        /// A snapshot report of all perf counters
        #[cfg(feature = "perf")]
        #[cfg_attr(feature = "defmt", derive(defmt::Format))]
        pub struct Report {
            $(
                pub $name: u32,
            )*
        }

        struct Counters {
            $(
                #[cfg(feature = "perf")]
                $name: AtomicU32,
            )*
        }
    };
}

// TODO: In the future, we may want to have more granular groupings of counters, behind
// features like `perf-interrupt`, `perf-interrupt-wake`, `perf-sleep`, etc. In that case,
// we might want a macro like the following, that enables a perf counter for ANY of the
// given features:
//
// ```rust
// define_counters! {
//   (["perf-interrupt", "perf-adc"], interrupt_adc0),
// };
//
// We can implement this later if we decide that "all of the perf counters" takes up too
// much static RAM space.
define_counters!(
    deep_sleeps,
    interrupt_adc0,
    interrupt_adc1,
    interrupt_adc2,
    interrupt_adc3,
    interrupt_cdog0,
    interrupt_ctimer0,
    interrupt_ctimer0_wake,
    interrupt_ctimer1,
    interrupt_ctimer1_wake,
    interrupt_ctimer2,
    interrupt_ctimer2_wake,
    interrupt_ctimer3,
    interrupt_ctimer3_wake,
    interrupt_ctimer4,
    interrupt_ctimer4_wake,
    interrupt_edma0,
    interrupt_edma0_wake,
    interrupt_gpio0,
    interrupt_gpio0_wake,
    interrupt_gpio1,
    interrupt_gpio1_wake,
    interrupt_gpio2,
    interrupt_gpio2_wake,
    interrupt_gpio3,
    interrupt_gpio3_wake,
    interrupt_gpio4,
    interrupt_gpio4_wake,
    interrupt_i2c0,
    interrupt_i2c0_wake,
    interrupt_i2c1,
    interrupt_i2c1_wake,
    interrupt_i2c2,
    interrupt_i2c2_wake,
    interrupt_i2c3,
    interrupt_i2c3_wake,
    interrupt_i3c0,
    interrupt_i3c0_wake,
    interrupt_lpuart0,
    interrupt_lpuart0_wake,
    interrupt_lpuart1,
    interrupt_lpuart1_wake,
    interrupt_lpuart2,
    interrupt_lpuart2_wake,
    interrupt_lpuart3,
    interrupt_lpuart3_wake,
    interrupt_lpuart4,
    interrupt_lpuart4_wake,
    interrupt_lpuart5,
    interrupt_lpuart5_wake,
    interrupt_ostimer,
    interrupt_ostimer_alarm,
    interrupt_rtc0,
    interrupt_rtc0_wake,
    interrupt_spi0,
    interrupt_spi0_wake,
    interrupt_spi1,
    interrupt_spi1_wake,
    interrupt_trng,
    interrupt_trng_wake,
    interrupt_wwdt,
    wfe_sleeps
);

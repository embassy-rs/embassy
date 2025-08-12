//! Helper functions for calculating PIO clock dividers

use fixed::traits::ToFixed;
use fixed::types::extra::U8;

use crate::clocks::clk_sys_freq;

/// Calculate a PIO clock divider value based on the desired target frequency.
///
/// # Arguments
///
/// * `target_hz` - The desired PIO clock frequency in Hz
///
/// # Returns
///
/// A fixed-point divider value suitable for use in a PIO state machine configuration
#[inline]
pub fn calculate_pio_clock_divider(target_hz: u32) -> fixed::FixedU32<U8> {
    // Requires a non-zero frequency
    assert!(target_hz > 0, "PIO clock frequency cannot be zero");

    // Calculate the divider
    let divider = (clk_sys_freq() + target_hz / 2) / target_hz;
    divider.to_fixed()
}

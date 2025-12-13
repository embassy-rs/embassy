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
    calculate_pio_clock_divider_inner(clk_sys_freq(), target_hz)
}

#[inline]
fn calculate_pio_clock_divider_inner(sys_freq: u32, target_hz: u32) -> fixed::FixedU32<U8> {
    // Requires a non-zero frequency
    assert!(target_hz > 0, "PIO clock frequency cannot be zero");

    // Calculate the divider
    let divider = (sys_freq + target_hz / 2) / target_hz;
    divider.to_fixed()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clock_divider_math() {
        // A simple divider that must have a fractional part.
        let divider = calculate_pio_clock_divider_inner(125_000_000, 40_000_000);
        let expected: fixed::FixedU32<U8> = 3.125.to_fixed();
        assert_eq!(divider, expected);

        // A system clk so high it would overflow a u32 if shifted left.
        let divider = calculate_pio_clock_divider_inner(2_000_000_000, 40_000);
        let expected: fixed::FixedU32<U8> = 50000.to_fixed();
        assert_eq!(divider, expected);

        // A divider that requires all 8 fractional bits.
        let divider = calculate_pio_clock_divider_inner(134_283_264, 16_777_216);
        let expected: fixed::FixedU32<U8> = 8.00390625.to_fixed();
        assert_eq!(divider, expected);
    }
}

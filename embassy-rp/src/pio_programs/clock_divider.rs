//! Helper functions for calculating PIO clock dividers

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
pub fn calculate_pio_clock_divider(target_hz: u32) -> fixed::FixedU32<U8> {
    calculate_pio_clock_divider_value(clk_sys_freq(), target_hz)
}

/// Calculate a PIO clock divider value based on the desired target frequency.
///
/// # Arguments
///
/// * `sys_hz` - The system clock frequency in Hz
/// * `target_hz` - The desired PIO clock frequency in Hz
///
/// # Returns
///
/// A fixed-point divider value suitable for use in a PIO state machine configuration
pub const fn calculate_pio_clock_divider_value(sys_hz: u32, target_hz: u32) -> fixed::FixedU32<U8> {
    // Requires a non-zero frequency
    core::assert!(target_hz > 0);

    // Compute the integer and fractional part of the divider.
    // Doing it this way allows us to avoid u64 division while
    // maintaining precision.
    let integer = sys_hz / target_hz;
    let remainder = sys_hz % target_hz;
    let frac = (remainder << 8) / target_hz;

    let result = integer << 8 | frac;

    // Ensure the result will fit in 16+8 bits.
    core::assert!(result <= 0xffff_ff);
    // The clock divider can't be used to go faster than the system clock.
    core::assert!(result >= 0x0001_00);

    fixed::FixedU32::from_bits(result)
}

#[cfg(test)]
mod tests {
    use fixed::traits::ToFixed;

    use super::*;

    #[test]
    fn clock_divider_math() {
        // A simple divider that must have a fractional part.
        let divider = calculate_pio_clock_divider_value(125_000_000, 40_000_000);
        let expected: fixed::FixedU32<U8> = 3.125.to_fixed();
        assert_eq!(divider, expected);

        // A system clk so high it would overflow a u32 if shifted left.
        let divider = calculate_pio_clock_divider_value(2_000_000_000, 40_000);
        let expected: fixed::FixedU32<U8> = 50000.to_fixed();
        assert_eq!(divider, expected);

        // A divider that requires all 8 fractional bits.
        let divider = calculate_pio_clock_divider_value(134_283_264, 16_777_216);
        let expected: fixed::FixedU32<U8> = 8.00390625.to_fixed();
        assert_eq!(divider, expected);
    }
}

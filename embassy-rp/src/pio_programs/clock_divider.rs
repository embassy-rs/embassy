//! Helper functions for calculating PIO clock dividers

use crate::clocks::clk_sys_freq;

/// Calculate a PIO clock divider value based on the desired target frequency.
///
/// # Arguments
///
/// * `target_hz` - The desired PIO clock frequency in Hz
///
/// # Returns
///
/// A divider value suitable for [`Config::clock_divider_x256`](crate::pio::Config::clock_divider_x256),
/// as a fixed-point number with 16 integer bits and 8 fractional bits.
pub fn calculate_pio_clock_divider_x256(target_hz: u32) -> u32 {
    calculate_pio_clock_divider_value_x256(clk_sys_freq(), target_hz)
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
/// A divider value suitable for [`Config::clock_divider_x256`](crate::pio::Config::clock_divider_x256),
/// as a fixed-point number with 16 integer bits and 8 fractional bits.
#[allow(clippy::unusual_byte_groupings)]
pub const fn calculate_pio_clock_divider_value_x256(sys_hz: u32, target_hz: u32) -> u32 {
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

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clock_divider_math() {
        // A simple divider that must have a fractional part.
        let divider = calculate_pio_clock_divider_value_x256(125_000_000, 40_000_000);
        assert_eq!(divider, 0x3_20); // 3.125

        // A system clk so high it would overflow a u32 if shifted left.
        let divider = calculate_pio_clock_divider_value_x256(2_000_000_000, 40_000);
        assert_eq!(divider, 50000 << 8);

        // A divider that requires all 8 fractional bits.
        let divider = calculate_pio_clock_divider_value_x256(134_283_264, 16_777_216);
        assert_eq!(divider, 0x8_01); // 8.00390625
    }
}

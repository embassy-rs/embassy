//! This module contains filter definitions and configuration that is common to both Classic CAN and FDCAN.

/// Possible errors when configuring a RX filter.
pub enum FilterConfigError {
    /// This error indicates that you have attempted to configure
    /// too many filters. Your split of standard and extended
    /// filters must adhere to the constaint:
    /// 2*(num_extended) + (num_standard) ≤ 32
    TooManyFilters,

    /// This error indicates that you have attempted configuring
    /// a standard ID that is over 11 bits long. Standard IDs must
    /// be <= 0x7FF.
    /// 
    /// The `u32` inside represents the standard ID that was rejected.
    StandardIdTooLarge(u32),

    /// This error indicates that you have attempted configuring
    /// an extended ID that is over 29 bits long. Extended IDs must
    /// be <= 0x1FFF_FFFF.
    /// 
    /// /// The `u32` inside represents the extended ID that was rejected.
    ExtendedIdTooLarge(u32),
}

/// Struct for configuring your standard and extended ID filters, via this general syntax:
/// ```rust
/// let config = FilterConfig {
///     standard_ids: [0x01, 0x02, 0x03],
///     extended_ids: [0xA1, 0xA2, 0xA3, 0xA4],
/// }
/// ```
/// Your config must adhere to the constaint: 2*(num_extended) + (num_standard) ≤ 32
pub struct FilterConfig<'a> {
    pub standard_ids: &'a [u32],
    pub extended_ids: &'a [u32],
}

impl FilterConfig<'_> {
    pub(in crate::flexcan) fn validate(&self) -> Result<(), FilterConfigError> {
            // Check if there are too many filters configured.
            if 2*self.extended_ids.len() + self.standard_ids.len() > 32 {
                return Err(FilterConfigError::TooManyFilters);
            }

            // Check if any of the standard IDs are too big.
            for &id in self.standard_ids {
                const MAX_STANDARD_ID: u32 = 0x7FF;
                if id > MAX_STANDARD_ID { return Err(FilterConfigError::StandardIdTooLarge(id)); }
            }

            // Check if any of the extended IDs are too big.
            for &id in self.extended_ids {
                const MAX_EXTENDED_ID: u32 = 0x1FFF_FFFF;
                if id > MAX_EXTENDED_ID { return Err(FilterConfigError::ExtendedIdTooLarge(id)); }
            }

            Ok(())
    }
}
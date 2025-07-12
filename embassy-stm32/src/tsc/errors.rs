/// Represents errors that can occur when configuring or validating TSC pin groups.
#[derive(Debug)]
pub enum GroupError {
    /// Error when a group has no sampling capacitor
    NoSamplingCapacitor,
    /// Error when a group has neither channel IOs nor a shield IO
    NoChannelOrShield,
    /// Error when a group has both channel IOs and a shield IO
    MixedChannelAndShield,
    /// Error when there is more than one shield IO across all groups
    MultipleShields,
}

/// Error returned when attempting to set an invalid channel pin as active in the TSC.
#[derive(Debug)]
pub enum AcquisitionBankError {
    /// Indicates that one or more of the provided pins is not a valid channel pin.
    InvalidChannelPin,
    /// Indicates that multiple channels from the same group were provided.
    MultipleChannelsPerGroup,
}

/// Peripheral state
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone, Copy)]
pub enum State {
    /// Peripheral is being setup or reconfigured
    Reset,
    /// Ready to start acquisition
    Ready,
    /// In process of sensor acquisition
    Busy,
    /// Error occured during acquisition
    Error,
}

/// Individual group status checked after acquisition reported as complete
/// For groups with multiple channel pins, may take longer because acquisitions
/// are done sequentially. Check this status before pulling count for each
/// sampled channel
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone, Copy)]
pub enum GroupStatus {
    /// Acquisition for channel still in progress
    Ongoing,
    /// Acquisition either not started or complete
    Complete,
}

/// Group identifier used to interrogate status
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[allow(missing_docs)]
#[derive(PartialEq, Clone, Copy)]
pub enum Group {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    #[cfg(any(tsc_v2, tsc_v3))]
    Seven,
    #[cfg(tsc_v3)]
    Eight,
}

impl Into<usize> for Group {
    fn into(self) -> usize {
        match self {
            Group::One => 0,
            Group::Two => 1,
            Group::Three => 2,
            Group::Four => 3,
            Group::Five => 4,
            Group::Six => 5,
            #[cfg(any(tsc_v2, tsc_v3))]
            Group::Seven => 6,
            #[cfg(tsc_v3)]
            Group::Eight => 7,
        }
    }
}

/// Error returned when attempting to create a Group from an invalid numeric value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidGroupError {
    invalid_value: usize,
}

impl InvalidGroupError {
    #[allow(missing_docs)]
    pub fn new(value: usize) -> Self {
        Self { invalid_value: value }
    }
}

impl TryFrom<usize> for Group {
    type Error = InvalidGroupError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Group::One),
            1 => Ok(Group::Two),
            2 => Ok(Group::Three),
            3 => Ok(Group::Four),
            4 => Ok(Group::Five),
            5 => Ok(Group::Six),
            #[cfg(any(tsc_v2, tsc_v3))]
            6 => Ok(Group::Two),
            #[cfg(tsc_v3)]
            7 => Ok(Group::Two),
            n => Err(InvalidGroupError::new(n)),
        }
    }
}

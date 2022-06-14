/// sub-GHz radio operating mode.
///
/// See `Get_Status` under section 5.8.5 "Communication status information commands"
/// in the reference manual.
///
/// This is returned by [`Status::mode`].
#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum StatusMode {
    /// Standby mode with RC 13MHz.
    StandbyRc = 0x2,
    /// Standby mode with HSE32.
    StandbyHse = 0x3,
    /// Frequency Synthesis mode.
    Fs = 0x4,
    /// Receive mode.
    Rx = 0x5,
    /// Transmit mode.
    Tx = 0x6,
}

impl StatusMode {
    /// Create a new `StatusMode` from bits.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::StatusMode;
    ///
    /// assert_eq!(StatusMode::from_raw(0x2), Ok(StatusMode::StandbyRc));
    /// assert_eq!(StatusMode::from_raw(0x3), Ok(StatusMode::StandbyHse));
    /// assert_eq!(StatusMode::from_raw(0x4), Ok(StatusMode::Fs));
    /// assert_eq!(StatusMode::from_raw(0x5), Ok(StatusMode::Rx));
    /// assert_eq!(StatusMode::from_raw(0x6), Ok(StatusMode::Tx));
    /// // Other values are reserved
    /// assert_eq!(StatusMode::from_raw(0), Err(0));
    /// ```
    pub const fn from_raw(bits: u8) -> Result<Self, u8> {
        match bits {
            0x2 => Ok(StatusMode::StandbyRc),
            0x3 => Ok(StatusMode::StandbyHse),
            0x4 => Ok(StatusMode::Fs),
            0x5 => Ok(StatusMode::Rx),
            0x6 => Ok(StatusMode::Tx),
            _ => Err(bits),
        }
    }
}

/// Command status.
///
/// See `Get_Status` under section 5.8.5 "Communication status information commands"
/// in the reference manual.
///
/// This is returned by [`Status::cmd`].
#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CmdStatus {
    /// Data available to host.
    ///
    /// Packet received successfully and data can be retrieved.
    Avaliable = 0x2,
    /// Command time out.
    ///
    /// Command took too long to complete triggering a sub-GHz radio watchdog
    /// timeout.
    Timeout = 0x3,
    /// Command processing error.
    ///
    /// Invalid opcode or incorrect number of parameters.
    ProcessingError = 0x4,
    /// Command execution failure.
    ///
    /// Command successfully received but cannot be executed at this time,
    /// requested operating mode cannot be entered or requested data cannot be
    /// sent.
    ExecutionFailure = 0x5,
    /// Transmit command completed.
    ///
    /// Current packet transmission completed.
    Complete = 0x6,
}

impl CmdStatus {
    /// Create a new `CmdStatus` from bits.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::CmdStatus;
    ///
    /// assert_eq!(CmdStatus::from_raw(0x2), Ok(CmdStatus::Avaliable));
    /// assert_eq!(CmdStatus::from_raw(0x3), Ok(CmdStatus::Timeout));
    /// assert_eq!(CmdStatus::from_raw(0x4), Ok(CmdStatus::ProcessingError));
    /// assert_eq!(CmdStatus::from_raw(0x5), Ok(CmdStatus::ExecutionFailure));
    /// assert_eq!(CmdStatus::from_raw(0x6), Ok(CmdStatus::Complete));
    /// // Other values are reserved
    /// assert_eq!(CmdStatus::from_raw(0), Err(0));
    /// ```
    pub const fn from_raw(bits: u8) -> Result<Self, u8> {
        match bits {
            0x2 => Ok(CmdStatus::Avaliable),
            0x3 => Ok(CmdStatus::Timeout),
            0x4 => Ok(CmdStatus::ProcessingError),
            0x5 => Ok(CmdStatus::ExecutionFailure),
            0x6 => Ok(CmdStatus::Complete),
            _ => Err(bits),
        }
    }
}

/// Radio status.
///
/// This is returned by [`status`].
///
/// [`status`]: super::SubGhz::status
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Status(u8);

impl From<u8> for Status {
    fn from(x: u8) -> Self {
        Status(x)
    }
}
impl From<Status> for u8 {
    fn from(x: Status) -> Self {
        x.0
    }
}

impl Status {
    /// Create a new `Status` from a raw `u8` value.
    ///
    /// This is the same as `Status::from(u8)`, but in a `const` function.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::{CmdStatus, Status, StatusMode};
    ///
    /// const STATUS: Status = Status::from_raw(0x54_u8);
    /// assert_eq!(STATUS.mode(), Ok(StatusMode::Rx));
    /// assert_eq!(STATUS.cmd(), Ok(CmdStatus::Avaliable));
    /// ```
    pub const fn from_raw(value: u8) -> Status {
        Status(value)
    }

    /// sub-GHz radio operating mode.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::{Status, StatusMode};
    ///
    /// let status: Status = 0xACu8.into();
    /// assert_eq!(status.mode(), Ok(StatusMode::StandbyRc));
    /// ```
    pub const fn mode(&self) -> Result<StatusMode, u8> {
        StatusMode::from_raw((self.0 >> 4) & 0b111)
    }

    /// Command status.
    ///
    /// This method frequently returns reserved values such as `Err(1)`.
    /// ST support has confirmed that this is normal and should be ignored.
    ///
    /// # Example
    ///
    /// ```
    /// use stm32wlxx_hal::subghz::{CmdStatus, Status};
    ///
    /// let status: Status = 0xACu8.into();
    /// assert_eq!(status.cmd(), Ok(CmdStatus::Complete));
    /// ```
    pub const fn cmd(&self) -> Result<CmdStatus, u8> {
        CmdStatus::from_raw((self.0 >> 1) & 0b111)
    }
}

impl core::fmt::Debug for Status {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Status")
            .field("mode", &self.mode())
            .field("cmd", &self.cmd())
            .finish()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for Status {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "Status {{ mode: {}, cmd: {} }}",
            self.mode(),
            self.cmd()
        )
    }
}

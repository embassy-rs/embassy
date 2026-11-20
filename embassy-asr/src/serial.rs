//! Configuration enums shared by the UART and LPUART drivers.

/// Number of data bits per character.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum DataBits {
    /// 5 data bits.
    DataBits5,
    /// 6 data bits.
    DataBits6,
    /// 7 data bits.
    DataBits7,
    /// 8 data bits.
    DataBits8,
}

/// Number of stop bits.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum StopBits {
    /// One stop bit.
    STOP1,
    /// Two stop bits.
    STOP2,
}

/// Hardware RTS/CTS flow control.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum FlowControl {
    /// No RTS/CTS.
    None,
    /// RTS only.
    Rts,
    /// CTS only.
    Cts,
    /// RTS and CTS.
    RtsCts,
}

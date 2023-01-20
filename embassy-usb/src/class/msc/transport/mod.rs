use embassy_usb_driver::EndpointError;

pub mod bulk_only;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum DataPipeError {
    /// Exceeded the host requested transfer size
    TransferSizeExceeded,
    /// Transfer was finalized by sending a short (non-full) packet
    TransferFinalized,
    /// USB driver endpoint error
    EndpointError(EndpointError),
}

impl From<EndpointError> for DataPipeError {
    fn from(e: EndpointError) -> Self {
        Self::EndpointError(e)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CommandError {
    PipeError(DataPipeError),
    CommandError,
}

/// A pipe that allows [CommandSetHandler] to write command-specific data.
pub trait DataPipeIn {
    /// Sends data to host.
    ///
    /// Must be called only once or in lengths multiple of maximum USB packet size.
    /// Otherwise, incomplete USB packet is interpreted as end of transfer.
    async fn write(&mut self, buf: &[u8]) -> Result<(), DataPipeError>;
}

/// A pipe that allows [CommandSetHandler] to read command-specific data.
pub trait DataPipeOut {
    /// Receives data to host.
    ///
    /// Must be called only once or in lengths multiple of maximum USB packet size.
    /// Otherwise, incomplete USB packet is interpreted as end of transfer.
    async fn read(&mut self, buf: &mut [u8]) -> Result<(), DataPipeError>;
}

/// Implemented by mass storage subclasses (i.e. SCSI).
///
/// This trait is tailored to bulk-only transport and may require changes for other transports.
pub trait CommandSetHandler {
    /// Handles command where data is sent to device.
    async fn command_out(&mut self, lun: u8, cmd: &[u8], pipe: &mut impl DataPipeOut) -> Result<(), CommandError>;

    /// Handles command where data is sent to host.
    async fn command_in(&mut self, lun: u8, cmd: &[u8], pipe: &mut impl DataPipeIn) -> Result<(), CommandError>;
}

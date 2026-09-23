use embassy_usb::driver::{Driver, Endpoint, EndpointError, EndpointIn, EndpointOut};
use embedded_io::{Error as IoError, ErrorKind as IoErrorKind};
use embedded_io_async::{ErrorType, Read, Write};

#[derive(Debug, defmt::Format)]
pub enum UsbIoError {
    Disconnected,
    Other,
}

impl core::fmt::Display for UsbIoError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            UsbIoError::Disconnected => f.write_str("Disconnected"),
            UsbIoError::Other => f.write_str("Other"),
        }
    }
}

impl core::error::Error for UsbIoError {}

impl IoError for UsbIoError {
    fn kind(&self) -> IoErrorKind {
        IoErrorKind::Other
    }
}

/// Interface class 0x08 (Mass Storage), subclass 0xFF (vendor), protocol 0xFF (vendor).
pub struct BulkReaderWriter<'d, D: Driver<'d>> {
    ep_out: D::EndpointOut, // host → device (commands)
    ep_in: D::EndpointIn,   // device → host (responses)
    /// Leftover bytes from a bulk packet that was larger than the requested read size.
    leftover: Option<(usize, usize, [u8; 512])>,
}

impl<'d, D: Driver<'d>> BulkReaderWriter<'d, D> {
    pub fn new(ep_out: D::EndpointOut, ep_in: D::EndpointIn) -> Self {
        Self {
            ep_out,
            ep_in,
            leftover: None,
        }
    }

    pub async fn wait_connection(&mut self) {
        self.ep_out.wait_enabled().await;
    }

    fn map_err(e: EndpointError) -> UsbIoError {
        match e {
            EndpointError::BufferOverflow => UsbIoError::Other,
            EndpointError::Disabled => UsbIoError::Disconnected,
        }
    }
}

impl<'d, D: Driver<'d>> ErrorType for BulkReaderWriter<'d, D> {
    type Error = UsbIoError;
}

impl<'d, D: Driver<'d>> Read for BulkReaderWriter<'d, D> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        if buf.is_empty() {
            return Ok(0);
        }

        // 1) Drain leftover bytes first if available.
        if let Some((offset, size, packet)) = &mut self.leftover {
            let available = *size - *offset;
            let to_copy = core::cmp::min(available, buf.len());
            buf[..to_copy].copy_from_slice(&packet[*offset..*offset + to_copy]);
            *offset += to_copy;

            if *offset >= *size {
                self.leftover = None;
            }
            return Ok(to_copy);
        }

        // 2) No leftover → read a fresh bulk packet (up to 512 bytes).
        let mut packet_buf = [0u8; 512];
        let n = self.ep_out.read(&mut packet_buf).await.map_err(Self::map_err)?;

        if n == 0 {
            return Ok(0);
        }

        let to_copy = core::cmp::min(n, buf.len());
        buf[..to_copy].copy_from_slice(&packet_buf[..to_copy]);

        // Store remaining bytes, if any.
        if n > to_copy {
            let mut leftover_buf = [0u8; 512];
            leftover_buf[..n - to_copy].copy_from_slice(&packet_buf[to_copy..n]);
            self.leftover = Some((0, n - to_copy, leftover_buf));
        }

        Ok(to_copy)
    }
}

impl<'d, D: Driver<'d>> Write for BulkReaderWriter<'d, D> {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        let to_write = core::cmp::min(buf.len(), 512);
        if to_write == 0 {
            return Ok(0);
        }
        self.ep_in.write(&buf[..to_write]).await.map_err(Self::map_err)?;
        Ok(to_write)
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

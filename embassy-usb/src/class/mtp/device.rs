//! Media Transfer Protocol (MTP / PTP) USB device class implementation.

use core::cell::Cell;
use core::mem::MaybeUninit;

use embassy_sync::blocking_mutex::CriticalSectionMutex;

use super::types::*;
use crate::class::read_packet;
use crate::control::{InResponse, OutResponse, Recipient, Request, RequestType};
use crate::driver::{Driver, Endpoint, EndpointIn};
use crate::types::{InterfaceNumber, StringIndex};
use crate::{Builder, EndpointOptions, Handler, InterfaceHandler, msos};

/// Configuration for the MTP device class.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Config<'d> {
    /// Optional interface string descriptor. Defaults to `Some("MTP")`.
    pub interface_name: Option<&'d str>,
    /// Whether to emit an MS OS 2.0 Compatible ID (`"MTP"`) descriptor so Windows
    /// binds its native MTP/WPD driver automatically.
    ///
    /// When enabled, the `Builder` must be supplied with a non-empty `msos_descriptor_buf`.
    pub msos_descriptor: bool,
    /// Polling interval in ms for the Interrupt-IN event endpoint. Defaults to `16`.
    pub interrupt_poll_ms: u8,
    /// Max packet size for the Interrupt-IN event endpoint (must be `>= 24`). Defaults to `64`.
    pub interrupt_max_packet_size: u16,
    /// Options for the Bulk-OUT endpoint.
    pub out_endpoint_options: EndpointOptions,
    /// Options for the Bulk-IN endpoint.
    pub in_endpoint_options: EndpointOptions,
}

impl<'d> Config<'d> {
    /// Creates a default MTP configuration.
    pub const fn new() -> Self {
        Self {
            interface_name: Some("MTP"),
            msos_descriptor: false,
            interrupt_poll_ms: 16,
            interrupt_max_packet_size: 64,
            out_endpoint_options: EndpointOptions::new(),
            in_endpoint_options: EndpointOptions::new(),
        }
    }

    /// Sets the interface string descriptor name.
    pub const fn with_interface_name(mut self, name: Option<&'d str>) -> Self {
        self.interface_name = name;
        self
    }

    /// Enables or disables emitting the MS OS 2.0 `"MTP"` compatible ID descriptor.
    pub const fn with_msos(mut self, enabled: bool) -> Self {
        self.msos_descriptor = enabled;
        self
    }
}

impl<'d> Default for Config<'d> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug)]
struct ControlInner {
    device_status: u16,
    cancel_tx_id: Option<u32>,
    reset_pending: bool,
}

impl ControlInner {
    const fn new() -> Self {
        Self {
            device_status: resp::OK,
            cancel_tx_id: None,
            reset_pending: false,
        }
    }
}

/// Shared state between the USB control handler and [`MtpClass`].
pub struct ControlShared {
    inner: CriticalSectionMutex<Cell<ControlInner>>,
}

impl ControlShared {
    /// Creates a new `ControlShared` with status [`resp::OK`].
    pub const fn new() -> Self {
        Self {
            inner: CriticalSectionMutex::new(Cell::new(ControlInner::new())),
        }
    }

    /// Sets the status code returned to the host by `REQ_GET_DEVICE_STATUS` (`0x67`).
    pub fn set_device_status(&self, status: u16) {
        self.inner.lock(|cell| {
            let mut inner = cell.get();
            inner.device_status = status;
            cell.set(inner);
        });
    }

    /// Returns the current device status code.
    pub fn device_status(&self) -> u16 {
        self.inner.lock(|cell| cell.get().device_status)
    }

    /// Returns and clears the cancelled transaction ID if the host sent `REQ_CANCEL` (`0x64`).
    pub fn take_cancel(&self) -> Option<u32> {
        self.inner.lock(|cell| {
            let mut inner = cell.get();
            let tx = inner.cancel_tx_id.take();
            cell.set(inner);
            tx
        })
    }

    /// Returns and clears whether the host sent `REQ_RESET` (`0x66`).
    pub fn take_reset(&self) -> bool {
        self.inner.lock(|cell| {
            let mut inner = cell.get();
            let reset = inner.reset_pending;
            inner.reset_pending = false;
            cell.set(inner);
            reset
        })
    }

    fn set_cancel(&self, tx_id: u32) {
        self.inner.lock(|cell| {
            let mut inner = cell.get();
            inner.cancel_tx_id = Some(tx_id);
            inner.device_status = resp::DEVICE_BUSY;
            cell.set(inner);
        });
    }

    fn request_reset(&self) {
        self.inner.lock(|cell| {
            let mut inner = cell.get();
            inner.device_status = resp::OK;
            inner.cancel_tx_id = None;
            inner.reset_pending = true;
            cell.set(inner);
        });
    }

    fn reset(&self) {
        self.inner.lock(|cell| {
            cell.set(ControlInner::new());
        });
    }
}

impl Default for ControlShared {
    fn default() -> Self {
        Self::new()
    }
}

/// Persistent state for [`MtpClass`].
pub struct State<'d> {
    control: MaybeUninit<Control<'d>>,
    shared: ControlShared,
}

impl<'d> State<'d> {
    /// Creates a new `State`.
    pub const fn new() -> Self {
        Self {
            control: MaybeUninit::uninit(),
            shared: ControlShared::new(),
        }
    }
}

impl<'d> Default for State<'d> {
    fn default() -> Self {
        Self::new()
    }
}

struct Control<'d> {
    iface: InterfaceNumber,
    iface_string: Option<StringIndex>,
    interface_name: Option<&'d str>,
    shared: &'d ControlShared,
}

impl<'d> Handler for Control<'d> {
    fn reset(&mut self) {
        self.shared.reset();
    }

    fn get_string(&mut self, index: StringIndex, _lang_id: u16) -> Option<&str> {
        if Some(index) == self.iface_string {
            self.interface_name
        } else {
            None
        }
    }

    fn control_out(&mut self, req: Request, data: &[u8]) -> Option<OutResponse> {
        if (req.request_type, req.recipient) != (RequestType::Class, Recipient::Interface)
            || req.index != u16::from(self.iface.0)
        {
            return None;
        }

        match req.request {
            REQ_CANCEL => {
                if data.len() != 6 {
                    return Some(OutResponse::Rejected);
                }
                let code = u16::from_le_bytes([data[0], data[1]]);
                if code != CANCEL_CODE_CANCELLATION {
                    return Some(OutResponse::Rejected);
                }
                let tx_id = u32::from_le_bytes([data[2], data[3], data[4], data[5]]);
                self.shared.set_cancel(tx_id);
                Some(OutResponse::Accepted)
            }
            REQ_RESET => {
                if !data.is_empty() || req.length != 0 {
                    return Some(OutResponse::Rejected);
                }
                self.shared.request_reset();
                Some(OutResponse::Accepted)
            }
            _ => Some(OutResponse::Rejected),
        }
    }

    fn control_in<'a>(&'a mut self, req: Request, buf: &'a mut [u8]) -> Option<InResponse<'a>> {
        if (req.request_type, req.recipient) != (RequestType::Class, Recipient::Interface)
            || req.index != u16::from(self.iface.0)
        {
            return None;
        }

        match req.request {
            REQ_GET_DEVICE_STATUS => {
                let status = self.shared.device_status();
                let mut payload = [0u8; 4];
                payload[0..2].copy_from_slice(&4u16.to_le_bytes());
                payload[2..4].copy_from_slice(&status.to_le_bytes());
                let len = core::cmp::min(usize::from(req.length), payload.len());
                if buf.len() < len {
                    return Some(InResponse::Rejected);
                }
                buf[..len].copy_from_slice(&payload[..len]);
                Some(InResponse::Accepted(&buf[..len]))
            }
            REQ_GET_EXT_EVENT_DATA => Some(InResponse::Rejected),
            _ => Some(InResponse::Rejected),
        }
    }
}

impl<'d> InterfaceHandler for Control<'d> {
    fn owns(&self, iface: InterfaceNumber) -> bool {
        iface == self.iface
    }
}

/// USB Media Transfer Protocol (MTP) device class.
pub struct MtpClass<'d, D: Driver<'d>> {
    read_ep: D::EndpointOut,
    write_ep: D::EndpointIn,
    int_ep: D::EndpointIn,
    iface: InterfaceNumber,
    shared: &'d ControlShared,
}

impl<'d, D: Driver<'d>> MtpClass<'d, D> {
    /// Creates a new `MtpClass` and registers its USB Still Image / MTP interface.
    pub fn new(builder: &mut Builder<'d, D>, state: &'d mut State<'d>, config: Config<'d>) -> Self {
        assert!(
            usize::from(config.interrupt_max_packet_size) >= MAX_EVENT_LEN,
            "MTP interrupt_max_packet_size must be at least {} bytes",
            MAX_EVENT_LEN
        );

        let iface_string = config.interface_name.map(|_| builder.string());

        if config.msos_descriptor && builder.msos_writer().is_empty() {
            builder.msos_descriptor(msos::windows_version::WIN8_1, 0x02);
        }

        let mut func = builder.function(USB_CLASS_STILL_IMAGE, USB_SUBCLASS_STILL_IMAGE, USB_PROTOCOL_MTP);
        if config.msos_descriptor {
            func.msos_feature(msos::CompatibleIdFeatureDescriptor::new("MTP", ""));
        }

        let mut interface = func.interface();
        let iface = interface.interface_number();
        let mut alt = interface.alt_setting(
            USB_CLASS_STILL_IMAGE,
            USB_SUBCLASS_STILL_IMAGE,
            USB_PROTOCOL_MTP,
            iface_string,
        );
        let write_ep = alt.endpoint_bulk_in(None, config.in_endpoint_options);
        let read_ep = alt.endpoint_bulk_out(None, config.out_endpoint_options);
        let int_ep = alt.endpoint_interrupt_in(None, config.interrupt_max_packet_size, config.interrupt_poll_ms);
        drop(func);

        let shared = &state.shared;
        let control = state.control.write(Control {
            iface,
            iface_string,
            interface_name: config.interface_name,
            shared,
        });
        builder.interface_handler(control);

        Self {
            read_ep,
            write_ep,
            int_ep,
            iface,
            shared,
        }
    }

    /// Returns the Bulk endpoint max packet size in bytes.
    pub fn max_packet_size(&self) -> u16 {
        self.read_ep.info().max_packet_size
    }

    /// Returns the USB interface number assigned to this class instance.
    pub const fn interface(&self) -> InterfaceNumber {
        self.iface
    }

    /// Returns a reference to the shared control state.
    pub const fn shared(&self) -> &'d ControlShared {
        self.shared
    }

    /// Sets the status code returned to the host by `REQ_GET_DEVICE_STATUS` (`0x67`).
    pub fn set_device_status(&self, status: u16) {
        self.shared.set_device_status(status);
    }

    /// Returns and clears the cancelled transaction ID if the host sent `REQ_CANCEL` (`0x64`).
    pub fn take_cancel(&self) -> Option<u32> {
        self.shared.take_cancel()
    }

    /// Returns and clears whether the host sent `REQ_RESET` (`0x66`).
    pub fn take_reset(&self) -> bool {
        self.shared.take_reset()
    }

    /// Waits until the USB host configures and enables the MTP interface endpoints.
    pub async fn wait_connection(&mut self) {
        self.read_ep.wait_enabled().await;
    }

    /// Reads the next MTP Command container from the Bulk-OUT endpoint.
    ///
    /// `buf` must be at least `max_packet_size()` bytes long. Any leading zero-length
    /// packet (for example, a trailing ZLP from a prior Bulk-OUT data container whose
    /// length was a multiple of `max_packet_size`) is skipped automatically.
    pub async fn read_command(&mut self, buf: &mut [u8]) -> Result<CommandBlock, MtpError> {
        let mps = usize::from(self.read_ep.info().max_packet_size);
        if buf.len() < mps || buf.len() < CONTAINER_HEADER_LEN {
            return Err(MtpError::BufferTooSmall);
        }

        let n = loop {
            let n = read_packet(&mut self.read_ep, buf).await?;
            if n > 0 {
                break n;
            }
        };

        CommandBlock::decode(&buf[..n])
    }

    /// Starts reading an MTP Data-OUT container from the Bulk-OUT endpoint.
    ///
    /// `buf` is used as a packet staging buffer and must be at least `max_packet_size()`
    /// bytes long. Returns a [`DataOutReader`] to stream the payload in arbitrary chunk sizes.
    pub async fn start_data_out<'a>(&'a mut self, buf: &'a mut [u8]) -> Result<DataOutReader<'a, 'd, D>, MtpError> {
        let mps = usize::from(self.read_ep.info().max_packet_size);
        if buf.len() < mps || buf.len() < CONTAINER_HEADER_LEN {
            return Err(MtpError::BufferTooSmall);
        }

        let n = loop {
            let n = read_packet(&mut self.read_ep, buf).await?;
            if n > 0 {
                break n;
            }
        };

        if n < CONTAINER_HEADER_LEN {
            return Err(MtpError::PacketTooShort);
        }
        let header = ContainerHeader::decode(&buf[..CONTAINER_HEADER_LEN])?;
        if header.container_type != ContainerType::Data {
            return Err(MtpError::UnexpectedContainerType {
                expected: ContainerType::Data,
                actual: header.container_type,
            });
        }

        let expected_payload = if header.length == STREAMING_CONTAINER_LEN {
            None
        } else if header.length < CONTAINER_HEADER_LEN as u32 {
            return Err(MtpError::InvalidLength);
        } else {
            Some(header.length - CONTAINER_HEADER_LEN as u32)
        };

        let initial_payload = n - CONTAINER_HEADER_LEN;
        let short = n < mps;

        let mut eof = false;
        let mut truncated = false;
        if let Some(expected) = expected_payload {
            if initial_payload as u32 > expected {
                return Err(MtpError::LengthMismatch);
            }
            if initial_payload as u32 == expected {
                eof = true;
            } else if short {
                eof = true;
                truncated = true;
            }
        } else if short {
            eof = true;
        }

        Ok(DataOutReader {
            ep: &mut self.read_ep,
            buf,
            header,
            expected_payload,
            pos: CONTAINER_HEADER_LEN,
            end: n,
            bytes_received: initial_payload as u64,
            bytes_read: 0,
            eof,
            truncated,
            mps,
        })
    }

    /// Reads an entire bounded MTP Data-OUT container into `payload_buf`.
    ///
    /// `pkt_buf` must be at least `max_packet_size()` bytes long. Returns the parsed
    /// [`ContainerHeader`] and the number of payload bytes written into `payload_buf`.
    pub async fn read_data_all(
        &mut self,
        pkt_buf: &mut [u8],
        payload_buf: &mut [u8],
    ) -> Result<(ContainerHeader, usize), MtpError> {
        let mut reader = self.start_data_out(pkt_buf).await?;
        let header = reader.header();
        if let Some(total) = reader.total_payload_len()
            && total as usize > payload_buf.len()
        {
            return Err(MtpError::BufferTooSmall);
        }

        let mut offset = 0usize;
        loop {
            if offset == payload_buf.len() {
                let mut probe = [0u8; 1];
                if reader.read(&mut probe).await? == 0 {
                    break;
                }
                return Err(MtpError::BufferTooSmall);
            }
            let n = reader.read(&mut payload_buf[offset..]).await?;
            if n == 0 {
                break;
            }
            offset += n;
        }
        Ok((header, offset))
    }

    /// Starts writing an MTP Data-IN container to the Bulk-IN endpoint.
    ///
    /// Pass `payload_len = Some(len)` when the total payload size is known (normal case),
    /// or `None` to write [`STREAMING_CONTAINER_LEN`] (`0xFFFF_FFFF`).
    ///
    /// `buf` is used as a transfer staging buffer and must be at least `max_packet_size()`
    /// bytes long. Larger multiples of `max_packet_size()` allow multi-packet USB transfers.
    pub fn start_data_in<'a>(
        &'a mut self,
        code: u16,
        transaction_id: u32,
        payload_len: Option<u32>,
        buf: &'a mut [u8],
    ) -> Result<DataInWriter<'a, 'd, D>, MtpError> {
        let mps = usize::from(self.write_ep.info().max_packet_size);
        let pkt_cap = (buf.len() / mps) * mps;
        if pkt_cap < mps || pkt_cap < CONTAINER_HEADER_LEN {
            return Err(MtpError::BufferTooSmall);
        }

        let container_len = match payload_len {
            Some(len) => len
                .checked_add(CONTAINER_HEADER_LEN as u32)
                .ok_or(MtpError::InvalidLength)?,
            None => STREAMING_CONTAINER_LEN,
        };

        ContainerHeader {
            length: container_len,
            container_type: ContainerType::Data,
            code,
            transaction_id,
        }
        .encode(buf)?;

        Ok(DataInWriter {
            ep: &mut self.write_ep,
            buf,
            pkt_cap,
            filled: CONTAINER_HEADER_LEN,
            expected_payload: payload_len,
            bytes_written: 0,
        })
    }

    /// Sends a complete MTP Data-IN container for an in-memory `payload`.
    ///
    /// `buf` must be at least `max_packet_size()` bytes long. Automatically appends a ZLP
    /// when `(12 + payload.len())` is an exact multiple of `max_packet_size()`.
    pub async fn write_data(
        &mut self,
        code: u16,
        transaction_id: u32,
        payload: &[u8],
        buf: &mut [u8],
    ) -> Result<(), MtpError> {
        let len = u32::try_from(payload.len()).map_err(|_| MtpError::InvalidLength)?;
        let mut writer = self.start_data_in(code, transaction_id, Some(len), buf)?;
        writer.write(payload).await?;
        writer.finish().await
    }

    /// Sends an MTP Response container on the Bulk-IN endpoint.
    pub async fn write_response(&mut self, response: &ResponseBlock) -> Result<(), MtpError> {
        let mut buf = [0u8; MAX_RESPONSE_LEN];
        let len = response.encode(&mut buf)?;
        self.write_ep.write_transfer(&buf[..len], true).await?;
        Ok(())
    }

    /// Sends an asynchronous MTP Event container on the Interrupt-IN endpoint.
    pub async fn write_event(&mut self, event: &EventBlock) -> Result<(), MtpError> {
        let mut buf = [0u8; MAX_EVENT_LEN];
        let len = event.encode(&mut buf)?;
        self.int_ep.write_transfer(&buf[..len], true).await?;
        Ok(())
    }
}

/// Streaming reader for an MTP Data-OUT container.
pub struct DataOutReader<'a, 'd, D: Driver<'d>> {
    ep: &'a mut D::EndpointOut,
    buf: &'a mut [u8],
    header: ContainerHeader,
    expected_payload: Option<u32>,
    pos: usize,
    end: usize,
    bytes_received: u64,
    bytes_read: u64,
    eof: bool,
    truncated: bool,
    mps: usize,
}

impl<'a, 'd, D: Driver<'d>> DataOutReader<'a, 'd, D> {
    /// Returns the decoded container header.
    pub const fn header(&self) -> ContainerHeader {
        self.header
    }

    /// Returns the operation code from the Data container header.
    pub const fn code(&self) -> u16 {
        self.header.code
    }

    /// Returns the transaction ID from the Data container header.
    pub const fn transaction_id(&self) -> u32 {
        self.header.transaction_id
    }

    /// Returns the declared payload length in bytes, or `None` for streaming containers.
    pub const fn total_payload_len(&self) -> Option<u32> {
        self.expected_payload
    }

    /// Returns the number of payload bytes consumed via [`read`](Self::read) so far.
    pub const fn bytes_read(&self) -> u64 {
        self.bytes_read
    }

    /// Returns the number of unread payload bytes remaining, if declared by the header.
    pub const fn remaining(&self) -> Option<u64> {
        match self.expected_payload {
            Some(total) => Some((total as u64).saturating_sub(self.bytes_read)),
            None => None,
        }
    }

    /// Returns `true` if all buffered and wire bytes of this Data container have been consumed.
    pub const fn is_eof(&self) -> bool {
        self.eof && self.pos >= self.end
    }

    /// Reads the next chunk of payload data into `out`.
    ///
    /// Returns `Ok(0)` once the entire Data container has been read.
    pub async fn read(&mut self, out: &mut [u8]) -> Result<usize, MtpError> {
        if out.is_empty() {
            return Ok(0);
        }

        if self.pos < self.end {
            let take = core::cmp::min(out.len(), self.end - self.pos);
            out[..take].copy_from_slice(&self.buf[self.pos..self.pos + take]);
            self.pos += take;
            self.bytes_read += take as u64;
            return Ok(take);
        }

        if self.truncated {
            return Err(MtpError::Truncated);
        }
        if self.eof {
            return Ok(0);
        }

        let n = read_packet(self.ep, self.buf).await?;
        let short = n < self.mps;
        if let Some(expected) = self.expected_payload {
            let new_total = self.bytes_received + n as u64;
            if new_total > expected as u64 {
                self.eof = true;
                return Err(MtpError::LengthMismatch);
            }
            self.bytes_received = new_total;
            if new_total == expected as u64 {
                self.eof = true;
            } else if short {
                self.eof = true;
                self.truncated = true;
                return Err(MtpError::Truncated);
            }
        } else if short {
            self.eof = true;
        }

        if n == 0 {
            return Ok(0);
        }

        let take = core::cmp::min(out.len(), n);
        out[..take].copy_from_slice(&self.buf[..take]);
        self.pos = take;
        self.end = n;
        self.bytes_read += take as u64;
        Ok(take)
    }

    /// Discards any remaining payload bytes in this Data container.
    pub async fn drain(mut self) -> Result<(), MtpError> {
        let mut sink = [0u8; 64];
        while self.read(&mut sink).await? > 0 {}
        Ok(())
    }
}

/// Streaming writer for an MTP Data-IN container.
pub struct DataInWriter<'a, 'd, D: Driver<'d>> {
    ep: &'a mut D::EndpointIn,
    buf: &'a mut [u8],
    pkt_cap: usize,
    filled: usize,
    expected_payload: Option<u32>,
    bytes_written: u64,
}

impl<'a, 'd, D: Driver<'d>> DataInWriter<'a, 'd, D> {
    /// Returns the number of payload bytes written so far.
    pub const fn bytes_written(&self) -> u64 {
        self.bytes_written
    }

    /// Queues a chunk of payload bytes, flushing full `max_packet_size`-aligned blocks to Bulk IN.
    pub async fn write(&mut self, mut data: &[u8]) -> Result<(), MtpError> {
        if let Some(expected) = self.expected_payload
            && self.bytes_written + data.len() as u64 > expected as u64
        {
            return Err(MtpError::LengthMismatch);
        }

        while !data.is_empty() {
            let space = self.pkt_cap - self.filled;
            let take = core::cmp::min(data.len(), space);
            self.buf[self.filled..self.filled + take].copy_from_slice(&data[..take]);
            self.filled += take;
            self.bytes_written += take as u64;
            data = &data[take..];

            if self.filled == self.pkt_cap {
                self.ep.write_transfer(&self.buf[..self.pkt_cap], false).await?;
                self.filled = 0;
            }
        }

        Ok(())
    }

    /// Completes the Data-IN container, flushing any remaining bytes and sending a ZLP if needed.
    pub async fn finish(self) -> Result<(), MtpError> {
        if let Some(expected) = self.expected_payload
            && self.bytes_written != expected as u64
        {
            return Err(MtpError::LengthMismatch);
        }

        if self.filled > 0 {
            self.ep.write_transfer(&self.buf[..self.filled], true).await?;
        } else {
            self.ep.write_transfer(&[], false).await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use embassy_futures::block_on;

    use super::*;
    use crate::device::Config as DeviceConfig;
    use crate::device::test_utils::TestDriver;
    use crate::driver::Direction;

    #[test]
    fn descriptors_and_msos_compatible_id() {
        let driver = TestDriver::default();
        let mut state = State::new();
        let mut config_descriptor = [0u8; 256];
        let mut bos_descriptor = [0u8; 256];
        let mut msos_descriptor = [0u8; 256];
        let mut control_buf = [0u8; 64];

        let mut builder = Builder::new(
            driver,
            DeviceConfig::new(0x1234, 0x5678),
            &mut config_descriptor,
            &mut bos_descriptor,
            &mut msos_descriptor,
            &mut control_buf,
        );

        let mtp = MtpClass::new(&mut builder, &mut state, Config::new().with_msos(true));
        assert_eq!(mtp.interface().0, 0);
        assert_eq!(mtp.max_packet_size(), 64);

        let usb = builder.build();
        drop(usb);

        // Still Image interface descriptor: class=0x06, subclass=0x01, protocol=0x01, 3 endpoints
        let iface = config_descriptor
            .windows(9)
            .find(|w| {
                w[0] == 9
                    && w[1] == 4
                    && w[5] == USB_CLASS_STILL_IMAGE
                    && w[6] == USB_SUBCLASS_STILL_IMAGE
                    && w[7] == USB_PROTOCOL_MTP
            })
            .expect("MTP interface descriptor not found");
        assert_eq!(iface[4], 3);

        // MS OS 2.0 compatible ID "MTP"
        assert!(msos_descriptor.windows(4).any(|w| w == b"MTP\0"));
    }

    #[test]
    fn control_requests_cancel_reset_and_status() {
        let shared = ControlShared::new();
        let mut control = Control {
            iface: InterfaceNumber(1),
            iface_string: Some(StringIndex(3)),
            interface_name: Some("MTP"),
            shared: &shared,
        };

        assert_eq!(control.get_string(StringIndex(3), 0x0409), Some("MTP"));
        assert_eq!(control.get_string(StringIndex(4), 0x0409), None);

        // Initial status = OK
        let mut buf = [0u8; 16];
        let status_req = Request {
            direction: Direction::In,
            request_type: RequestType::Class,
            recipient: Recipient::Interface,
            request: REQ_GET_DEVICE_STATUS,
            value: 0,
            index: 1,
            length: 4,
        };
        match control.control_in(status_req, &mut buf) {
            Some(InResponse::Accepted(d)) => {
                assert_eq!(d, &[0x04, 0x00, 0x01, 0x20]); // len=4, resp::OK (0x2001)
            }
            _ => panic!("expected status response"),
        }

        // Cancel transaction 0x11223344
        let cancel_req = Request {
            direction: Direction::Out,
            request_type: RequestType::Class,
            recipient: Recipient::Interface,
            request: REQ_CANCEL,
            value: 0,
            index: 1,
            length: 6,
        };
        let cancel_payload = [0x01, 0x40, 0x44, 0x33, 0x22, 0x11];
        assert!(matches!(
            control.control_out(cancel_req, &cancel_payload),
            Some(OutResponse::Accepted)
        ));
        assert_eq!(shared.device_status(), resp::DEVICE_BUSY);
        assert_eq!(shared.take_cancel(), Some(0x1122_3344));
        assert_eq!(shared.take_cancel(), None);

        // Reset clears busy/cancel and sets reset_pending
        let reset_req = Request {
            direction: Direction::Out,
            request_type: RequestType::Class,
            recipient: Recipient::Interface,
            request: REQ_RESET,
            value: 0,
            index: 1,
            length: 0,
        };
        assert!(matches!(
            control.control_out(reset_req, &[]),
            Some(OutResponse::Accepted)
        ));
        assert_eq!(shared.device_status(), resp::OK);
        assert!(shared.take_reset());
        assert!(!shared.take_reset());
    }

    #[test]
    fn bulk_command_data_streaming_and_zlp_round_trip() {
        let driver = TestDriver::default();
        let inner = driver.io();
        let mut state = State::new();
        let mut config_descriptor = [0u8; 256];
        let mut bos_descriptor = [0u8; 16];
        let mut msos_descriptor = [0u8; 16];
        let mut control_buf = [0u8; 64];

        let mut builder = Builder::new(
            driver,
            DeviceConfig::new(0x1234, 0x5678),
            &mut config_descriptor,
            &mut bos_descriptor,
            &mut msos_descriptor,
            &mut control_buf,
        );
        let mut mtp = MtpClass::new(&mut builder, &mut state, Config::new());
        let _usb = builder.build();
        // Configured by the host.
        for addr in [0x01, 0x81, 0x82] {
            inner.set_enabled(addr, true);
        }

        // Enqueue:
        // 1) A 64-byte Data-OUT packet (12-byte header + 52-byte payload) followed by trailing ZLP
        // 2) A subsequent Command container (SendObject)
        let mut data_pkt = [0u8; 64];
        ContainerHeader {
            length: 64,
            container_type: ContainerType::Data,
            code: op::SEND_OBJECT,
            transaction_id: 9,
        }
        .encode(&mut data_pkt)
        .unwrap();
        for (i, b) in data_pkt[12..].iter_mut().enumerate() {
            *b = i as u8;
        }

        let cmd = CommandBlock::new(op::GET_DEVICE_INFO, 10, &[]).unwrap();
        let mut cmd_pkt = [0u8; MAX_COMMAND_LEN];
        let cmd_len = cmd.encode(&mut cmd_pkt).unwrap();

        inner.push_read(0x01, &data_pkt);
        inner.push_read(0x01, &[]); // host trailing ZLP
        inner.push_read(0x01, &cmd_pkt[..cmd_len]);

        let mut pkt_buf = [0u8; 64];
        let mut payload_out = [0u8; 64];
        let (hdr, n) = block_on(mtp.read_data_all(&mut pkt_buf, &mut payload_out)).unwrap();
        assert_eq!(hdr.transaction_id, 9);
        assert_eq!(n, 52);
        assert_eq!(payload_out[0], 0);
        assert_eq!(payload_out[51], 51);

        // Next read_command must skip the trailing ZLP and decode GET_DEVICE_INFO
        let got_cmd = block_on(mtp.read_command(&mut pkt_buf)).unwrap();
        assert_eq!(got_cmd, cmd);

        // Stream 116-byte payload + 12-byte header = 128 bytes (2x 64-byte MPS -> requires ZLP)
        let payload_in = [0xABu8; 116];
        let mut in_buf = [0u8; 64];
        block_on(async {
            let mut writer = mtp.start_data_in(op::GET_OBJECT, 10, Some(116), &mut in_buf).unwrap();
            writer.write(&payload_in[..50]).await.unwrap();
            writer.write(&payload_in[50..]).await.unwrap();
            writer.finish().await.unwrap();
            mtp.write_response(&ResponseBlock::ok(10)).await.unwrap();
            mtp.write_event(&EventBlock::new(event::STORE_ADDED, 0xFFFF_FFFF))
                .await
                .unwrap();
        });

        let in_transfers = inner.writes(0x81);
        let int_transfers = inner.writes(0x82);
        // Expect 2x 64-byte chunks + 1 empty ZLP + 1 response
        assert_eq!(in_transfers.len(), 4);
        assert_eq!(in_transfers[0].0.len(), 64);
        assert_eq!(in_transfers[1].0.len(), 64);
        assert_eq!(in_transfers[2].0.len(), 0); // trailing ZLP
        assert_eq!(in_transfers[3].0.len(), 12); // ResponseBlock::ok(10)
        assert_eq!(int_transfers.len(), 1);
        assert_eq!(int_transfers[0].0.len(), 12);
    }
}

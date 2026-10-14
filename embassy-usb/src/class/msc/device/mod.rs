//! USB Mass Storage Class (MSC) implementation.
//!
//! This implements the USB Bulk-Only Transport (BOT) protocol with a
//! SCSI transparent command set suitable for a simple block device.

use core::cell::Cell;
use core::cmp::min;
use core::mem::MaybeUninit;

use embassy_futures::join::join;
use embassy_sync::blocking_mutex::CriticalSectionMutex;

use super::bot::{CBW_LEN, Cbw, Csw, CswStatus};
use super::scsi::*;
use super::{
    BOT_REQ_GET_MAX_LUN, BOT_REQ_RESET, SenseData, USB_CLASS_MSC, USB_PROTOCOL_BULK_ONLY, USB_SUBCLASS_SCSI_TRANSPARENT,
};
use crate::control::{InResponse, OutResponse, Recipient, Request, RequestType};
use crate::driver::{Driver, Endpoint, EndpointError, EndpointIn, EndpointOut};
use crate::types::InterfaceNumber;
use crate::{Builder, Handler};

mod block_device;
pub use block_device::*;

const VPD_PAGE_SUPPORTED_PAGES: u8 = 0x00;
const VPD_PAGE_UNIT_SERIAL_NUMBER: u8 = 0x80;
const VPD_PAGE_DEVICE_IDENTIFICATION: u8 = 0x83;

/// Configuration for the USB Mass Storage Class.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Config {
    /// Maximum packet size for Bulk endpoints (typically 64 for Full Speed, 512 for High Speed).
    pub max_packet_size: u16,
    /// Vendor identification (8 ASCII characters, space-padded per SPC-2).
    pub vendor_id: [u8; 8],
    /// Product identification (16 ASCII characters, space-padded per SPC-2).
    pub product_id: [u8; 16],
    /// Product revision level (4 ASCII characters, space-padded per SPC-2).
    pub product_revision_level: [u8; 4],
    /// Unit serial number (up to 32 ASCII characters).
    pub serial_number: [u8; 32],
    /// Length of the unit serial number in bytes (0..=32).
    pub serial_number_len: u8,
    /// Ask the driver to double-buffer the bulk endpoints. See
    /// [`Driver::alloc_endpoint_bulk_in_double_buffered`].
    pub double_buffered: bool,
}

impl Config {
    /// Creates a new `Config` with the given `max_packet_size` and empty vendor/product/serial strings.
    pub const fn new(max_packet_size: u16) -> Self {
        core::assert!(
            max_packet_size == 64 || max_packet_size == 512,
            "max_packet_size must be 64 (Full Speed) or 512 (High Speed)"
        );
        Self {
            max_packet_size,
            vendor_id: [b' '; 8],
            product_id: [b' '; 16],
            product_revision_level: [b' '; 4],
            serial_number: [0u8; 32],
            serial_number_len: 0,
            double_buffered: false,
        }
    }

    /// Sets the SCSI Vendor Identification (space-padded or truncated to 8 bytes).
    pub fn vendor_id(mut self, vendor_id: &str) -> Self {
        self.set_vendor_id(vendor_id);
        self
    }

    /// Sets the SCSI Vendor Identification (space-padded or truncated to 8 bytes).
    pub fn set_vendor_id(&mut self, vendor_id: &str) {
        copy_pad_ascii(&mut self.vendor_id, vendor_id);
    }

    /// Sets the SCSI Product Identification (space-padded or truncated to 16 bytes).
    pub fn product_id(mut self, product_id: &str) -> Self {
        self.set_product_id(product_id);
        self
    }

    /// Sets the SCSI Product Identification (space-padded or truncated to 16 bytes).
    pub fn set_product_id(&mut self, product_id: &str) {
        copy_pad_ascii(&mut self.product_id, product_id);
    }

    /// Sets the SCSI Product Revision Level (space-padded or truncated to 4 bytes).
    pub fn product_revision_level(mut self, revision: &str) -> Self {
        self.set_product_revision_level(revision);
        self
    }

    /// Sets the SCSI Product Revision Level (space-padded or truncated to 4 bytes).
    pub fn set_product_revision_level(&mut self, revision: &str) {
        copy_pad_ascii(&mut self.product_revision_level, revision);
    }

    /// Asks the driver to double-buffer the bulk endpoints.
    ///
    /// This improves throughput on drivers that support it, at the cost of more endpoint
    /// resources. Drivers that don't support it allocate ordinary endpoints.
    pub fn double_buffered(mut self, double_buffered: bool) -> Self {
        self.double_buffered = double_buffered;
        self
    }

    /// Sets the Unit Serial Number (up to 32 ASCII characters).
    pub fn serial_number(mut self, serial_number: &str) -> Self {
        self.set_serial_number(serial_number);
        self
    }

    /// Sets the Unit Serial Number (up to 32 ASCII characters).
    pub fn set_serial_number(&mut self, serial_number: &str) {
        let bytes = serial_number.as_bytes();
        let len = min(bytes.len(), self.serial_number.len());
        self.serial_number[..len].copy_from_slice(&bytes[..len]);
        self.serial_number_len = len as u8;
    }

    /// Returns the unit serial number bytes.
    pub fn serial_number_bytes(&self) -> &[u8] {
        &self.serial_number[..self.serial_number_len as usize]
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new(64)
    }
}

fn copy_pad_ascii(dest: &mut [u8], src: &str) {
    dest.fill(b' ');
    let len = min(dest.len(), src.len());
    dest[..len].copy_from_slice(&src.as_bytes()[..len]);
}

/// Trait implemented by synchronous block devices used by [`MscClass`].
pub trait BlockDevice {
    /// Error type returned by storage operations.
    type Error;

    /// Returns the block size in bytes.
    fn block_size(&self) -> u32;

    /// Returns the total amount of logical blocks.
    fn block_count(&self) -> u32;

    /// Reads consecutive logical blocks starting at `lba` into `buf`.
    ///
    /// `buf.len()` is a non-zero multiple of `self.block_size()`.
    fn read_blocks(&mut self, lba: u32, buf: &mut [u8]) -> Result<(), Self::Error>;

    /// Writes consecutive logical blocks starting at `lba` from `data`.
    ///
    /// `data.len()` is a non-zero multiple of `self.block_size()`.
    fn write_blocks(&mut self, lba: u32, data: &[u8]) -> Result<(), Self::Error>;

    /// Flushes pending writes to backing storage.
    fn flush(&mut self) -> Result<(), Self::Error>;

    /// Returns whether the media is write-protected.
    fn is_write_protected(&self) -> bool {
        false
    }
}

/// Async block device abstraction used by [`MscClass`].
///
/// You can implement this trait directly for asynchronous storage, or
/// implement [`BlockDevice`] and rely on the blanket adapter.
pub trait AsyncBlockDevice {
    /// Error type returned by storage operations.
    type Error;

    /// Returns the block size in bytes.
    fn block_size(&self) -> u32;

    /// Returns the total amount of logical blocks.
    fn block_count(&self) -> u32;

    /// Reads consecutive logical blocks starting at `lba` into `buf`.
    ///
    /// `buf.len()` is a non-zero multiple of `self.block_size()`. Transferring
    /// several blocks per call lets storage use multi-block commands.
    async fn read_blocks(&mut self, lba: u32, buf: &mut [u8]) -> Result<(), Self::Error>;

    /// Writes consecutive logical blocks starting at `lba` from `data`.
    ///
    /// `data.len()` is a non-zero multiple of `self.block_size()`.
    async fn write_blocks(&mut self, lba: u32, data: &[u8]) -> Result<(), Self::Error>;

    /// Flushes pending writes to backing storage.
    async fn flush(&mut self) -> Result<(), Self::Error>;

    /// Returns whether the media is write-protected.
    fn is_write_protected(&self) -> bool {
        false
    }
}

impl<T: BlockDevice + ?Sized> AsyncBlockDevice for T {
    type Error = T::Error;

    fn block_size(&self) -> u32 {
        BlockDevice::block_size(self)
    }

    fn block_count(&self) -> u32 {
        BlockDevice::block_count(self)
    }

    async fn read_blocks(&mut self, lba: u32, buf: &mut [u8]) -> Result<(), Self::Error> {
        BlockDevice::read_blocks(self, lba, buf)
    }

    async fn write_blocks(&mut self, lba: u32, data: &[u8]) -> Result<(), Self::Error> {
        BlockDevice::write_blocks(self, lba, data)
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        BlockDevice::flush(self)
    }

    fn is_write_protected(&self) -> bool {
        BlockDevice::is_write_protected(self)
    }
}

/// Why [`MscClass::run`] returned.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum StopReason {
    /// The host ejected the medium with START STOP UNIT (LoEj = 1, Start = 0).
    ///
    /// Pending writes were flushed before the command completed.
    Ejected,
}

/// Internal state for the MSC class.
pub struct State<'a> {
    control: MaybeUninit<Control<'a>>,
    reset_requested: CriticalSectionMutex<Cell<bool>>,
}

impl<'a> Default for State<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> State<'a> {
    /// Create a new `State`.
    pub const fn new() -> Self {
        Self {
            control: MaybeUninit::uninit(),
            reset_requested: CriticalSectionMutex::new(Cell::new(false)),
        }
    }
}

struct Control<'a> {
    interface: InterfaceNumber,
    reset_requested: &'a CriticalSectionMutex<Cell<bool>>,
}

impl<'a> Handler for Control<'a> {
    fn control_out(&mut self, req: Request, data: &[u8]) -> Option<OutResponse> {
        if (req.request_type, req.recipient, req.index)
            != (RequestType::Class, Recipient::Interface, self.interface.0 as u16)
        {
            return None;
        }

        match req.request {
            BOT_REQ_RESET if req.length == 0 && req.value == 0 && data.is_empty() => {
                self.reset_requested.lock(|flag| flag.set(true));
                Some(OutResponse::Accepted)
            }
            BOT_REQ_RESET => Some(OutResponse::Rejected),
            _ => Some(OutResponse::Rejected),
        }
    }

    fn control_in<'b>(&'b mut self, req: Request, buf: &'b mut [u8]) -> Option<InResponse<'b>> {
        if (req.request_type, req.recipient, req.index)
            != (RequestType::Class, Recipient::Interface, self.interface.0 as u16)
        {
            return None;
        }

        match req.request {
            BOT_REQ_GET_MAX_LUN if req.length == 1 && req.value == 0 && !buf.is_empty() => {
                buf[0] = 0;
                Some(InResponse::Accepted(&buf[..1]))
            }
            BOT_REQ_GET_MAX_LUN => Some(InResponse::Rejected),
            _ => Some(InResponse::Rejected),
        }
    }
}

/// USB Mass Storage Class (MSC) implementation.
pub struct MscClass<'d, D: Driver<'d>> {
    read_ep: D::EndpointOut,
    write_ep: D::EndpointIn,
    _interface: InterfaceNumber,
    reset_requested: &'d CriticalSectionMutex<Cell<bool>>,
    config: Config,
    sense: SenseData,
}

impl<'d, D: Driver<'d>> MscClass<'d, D> {
    /// Creates a new MSC class with configuration.
    pub fn new(builder: &mut Builder<'d, D>, state: &'d mut State<'d>, config: Config) -> Self {
        let mut func = builder.function(USB_CLASS_MSC, USB_SUBCLASS_SCSI_TRANSPARENT, USB_PROTOCOL_BULK_ONLY);

        let mut iface = func.interface();
        let interface = iface.interface_number();
        let mut alt = iface.alt_setting(
            USB_CLASS_MSC,
            USB_SUBCLASS_SCSI_TRANSPARENT,
            USB_PROTOCOL_BULK_ONLY,
            None,
        );

        let (read_ep, write_ep) = if config.double_buffered {
            (
                alt.endpoint_bulk_out_double_buffered(None, config.max_packet_size),
                alt.endpoint_bulk_in_double_buffered(None, config.max_packet_size),
            )
        } else {
            (
                alt.endpoint_bulk_out(None, config.max_packet_size),
                alt.endpoint_bulk_in(None, config.max_packet_size),
            )
        };

        drop(func);

        let control = state.control.write(Control {
            interface,
            reset_requested: &state.reset_requested,
        });
        builder.handler(control);

        Self {
            read_ep,
            write_ep,
            _interface: interface,
            reset_requested: &state.reset_requested,
            config,
            sense: SenseData::NO_SENSE,
        }
    }

    /// Gets the endpoint max packet size in bytes.
    pub fn max_packet_size(&self) -> u16 {
        self.read_ep.info().max_packet_size
    }

    /// Waits for the USB host to enable this interface.
    pub async fn wait_connection(&mut self) {
        self.read_ep.wait_enabled().await;
    }

    /// Runs the MSC BOT state machine until the host ejects the medium.
    ///
    /// Disconnects and reconnects are handled internally. When this returns, the
    /// host has received the status of the eject command but the device is still
    /// attached: disable the USB device, or call `run` again to keep serving commands.
    ///
    /// `block_buf` is a temporary buffer used for block transfers and must be at
    /// least `block_device.block_size()` and `max_packet_size` bytes long.
    ///
    /// A buffer of two or more blocks is split in two halves, so the USB transfer of
    /// one half overlaps with `block_device` reading or writing the other. Each call
    /// to `block_device` moves up to half the buffer, so a buffer of several blocks
    /// speeds up large transfers.
    pub async fn run<B: AsyncBlockDevice>(&mut self, block_device: &mut B, block_buf: &mut [u8]) -> StopReason {
        assert!(
            block_buf.len() >= block_device.block_size() as usize
                && block_buf.len() >= self.config.max_packet_size as usize,
            "block_buf must be at least block_size and max_packet_size"
        );
        loop {
            self.wait_connection().await;
            info!("msc: connected");

            if let Ok(reason) = self.run_connected(block_device, block_buf).await {
                info!("msc: stopped: {:?}", reason);
                return reason;
            }

            info!("msc: disconnected");
        }
    }

    async fn run_connected<B: AsyncBlockDevice>(
        &mut self,
        block_device: &mut B,
        block_buf: &mut [u8],
    ) -> Result<StopReason, EndpointError> {
        loop {
            let Some(cbw) = self.read_cbw(block_buf).await? else {
                continue;
            };

            let reset_requested = self.reset_requested.lock(|flag| {
                let was_requested = flag.get();
                flag.set(false);
                was_requested
            });
            if reset_requested {
                self.sense = SenseData::NO_SENSE;
            }

            let result = self.process_cbw(block_device, block_buf, &cbw).await?;

            self.write_csw(cbw.tag, result.residue, result.status).await?;

            if let Some(reason) = result.stop {
                return Ok(reason);
            }
        }
    }

    async fn read_cbw(&mut self, buf: &mut [u8]) -> Result<Option<Cbw>, EndpointError> {
        let n = self
            .read_ep
            .read(&mut buf[..self.config.max_packet_size as usize])
            .await?;

        if n != CBW_LEN {
            warn!("msc: invalid CBW size {}", n);
            return Ok(None);
        }

        let cbw = match Cbw::parse(&buf[..CBW_LEN]) {
            Ok(cbw) => cbw,
            Err(_) => {
                warn!("msc: invalid CBW payload");
                return Ok(None);
            }
        };

        Ok(Some(cbw))
    }

    async fn write_csw(&mut self, tag: u32, residue: u32, status: CswStatus) -> Result<(), EndpointError> {
        let csw = Csw { tag, residue, status };
        self.write_ep.write(&csw.encode()).await
    }

    async fn process_cbw<B: AsyncBlockDevice>(
        &mut self,
        block_device: &mut B,
        block_buf: &mut [u8],
        cbw: &Cbw,
    ) -> Result<CommandResult, EndpointError> {
        let command = Command::parse(&cbw.cb, cbw.cb_length as usize);
        let is_write = matches!(command, Command::Write(_));

        // Drain unexpected OUT data for any non-WRITE command upfront, or any command to an unsupported LUN (BOT Cases 9 & 10)
        if !cbw.direction_in() && cbw.data_transfer_length > 0 && (!is_write || cbw.lun != 0) {
            self.discard_out_data(cbw.data_transfer_length, block_buf).await?;
        }

        // Only LUN 0 is supported.
        if cbw.lun != 0 {
            return match command {
                Command::Inquiry(Inquiry { allocation_length, .. }) => {
                    if let Some(result) = self.check_phase(cbw, DataPhase::DataIn) {
                        return Ok(result);
                    }
                    // SPC-2: Unsupported LUN returns 0x7F (0x60 peripheral not connected | 0x1F unknown device type).
                    // Leaves the sense data alone.
                    let mut data = [0u8; 36];
                    data[0] = 0x7f;
                    let len = min(data.len(), allocation_length);
                    self.send_in_data(cbw, &data[..len]).await
                }
                // SPC-4 processes REPORT LUNS sent to an incorrect logical unit normally.
                Command::ReportLuns { .. } => self.execute(cbw, command, block_device, block_buf).await,
                _ => {
                    self.sense = SenseData::LOGICAL_UNIT_NOT_SUPPORTED;
                    Ok(CommandResult::failed(cbw.data_transfer_length))
                }
            };
        }

        let result = self.execute(cbw, command, block_device, block_buf).await?;

        // If an IN command failed and no data was transferred, terminate host's Data-In
        // phase with a ZLP before the CSW is sent (prevents host reading CSW as data).
        if result.status != CswStatus::Passed
            && cbw.direction_in()
            && result.residue == cbw.data_transfer_length
            && cbw.data_transfer_length > 0
        {
            let _ = self.write_ep.write(&[]).await;
        }

        Ok(result)
    }

    /// Runs `command` against `block_device`, including its data phase.
    async fn execute<B: AsyncBlockDevice>(
        &mut self,
        cbw: &Cbw,
        command: Command,
        block_device: &mut B,
        block_buf: &mut [u8],
    ) -> Result<CommandResult, EndpointError> {
        if let Some(phase) = command.data_phase()
            && let Some(result) = self.check_phase(cbw, phase)
        {
            return Ok(result);
        }

        let mut response = [0u8; RESPONSE_LEN];
        let reply = match command {
            // READ and WRITE stream their data and handle sense themselves.
            Command::Read(rw) => return self.read(cbw, rw, block_device, block_buf).await,
            Command::Write(rw) => return self.write(cbw, rw, block_device, block_buf).await,
            Command::TestUnitReady | Command::PreventAllowMediumRemoval => Ok(Reply::Done),
            Command::RequestSense { allocation_length } => self.request_sense(allocation_length, &mut response),
            Command::Inquiry(inquiry) => self.inquiry(inquiry, &mut response),
            Command::ModeSense6 { allocation_length } => mode_sense_6(allocation_length, block_device, &mut response),
            Command::ModeSense10 { allocation_length } => mode_sense_10(allocation_length, block_device, &mut response),
            Command::ReadFormatCapacities { allocation_length } => {
                read_format_capacities(allocation_length, block_device, &mut response)
            }
            Command::ReadCapacity10 => read_capacity_10(block_device, &mut response),
            Command::ReadCapacity16 { allocation_length } => {
                read_capacity_16(allocation_length, block_device, &mut response)
            }
            Command::StartStopUnit { eject } => start_stop_unit(eject, block_device).await,
            Command::SynchronizeCache10 => synchronize_cache_10(block_device).await,
            Command::ReportLuns {
                select_report,
                allocation_length,
            } => report_luns(select_report, allocation_length, &mut response),
            Command::Unsupported { opcode } => {
                debug!("msc: unsupported opcode 0x{:02x}", opcode);
                Err(SenseData::INVALID_COMMAND_OPERATION_CODE)
            }
        };

        match reply {
            Ok(reply) => {
                self.sense = SenseData::NO_SENSE;
                match reply {
                    Reply::Done => Ok(CommandResult::passed(0)),
                    Reply::Data(len) => self.send_in_data(cbw, &response[..len]).await,
                    Reply::Stop(reason) => Ok(CommandResult {
                        stop: Some(reason),
                        ..CommandResult::passed(0)
                    }),
                }
            }
            Err(sense) => {
                self.sense = sense;
                Ok(CommandResult::failed(cbw.data_transfer_length))
            }
        }
    }

    /// Checks the data phase the host announced in `cbw` against the one the command expects.
    ///
    /// Returns the result to report if they disagree.
    fn check_phase(&mut self, cbw: &Cbw, phase: DataPhase) -> Option<CommandResult> {
        let len = cbw.data_transfer_length;
        match phase {
            DataPhase::NoData if len != 0 => {
                self.sense = SenseData::INVALID_FIELD_IN_CDB;
                Some(CommandResult::failed(len))
            }
            DataPhase::DataIn if len > 0 && !cbw.direction_in() => Some(CommandResult::phase_error(len)),
            DataPhase::DataOut if len > 0 && cbw.direction_in() => Some(CommandResult::phase_error(len)),
            _ => None,
        }
    }

    fn request_sense(&self, allocation_length: usize, buf: &mut Response) -> Result<Reply, SenseData> {
        buf[0] = 0x70;
        buf[2] = self.sense.key as u8;
        buf[7] = 10;
        buf[12] = self.sense.asc;
        buf[13] = self.sense.ascq;
        // The dispatcher then resets the sense data, per SCSI SPC-2.
        Ok(Reply::data(18, allocation_length))
    }

    fn inquiry(&self, inquiry: Inquiry, buf: &mut Response) -> Result<Reply, SenseData> {
        // CmdDt is obsolete and unsupported.
        if inquiry.cmd_dt {
            return Err(SenseData::INVALID_FIELD_IN_CDB);
        }

        let len = if inquiry.evpd {
            match inquiry.page_code {
                VPD_PAGE_SUPPORTED_PAGES => {
                    let data = [
                        0x00, // direct-access block device
                        VPD_PAGE_SUPPORTED_PAGES,
                        0x00,
                        0x03, // 3 bytes follow
                        VPD_PAGE_SUPPORTED_PAGES,
                        VPD_PAGE_UNIT_SERIAL_NUMBER,
                        VPD_PAGE_DEVICE_IDENTIFICATION,
                    ];
                    buf[..data.len()].copy_from_slice(&data);
                    data.len()
                }
                VPD_PAGE_UNIT_SERIAL_NUMBER => {
                    let serial = self.config.serial_number_bytes();
                    buf[1] = VPD_PAGE_UNIT_SERIAL_NUMBER;
                    buf[3] = serial.len() as u8;
                    buf[4..4 + serial.len()].copy_from_slice(serial);
                    4 + serial.len()
                }
                VPD_PAGE_DEVICE_IDENTIFICATION => {
                    let id = self.config.serial_number_bytes();
                    buf[1] = VPD_PAGE_DEVICE_IDENTIFICATION;
                    if id.is_empty() {
                        4
                    } else {
                        buf[3] = (4 + id.len()) as u8; // page length
                        buf[4] = 0x02; // ASCII code set
                        buf[5] = 0x00; // logical unit, vendor-specific identifier type
                        buf[7] = id.len() as u8;
                        buf[8..8 + id.len()].copy_from_slice(id);
                        8 + id.len()
                    }
                }
                _ => return Err(SenseData::INVALID_FIELD_IN_CDB),
            }
        } else {
            // Standard inquiry should request page code 0.
            if inquiry.page_code != 0 {
                return Err(SenseData::INVALID_FIELD_IN_CDB);
            }

            buf[0] = 0x00; // direct-access block device
            buf[1] = 0x80; // removable medium
            buf[2] = 0x04; // SPC-2
            buf[3] = 0x02; // response data format
            buf[4] = 31; // additional length
            buf[8..16].copy_from_slice(&self.config.vendor_id);
            buf[16..32].copy_from_slice(&self.config.product_id);
            buf[32..36].copy_from_slice(&self.config.product_revision_level);
            36
        };

        Ok(Reply::data(len, inquiry.allocation_length))
    }

    async fn read<B: AsyncBlockDevice>(
        &mut self,
        cbw: &Cbw,
        rw: ReadWrite,
        block_device: &mut B,
        block_buf: &mut [u8],
    ) -> Result<CommandResult, EndpointError> {
        let lba = match check_read_write(cbw, rw, block_device, block_buf) {
            Ok(lba) => lba,
            Err(sense) => {
                self.sense = sense;
                return Ok(CommandResult::failed(cbw.data_transfer_length));
            }
        };

        if rw.blocks == 0 {
            self.sense = SenseData::NO_SENSE;
            return Ok(CommandResult::passed(0));
        }

        let blocks = rw.blocks;
        let block_size = block_device.block_size() as usize;
        let (mut front, mut back) = split_chunks(block_buf, block_size);
        let chunk_blocks = (front.len() / block_size) as u32;

        let mut residue = cbw.data_transfer_length;
        let mut chunk_lba = lba;
        let mut len = min(blocks, chunk_blocks) as usize * block_size;
        let mut filled = block_device.read_blocks(chunk_lba, &mut front[..len]).await;

        loop {
            if filled.is_err() {
                warn!("msc: read_blocks failed at lba {}", chunk_lba);
                self.sense = SenseData::UNRECOVERED_READ_ERROR;
                if residue < cbw.data_transfer_length {
                    let _ = self.write_ep.write(&[]).await;
                }
                return Ok(CommandResult::failed(residue));
            }

            let next_lba = chunk_lba + (len / block_size) as u32;
            let next = min(lba + blocks - next_lba, chunk_blocks) as usize * block_size;

            if back.is_empty() {
                self.write_ep.write_transfer(&front[..len], false).await?;
                filled = read_chunk(block_device, next_lba, &mut front[..next]).await;
            } else {
                // Send this chunk while storage reads the next one.
                let (sent, read) = join(
                    self.write_ep.write_transfer(&front[..len], false),
                    read_chunk(block_device, next_lba, &mut back[..next]),
                )
                .await;
                sent?;
                filled = read;
                core::mem::swap(&mut front, &mut back);
            }

            residue -= len as u32;

            if next == 0 {
                break;
            }

            chunk_lba = next_lba;
            len = next;
        }

        self.sense = SenseData::NO_SENSE;
        Ok(CommandResult::passed(residue))
    }

    async fn write<B: AsyncBlockDevice>(
        &mut self,
        cbw: &Cbw,
        rw: ReadWrite,
        block_device: &mut B,
        block_buf: &mut [u8],
    ) -> Result<CommandResult, EndpointError> {
        let checked = check_read_write(cbw, rw, block_device, block_buf).and_then(|lba| {
            if block_device.is_write_protected() {
                Err(SenseData::WRITE_PROTECTED)
            } else {
                Ok(lba)
            }
        });
        let lba = match checked {
            Ok(lba) => lba,
            Err(sense) => {
                self.sense = sense;
                self.discard_out_data(cbw.data_transfer_length, block_buf).await?;
                return Ok(CommandResult::failed(cbw.data_transfer_length));
            }
        };

        if rw.blocks == 0 {
            self.sense = SenseData::NO_SENSE;
            return Ok(CommandResult::passed(0));
        }

        let blocks = rw.blocks;
        let block_size = block_device.block_size() as usize;
        let (mut front, mut back) = split_chunks(block_buf, block_size);
        let chunk_blocks = (front.len() / block_size) as u32;

        let mut received = 0;
        let mut chunk_lba = lba;
        let mut len = min(blocks, chunk_blocks) as usize * block_size;

        read_exact_out(&mut self.read_ep, &mut front[..len]).await?;

        received += len as u32;

        let failed = loop {
            let next_lba = chunk_lba + (len / block_size) as u32;
            let next = min(lba + blocks - next_lba, chunk_blocks) as usize * block_size;

            let stored = if back.is_empty() {
                block_device.write_blocks(chunk_lba, &front[..len]).await
            } else {
                // Receive the next chunk while storage writes this one.
                let (got, stored) = join(
                    read_exact_out(&mut self.read_ep, &mut back[..next]),
                    block_device.write_blocks(chunk_lba, &front[..len]),
                )
                .await;

                got?;
                received += next as u32;

                stored
            };

            if stored.is_err() {
                break Some(chunk_lba);
            }
            if next == 0 {
                break None;
            }

            if back.is_empty() {
                read_exact_out(&mut self.read_ep, &mut front[..next]).await?;
                received += next as u32;
            } else {
                core::mem::swap(&mut front, &mut back);
            }

            chunk_lba = next_lba;
            len = next;
        };

        if let Some(failed_lba) = failed {
            warn!("msc: write_blocks failed at lba {}", failed_lba);
            self.sense = SenseData::WRITE_ERROR;

            let remaining_to_drain = cbw.data_transfer_length - received;
            if remaining_to_drain > 0 {
                self.discard_out_data(remaining_to_drain, block_buf).await?;
            }

            let written = (failed_lba - lba) * block_size as u32;
            return Ok(CommandResult::failed(cbw.data_transfer_length - written));
        }

        if block_device.flush().await.is_err() {
            warn!("msc: flush after write failed");
            self.sense = SenseData::WRITE_ERROR;
            return Ok(CommandResult::failed(0));
        }

        self.sense = SenseData::NO_SENSE;
        Ok(CommandResult::passed(0))
    }

    async fn send_in_data(&mut self, cbw: &Cbw, data: &[u8]) -> Result<CommandResult, EndpointError> {
        let transfer_len = min(data.len(), cbw.data_transfer_length as usize);
        self.write_ep.write_transfer(&data[..transfer_len], false).await?;

        // If transfer completed with fewer bytes than expected by the host, and the
        // length transferred is a multiple of max_packet_size (including 0), send a ZLP
        // to terminate the USB transfer.
        if (transfer_len as u32) < cbw.data_transfer_length
            && transfer_len.is_multiple_of(self.config.max_packet_size as usize)
        {
            self.write_ep.write(&[]).await?;
        }

        let residue = cbw.data_transfer_length.saturating_sub(transfer_len as u32);
        Ok(CommandResult::passed(residue))
    }

    async fn discard_out_data(&mut self, mut len: u32, scratch: &mut [u8]) -> Result<(), EndpointError> {
        if scratch.is_empty() {
            return Ok(());
        }

        let max_pkt = self.config.max_packet_size as usize;
        while len > 0 {
            // Read at least max_packet_size into scratch if possible to avoid BufferOverflow.
            let chunk_len = min(scratch.len(), core::cmp::max(len as usize, max_pkt));
            let n = self.read_ep.read(&mut scratch[..chunk_len]).await?;
            len = len.saturating_sub(n as u32);
            if n < max_pkt {
                break;
            }
        }

        Ok(())
    }
}

fn mode_sense_6<B: AsyncBlockDevice>(
    allocation_length: usize,
    block_device: &B,
    buf: &mut Response,
) -> Result<Reply, SenseData> {
    buf[0] = 3; // Mode data length
    buf[2] = if block_device.is_write_protected() { 0x80 } else { 0x00 };
    Ok(Reply::data(4, allocation_length))
}

fn mode_sense_10<B: AsyncBlockDevice>(
    allocation_length: usize,
    block_device: &B,
    buf: &mut Response,
) -> Result<Reply, SenseData> {
    buf[1] = 6; // Mode data length (6 bytes follow)
    buf[2] = 0; // medium type
    buf[3] = if block_device.is_write_protected() { 0x80 } else { 0x00 };
    Ok(Reply::data(8, allocation_length))
}

fn read_format_capacities<B: AsyncBlockDevice>(
    allocation_length: usize,
    block_device: &B,
    buf: &mut Response,
) -> Result<Reply, SenseData> {
    let block_size = block_device.block_size();

    buf[3] = 8;
    buf[4..8].copy_from_slice(&block_device.block_count().to_be_bytes());
    buf[8] = 0x02; // formatted media, current capacity
    buf[9] = (block_size >> 16) as u8;
    buf[10] = (block_size >> 8) as u8;
    buf[11] = block_size as u8;
    Ok(Reply::data(12, allocation_length))
}

fn read_capacity_10<B: AsyncBlockDevice>(block_device: &B, buf: &mut Response) -> Result<Reply, SenseData> {
    let last_lba = block_device.block_count().saturating_sub(1);

    buf[0..4].copy_from_slice(&last_lba.to_be_bytes());
    buf[4..8].copy_from_slice(&block_device.block_size().to_be_bytes());
    Ok(Reply::Data(8))
}

fn read_capacity_16<B: AsyncBlockDevice>(
    allocation_length: usize,
    block_device: &B,
    buf: &mut Response,
) -> Result<Reply, SenseData> {
    let last_lba = block_device.block_count().saturating_sub(1) as u64;

    buf[0..8].copy_from_slice(&last_lba.to_be_bytes());
    buf[8..12].copy_from_slice(&block_device.block_size().to_be_bytes());
    Ok(Reply::data(32, allocation_length))
}

async fn start_stop_unit<B: AsyncBlockDevice>(eject: bool, block_device: &mut B) -> Result<Reply, SenseData> {
    if !eject {
        return Ok(Reply::Done);
    }

    if block_device.flush().await.is_err() {
        warn!("msc: flush before eject failed");
        return Err(SenseData::WRITE_ERROR);
    }

    Ok(Reply::Stop(StopReason::Ejected))
}

async fn synchronize_cache_10<B: AsyncBlockDevice>(block_device: &mut B) -> Result<Reply, SenseData> {
    if block_device.flush().await.is_err() {
        warn!("msc: flush failed");
        return Err(SenseData::WRITE_ERROR);
    }

    Ok(Reply::Done)
}

fn report_luns(select_report: u8, allocation_length: usize, buf: &mut Response) -> Result<Reply, SenseData> {
    let Some(data) = lun_list(select_report).filter(|_| allocation_length >= RL_MIN_ALLOCATION_LENGTH) else {
        return Err(SenseData::INVALID_FIELD_IN_CDB);
    };

    buf[..data.len()].copy_from_slice(data);
    Ok(Reply::data(data.len(), allocation_length))
}

/// Checks a READ or WRITE command against `block_device` and `block_buf`.
///
/// Returns the starting LBA, or the sense data to report if the command can't run.
fn check_read_write<B: AsyncBlockDevice>(
    cbw: &Cbw,
    rw: ReadWrite,
    block_device: &B,
    block_buf: &[u8],
) -> Result<u32, SenseData> {
    let block_size = block_device.block_size();
    if block_size == 0
        || block_buf.len() < block_size as usize
        || rw.blocks.checked_mul(block_size) != Some(cbw.data_transfer_length)
    {
        return Err(SenseData::INVALID_FIELD_IN_CDB);
    }

    if rw
        .lba
        .checked_add(rw.blocks as u64)
        .is_none_or(|end| end > block_device.block_count() as u64)
    {
        return Err(SenseData::LOGICAL_BLOCK_ADDRESS_OUT_OF_RANGE);
    }

    // In range, so it fits in u32 like the block count.
    Ok(rw.lba as u32)
}

/// Splits `buf` into two chunks of whole blocks so USB transfers can overlap with storage access.
///
/// Both chunks are the same size. If `buf` holds a single block, the second chunk is empty.
fn split_chunks(buf: &mut [u8], block_size: usize) -> (&mut [u8], &mut [u8]) {
    let blocks = buf.len() / block_size;
    let back_len = blocks / 2 * block_size;
    let (front, back) = buf.split_at_mut(blocks.div_ceil(2) * block_size);

    (&mut front[..back_len.max(block_size)], &mut back[..back_len])
}

async fn read_chunk<B: AsyncBlockDevice>(block_device: &mut B, lba: u32, buf: &mut [u8]) -> Result<(), B::Error> {
    if buf.is_empty() {
        return Ok(());
    }

    block_device.read_blocks(lba, buf).await
}

async fn read_exact_out(ep: &mut impl EndpointOut, mut buf: &mut [u8]) -> Result<(), EndpointError> {
    while !buf.is_empty() {
        let n = ep.read_transfer(buf).await?;
        buf = &mut buf[n..];
    }

    Ok(())
}

/// REPORT LUNS parameter data for a SELECT REPORT value, or `None` if the value is unsupported.
///
/// LUN 0 is the only logical unit, and there are no well-known logical units.
fn lun_list(select_report: u8) -> Option<&'static [u8]> {
    match select_report {
        // LUN LIST LENGTH, 4 reserved bytes, then LUN 0.
        RL_SELECT_ALL | RL_SELECT_ALL_AND_WELL_KNOWN => Some(&[0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        RL_SELECT_WELL_KNOWN => Some(&[0; 8]),
        _ => None,
    }
}

/// Size of the buffer command handlers write their Data-In response to.
const RESPONSE_LEN: usize = 40;

type Response = [u8; RESPONSE_LEN];

/// What a command handler asks the dispatcher to do once the command succeeded.
#[derive(Clone, Copy)]
enum Reply {
    /// Complete the command with no data phase.
    Done,
    /// Send this many bytes of the response buffer to the host.
    Data(usize),
    /// Complete the command, then stop [`MscClass::run`].
    Stop(StopReason),
}

impl Reply {
    /// Sends `len` bytes of the response, truncated to the CDB's allocation length.
    fn data(len: usize, allocation_length: usize) -> Self {
        Self::Data(min(len, allocation_length))
    }
}

#[derive(Clone, Copy)]
struct CommandResult {
    residue: u32,
    status: CswStatus,
    stop: Option<StopReason>,
}

impl CommandResult {
    const fn passed(residue: u32) -> Self {
        Self {
            residue,
            status: CswStatus::Passed,
            stop: None,
        }
    }

    const fn failed(residue: u32) -> Self {
        Self {
            residue,
            status: CswStatus::Failed,
            stop: None,
        }
    }

    const fn phase_error(residue: u32) -> Self {
        Self {
            residue,
            status: CswStatus::PhaseError,
            stop: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pads_ascii() {
        let mut buf = [0u8; 8];
        copy_pad_ascii(&mut buf, "Embassy");
        core::assert_eq!(&buf, b"Embassy ");

        let mut buf16 = [0u8; 16];
        copy_pad_ascii(&mut buf16, "MSC Disk");
        core::assert_eq!(&buf16, b"MSC Disk        ");

        let mut buf4 = [0u8; 4];
        copy_pad_ascii(&mut buf4, "0.1");
        core::assert_eq!(&buf4, b"0.1 ");
    }

    #[test]
    fn default_config() {
        let config = Config::default();
        core::assert_eq!(config.max_packet_size, 64);
        core::assert_eq!(&config.vendor_id, b"        ");
        core::assert_eq!(&config.product_id, b"                ");
        core::assert_eq!(&config.product_revision_level, b"    ");
        core::assert_eq!(config.serial_number_bytes(), b"");
    }

    #[test]
    fn config_builder_dynamic_strings() {
        let dynamic_serial = "CHIP-9876543210";
        let config = Config::new(64)
            .vendor_id("ACME")
            .product_id("Storage Device")
            .product_revision_level("2.0")
            .serial_number(dynamic_serial);

        core::assert_eq!(&config.vendor_id, b"ACME    ");
        core::assert_eq!(&config.product_id, b"Storage Device  ");
        core::assert_eq!(&config.product_revision_level, b"2.0 ");
        core::assert_eq!(config.serial_number_bytes(), dynamic_serial.as_bytes());
    }

    #[test]
    fn lun_list_reports_lun_0() {
        let lun_0 = [0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        core::assert_eq!(lun_list(RL_SELECT_ALL), Some(&lun_0[..]));
        core::assert_eq!(lun_list(RL_SELECT_ALL_AND_WELL_KNOWN), Some(&lun_0[..]));
        core::assert_eq!(lun_list(RL_SELECT_WELL_KNOWN), Some(&[0u8; 8][..]));
        core::assert_eq!(lun_list(0x03), None);
    }

    struct MockBlockDevice {
        blocks: [[u8; 512]; 4],
    }

    impl BlockDevice for MockBlockDevice {
        type Error = ();
        fn block_size(&self) -> u32 {
            512
        }
        fn block_count(&self) -> u32 {
            4
        }
        fn read_blocks(&mut self, lba: u32, buf: &mut [u8]) -> Result<(), Self::Error> {
            buf.copy_from_slice(self.blocks[lba as usize..][..buf.len() / 512].as_flattened());
            Ok(())
        }
        fn write_blocks(&mut self, lba: u32, data: &[u8]) -> Result<(), Self::Error> {
            self.blocks[lba as usize..][..data.len() / 512]
                .as_flattened_mut()
                .copy_from_slice(data);
            Ok(())
        }
        fn flush(&mut self) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    #[test]
    fn sync_block_device_blanket_impl() {
        let mut dev = MockBlockDevice {
            blocks: [[0u8; 512]; 4],
        };
        dev.blocks[1][0] = 42;
        dev.blocks[2][0] = 43;
        let mut buf = [0u8; 1024];
        embassy_futures::block_on(async {
            AsyncBlockDevice::read_blocks(&mut dev, 1, &mut buf).await.unwrap();
            core::assert_eq!((buf[0], buf[512]), (42, 43));
        });
    }
}

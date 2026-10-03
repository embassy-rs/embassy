//! USB Mass Storage Class (MSC) implementation.
//!
//! This implements the USB Bulk-Only Transport (BOT) protocol with a
//! SCSI transparent command set suitable for a simple block device.

use core::cell::Cell;
use core::cmp::min;
use core::mem::MaybeUninit;

use embassy_sync::blocking_mutex::CriticalSectionMutex;

use super::bot::{CSW_STATUS_FAILED, CSW_STATUS_PASSED, CSW_STATUS_PHASE_ERROR, Cbw, Csw, encode_csw, parse_cbw};
use super::scsi::*;
use super::{
    BOT_REQ_GET_MAX_LUN, BOT_REQ_RESET, SenseData, SenseKey, USB_CLASS_MSC, USB_PROTOCOL_BULK_ONLY,
    USB_SUBCLASS_SCSI_TRANSPARENT,
};
use crate::control::{InResponse, OutResponse, Recipient, Request, RequestType};
use crate::driver::{Driver, Endpoint, EndpointError, EndpointIn, EndpointOut};
use crate::types::InterfaceNumber;
use crate::{Builder, Handler};

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

    /// Reads one logical block at `lba` into `buf`.
    ///
    /// Implementations should expect `buf.len() == self.block_size() as usize`.
    fn read_block(&mut self, lba: u32, buf: &mut [u8]) -> Result<(), Self::Error>;

    /// Writes one logical block at `lba` from `data`.
    ///
    /// Implementations should expect `data.len() == self.block_size() as usize`.
    fn write_block(&mut self, lba: u32, data: &[u8]) -> Result<(), Self::Error>;

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

    /// Reads one logical block at `lba` into `buf`.
    async fn read_block(&mut self, lba: u32, buf: &mut [u8]) -> Result<(), Self::Error>;

    /// Writes one logical block at `lba` from `data`.
    async fn write_block(&mut self, lba: u32, data: &[u8]) -> Result<(), Self::Error>;

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

    async fn read_block(&mut self, lba: u32, buf: &mut [u8]) -> Result<(), Self::Error> {
        BlockDevice::read_block(self, lba, buf)
    }

    async fn write_block(&mut self, lba: u32, data: &[u8]) -> Result<(), Self::Error> {
        BlockDevice::write_block(self, lba, data)
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        BlockDevice::flush(self)
    }

    fn is_write_protected(&self) -> bool {
        BlockDevice::is_write_protected(self)
    }
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

        let read_ep = alt.endpoint_bulk_out(None, config.max_packet_size);
        let write_ep = alt.endpoint_bulk_in(None, config.max_packet_size);

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

    /// Runs the MSC BOT state machine forever.
    ///
    /// `block_buf` is a temporary buffer used for block transfers and must be at
    /// least `block_device.block_size()` and `max_packet_size` bytes long.
    pub async fn run<B: AsyncBlockDevice>(&mut self, block_device: &mut B, block_buf: &mut [u8]) -> ! {
        assert!(
            block_buf.len() >= block_device.block_size() as usize
                && block_buf.len() >= self.config.max_packet_size as usize,
            "block_buf must be at least block_size and max_packet_size"
        );
        loop {
            self.wait_connection().await;
            info!("msc: connected");

            let _ = self.run_connected(block_device, block_buf).await;

            info!("msc: disconnected");
        }
    }

    async fn run_connected<B: AsyncBlockDevice>(
        &mut self,
        block_device: &mut B,
        block_buf: &mut [u8],
    ) -> Result<(), EndpointError> {
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
        }
    }

    async fn read_cbw(&mut self, buf: &mut [u8]) -> Result<Option<Cbw>, EndpointError> {
        let n = self
            .read_ep
            .read(&mut buf[..self.config.max_packet_size as usize])
            .await?;

        if n != 31 {
            warn!("msc: invalid CBW size {}", n);
            return Ok(None);
        }

        let cbw = match parse_cbw(&buf[..31]) {
            Ok(cbw) => cbw,
            Err(_) => {
                warn!("msc: invalid CBW payload");
                return Ok(None);
            }
        };

        Ok(Some(cbw))
    }

    async fn write_csw(&mut self, tag: u32, residue: u32, status: u8) -> Result<(), EndpointError> {
        let csw = encode_csw(Csw { tag, residue, status });
        self.write_ep.write(&csw).await
    }

    async fn process_cbw<B: AsyncBlockDevice>(
        &mut self,
        block_device: &mut B,
        block_buf: &mut [u8],
        cbw: &Cbw,
    ) -> Result<CommandResult, EndpointError> {
        // Drain unexpected OUT data for any non-WRITE command upfront, or any command to an unsupported LUN (BOT Cases 9 & 10)
        if !cbw.direction_in() && cbw.data_transfer_length > 0 && (cbw.cb[0] != SCSI_WRITE_10 || cbw.lun != 0) {
            self.discard_out_data(cbw.data_transfer_length, block_buf).await?;
        }

        // Only LUN 0 is supported.
        if cbw.lun != 0 {
            if cbw.cb[0] == SCSI_INQUIRY {
                if cbw.data_transfer_length > 0 && !cbw.direction_in() {
                    return Ok(CommandResult::phase_error(cbw.data_transfer_length));
                }
                // SPC-2: Unsupported LUN returns 0x7F (0x60 peripheral not connected | 0x1F unknown device type).
                let mut data = [0u8; 36];
                data[0] = 0x7f;
                let allocation_len = u16::from_be_bytes([cbw.cb[3], cbw.cb[4]]) as usize;
                let len = min(data.len(), allocation_len);
                return self.send_in_data(cbw, &data[..len]).await;
            }

            self.set_sense(SENSE_KEY_ILLEGAL_REQUEST, ASC_LOGICAL_UNIT_NOT_SUPPORTED, ASCQ_NONE);
            return Ok(CommandResult::failed(cbw.data_transfer_length));
        }

        let result = match cbw.cb[0] {
            SCSI_TEST_UNIT_READY => Ok(self.test_unit_ready(cbw)),
            SCSI_REQUEST_SENSE => self.request_sense(cbw).await,
            SCSI_INQUIRY => self.inquiry(cbw).await,
            SCSI_MODE_SENSE_6 => self.mode_sense_6(cbw, block_device).await,
            SCSI_MODE_SENSE_10 => self.mode_sense_10(cbw, block_device).await,
            SCSI_READ_FORMAT_CAPACITIES => self.read_format_capacities(cbw, block_device).await,
            SCSI_READ_CAPACITY_10 => self.read_capacity_10(cbw, block_device).await,
            SCSI_READ_10 => self.read_10(cbw, block_device, block_buf).await,
            SCSI_WRITE_10 => self.write_10(cbw, block_device, block_buf).await,
            SCSI_START_STOP_UNIT => Ok(self.start_stop_unit(cbw)),
            SCSI_PREVENT_ALLOW_MEDIUM_REMOVAL => Ok(self.prevent_allow_medium_removal(cbw)),
            SCSI_SYNCHRONIZE_CACHE_10 => self.synchronize_cache_10(cbw, block_device).await,
            _ => {
                self.set_sense(SENSE_KEY_ILLEGAL_REQUEST, ASC_INVALID_COMMAND_OPERATION_CODE, ASCQ_NONE);
                Ok(CommandResult::failed(cbw.data_transfer_length))
            }
        };

        // If an IN command failed and no data was transferred, terminate host's Data-In
        // phase with a ZLP before the CSW is sent (prevents host reading CSW as data).
        if let Ok(r) = &result {
            if r.status != CSW_STATUS_PASSED
                && cbw.direction_in()
                && r.residue == cbw.data_transfer_length
                && cbw.data_transfer_length > 0
            {
                let _ = self.write_ep.write(&[]).await;
            }
        }

        result
    }

    fn test_unit_ready(&mut self, cbw: &Cbw) -> CommandResult {
        if cbw.data_transfer_length != 0 {
            self.set_sense(SENSE_KEY_ILLEGAL_REQUEST, ASC_INVALID_FIELD_IN_CDB, ASCQ_NONE);
            return CommandResult::failed(cbw.data_transfer_length);
        }

        self.sense = SenseData::NO_SENSE;
        CommandResult::passed(0)
    }

    async fn request_sense(&mut self, cbw: &Cbw) -> Result<CommandResult, EndpointError> {
        if cbw.data_transfer_length > 0 && !cbw.direction_in() {
            return Ok(CommandResult::phase_error(cbw.data_transfer_length));
        }

        let allocation_len = cbw.cb[4] as usize;
        let mut data = [0u8; 18];
        data[0] = 0x70;
        data[2] = self.sense.key as u8;
        data[7] = 10;
        data[12] = self.sense.asc;
        data[13] = self.sense.ascq;

        // Reset sense data after returning it, per SCSI SPC-2 spec.
        self.sense = SenseData::NO_SENSE;

        let len = min(data.len(), allocation_len);
        let result = self.send_in_data(cbw, &data[..len]).await?;

        Ok(result)
    }

    async fn inquiry(&mut self, cbw: &Cbw) -> Result<CommandResult, EndpointError> {
        if cbw.data_transfer_length > 0 && !cbw.direction_in() {
            return Ok(CommandResult::phase_error(cbw.data_transfer_length));
        }

        let evpd = cbw.cb[1] & 0x01 != 0;
        let cmd_dt = cbw.cb[1] & 0x02 != 0;
        let page_code = cbw.cb[2];
        let allocation_len = u16::from_be_bytes([cbw.cb[3], cbw.cb[4]]) as usize;

        // CmdDt is obsolete and unsupported.
        if cmd_dt {
            self.set_sense(SENSE_KEY_ILLEGAL_REQUEST, ASC_INVALID_FIELD_IN_CDB, ASCQ_NONE);
            return Ok(CommandResult::failed(cbw.data_transfer_length));
        }

        if evpd {
            let result = match page_code {
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
                    let len = min(data.len(), allocation_len);
                    self.send_in_data(cbw, &data[..len]).await?
                }
                VPD_PAGE_UNIT_SERIAL_NUMBER => {
                    let serial_bytes = self.config.serial_number_bytes();
                    let serial_len = serial_bytes.len();
                    let mut data = [0u8; 36];
                    data[0] = 0x00;
                    data[1] = VPD_PAGE_UNIT_SERIAL_NUMBER;
                    data[2] = 0x00;
                    data[3] = serial_len as u8;
                    data[4..4 + serial_len].copy_from_slice(serial_bytes);
                    let len = min(4 + serial_len, allocation_len);
                    self.send_in_data(cbw, &data[..len]).await?
                }
                VPD_PAGE_DEVICE_IDENTIFICATION => {
                    let id_bytes = self.config.serial_number_bytes();
                    let id_len = id_bytes.len();
                    let mut data = [0u8; 40];
                    data[0] = 0x00;
                    data[1] = VPD_PAGE_DEVICE_IDENTIFICATION;
                    data[2] = 0x00;
                    if id_len == 0 {
                        data[3] = 0;
                        let len = min(4, allocation_len);
                        self.send_in_data(cbw, &data[..len]).await?
                    } else {
                        data[3] = (4 + id_len) as u8; // page length
                        data[4] = 0x02; // ASCII code set
                        data[5] = 0x00; // logical unit, vendor-specific identifier type
                        data[6] = 0x00;
                        data[7] = id_len as u8;
                        data[8..8 + id_len].copy_from_slice(id_bytes);
                        let len = min(8 + id_len, allocation_len);
                        self.send_in_data(cbw, &data[..len]).await?
                    }
                }
                _ => {
                    self.set_sense(SENSE_KEY_ILLEGAL_REQUEST, ASC_INVALID_FIELD_IN_CDB, ASCQ_NONE);
                    return Ok(CommandResult::failed(cbw.data_transfer_length));
                }
            };

            self.sense = SenseData::NO_SENSE;
            return Ok(result);
        }

        // Standard inquiry should request page code 0.
        if page_code != 0 {
            self.set_sense(SENSE_KEY_ILLEGAL_REQUEST, ASC_INVALID_FIELD_IN_CDB, ASCQ_NONE);
            return Ok(CommandResult::failed(cbw.data_transfer_length));
        }

        let mut data = [0u8; 36];
        data[0] = 0x00; // direct-access block device
        data[1] = 0x80; // removable medium
        data[2] = 0x04; // SPC-2
        data[3] = 0x02; // response data format
        data[4] = 31; // additional length

        data[8..16].copy_from_slice(&self.config.vendor_id);
        data[16..32].copy_from_slice(&self.config.product_id);
        data[32..36].copy_from_slice(&self.config.product_revision_level);

        let len = min(data.len(), allocation_len);
        let result = self.send_in_data(cbw, &data[..len]).await?;
        self.sense = SenseData::NO_SENSE;
        Ok(result)
    }

    async fn mode_sense_6<B: AsyncBlockDevice>(
        &mut self,
        cbw: &Cbw,
        block_device: &B,
    ) -> Result<CommandResult, EndpointError> {
        if cbw.data_transfer_length > 0 && !cbw.direction_in() {
            return Ok(CommandResult::phase_error(cbw.data_transfer_length));
        }

        let allocation_len = cbw.cb[4] as usize;
        let mut data = [0u8; 4];
        data[0] = 3; // Mode data length
        data[2] = if block_device.is_write_protected() { 0x80 } else { 0x00 };

        let len = min(data.len(), allocation_len);
        let result = self.send_in_data(cbw, &data[..len]).await?;
        self.sense = SenseData::NO_SENSE;
        Ok(result)
    }

    async fn mode_sense_10<B: AsyncBlockDevice>(
        &mut self,
        cbw: &Cbw,
        block_device: &B,
    ) -> Result<CommandResult, EndpointError> {
        if cbw.data_transfer_length > 0 && !cbw.direction_in() {
            return Ok(CommandResult::phase_error(cbw.data_transfer_length));
        }

        let allocation_len = u16::from_be_bytes([cbw.cb[7], cbw.cb[8]]) as usize;
        let mut data = [0u8; 8];
        data[0] = 0;
        data[1] = 6; // Mode data length (6 bytes follow)
        data[2] = 0; // medium type
        data[3] = if block_device.is_write_protected() { 0x80 } else { 0x00 };

        let len = min(data.len(), allocation_len);
        let result = self.send_in_data(cbw, &data[..len]).await?;
        self.sense = SenseData::NO_SENSE;
        Ok(result)
    }

    async fn read_format_capacities<B: AsyncBlockDevice>(
        &mut self,
        cbw: &Cbw,
        block_device: &B,
    ) -> Result<CommandResult, EndpointError> {
        if cbw.data_transfer_length > 0 && !cbw.direction_in() {
            return Ok(CommandResult::phase_error(cbw.data_transfer_length));
        }

        let mut data = [0u8; 12];
        data[3] = 8;

        let blocks = block_device.block_count();
        let block_size = block_device.block_size();

        data[4..8].copy_from_slice(&blocks.to_be_bytes());
        data[8] = 0x02; // formatted media, current capacity
        data[9] = (block_size >> 16) as u8;
        data[10] = (block_size >> 8) as u8;
        data[11] = block_size as u8;

        let allocation_len = u16::from_be_bytes([cbw.cb[7], cbw.cb[8]]) as usize;
        let len = min(data.len(), allocation_len);
        let result = self.send_in_data(cbw, &data[..len]).await?;
        self.sense = SenseData::NO_SENSE;
        Ok(result)
    }

    async fn read_capacity_10<B: AsyncBlockDevice>(
        &mut self,
        cbw: &Cbw,
        block_device: &B,
    ) -> Result<CommandResult, EndpointError> {
        if cbw.data_transfer_length > 0 && !cbw.direction_in() {
            return Ok(CommandResult::phase_error(cbw.data_transfer_length));
        }

        let mut data = [0u8; 8];
        let block_count = block_device.block_count();
        let last_lba = block_count.saturating_sub(1);
        let block_size = block_device.block_size();

        data[0..4].copy_from_slice(&last_lba.to_be_bytes());
        data[4..8].copy_from_slice(&block_size.to_be_bytes());

        let result = self.send_in_data(cbw, &data).await?;
        self.sense = SenseData::NO_SENSE;
        Ok(result)
    }

    async fn read_10<B: AsyncBlockDevice>(
        &mut self,
        cbw: &Cbw,
        block_device: &mut B,
        block_buf: &mut [u8],
    ) -> Result<CommandResult, EndpointError> {
        if cbw.data_transfer_length > 0 && !cbw.direction_in() {
            return Ok(CommandResult::phase_error(cbw.data_transfer_length));
        }

        let block_size = block_device.block_size() as usize;
        if block_size == 0 || block_buf.len() < block_size {
            self.set_sense(SENSE_KEY_ILLEGAL_REQUEST, ASC_INVALID_FIELD_IN_CDB, ASCQ_NONE);
            return Ok(CommandResult::failed(cbw.data_transfer_length));
        }

        let lba = u32::from_be_bytes([cbw.cb[2], cbw.cb[3], cbw.cb[4], cbw.cb[5]]);
        let blocks = u16::from_be_bytes([cbw.cb[7], cbw.cb[8]]) as u32;

        let Some(total_bytes) = blocks.checked_mul(block_device.block_size()) else {
            self.set_sense(SENSE_KEY_ILLEGAL_REQUEST, ASC_INVALID_FIELD_IN_CDB, ASCQ_NONE);
            return Ok(CommandResult::failed(cbw.data_transfer_length));
        };

        if cbw.data_transfer_length != total_bytes {
            self.set_sense(SENSE_KEY_ILLEGAL_REQUEST, ASC_INVALID_FIELD_IN_CDB, ASCQ_NONE);
            return Ok(CommandResult::failed(cbw.data_transfer_length));
        }

        let Some(last_lba) = lba.checked_add(blocks) else {
            self.set_sense(
                SENSE_KEY_ILLEGAL_REQUEST,
                ASC_LOGICAL_BLOCK_ADDRESS_OUT_OF_RANGE,
                ASCQ_NONE,
            );
            return Ok(CommandResult::failed(cbw.data_transfer_length));
        };

        if last_lba > block_device.block_count() {
            self.set_sense(
                SENSE_KEY_ILLEGAL_REQUEST,
                ASC_LOGICAL_BLOCK_ADDRESS_OUT_OF_RANGE,
                ASCQ_NONE,
            );
            return Ok(CommandResult::failed(cbw.data_transfer_length));
        }

        let mut residue = cbw.data_transfer_length;
        for i in 0..blocks {
            if block_device
                .read_block(lba + i, &mut block_buf[..block_size])
                .await
                .is_err()
            {
                warn!("msc: read_block failed at lba {}", lba + i);
                self.set_sense(SENSE_KEY_MEDIUM_ERROR, ASC_UNRECOVERED_READ_ERROR, ASCQ_NONE);
                if residue < cbw.data_transfer_length {
                    let _ = self.write_ep.write(&[]).await;
                }
                return Ok(CommandResult::failed(residue));
            }

            self.write_all_in(&block_buf[..block_size]).await?;
            residue = residue.saturating_sub(block_size as u32);
        }

        self.sense = SenseData::NO_SENSE;
        Ok(CommandResult::passed(residue))
    }

    async fn write_10<B: AsyncBlockDevice>(
        &mut self,
        cbw: &Cbw,
        block_device: &mut B,
        block_buf: &mut [u8],
    ) -> Result<CommandResult, EndpointError> {
        if cbw.data_transfer_length > 0 && cbw.direction_in() {
            return Ok(CommandResult::phase_error(cbw.data_transfer_length));
        }

        if block_buf.is_empty() {
            self.set_sense(SENSE_KEY_ILLEGAL_REQUEST, ASC_INVALID_FIELD_IN_CDB, ASCQ_NONE);
            return Ok(CommandResult::failed(cbw.data_transfer_length));
        }

        let block_size = block_device.block_size() as usize;
        if block_size == 0 || block_buf.len() < block_size {
            self.set_sense(SENSE_KEY_ILLEGAL_REQUEST, ASC_INVALID_FIELD_IN_CDB, ASCQ_NONE);
            self.discard_out_data(cbw.data_transfer_length, block_buf).await?;
            return Ok(CommandResult::failed(cbw.data_transfer_length));
        }

        let lba = u32::from_be_bytes([cbw.cb[2], cbw.cb[3], cbw.cb[4], cbw.cb[5]]);
        let blocks = u16::from_be_bytes([cbw.cb[7], cbw.cb[8]]) as u32;

        let Some(total_bytes) = blocks.checked_mul(block_device.block_size()) else {
            self.set_sense(SENSE_KEY_ILLEGAL_REQUEST, ASC_INVALID_FIELD_IN_CDB, ASCQ_NONE);
            self.discard_out_data(cbw.data_transfer_length, block_buf).await?;
            return Ok(CommandResult::failed(cbw.data_transfer_length));
        };

        if cbw.data_transfer_length != total_bytes {
            self.set_sense(SENSE_KEY_ILLEGAL_REQUEST, ASC_INVALID_FIELD_IN_CDB, ASCQ_NONE);
            self.discard_out_data(cbw.data_transfer_length, block_buf).await?;
            return Ok(CommandResult::failed(cbw.data_transfer_length));
        }

        let Some(last_lba) = lba.checked_add(blocks) else {
            self.set_sense(
                SENSE_KEY_ILLEGAL_REQUEST,
                ASC_LOGICAL_BLOCK_ADDRESS_OUT_OF_RANGE,
                ASCQ_NONE,
            );
            self.discard_out_data(cbw.data_transfer_length, block_buf).await?;
            return Ok(CommandResult::failed(cbw.data_transfer_length));
        };

        if last_lba > block_device.block_count() {
            self.set_sense(
                SENSE_KEY_ILLEGAL_REQUEST,
                ASC_LOGICAL_BLOCK_ADDRESS_OUT_OF_RANGE,
                ASCQ_NONE,
            );
            self.discard_out_data(cbw.data_transfer_length, block_buf).await?;
            return Ok(CommandResult::failed(cbw.data_transfer_length));
        }

        if block_device.is_write_protected() {
            self.set_sense(SENSE_KEY_DATA_PROTECT, ASC_WRITE_PROTECTED, ASCQ_NONE);
            self.discard_out_data(cbw.data_transfer_length, block_buf).await?;
            return Ok(CommandResult::failed(cbw.data_transfer_length));
        }

        let mut residue = cbw.data_transfer_length;

        for i in 0..blocks {
            self.read_exact_out(&mut block_buf[..block_size]).await?;

            if block_device
                .write_block(lba + i, &block_buf[..block_size])
                .await
                .is_err()
            {
                warn!("msc: write_block failed at lba {}", lba + i);
                self.set_sense(SENSE_KEY_MEDIUM_ERROR, ASC_WRITE_ERROR, ASCQ_NONE);

                let unwritten_residue = residue;
                let remaining_to_drain = residue.saturating_sub(block_size as u32);
                if remaining_to_drain > 0 {
                    self.discard_out_data(remaining_to_drain, block_buf).await?;
                }

                return Ok(CommandResult::failed(unwritten_residue));
            }

            residue = residue.saturating_sub(block_size as u32);
        }

        if block_device.flush().await.is_err() {
            warn!("msc: flush after write failed");
            self.set_sense(SENSE_KEY_MEDIUM_ERROR, ASC_WRITE_ERROR, ASCQ_NONE);
            return Ok(CommandResult::failed(0));
        }

        self.sense = SenseData::NO_SENSE;
        Ok(CommandResult::passed(0))
    }

    fn start_stop_unit(&mut self, cbw: &Cbw) -> CommandResult {
        if cbw.data_transfer_length != 0 {
            self.set_sense(SENSE_KEY_ILLEGAL_REQUEST, ASC_INVALID_FIELD_IN_CDB, ASCQ_NONE);
            return CommandResult::failed(cbw.data_transfer_length);
        }

        self.sense = SenseData::NO_SENSE;
        CommandResult::passed(0)
    }

    fn prevent_allow_medium_removal(&mut self, cbw: &Cbw) -> CommandResult {
        if cbw.data_transfer_length != 0 {
            self.set_sense(SENSE_KEY_ILLEGAL_REQUEST, ASC_INVALID_FIELD_IN_CDB, ASCQ_NONE);
            return CommandResult::failed(cbw.data_transfer_length);
        }

        self.sense = SenseData::NO_SENSE;
        CommandResult::passed(0)
    }

    async fn synchronize_cache_10<B: AsyncBlockDevice>(
        &mut self,
        cbw: &Cbw,
        block_device: &mut B,
    ) -> Result<CommandResult, EndpointError> {
        if cbw.data_transfer_length != 0 {
            self.set_sense(SENSE_KEY_ILLEGAL_REQUEST, ASC_INVALID_FIELD_IN_CDB, ASCQ_NONE);
            return Ok(CommandResult::failed(cbw.data_transfer_length));
        }

        if block_device.flush().await.is_err() {
            warn!("msc: flush failed");
            self.set_sense(SENSE_KEY_MEDIUM_ERROR, ASC_WRITE_ERROR, ASCQ_NONE);
            return Ok(CommandResult::failed(0));
        }

        self.sense = SenseData::NO_SENSE;
        Ok(CommandResult::passed(0))
    }

    async fn send_in_data(&mut self, cbw: &Cbw, data: &[u8]) -> Result<CommandResult, EndpointError> {
        let transfer_len = min(data.len(), cbw.data_transfer_length as usize);
        self.write_all_in(&data[..transfer_len]).await?;

        // If transfer completed with fewer bytes than expected by the host, and the
        // length transferred is a multiple of max_packet_size (including 0), send a ZLP
        // to terminate the USB transfer.
        if (transfer_len as u32) < cbw.data_transfer_length && transfer_len % self.config.max_packet_size as usize == 0
        {
            self.write_ep.write(&[]).await?;
        }

        let residue = cbw.data_transfer_length.saturating_sub(transfer_len as u32);
        Ok(CommandResult::passed(residue))
    }

    async fn write_all_in(&mut self, data: &[u8]) -> Result<(), EndpointError> {
        let max_packet_size = self.config.max_packet_size as usize;
        for chunk in data.chunks(max_packet_size) {
            self.write_ep.write(chunk).await?;
        }
        Ok(())
    }

    async fn read_exact_out(&mut self, mut buf: &mut [u8]) -> Result<(), EndpointError> {
        while !buf.is_empty() {
            let n = self.read_ep.read(buf).await?;
            if n == 0 {
                continue;
            }
            buf = &mut buf[n..];
        }

        Ok(())
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

    fn set_sense(&mut self, key: SenseKey, asc: u8, ascq: u8) {
        self.sense = SenseData { key, asc, ascq };
    }
}

#[derive(Clone, Copy)]
struct CommandResult {
    residue: u32,
    status: u8,
}

impl CommandResult {
    const fn passed(residue: u32) -> Self {
        Self {
            residue,
            status: CSW_STATUS_PASSED,
        }
    }

    const fn failed(residue: u32) -> Self {
        Self {
            residue,
            status: CSW_STATUS_FAILED,
        }
    }

    const fn phase_error(residue: u32) -> Self {
        Self {
            residue,
            status: CSW_STATUS_PHASE_ERROR,
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
        fn read_block(&mut self, lba: u32, buf: &mut [u8]) -> Result<(), Self::Error> {
            buf.copy_from_slice(&self.blocks[lba as usize]);
            Ok(())
        }
        fn write_block(&mut self, lba: u32, data: &[u8]) -> Result<(), Self::Error> {
            self.blocks[lba as usize].copy_from_slice(data);
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
        let mut buf = [0u8; 512];
        embassy_futures::block_on(async {
            AsyncBlockDevice::read_block(&mut dev, 1, &mut buf).await.unwrap();
            core::assert_eq!(buf[0], 42);
        });
    }
}

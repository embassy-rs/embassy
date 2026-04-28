//! USB Mass Storage Class (MSC) implementation.
//!
//! This implements the USB Bulk-Only Transport (BOT) protocol with a
//! SCSI transparent command set suitable for a simple block device.

use core::cell::Cell;
use core::cmp::min;
use core::future::{Future, Ready, ready};
use core::mem::MaybeUninit;

use embassy_sync::blocking_mutex::CriticalSectionMutex;

use crate::control::{InResponse, OutResponse, Recipient, Request, RequestType};
use crate::driver::{Driver, Endpoint, EndpointError, EndpointIn, EndpointOut};
use crate::types::InterfaceNumber;
use crate::{Builder, Handler};

/// This should be used as `device_class` when building a pure MSC device.
pub const USB_CLASS_MSC: u8 = 0x08;

const USB_SUBCLASS_SCSI_TRANSPARENT: u8 = 0x06;
const USB_PROTOCOL_BULK_ONLY: u8 = 0x50;

const BOT_REQ_RESET: u8 = 0xff;
const BOT_REQ_GET_MAX_LUN: u8 = 0xfe;

const CBW_SIGNATURE: u32 = 0x4342_5355;
const CSW_SIGNATURE: u32 = 0x5342_5355;

const CSW_STATUS_PASSED: u8 = 0x00;
const CSW_STATUS_FAILED: u8 = 0x01;
const CSW_STATUS_PHASE_ERROR: u8 = 0x02;

const SCSI_TEST_UNIT_READY: u8 = 0x00;
const SCSI_REQUEST_SENSE: u8 = 0x03;
const SCSI_INQUIRY: u8 = 0x12;
const SCSI_MODE_SENSE_6: u8 = 0x1a;
const SCSI_START_STOP_UNIT: u8 = 0x1b;
const SCSI_PREVENT_ALLOW_MEDIUM_REMOVAL: u8 = 0x1e;
const SCSI_READ_FORMAT_CAPACITIES: u8 = 0x23;
const SCSI_READ_CAPACITY_10: u8 = 0x25;
const SCSI_READ_10: u8 = 0x28;
const SCSI_WRITE_10: u8 = 0x2a;
const SCSI_SYNCHRONIZE_CACHE_10: u8 = 0x35;

const SENSE_KEY_NO_SENSE: u8 = 0x00;
const SENSE_KEY_MEDIUM_ERROR: u8 = 0x03;
const SENSE_KEY_ILLEGAL_REQUEST: u8 = 0x05;
const SENSE_KEY_DATA_PROTECT: u8 = 0x07;

const ASC_INVALID_COMMAND_OPERATION_CODE: u8 = 0x20;
const ASC_LOGICAL_BLOCK_ADDRESS_OUT_OF_RANGE: u8 = 0x21;
const ASC_INVALID_FIELD_IN_CDB: u8 = 0x24;
const ASC_WRITE_PROTECTED: u8 = 0x27;
const ASC_UNRECOVERED_READ_ERROR: u8 = 0x11;
const ASC_WRITE_ERROR: u8 = 0x0c;
const ASCQ_NONE: u8 = 0x00;

const VPD_PAGE_SUPPORTED_PAGES: u8 = 0x00;
const VPD_PAGE_UNIT_SERIAL_NUMBER: u8 = 0x80;
const VPD_PAGE_DEVICE_IDENTIFICATION: u8 = 0x83;

/// Trait implemented by block devices used by [`MscClass`].
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
/// You can implement this trait directly for truly asynchronous storage, or
/// implement [`BlockDevice`] and rely on the blanket adapter below.
pub trait AsyncBlockDevice {
    /// Error type returned by storage operations.
    type Error;

    /// Returns the block size in bytes.
    fn block_size(&self) -> u32;

    /// Returns the total amount of logical blocks.
    fn block_count(&self) -> u32;

    /// Future returned by [`Self::read_block`].
    type ReadFuture<'a>: Future<Output = Result<(), Self::Error>>
    where
        Self: 'a;

    /// Reads one logical block at `lba` into `buf`.
    fn read_block<'a>(&'a mut self, lba: u32, buf: &'a mut [u8]) -> Self::ReadFuture<'a>;

    /// Future returned by [`Self::write_block`].
    type WriteFuture<'a>: Future<Output = Result<(), Self::Error>>
    where
        Self: 'a;

    /// Writes one logical block at `lba` from `data`.
    fn write_block<'a>(&'a mut self, lba: u32, data: &'a [u8]) -> Self::WriteFuture<'a>;

    /// Future returned by [`Self::flush`].
    type FlushFuture<'a>: Future<Output = Result<(), Self::Error>>
    where
        Self: 'a;

    /// Flushes pending writes to backing storage.
    fn flush<'a>(&'a mut self) -> Self::FlushFuture<'a>;

    /// Returns whether the media is write-protected.
    fn is_write_protected(&self) -> bool {
        false
    }
}

impl<T: BlockDevice + ?Sized> AsyncBlockDevice for T {
    type Error = T::Error;
    type ReadFuture<'a>
        = Ready<Result<(), Self::Error>>
    where
        Self: 'a;
    type WriteFuture<'a>
        = Ready<Result<(), Self::Error>>
    where
        Self: 'a;
    type FlushFuture<'a>
        = Ready<Result<(), Self::Error>>
    where
        Self: 'a;

    fn block_size(&self) -> u32 {
        BlockDevice::block_size(self)
    }

    fn block_count(&self) -> u32 {
        BlockDevice::block_count(self)
    }

    fn read_block<'a>(&'a mut self, lba: u32, buf: &'a mut [u8]) -> Self::ReadFuture<'a> {
        ready(BlockDevice::read_block(self, lba, buf))
    }

    fn write_block<'a>(&'a mut self, lba: u32, data: &'a [u8]) -> Self::WriteFuture<'a> {
        ready(BlockDevice::write_block(self, lba, data))
    }

    fn flush<'a>(&'a mut self) -> Self::FlushFuture<'a> {
        ready(BlockDevice::flush(self))
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
            BOT_REQ_RESET if req.length == 0 && data.is_empty() => {
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
            BOT_REQ_GET_MAX_LUN if req.length == 1 && !buf.is_empty() => {
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
    max_packet_size: usize,
    sense: SenseData,
}

impl<'d, D: Driver<'d>> MscClass<'d, D> {
    /// Creates a new MSC class with one BULK OUT and one BULK IN endpoint.
    pub fn new(builder: &mut Builder<'d, D>, state: &'d mut State<'d>, max_packet_size: u16) -> Self {
        let mut func = builder.function(USB_CLASS_MSC, USB_SUBCLASS_SCSI_TRANSPARENT, USB_PROTOCOL_BULK_ONLY);

        let mut iface = func.interface();
        let interface = iface.interface_number();
        let mut alt = iface.alt_setting(
            USB_CLASS_MSC,
            USB_SUBCLASS_SCSI_TRANSPARENT,
            USB_PROTOCOL_BULK_ONLY,
            None,
        );

        let read_ep = alt.endpoint_bulk_out(None, max_packet_size);
        let write_ep = alt.endpoint_bulk_in(None, max_packet_size);

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
            max_packet_size: max_packet_size as usize,
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
    /// `block_buf` is a temporary buffer used for one block transfer and must be at
    /// least `block_device.block_size()` bytes long.
    pub async fn run<B: AsyncBlockDevice>(&mut self, block_device: &mut B, block_buf: &mut [u8]) -> !
    where
        B::Error: core::fmt::Debug,
    {
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
    ) -> Result<(), EndpointError>
    where
        B::Error: core::fmt::Debug,
    {
        loop {
            let reset_requested = self.reset_requested.lock(|flag| {
                let was_requested = flag.get();
                flag.set(false);
                was_requested
            });
            if reset_requested {
                self.sense = SenseData::NO_SENSE;
            }

            let Some(cbw) = self.read_cbw().await? else {
                continue;
            };

            let result = self.process_cbw(block_device, block_buf, &cbw).await?;

            self.write_csw(cbw.tag, result.residue, result.status).await?;
        }
    }

    async fn read_cbw(&mut self) -> Result<Option<Cbw>, EndpointError> {
        let mut raw = [0u8; 31];
        let n = self.read_ep.read(&mut raw).await?;

        if n != raw.len() {
            warn!("msc: invalid CBW size {}", n);
            return Ok(None);
        }

        let cbw = match parse_cbw(&raw) {
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
    ) -> Result<CommandResult, EndpointError>
    where
        B::Error: core::fmt::Debug,
    {
        match cbw.cb[0] {
            SCSI_TEST_UNIT_READY => Ok(self.test_unit_ready(cbw)),
            SCSI_REQUEST_SENSE => self.request_sense(cbw).await,
            SCSI_INQUIRY => self.inquiry(cbw).await,
            SCSI_MODE_SENSE_6 => self.mode_sense_6(cbw, block_device).await,
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
        }
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
        data[2] = self.sense.key;
        data[7] = 10;
        data[12] = self.sense.asc;
        data[13] = self.sense.ascq;

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
        let allocation_len = cbw.cb[4] as usize;

        // CmdDt is obsolete and unsupported.
        if cmd_dt {
            self.set_sense(SENSE_KEY_ILLEGAL_REQUEST, ASC_INVALID_FIELD_IN_CDB, ASCQ_NONE);
            return Ok(CommandResult::failed(cbw.data_transfer_length));
        }

        if evpd {
            let result = match page_code {
                VPD_PAGE_SUPPORTED_PAGES => {
                    // List only the pages we implement.
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
                    // Stable short serial so host can cache device identity.
                    const SERIAL: &[u8; 8] = b"EMBMSC01";
                    let mut data = [0u8; 12];
                    data[0] = 0x00;
                    data[1] = VPD_PAGE_UNIT_SERIAL_NUMBER;
                    data[2] = 0x00;
                    data[3] = SERIAL.len() as u8;
                    data[4..12].copy_from_slice(SERIAL);
                    let len = min(data.len(), allocation_len);
                    self.send_in_data(cbw, &data[..len]).await?
                }
                VPD_PAGE_DEVICE_IDENTIFICATION => {
                    // One ASCII vendor-specific identifier descriptor.
                    // Descriptor length = 16, page length = 4 + 16.
                    let mut data = [0u8; 24];
                    data[0] = 0x00;
                    data[1] = VPD_PAGE_DEVICE_IDENTIFICATION;
                    data[2] = 0x00;
                    data[3] = 20;
                    data[4] = 0x01; // binary protocol, association: logical unit
                    data[5] = 0x00; // identifier type: vendor specific
                    data[6] = 0x00;
                    data[7] = 16; // identifier length
                    data[8..24].copy_from_slice(b"EMBASSY_MSC_DISK");
                    let len = min(data.len(), allocation_len);
                    self.send_in_data(cbw, &data[..len]).await?
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
        data[8..16].copy_from_slice(b"Embassy ");
        data[16..32].copy_from_slice(b"MSC Disk        ");
        data[32..36].copy_from_slice(b"0.1 ");

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
        data[0] = 3;
        data[2] = if block_device.is_write_protected() { 0x80 } else { 0x00 };

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

        let result = self.send_in_data(cbw, &data).await?;
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
    ) -> Result<CommandResult, EndpointError>
    where
        B::Error: core::fmt::Debug,
    {
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
                warn!("msc: read_block failed");
                self.set_sense(SENSE_KEY_MEDIUM_ERROR, ASC_UNRECOVERED_READ_ERROR, ASCQ_NONE);
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
    ) -> Result<CommandResult, EndpointError>
    where
        B::Error: core::fmt::Debug,
    {
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
                warn!("msc: write_block failed");
                self.set_sense(SENSE_KEY_MEDIUM_ERROR, ASC_WRITE_ERROR, ASCQ_NONE);

                residue = residue.saturating_sub(block_size as u32);
                if residue > 0 {
                    self.discard_out_data(residue, block_buf).await?;
                }

                return Ok(CommandResult::failed(0));
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
    ) -> Result<CommandResult, EndpointError>
    where
        B::Error: core::fmt::Debug,
    {
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

        let residue = cbw.data_transfer_length.saturating_sub(transfer_len as u32);
        Ok(CommandResult::passed(residue))
    }

    async fn write_all_in(&mut self, data: &[u8]) -> Result<(), EndpointError> {
        for chunk in data.chunks(self.max_packet_size) {
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

        while len > 0 {
            let want = min(len as usize, scratch.len());
            let n = self.read_ep.read(&mut scratch[..want]).await?;
            if n == 0 {
                continue;
            }
            len -= n as u32;
        }

        Ok(())
    }

    fn set_sense(&mut self, key: u8, asc: u8, ascq: u8) {
        self.sense = SenseData { key, asc, ascq };
    }
}

#[derive(Clone, Copy)]
struct SenseData {
    key: u8,
    asc: u8,
    ascq: u8,
}

impl SenseData {
    const NO_SENSE: Self = Self {
        key: SENSE_KEY_NO_SENSE,
        asc: 0,
        ascq: 0,
    };
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Cbw {
    tag: u32,
    data_transfer_length: u32,
    flags: u8,
    lun: u8,
    cb_length: u8,
    cb: [u8; 16],
}

impl Cbw {
    fn direction_in(&self) -> bool {
        self.flags & 0x80 != 0
    }
}

#[derive(Clone, Copy)]
struct Csw {
    tag: u32,
    residue: u32,
    status: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ParseCbwError {
    InvalidLength,
    InvalidSignature,
    InvalidCbLength,
}

fn parse_cbw(raw: &[u8]) -> Result<Cbw, ParseCbwError> {
    if raw.len() != 31 {
        return Err(ParseCbwError::InvalidLength);
    }

    let signature = u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]);
    if signature != CBW_SIGNATURE {
        return Err(ParseCbwError::InvalidSignature);
    }

    let cb_length = raw[14] & 0x1f;
    if cb_length == 0 || cb_length > 16 {
        return Err(ParseCbwError::InvalidCbLength);
    }

    let mut cb = [0u8; 16];
    cb.copy_from_slice(&raw[15..31]);

    Ok(Cbw {
        tag: u32::from_le_bytes([raw[4], raw[5], raw[6], raw[7]]),
        data_transfer_length: u32::from_le_bytes([raw[8], raw[9], raw[10], raw[11]]),
        flags: raw[12],
        lun: raw[13],
        cb_length,
        cb,
    })
}

fn encode_csw(csw: Csw) -> [u8; 13] {
    let mut out = [0u8; 13];
    out[0..4].copy_from_slice(&CSW_SIGNATURE.to_le_bytes());
    out[4..8].copy_from_slice(&csw.tag.to_le_bytes());
    out[8..12].copy_from_slice(&csw.residue.to_le_bytes());
    out[12] = csw.status;
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_cbw() {
        let mut raw = [0u8; 31];
        raw[0..4].copy_from_slice(&CBW_SIGNATURE.to_le_bytes());
        raw[4..8].copy_from_slice(&0x1122_3344u32.to_le_bytes());
        raw[8..12].copy_from_slice(&0x5566_7788u32.to_le_bytes());
        raw[12] = 0x80;
        raw[13] = 0x00;
        raw[14] = 10;
        raw[15] = SCSI_INQUIRY;

        let cbw = parse_cbw(&raw).unwrap();
        assert_eq!(cbw.tag, 0x1122_3344);
        assert_eq!(cbw.data_transfer_length, 0x5566_7788);
        assert!(cbw.direction_in());
        assert_eq!(cbw.cb_length, 10);
        assert_eq!(cbw.cb[0], SCSI_INQUIRY);
    }

    #[test]
    fn rejects_bad_signature() {
        let mut raw = [0u8; 31];
        raw[14] = 6;
        assert_eq!(parse_cbw(&raw), Err(ParseCbwError::InvalidSignature));
    }

    #[test]
    fn rejects_bad_cb_length() {
        let mut raw = [0u8; 31];
        raw[0..4].copy_from_slice(&CBW_SIGNATURE.to_le_bytes());
        raw[14] = 0;
        assert_eq!(parse_cbw(&raw), Err(ParseCbwError::InvalidCbLength));
    }

    #[test]
    fn encodes_csw() {
        let raw = encode_csw(Csw {
            tag: 0xAABB_CCDD,
            residue: 0x0102_0304,
            status: CSW_STATUS_FAILED,
        });

        assert_eq!(&raw[0..4], &CSW_SIGNATURE.to_le_bytes());
        assert_eq!(&raw[4..8], &0xAABB_CCDDu32.to_le_bytes());
        assert_eq!(&raw[8..12], &0x0102_0304u32.to_le_bytes());
        assert_eq!(raw[12], CSW_STATUS_FAILED);
    }
}

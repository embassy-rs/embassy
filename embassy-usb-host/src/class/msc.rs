//! USB Mass Storage Class host driver (Bulk-Only Transport, SCSI transparent).
//!
//! Implements the MSC BBB transport (USB MSC BBB r1.0): every SCSI
//! command runs as a CBW → optional data phase → CSW cycle on a pair
//! of bulk endpoints, with stall recovery via `CLEAR_FEATURE` and
//! phase-error recovery via the class-specific Bulk-Only Mass Storage
//! Reset. Only subclass `0x06` (SCSI transparent) / protocol `0x50`
//! (BBB) interfaces are recognized.
//!
//! An [`MscDevice`] owns the control and bulk pipes for one MSC
//! interface. Per-LUN handles are opened with [`MscDevice::lun`];
//! all LUNs share the same transport and their commands serialize
//! through the device's internal async mutex.
//!
//! [`MscDevice::new`] guards the transport with a [`NoopRawMutex`];
//! use [`MscDevice::new_with_raw_mutex`] with a `Sync` raw mutex
//! (e.g. `CriticalSectionRawMutex`) when LUNs are driven from
//! multiple tasks.
//!
//! # Example
//!
//! ```rust,ignore
//! use embassy_usb_host::class::msc::MscDevice;
//!
//! let device = MscDevice::new(&bus, &enum_info, &config_buf[..config_len]).await?;
//! let mut lun = device.lun(0)?;
//!
//! let mut inq = [0u8; 36];
//! let info = lun.inquiry(&mut inq).await?;
//! let cap = lun.capacity().await?;
//!
//! let mut block = [0u8; 512];
//! lun.read_blocks(0, &mut block).await?;
//! ```

use core::marker::PhantomData;

use embassy_sync::blocking_mutex::raw::{NoopRawMutex, RawMutex};
use embassy_sync::mutex::Mutex;
use embassy_usb_driver::host::{PipeError, SplitInfo, UsbHostAllocator, UsbPipe, pipe};
use embassy_usb_driver::{Direction as UsbDirection, EndpointAddress, EndpointInfo, EndpointType};

use crate::control::{ControlType, Recipient, RequestType, SetupPacket};
use crate::descriptor::ConfigurationDescriptorChain;
use crate::handler::EnumerationInfo;

// MSC BBB r1.0 §4.
const CLASS_MSC: u8 = 0x08;
const SUBCLASS_SCSI: u8 = 0x06;
const PROTOCOL_BBB: u8 = 0x50;

// Class-specific requests (MSC BBB r1.0 §3).
const REQ_GET_MAX_LUN: u8 = 0xFE;
const REQ_BULK_ONLY_RESET: u8 = 0xFF;

// Standard endpoint requests for stall recovery (USB 2.0 §9.4).
const REQ_CLEAR_FEATURE: u8 = 0x01;
const FEATURE_ENDPOINT_HALT: u16 = 0x0000;

// CBW / CSW (MSC BBB r1.0 §5).
const CBW_SIGNATURE: u32 = 0x43425355; // "USBC"
const CSW_SIGNATURE: u32 = 0x53425355; // "USBS"
const CBW_LEN: usize = 31;
const CSW_LEN: usize = 13;
const CBW_FLAG_IN: u8 = 0x80;

// CSW status values.
const CSW_PASSED: u8 = 0x00;
const CSW_FAILED: u8 = 0x01;
const CSW_PHASE_ERROR: u8 = 0x02;

// SCSI opcodes (SPC-3 / SBC-3).
const SCSI_TEST_UNIT_READY: u8 = 0x00;
const SCSI_REQUEST_SENSE: u8 = 0x03;
const SCSI_INQUIRY: u8 = 0x12;
const SCSI_PREVENT_ALLOW_REMOVAL: u8 = 0x1E;
const SCSI_READ_CAPACITY_10: u8 = 0x25;
const SCSI_READ_10: u8 = 0x28;
const SCSI_WRITE_10: u8 = 0x2A;
const SCSI_SYNCHRONIZE_CACHE_10: u8 = 0x35;
const SCSI_READ_16: u8 = 0x88;
const SCSI_WRITE_16: u8 = 0x8A;
const SCSI_SERVICE_ACTION_IN_16: u8 = 0x9E;
const SCSI_SA_READ_CAPACITY_16: u8 = 0x10;

/// MSC host driver error.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum MscError {
    /// Transfer error.
    Transfer(PipeError),
    /// No SCSI/BBB interface in the configuration descriptor.
    NoInterface,
    /// Failed to allocate a pipe.
    NoPipe,
    /// Device response had unexpected length or out-of-range field.
    InvalidResponse,
    /// CBW/CSW signature or tag mismatch.
    Protocol,
    /// Device reported CSW status = 2 (phase error). The transport has
    /// been reset; retry the command.
    PhaseError,
    /// SCSI command failed; sense data was fetched via `REQUEST SENSE`.
    Scsi(SenseData),
    /// Buffer length is not a multiple of the LUN's block size.
    Unaligned,
    /// LBA or block count is out of range for the LUN's capacity.
    OutOfRange,
    /// LUN index >= `num_luns()`.
    NoSuchLun,
    /// CDB length must be in `1..=16`.
    InvalidCdb,
    /// The LUN's reported block size does not match the size requested
    /// by the caller (e.g. the `SIZE` generic on a
    /// [`block_device_driver::BlockDevice`] impl).
    #[cfg(feature = "block-device-driver")]
    BlockSizeMismatch,
}

impl From<PipeError> for MscError {
    fn from(e: PipeError) -> Self {
        Self::Transfer(e)
    }
}

impl core::fmt::Display for MscError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Transfer(_) => write!(f, "Transfer error"),
            Self::NoInterface => write!(f, "No MSC BBB/SCSI interface found"),
            Self::NoPipe => write!(f, "No free pipe"),
            Self::InvalidResponse => write!(f, "Invalid response from device"),
            Self::Protocol => write!(f, "BBB protocol violation"),
            Self::PhaseError => write!(f, "BBB phase error"),
            Self::Scsi(_) => write!(f, "SCSI command failed"),
            Self::Unaligned => write!(f, "Buffer is not block-aligned"),
            Self::OutOfRange => write!(f, "LBA out of range"),
            Self::NoSuchLun => write!(f, "No such LUN"),
            Self::InvalidCdb => write!(f, "Invalid CDB length"),
            #[cfg(feature = "block-device-driver")]
            Self::BlockSizeMismatch => write!(f, "Block size mismatch"),
        }
    }
}

impl core::error::Error for MscError {}

/// Direction and buffer for the optional data phase of a SCSI command.
pub enum DataDir<'a> {
    /// No data phase (e.g. `TEST UNIT READY`).
    None,
    /// Device-to-host: bytes are read into `buf`.
    In(&'a mut [u8]),
    /// Host-to-device: bytes in `buf` are sent.
    Out(&'a [u8]),
}

impl DataDir<'_> {
    fn len(&self) -> u32 {
        match self {
            Self::None => 0,
            Self::In(b) => b.len() as u32,
            Self::Out(b) => b.len() as u32,
        }
    }

    fn cbw_flags(&self) -> u8 {
        match self {
            Self::In(_) => CBW_FLAG_IN,
            _ => 0,
        }
    }
}

/// Result of a [`MscDevice::command`] cycle whose CSW was received.
///
/// Transport-level problems (stall without recovery, protocol violation,
/// phase error) return [`MscError`] instead.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CommandOutcome {
    /// CSW status = `0x00` (Passed).
    Ok {
        /// `dCSWDataResidue`: bytes of the announced data transfer that
        /// were not consumed.
        residue: u32,
    },
    /// CSW status = `0x01` (Failed). Issue `REQUEST SENSE` for detail.
    Failed {
        /// `dCSWDataResidue`.
        residue: u32,
    },
}

// --------------------------------------------------------------------------
// SCSI types.
// --------------------------------------------------------------------------

/// SCSI peripheral-device type (SPC-3 §6.4.2, bits 0..4 of INQUIRY byte 0).
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum PeripheralType {
    /// Direct-access block device (USB sticks, HDDs, SSDs, SD cards).
    DirectAccess,
    /// Sequential-access device (tape).
    SequentialAccess,
    /// CD/DVD.
    CdDvd,
    /// Optical memory device.
    Optical,
    /// Reduced-block-command (RBC) direct-access device.
    SimplifiedDirectAccess,
    /// Any other peripheral type.
    Other(u8),
}

impl PeripheralType {
    fn from_bits(b: u8) -> Self {
        match b & 0x1F {
            0x00 => Self::DirectAccess,
            0x01 => Self::SequentialAccess,
            0x05 => Self::CdDvd,
            0x07 => Self::Optical,
            0x0E => Self::SimplifiedDirectAccess,
            v => Self::Other(v),
        }
    }
}

/// Decoded `INQUIRY` standard response (SPC-3 §6.4.2).
///
/// String fields reference the 36-byte buffer passed to
/// [`MscLun::inquiry`]. They are ASCII, space-padded, not
/// NUL-terminated.
#[derive(Copy, Clone, Debug)]
pub struct InquiryData<'a> {
    /// Peripheral-device type (byte 0, bits 0..4).
    pub peripheral: PeripheralType,
    /// RMB bit (byte 1, bit 7): `true` if the medium can be removed.
    pub removable: bool,
    /// Vendor identification (8 bytes).
    pub vendor: &'a [u8],
    /// Product identification (16 bytes).
    pub product: &'a [u8],
    /// Product revision level (4 bytes).
    pub revision: &'a [u8],
}

/// SCSI sense key (SPC-3 §4.5.6, Table 27).
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum SenseKey {
    /// `0x0` — no error.
    NoSense = 0x0,
    /// `0x1` — command succeeded with automatic recovery.
    RecoveredError = 0x1,
    /// `0x2` — the medium is not ready.
    NotReady = 0x2,
    /// `0x3` — unrecoverable medium error.
    MediumError = 0x3,
    /// `0x4` — non-medium hardware error.
    HardwareError = 0x4,
    /// `0x5` — illegal CDB or parameter.
    IllegalRequest = 0x5,
    /// `0x6` — reset, medium change, or parameter change.
    UnitAttention = 0x6,
    /// `0x7` — write-protected medium.
    DataProtect = 0x7,
    /// `0x8` — blank medium on a device that expected data.
    BlankCheck = 0x8,
    /// `0x9` — vendor-specific.
    VendorSpecific = 0x9,
    /// `0xA` — COPY/COMPARE aborted.
    CopyAborted = 0xA,
    /// `0xB` — target aborted the command.
    AbortedCommand = 0xB,
    /// `0xD` — volume overflow on a sequential device.
    VolumeOverflow = 0xD,
    /// `0xE` — data did not match expected values.
    Miscompare = 0xE,
    /// Any reserved sense key value.
    Reserved = 0xF,
}

impl SenseKey {
    fn from_bits(b: u8) -> Self {
        match b & 0x0F {
            0x0 => Self::NoSense,
            0x1 => Self::RecoveredError,
            0x2 => Self::NotReady,
            0x3 => Self::MediumError,
            0x4 => Self::HardwareError,
            0x5 => Self::IllegalRequest,
            0x6 => Self::UnitAttention,
            0x7 => Self::DataProtect,
            0x8 => Self::BlankCheck,
            0x9 => Self::VendorSpecific,
            0xA => Self::CopyAborted,
            0xB => Self::AbortedCommand,
            0xD => Self::VolumeOverflow,
            0xE => Self::Miscompare,
            _ => Self::Reserved,
        }
    }
}

/// Decoded fixed-format sense data (SPC-3 §4.5.3).
///
/// Use the raw REQUEST SENSE response if more detail is needed.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct SenseData {
    /// Sense key (byte 2, bits 0..3).
    pub key: SenseKey,
    /// Additional Sense Code (byte 12).
    pub asc: u8,
    /// Additional Sense Code Qualifier (byte 13).
    pub ascq: u8,
}

/// Block-device capacity derived from `READ CAPACITY(10)` or `(16)`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct BlockCapacity {
    /// Total number of addressable blocks.
    pub block_count: u64,
    /// Block size in bytes.
    pub block_size: u32,
}

// --------------------------------------------------------------------------
// Descriptor walker.
// --------------------------------------------------------------------------

/// Descriptor-located info for the MSC interface.
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct MscInfo {
    /// USB interface number.
    pub interface: u8,
    /// Bulk IN endpoint address (with direction bit).
    pub bulk_in_ep: u8,
    /// Bulk IN max packet size.
    pub bulk_in_mps: u16,
    /// Bulk OUT endpoint address.
    pub bulk_out_ep: u8,
    /// Bulk OUT max packet size.
    pub bulk_out_mps: u16,
}

/// Locate the first SCSI/BBB interface in `config_desc`.
pub fn find_msc(config_desc: &[u8]) -> Option<MscInfo> {
    let cfg = ConfigurationDescriptorChain::try_from_slice(config_desc).ok()?;

    for iface in cfg.iter_interface() {
        if iface.interface_class != CLASS_MSC
            || iface.interface_subclass != SUBCLASS_SCSI
            || iface.interface_protocol != PROTOCOL_BBB
            || iface.alternate_setting != 0
        {
            continue;
        }

        let mut in_ep = None;
        let mut out_ep = None;
        for ep in iface.iter_endpoints() {
            if ep.ep_type() != EndpointType::Bulk {
                continue;
            }
            if ep.is_in() {
                in_ep = Some((ep.endpoint_address, ep.max_packet_size));
            } else {
                out_ep = Some((ep.endpoint_address, ep.max_packet_size));
            }
        }

        if let (Some((in_a, in_m)), Some((out_a, out_m))) = (in_ep, out_ep) {
            return Some(MscInfo {
                interface: iface.interface_number,
                bulk_in_ep: in_a,
                bulk_in_mps: in_m,
                bulk_out_ep: out_a,
                bulk_out_mps: out_m,
            });
        }
    }

    None
}

// --------------------------------------------------------------------------
// Transport.
// --------------------------------------------------------------------------

/// Shared, mutex-protected BBB transport state.
struct Transport<'d, A>
where
    A: UsbHostAllocator<'d>,
{
    ctrl: A::Pipe<pipe::Control, pipe::InOut>,
    bulk_in: A::Pipe<pipe::Bulk, pipe::In>,
    bulk_out: A::Pipe<pipe::Bulk, pipe::Out>,
    interface: u8,
    bulk_in_ep: u8,
    bulk_out_ep: u8,
    next_tag: u32,
    /// Set when a command was cancelled mid-transfer; the next command
    /// resets the transport before running.
    dirty: bool,
    _phantom: PhantomData<&'d ()>,
}

impl<'d, A> Transport<'d, A>
where
    A: UsbHostAllocator<'d>,
{
    async fn clear_halt_in(&mut self) -> Result<(), MscError> {
        clear_endpoint_halt(&mut self.ctrl, self.bulk_in_ep).await?;
        self.bulk_in.reset_data_toggle();
        Ok(())
    }

    async fn clear_halt_out(&mut self) -> Result<(), MscError> {
        clear_endpoint_halt(&mut self.ctrl, self.bulk_out_ep).await?;
        self.bulk_out.reset_data_toggle();
        Ok(())
    }

    async fn mass_storage_reset(&mut self) -> Result<(), MscError> {
        let setup = SetupPacket::class_interface_out(REQ_BULK_ONLY_RESET, 0, self.interface as u16, 0);
        self.ctrl.control_out(&setup.to_bytes(), &[]).await?;
        self.clear_halt_in().await?;
        self.clear_halt_out().await?;
        Ok(())
    }
}

async fn clear_endpoint_halt<P>(ctrl: &mut P, ep_addr: u8) -> Result<(), MscError>
where
    P: UsbPipe<pipe::Control, pipe::InOut>,
{
    let setup = SetupPacket {
        request_type: RequestType {
            direction: UsbDirection::Out,
            control_type: ControlType::Standard,
            recipient: Recipient::Endpoint,
        },
        request: REQ_CLEAR_FEATURE,
        value: FEATURE_ENDPOINT_HALT,
        index: ep_addr as u16,
        length: 0,
    };
    ctrl.control_out(&setup.to_bytes(), &[]).await?;
    Ok(())
}

async fn get_max_lun<P>(ctrl: &mut P, interface: u8) -> Result<u8, MscError>
where
    P: UsbPipe<pipe::Control, pipe::InOut>,
{
    let setup = SetupPacket::class_interface_in(REQ_GET_MAX_LUN, 0, interface as u16, 1);
    let mut buf = [0u8; 1];
    // Many devices stall GET_MAX_LUN — treat that as "single LUN".
    match ctrl.control_in(&setup.to_bytes(), &mut buf).await {
        Ok(1) if buf[0] <= 15 => Ok(buf[0]),
        Ok(_) => Err(MscError::InvalidResponse),
        Err(PipeError::Stall) => Ok(0),
        Err(e) => Err(e.into()),
    }
}

/// Run one CBW → data → CSW cycle. Caller holds the transport lock.
async fn run_cycle<'d, A>(
    t: &mut Transport<'d, A>,
    lun: u8,
    cdb: &[u8],
    data: DataDir<'_>,
) -> Result<CommandOutcome, MscError>
where
    A: UsbHostAllocator<'d>,
{
    let tag = {
        let tag = t.next_tag;
        t.next_tag = t.next_tag.wrapping_add(1);
        tag
    };

    let mut cbw = [0u8; CBW_LEN];
    cbw[0..4].copy_from_slice(&CBW_SIGNATURE.to_le_bytes());
    cbw[4..8].copy_from_slice(&tag.to_le_bytes());
    cbw[8..12].copy_from_slice(&data.len().to_le_bytes());
    cbw[12] = data.cbw_flags();
    cbw[13] = lun & 0x0F;
    cbw[14] = cdb.len() as u8;
    cbw[15..15 + cdb.len()].copy_from_slice(cdb);

    trace!(
        "MSC: CBW tag={:#010x} lun={} op={:#04x} data_len={} dir={}",
        tag,
        lun,
        cdb[0],
        data.len(),
        match data {
            DataDir::None => "none",
            DataDir::In(_) => "in",
            DataDir::Out(_) => "out",
        },
    );

    // CBW phase.
    if let Err(e) = t.bulk_out.request_out(&cbw, false).await {
        if matches!(e, PipeError::Stall) {
            t.clear_halt_out().await?;
        }
        return Err(e.into());
    }

    // Data phase. A stall here is recoverable: clear the halt, then
    // read the CSW.
    match data {
        DataDir::None => {}
        DataDir::In(buf) => match t.bulk_in.request_in(buf).await {
            Ok(_) => {}
            Err(PipeError::Stall) => t.clear_halt_in().await?,
            Err(e) => return Err(e.into()),
        },
        DataDir::Out(buf) => match t.bulk_out.request_out(buf, false).await {
            Ok(()) => {}
            Err(PipeError::Stall) => t.clear_halt_out().await?,
            Err(e) => return Err(e.into()),
        },
    }

    // CSW phase. A stall here gets one retry after clearing.
    let csw = match read_csw(t).await {
        Ok(b) => b,
        Err(MscError::Transfer(PipeError::Stall)) => {
            t.clear_halt_in().await?;
            read_csw(t).await?
        }
        Err(e) => return Err(e),
    };

    let signature = u32::from_le_bytes([csw[0], csw[1], csw[2], csw[3]]);
    let csw_tag = u32::from_le_bytes([csw[4], csw[5], csw[6], csw[7]]);
    let residue = u32::from_le_bytes([csw[8], csw[9], csw[10], csw[11]]);
    let status = csw[12];

    if signature != CSW_SIGNATURE || csw_tag != tag {
        warn!(
            "MSC: CSW mismatch (expected sig={:#010x} tag={:#010x}, got sig={:#010x} tag={:#010x} residue={} status={:#04x}, raw={:?})",
            CSW_SIGNATURE, tag, signature, csw_tag, residue, status, csw,
        );
        t.mass_storage_reset().await.ok();
        return Err(MscError::Protocol);
    }

    match status {
        CSW_PASSED => Ok(CommandOutcome::Ok { residue }),
        CSW_FAILED => Ok(CommandOutcome::Failed { residue }),
        CSW_PHASE_ERROR => {
            t.mass_storage_reset().await?;
            Err(MscError::PhaseError)
        }
        _ => {
            t.mass_storage_reset().await.ok();
            Err(MscError::Protocol)
        }
    }
}

async fn read_csw<'d, A>(t: &mut Transport<'d, A>) -> Result<[u8; CSW_LEN], MscError>
where
    A: UsbHostAllocator<'d>,
{
    let mut buf = [0u8; CSW_LEN];
    let n = t.bulk_in.request_in(&mut buf).await?;
    if n == CSW_LEN {
        trace!("MSC: CSW raw={:?}", buf);
        Ok(buf)
    } else {
        warn!(
            "MSC: short CSW ({} bytes, expected {}), data={:?}",
            n,
            CSW_LEN,
            &buf[..n]
        );
        Err(MscError::Protocol)
    }
}

/// Run one command on an already-locked transport: dirty-check,
/// recover if needed, run the cycle, and clear dirty on success.
async fn command_locked<'d, A>(
    t: &mut Transport<'d, A>,
    lun: u8,
    cdb: &[u8],
    data: DataDir<'_>,
) -> Result<CommandOutcome, MscError>
where
    A: UsbHostAllocator<'d>,
{
    if t.dirty {
        // A previous command was cancelled or wedged the transport;
        // re-sync before starting.
        t.mass_storage_reset().await?;
        t.dirty = false;
    }
    t.dirty = true;
    let result = run_cycle(t, lun, cdb, data).await;
    if result.is_ok() {
        t.dirty = false;
    }
    result
}

/// Issue `REQUEST SENSE` on a locked transport.
async fn request_sense_locked<'d, A>(t: &mut Transport<'d, A>, lun: u8) -> Result<SenseData, MscError>
where
    A: UsbHostAllocator<'d>,
{
    let cdb = [SCSI_REQUEST_SENSE, 0, 0, 0, 18, 0];
    let mut buf = [0u8; 18];
    // Accept Failed here too — a second failure would loop.
    let _ = command_locked(t, lun, &cdb, DataDir::In(&mut buf)).await?;
    Ok(SenseData {
        key: SenseKey::from_bits(buf[2]),
        asc: buf[12],
        ascq: buf[13],
    })
}

/// Run a command on an already-locked transport, auto-issuing
/// `REQUEST SENSE` on CSW Failed so the sense data reports the
/// command we just ran (no other task can interleave).
async fn run_with_sense_locked<'d, A>(
    t: &mut Transport<'d, A>,
    lun: u8,
    cdb: &[u8],
    data: DataDir<'_>,
) -> Result<u32, MscError>
where
    A: UsbHostAllocator<'d>,
{
    match command_locked(t, lun, cdb, data).await? {
        CommandOutcome::Ok { residue } => Ok(residue),
        CommandOutcome::Failed { .. } => Err(MscError::Scsi(request_sense_locked(t, lun).await?)),
    }
}

// --------------------------------------------------------------------------
// MscDevice.
// --------------------------------------------------------------------------

/// USB Mass Storage Class host device.
///
/// Owns the control and bulk pipes for one BBB/SCSI interface and
/// serializes all command traffic through an internal async mutex.
/// Open per-LUN handles with [`MscDevice::lun`].
///
/// Use [`MscDevice::new`] for the common single-task case; the
/// transport mutex is a [`NoopRawMutex`] with no runtime cost. To
/// share LUNs across tasks, construct with
/// [`MscDevice::new_with_raw_mutex`] and pick a `Sync` raw mutex.
pub struct MscDevice<'d, A, M = NoopRawMutex>
where
    A: UsbHostAllocator<'d>,
    M: RawMutex,
{
    transport: Mutex<M, Transport<'d, A>>,
    interface: u8,
    max_lun: u8,
    _phantom: PhantomData<&'d ()>,
}

impl<'d, A> MscDevice<'d, A, NoopRawMutex>
where
    A: UsbHostAllocator<'d>,
{
    /// Allocate the control and bulk pipes for the first BBB/SCSI
    /// interface in `config_desc`, probe `GET_MAX_LUN`, and wrap the
    /// transport in a [`NoopRawMutex`].
    ///
    /// The resulting device is `!Sync`. Use this constructor when all
    /// LUNs stay in one task. For multi-task sharing, use
    /// [`MscDevice::new_with_raw_mutex`] instead.
    pub async fn new(alloc: &A, enum_info: &EnumerationInfo, config_desc: &[u8]) -> Result<Self, MscError> {
        Self::new_with_raw_mutex(alloc, enum_info, config_desc).await
    }
}

impl<'d, A, M> MscDevice<'d, A, M>
where
    A: UsbHostAllocator<'d>,
    M: RawMutex,
{
    /// Allocate the control and bulk pipes for the first BBB/SCSI
    /// interface in `config_desc`, probe `GET_MAX_LUN`, and wrap the
    /// transport in the caller-chosen raw mutex `M`.
    ///
    /// Pick a `Sync` raw mutex (e.g. `CriticalSectionRawMutex`) to
    /// drive LUNs from multiple tasks. For single-task use, prefer
    /// [`MscDevice::new`].
    pub async fn new_with_raw_mutex(
        alloc: &A,
        enum_info: &EnumerationInfo,
        config_desc: &[u8],
    ) -> Result<Self, MscError> {
        let info = find_msc(config_desc).ok_or(MscError::NoInterface)?;

        let ctrl_ep_info = EndpointInfo {
            addr: EndpointAddress::from_parts(0, UsbDirection::In),
            ep_type: EndpointType::Control,
            max_packet_size: enum_info.device_desc.max_packet_size0 as u16,
            interval_ms: 0,
        };

        let in_ep_info = EndpointInfo {
            addr: EndpointAddress::from_parts((info.bulk_in_ep & 0x0F) as usize, UsbDirection::In),
            ep_type: EndpointType::Bulk,
            max_packet_size: info.bulk_in_mps,
            interval_ms: 0,
        };

        let out_ep_info = EndpointInfo {
            addr: EndpointAddress::from_parts((info.bulk_out_ep & 0x0F) as usize, UsbDirection::Out),
            ep_type: EndpointType::Bulk,
            max_packet_size: info.bulk_out_mps,
            interval_ms: 0,
        };

        let device_address = enum_info.device_address;
        let split: Option<SplitInfo> = enum_info.split();

        let mut ctrl = alloc
            .alloc_pipe::<pipe::Control, pipe::InOut>(device_address, &ctrl_ep_info, split)
            .map_err(|_| MscError::NoPipe)?;
        let bulk_in = alloc
            .alloc_pipe::<pipe::Bulk, pipe::In>(device_address, &in_ep_info, split)
            .map_err(|_| MscError::NoPipe)?;
        let bulk_out = alloc
            .alloc_pipe::<pipe::Bulk, pipe::Out>(device_address, &out_ep_info, split)
            .map_err(|_| MscError::NoPipe)?;

        let max_lun = get_max_lun(&mut ctrl, info.interface).await?;

        let device = Self {
            transport: Mutex::new(Transport {
                ctrl,
                bulk_in,
                bulk_out,
                interface: info.interface,
                bulk_in_ep: info.bulk_in_ep,
                bulk_out_ep: info.bulk_out_ep,
                next_tag: 1,
                dirty: false,
                _phantom: PhantomData,
            }),
            interface: info.interface,
            max_lun,
            _phantom: PhantomData,
        };

        // Put the BBB transport into a known-clean state before any real
        // command. Best-effort: a few devices stall Bulk-Only Reset — that's
        // fine, the state they care about is whatever they reset during
        // enumeration.
        if let Err(e) = device.reset().await {
            debug!("MSC: initial Bulk-Only Reset failed ({:?}); continuing anyway", e);
        }

        Ok(device)
    }

    /// USB interface number this device is bound to.
    pub fn interface(&self) -> u8 {
        self.interface
    }

    /// Highest valid LUN index (as returned by `GET_MAX_LUN`).
    pub fn max_lun(&self) -> u8 {
        self.max_lun
    }

    /// Number of LUNs exposed by the device (`max_lun() + 1`).
    pub fn num_luns(&self) -> u8 {
        self.max_lun + 1
    }

    /// Handle to the given LUN.
    ///
    /// LUN handles are cheap and do not reserve any transport
    /// resource; issuing more than one for the same LUN is permitted
    /// but only useful if the caller manages the split state between
    /// them.
    pub fn lun(&self, lun: u8) -> Result<MscLun<'_, 'd, A, M>, MscError> {
        if lun > self.max_lun {
            return Err(MscError::NoSuchLun);
        }
        Ok(MscLun {
            device: self,
            lun,
            capacity: None,
        })
    }

    /// Run one Bulk-Only command cycle and return the outcome.
    ///
    /// `cdb` must be 1..=16 bytes. The length of `data` is reported as
    /// `dCBWDataTransferLength` and drives the data phase.
    ///
    /// Recovers from endpoint stalls (via `CLEAR_FEATURE(ENDPOINT_HALT)`
    /// plus data-toggle reset) and from CSW signature/tag mismatches
    /// (via Bulk-Only Mass Storage Reset). A CSW status of `0x02`
    /// (phase error) triggers a reset and returns
    /// [`MscError::PhaseError`].
    ///
    /// # Cancellation
    ///
    /// Not cancel-safe: dropping the future mid-cycle leaves the
    /// device in an undefined state. The transport marks itself dirty
    /// and issues a Mass Storage Reset before the next command.
    pub async fn command(&self, lun: u8, cdb: &[u8], data: DataDir<'_>) -> Result<CommandOutcome, MscError> {
        if cdb.is_empty() || cdb.len() > 16 {
            return Err(MscError::InvalidCdb);
        }
        if lun > self.max_lun {
            return Err(MscError::NoSuchLun);
        }

        let mut t = self.transport.lock().await;
        command_locked(&mut t, lun, cdb, data).await
    }

    /// Issue a Bulk-Only Mass Storage Reset followed by
    /// `CLEAR_FEATURE(ENDPOINT_HALT)` on both bulk endpoints.
    pub async fn reset(&self) -> Result<(), MscError> {
        let mut t = self.transport.lock().await;
        t.mass_storage_reset().await?;
        t.dirty = false;
        Ok(())
    }
}

// --------------------------------------------------------------------------
// MscLun.
// --------------------------------------------------------------------------

/// Handle for a single Logical Unit.
///
/// Borrows the [`MscDevice`] for transport access. Convenience methods
/// wrap the SCSI commands a storage host typically needs; power users
/// can issue arbitrary CDBs via [`MscDevice::command`].
///
/// Block-I/O methods use the LUN's cached capacity. The first call to
/// [`MscLun::capacity`] fills it; [`MscLun::invalidate_capacity`]
/// clears it (useful after a `UnitAttention` indicating media change).
pub struct MscLun<'dev, 'd, A, M = NoopRawMutex>
where
    A: UsbHostAllocator<'d>,
    M: RawMutex,
{
    device: &'dev MscDevice<'d, A, M>,
    lun: u8,
    capacity: Option<BlockCapacity>,
}

impl<'dev, 'd, A, M> MscLun<'dev, 'd, A, M>
where
    A: UsbHostAllocator<'d>,
    M: RawMutex,
{
    /// LUN index.
    pub fn lun(&self) -> u8 {
        self.lun
    }

    /// Cached [`BlockCapacity`], if [`capacity`](Self::capacity) has
    /// been called.
    pub fn cached_capacity(&self) -> Option<BlockCapacity> {
        self.capacity
    }

    /// Clear the cached capacity so the next I/O re-fetches it.
    pub fn invalidate_capacity(&mut self) {
        self.capacity = None;
    }

    /// Run a single command with auto-sense on failure. Takes and
    /// releases the transport lock.
    async fn run(&mut self, cdb: &[u8], data: DataDir<'_>) -> Result<u32, MscError> {
        let mut t = self.device.transport.lock().await;
        run_with_sense_locked(&mut t, self.lun, cdb, data).await
    }

    /// Run `INQUIRY` (standard data, 36 bytes).
    pub async fn inquiry<'a>(&mut self, buf: &'a mut [u8; 36]) -> Result<InquiryData<'a>, MscError> {
        let cdb = [SCSI_INQUIRY, 0, 0, 0, 36, 0];
        self.run(&cdb, DataDir::In(&mut buf[..])).await?;
        Ok(InquiryData {
            peripheral: PeripheralType::from_bits(buf[0]),
            removable: buf[1] & 0x80 != 0,
            vendor: &buf[8..16],
            product: &buf[16..32],
            revision: &buf[32..36],
        })
    }

    /// Read SCSI sense data via `REQUEST SENSE`.
    pub async fn request_sense(&mut self) -> Result<SenseData, MscError> {
        let mut t = self.device.transport.lock().await;
        request_sense_locked(&mut t, self.lun).await
    }

    /// Probe with `TEST UNIT READY`.
    ///
    /// Returns `Ok(true)` when the unit is ready, `Ok(false)` on a
    /// transient `NotReady` or `UnitAttention` sense (e.g. medium
    /// not yet spun up or just inserted). Other failures surface as
    /// [`MscError::Scsi`].
    pub async fn test_unit_ready(&mut self) -> Result<bool, MscError> {
        let cdb = [SCSI_TEST_UNIT_READY, 0, 0, 0, 0, 0];
        // Hold the lock across the TUR → REQUEST SENSE pair so the
        // sense data reports our TUR rather than any interleaved
        // command.
        let sense = {
            let mut t = self.device.transport.lock().await;
            match command_locked(&mut t, self.lun, &cdb, DataDir::None).await? {
                CommandOutcome::Ok { .. } => return Ok(true),
                CommandOutcome::Failed { .. } => request_sense_locked(&mut t, self.lun).await?,
            }
        };
        match sense.key {
            SenseKey::NotReady | SenseKey::UnitAttention => {
                self.invalidate_capacity();
                Ok(false)
            }
            _ => Err(MscError::Scsi(sense)),
        }
    }

    /// Enable or disable medium removal by the user.
    pub async fn prevent_medium_removal(&mut self, prevent: bool) -> Result<(), MscError> {
        let cdb = [SCSI_PREVENT_ALLOW_REMOVAL, 0, 0, 0, prevent as u8, 0];
        self.run(&cdb, DataDir::None).await?;
        Ok(())
    }

    /// Fetch and cache the LUN's block capacity.
    ///
    /// Uses `READ CAPACITY(10)`; falls back to `READ CAPACITY(16)` when
    /// the device reports the sentinel `0xFFFFFFFF` (i.e. the LUN is
    /// larger than 2 TiB at 512-byte blocks). The two probes run
    /// under a single lock hold.
    pub async fn capacity(&mut self) -> Result<BlockCapacity, MscError> {
        if let Some(c) = self.capacity {
            return Ok(c);
        }

        let cap = {
            let mut t = self.device.transport.lock().await;

            let cdb = [SCSI_READ_CAPACITY_10, 0, 0, 0, 0, 0, 0, 0, 0, 0];
            let mut buf = [0u8; 8];
            run_with_sense_locked(&mut t, self.lun, &cdb, DataDir::In(&mut buf)).await?;
            let last_lba = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
            let block_size = u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]);

            if last_lba == 0xFFFF_FFFF {
                let mut cdb16 = [0u8; 16];
                cdb16[0] = SCSI_SERVICE_ACTION_IN_16;
                cdb16[1] = SCSI_SA_READ_CAPACITY_16;
                cdb16[13] = 32;
                let mut buf16 = [0u8; 32];
                run_with_sense_locked(&mut t, self.lun, &cdb16, DataDir::In(&mut buf16)).await?;
                let last_lba = u64::from_be_bytes([
                    buf16[0], buf16[1], buf16[2], buf16[3], buf16[4], buf16[5], buf16[6], buf16[7],
                ]);
                let block_size = u32::from_be_bytes([buf16[8], buf16[9], buf16[10], buf16[11]]);
                BlockCapacity {
                    block_count: last_lba.saturating_add(1),
                    block_size,
                }
            } else {
                BlockCapacity {
                    block_count: last_lba as u64 + 1,
                    block_size,
                }
            }
        };

        if cap.block_size == 0 {
            return Err(MscError::InvalidResponse);
        }

        self.capacity = Some(cap);
        Ok(cap)
    }

    /// Read `buf.len() / block_size` blocks starting at `lba`.
    ///
    /// `buf.len()` must be a non-zero multiple of `block_size`.
    ///
    /// Uses `READ(10)` when `lba` fits in `u32` and the chunk count
    /// fits in `u16`, else `READ(16)`. Large reads are transparently
    /// split into chunks of up to 65 535 blocks per command, all
    /// issued under a single transport lock hold.
    pub async fn read_blocks(&mut self, lba: u64, buf: &mut [u8]) -> Result<(), MscError> {
        let cap = self.capacity().await?;
        let (block_size, total_blocks) = check_block_args(lba, buf.len(), &cap)?;

        let mut t = self.device.transport.lock().await;

        let mut cur_lba = lba;
        let mut offset = 0usize;
        let mut remaining = total_blocks;

        while remaining > 0 {
            let (n, use_10) = chunk_blocks(cur_lba, remaining);
            let bytes = n as usize * block_size;
            let chunk = &mut buf[offset..offset + bytes];

            let residue = if use_10 {
                let cdb = read10_cdb(cur_lba as u32, n as u16);
                run_with_sense_locked(&mut t, self.lun, &cdb, DataDir::In(chunk)).await?
            } else {
                let cdb = read16_cdb(cur_lba, n);
                run_with_sense_locked(&mut t, self.lun, &cdb, DataDir::In(chunk)).await?
            };
            if residue != 0 {
                return Err(MscError::InvalidResponse);
            }

            offset += bytes;
            cur_lba += n as u64;
            remaining -= n as u64;
        }
        Ok(())
    }

    /// Write `buf.len() / block_size` blocks starting at `lba`.
    ///
    /// `buf.len()` must be a non-zero multiple of `block_size`.
    ///
    /// Uses `WRITE(10)` when `lba` fits in `u32` and the chunk count
    /// fits in `u16`, else `WRITE(16)`. Large writes are transparently
    /// split into chunks of up to 65 535 blocks per command, all
    /// issued under a single transport lock hold.
    pub async fn write_blocks(&mut self, lba: u64, buf: &[u8]) -> Result<(), MscError> {
        let cap = self.capacity().await?;
        let (block_size, total_blocks) = check_block_args(lba, buf.len(), &cap)?;

        let mut t = self.device.transport.lock().await;

        let mut cur_lba = lba;
        let mut offset = 0usize;
        let mut remaining = total_blocks;

        while remaining > 0 {
            let (n, use_10) = chunk_blocks(cur_lba, remaining);
            let bytes = n as usize * block_size;
            let chunk = &buf[offset..offset + bytes];

            let residue = if use_10 {
                let cdb = write10_cdb(cur_lba as u32, n as u16);
                run_with_sense_locked(&mut t, self.lun, &cdb, DataDir::Out(chunk)).await?
            } else {
                let cdb = write16_cdb(cur_lba, n);
                run_with_sense_locked(&mut t, self.lun, &cdb, DataDir::Out(chunk)).await?
            };
            if residue != 0 {
                return Err(MscError::InvalidResponse);
            }

            offset += bytes;
            cur_lba += n as u64;
            remaining -= n as u64;
        }
        Ok(())
    }

    /// Flush the device's write cache (`SYNCHRONIZE CACHE(10)` over
    /// the entire LUN).
    pub async fn flush(&mut self) -> Result<(), MscError> {
        let cdb = [SCSI_SYNCHRONIZE_CACHE_10, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        self.run(&cdb, DataDir::None).await?;
        Ok(())
    }

    /// Wrap this LUN as a [`block_device_driver::BlockDevice`].
    ///
    /// `ALIGN` selects the alignment of the caller's block buffers
    /// (must divide `SIZE`). The `SIZE` const generic on the trait
    /// impl is the block size in bytes; it is checked against the
    /// LUN's reported block size on every call and
    /// [`MscError::BlockSizeMismatch`] is returned on mismatch.
    #[cfg(feature = "block-device-driver")]
    pub fn as_block_device<ALIGN>(&mut self) -> MscBlockDevice<'_, 'dev, 'd, A, M, ALIGN>
    where
        ALIGN: aligned::Alignment,
    {
        MscBlockDevice {
            lun: self,
            _align: PhantomData,
        }
    }
}

// --------------------------------------------------------------------------
// block-device-driver integration.
// --------------------------------------------------------------------------

/// [`block_device_driver::BlockDevice`] adapter for an [`MscLun`].
///
/// Constructed by [`MscLun::as_block_device`]. The `ALIGN` type
/// parameter picks the buffer alignment required by the caller (for
/// example the DMA alignment of the downstream consumer). The
/// `BlockDevice` trait is implemented for every `SIZE` — the LUN's
/// reported block size is validated on every call.
#[cfg(feature = "block-device-driver")]
pub struct MscBlockDevice<'lun, 'dev, 'd, A, M, ALIGN>
where
    A: UsbHostAllocator<'d>,
    M: RawMutex,
    ALIGN: aligned::Alignment,
{
    lun: &'lun mut MscLun<'dev, 'd, A, M>,
    _align: PhantomData<fn() -> ALIGN>,
}

#[cfg(feature = "block-device-driver")]
impl<'lun, 'dev, 'd, A, M, ALIGN, const SIZE: usize> block_device_driver::BlockDevice<SIZE>
    for MscBlockDevice<'lun, 'dev, 'd, A, M, ALIGN>
where
    A: UsbHostAllocator<'d>,
    M: RawMutex,
    ALIGN: aligned::Alignment,
{
    type Error = MscError;
    type Align = ALIGN;

    async fn read(
        &mut self,
        block_address: u32,
        data: &mut [aligned::Aligned<ALIGN, [u8; SIZE]>],
    ) -> Result<(), MscError> {
        let cap = self.lun.capacity().await?;
        if cap.block_size as usize != SIZE {
            return Err(MscError::BlockSizeMismatch);
        }
        let bytes = block_device_driver::blocks_to_slice_mut(data);
        self.lun.read_blocks(block_address as u64, bytes).await
    }

    async fn write(
        &mut self,
        block_address: u32,
        data: &[aligned::Aligned<ALIGN, [u8; SIZE]>],
    ) -> Result<(), MscError> {
        let cap = self.lun.capacity().await?;
        if cap.block_size as usize != SIZE {
            return Err(MscError::BlockSizeMismatch);
        }
        let bytes = block_device_driver::blocks_to_slice(data);
        self.lun.write_blocks(block_address as u64, bytes).await
    }

    async fn size(&mut self) -> Result<u64, MscError> {
        let cap = self.lun.capacity().await?;
        Ok(cap.block_count.saturating_mul(cap.block_size as u64))
    }
}

/// Pick the next chunk size and CDB flavour.
///
/// Prefers `READ/WRITE(10)` whenever the starting LBA fits in a `u32`
/// (clamping the chunk to `u16::MAX` blocks) for maximum device
/// compatibility; only falls back to the 16-byte variants when the LBA
/// itself exceeds the 32-bit range.
fn chunk_blocks(lba: u64, remaining: u64) -> (u32, bool) {
    const MAX_BLOCKS_10: u64 = u16::MAX as u64;
    let use_10 = lba <= u32::MAX as u64;
    let n = if use_10 {
        remaining.min(MAX_BLOCKS_10) as u32
    } else {
        remaining.min(u32::MAX as u64) as u32
    };
    (n, use_10)
}

fn check_block_args(lba: u64, bytes: usize, cap: &BlockCapacity) -> Result<(usize, u64), MscError> {
    let block_size = cap.block_size as usize;
    if bytes == 0 || !bytes.is_multiple_of(block_size) {
        return Err(MscError::Unaligned);
    }
    let total_blocks = (bytes / block_size) as u64;
    if lba.checked_add(total_blocks).is_none_or(|end| end > cap.block_count) {
        return Err(MscError::OutOfRange);
    }
    Ok((block_size, total_blocks))
}

fn read10_cdb(lba: u32, blocks: u16) -> [u8; 10] {
    let lba = lba.to_be_bytes();
    let bl = blocks.to_be_bytes();
    [SCSI_READ_10, 0, lba[0], lba[1], lba[2], lba[3], 0, bl[0], bl[1], 0]
}

fn write10_cdb(lba: u32, blocks: u16) -> [u8; 10] {
    let lba = lba.to_be_bytes();
    let bl = blocks.to_be_bytes();
    [SCSI_WRITE_10, 0, lba[0], lba[1], lba[2], lba[3], 0, bl[0], bl[1], 0]
}

fn read16_cdb(lba: u64, blocks: u32) -> [u8; 16] {
    let lba = lba.to_be_bytes();
    let bl = blocks.to_be_bytes();
    [
        SCSI_READ_16,
        0,
        lba[0],
        lba[1],
        lba[2],
        lba[3],
        lba[4],
        lba[5],
        lba[6],
        lba[7],
        bl[0],
        bl[1],
        bl[2],
        bl[3],
        0,
        0,
    ]
}

fn write16_cdb(lba: u64, blocks: u32) -> [u8; 16] {
    let lba = lba.to_be_bytes();
    let bl = blocks.to_be_bytes();
    [
        SCSI_WRITE_16,
        0,
        lba[0],
        lba[1],
        lba[2],
        lba[3],
        lba[4],
        lba[5],
        lba[6],
        lba[7],
        bl[0],
        bl[1],
        bl[2],
        bl[3],
        0,
        0,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    // ----------------------------------------------------------------------
    // find_msc
    // ----------------------------------------------------------------------

    /// Single MSC/SCSI/BBB interface, bulk IN (0x81, mps 64) + bulk OUT (0x01, mps 64).
    #[rustfmt::skip]
    const CFG_SIMPLE_MSC: [u8; 32] = [
        9, 0x02, 32, 0, 1, 1, 0, 0x80, 50,
        9, 0x04, 0, 0, 2, 0x08, 0x06, 0x50, 0,
        7, 0x05, 0x81, 0x02, 0x40, 0x00, 0,
        7, 0x05, 0x01, 0x02, 0x40, 0x00, 0,
    ];

    #[test]
    fn find_msc_simple() {
        let info = find_msc(&CFG_SIMPLE_MSC).unwrap();
        assert_eq!(info.interface, 0);
        assert_eq!(info.bulk_in_ep, 0x81);
        assert_eq!(info.bulk_in_mps, 64);
        assert_eq!(info.bulk_out_ep, 0x01);
        assert_eq!(info.bulk_out_mps, 64);
    }

    #[test]
    fn find_msc_rejects_non_matching_interface() {
        // Empty / header-only.
        assert!(find_msc(&[]).is_none());

        // HID interface, no MSC anywhere.
        #[rustfmt::skip]
        let hid: [u8; 25] = [
            9, 0x02, 25, 0, 1, 1, 0, 0x80, 50,
            9, 0x04, 0, 0, 1, 0x03, 0x01, 0x01, 0,
            7, 0x05, 0x81, 0x03, 0x08, 0x00, 10,
        ];
        assert!(find_msc(&hid).is_none());

        // MSC class but wrong subclass (UFI), wrong protocol (CBI), or non-zero alt.
        for (offset, value) in [(6, 0x08), (7, 0x01), (3, 1)] {
            let mut cfg = CFG_SIMPLE_MSC;
            cfg[9 + offset] = value;
            assert!(find_msc(&cfg).is_none());
        }
    }

    #[test]
    fn find_msc_requires_both_bulk_endpoints() {
        // Only bulk OUT.
        #[rustfmt::skip]
        let out_only: [u8; 25] = [
            9, 0x02, 25, 0, 1, 1, 0, 0x80, 50,
            9, 0x04, 0, 0, 1, 0x08, 0x06, 0x50, 0,
            7, 0x05, 0x01, 0x02, 0x40, 0x00, 0,
        ];
        assert!(find_msc(&out_only).is_none());

        // Only bulk IN.
        #[rustfmt::skip]
        let in_only: [u8; 25] = [
            9, 0x02, 25, 0, 1, 1, 0, 0x80, 50,
            9, 0x04, 0, 0, 1, 0x08, 0x06, 0x50, 0,
            7, 0x05, 0x81, 0x02, 0x40, 0x00, 0,
        ];
        assert!(find_msc(&in_only).is_none());

        // Interrupt endpoints only (not bulk).
        #[rustfmt::skip]
        let intr: [u8; 32] = [
            9, 0x02, 32, 0, 1, 1, 0, 0x80, 50,
            9, 0x04, 0, 0, 2, 0x08, 0x06, 0x50, 0,
            7, 0x05, 0x81, 0x03, 0x08, 0x00, 10,
            7, 0x05, 0x01, 0x03, 0x08, 0x00, 10,
        ];
        assert!(find_msc(&intr).is_none());
    }

    #[test]
    fn find_msc_skips_preceding_interfaces() {
        // Composite: iface 0 = HID, iface 1 alt 1 = MSC (ignored because alt != 0),
        // iface 1 alt 0 = MSC (selected).
        #[rustfmt::skip]
        let cfg: [u8; 71] = [
            9, 0x02, 71, 0, 2, 1, 0, 0x80, 50,
            9, 0x04, 0, 0, 1, 0x03, 0x01, 0x01, 0,
            7, 0x05, 0x82, 0x03, 0x08, 0x00, 10,
            9, 0x04, 1, 1, 2, 0x08, 0x06, 0x50, 0,
            7, 0x05, 0x83, 0x02, 0x20, 0x00, 0,
            7, 0x05, 0x03, 0x02, 0x20, 0x00, 0,
            9, 0x04, 1, 0, 2, 0x08, 0x06, 0x50, 0,
            7, 0x05, 0x81, 0x02, 0x40, 0x00, 0,
            7, 0x05, 0x01, 0x02, 0x40, 0x00, 0,
        ];
        let info = find_msc(&cfg).unwrap();
        assert_eq!(info.interface, 1);
        assert_eq!(info.bulk_in_ep, 0x81);
        assert_eq!(info.bulk_in_mps, 64);
        assert_eq!(info.bulk_out_ep, 0x01);
    }

    // ----------------------------------------------------------------------
    // chunk_blocks
    // ----------------------------------------------------------------------

    #[test]
    fn chunk_blocks_prefers_read10_while_lba_fits_u32() {
        // Small chunk passes through verbatim.
        assert_eq!(chunk_blocks(0, 1), (1, true));
        // Clamped to u16::MAX even for huge remaining counts and high LBA.
        assert_eq!(chunk_blocks(0, u16::MAX as u64), (u16::MAX as u32, true));
        assert_eq!(chunk_blocks(0, u64::MAX), (u16::MAX as u32, true));
        assert_eq!(chunk_blocks(u32::MAX as u64, u64::MAX), (u16::MAX as u32, true));
    }

    #[test]
    fn chunk_blocks_falls_back_to_read16_above_u32_max() {
        assert_eq!(chunk_blocks(u32::MAX as u64 + 1, 100), (100, false));
        assert_eq!(chunk_blocks(u64::MAX - 10, u64::MAX), (u32::MAX, false));
    }

    // ----------------------------------------------------------------------
    // check_block_args
    // ----------------------------------------------------------------------

    const CAP_1K_512: BlockCapacity = BlockCapacity {
        block_count: 1000,
        block_size: 512,
    };

    #[test]
    fn check_block_args_accepts_aligned_in_range() {
        assert_eq!(check_block_args(0, 512, &CAP_1K_512).unwrap(), (512, 1));
        assert_eq!(check_block_args(0, 10 * 512, &CAP_1K_512).unwrap(), (512, 10));
        // Exact fit at end of device.
        assert_eq!(check_block_args(999, 512, &CAP_1K_512).unwrap(), (512, 1));
    }

    #[test]
    fn check_block_args_rejects_unaligned() {
        for bytes in [0, 511, 513] {
            assert!(matches!(
                check_block_args(0, bytes, &CAP_1K_512),
                Err(MscError::Unaligned)
            ));
        }
    }

    #[test]
    fn check_block_args_rejects_out_of_range() {
        for (lba, bytes) in [(1000, 512), (999, 1024), (u64::MAX, 512)] {
            assert!(matches!(
                check_block_args(lba, bytes, &CAP_1K_512),
                Err(MscError::OutOfRange)
            ));
        }
    }

    // ----------------------------------------------------------------------
    // CDB encoders
    // ----------------------------------------------------------------------

    #[test]
    fn read_write_10_cdb_encoding() {
        let expected = [0, 0, 0x12, 0x34, 0x56, 0x78, 0, 0x12, 0x34, 0];
        for (op, cdb) in [
            (SCSI_READ_10, read10_cdb(0x1234_5678, 0x1234)),
            (SCSI_WRITE_10, write10_cdb(0x1234_5678, 0x1234)),
        ] {
            let mut want = expected;
            want[0] = op;
            assert_eq!(cdb, want);
        }
    }

    #[test]
    fn read_write_16_cdb_encoding() {
        #[rustfmt::skip]
        let expected = [
            0, 0,
            0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF,
            0xDE, 0xAD, 0xBE, 0xEF,
            0, 0,
        ];
        for (op, cdb) in [
            (SCSI_READ_16, read16_cdb(0x0123_4567_89AB_CDEF, 0xDEAD_BEEF)),
            (SCSI_WRITE_16, write16_cdb(0x0123_4567_89AB_CDEF, 0xDEAD_BEEF)),
        ] {
            let mut want = expected;
            want[0] = op;
            assert_eq!(cdb, want);
        }
    }
}

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
//! use embassy_usb::class::msc::host::MscDevice;
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
use core::ops::Range;

use embassy_sync::blocking_mutex::raw::{NoopRawMutex, RawMutex};
use embassy_sync::mutex::Mutex;
use embassy_usb_driver::host::{PipeError, SplitInfo, UsbHostAllocator, pipe};
use embassy_usb_driver::{Direction as UsbDirection, EndpointAddress, EndpointInfo, EndpointType};

use super::bot::CBW_FLAG_IN;
use super::scsi::*;
pub use super::{SenseData, SenseKey};
use super::{USB_CLASS_MSC, USB_PROTOCOL_BULK_ONLY, USB_SUBCLASS_SCSI_TRANSPARENT};
use crate::host::descriptor::ConfigurationDescriptorChain;
use crate::host::handler::EnumerationInfo;

mod lun;
mod transport;
pub use lun::*;
use transport::{Transport, get_max_lun};

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
    fn len(&self) -> usize {
        match self {
            Self::None => 0,
            Self::In(b) => b.len(),
            Self::Out(b) => b.len(),
        }
    }

    /// The `range` part of the buffer, in the same direction.
    fn slice(&mut self, range: Range<usize>) -> DataDir<'_> {
        match self {
            Self::None => DataDir::None,
            Self::In(b) => DataDir::In(&mut b[range]),
            Self::Out(b) => DataDir::Out(&b[range]),
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
        match b & PERIPHERAL_DEVICE_TYPE_MASK {
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
        if iface.interface_class != USB_CLASS_MSC
            || iface.interface_subclass != USB_SUBCLASS_SCSI_TRANSPARENT
            || iface.interface_protocol != USB_PROTOCOL_BULK_ONLY
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

        self.transport.lock().await.command(lun, cdb, data).await
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
}

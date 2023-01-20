pub mod subclass;
pub mod transport;

use core::marker::PhantomData;

use crate::driver::Driver;
use crate::types::InterfaceNumber;
use crate::Builder;

/// USB Mass Storage Class ID
///
/// Section 4.3 [USB Bulk Only Transport Spec](https://www.usb.org/document-library/mass-storage-bulk-only-10)
pub const USB_CLASS_MSC: u8 = 0x08;

/// Command set used by the MSC interface.
///
/// Reported in `bInterfaceSubclass` field.
///
/// Section 2 [USB Mass Storage Class Overview](https://www.usb.org/document-library/mass-storage-class-specification-overview-14)
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum MscSubclass {
    /// SCSI command set not reported. De facto use
    ScsiCommandSetNotReported = 0x00,
    /// Allocated by USB-IF for RBC. RBC is defined outside of USB
    Rbc = 0x01,
    /// Allocated by USB-IF for MMC-5. MMC-5 is defined outside of USB
    Mmc5Atapi = 0x02,
    /// Specifies how to interface Floppy Disk Drives to USB
    Ufi = 0x04,
    /// Allocated by USB-IF for SCSI. SCSI standards are defined outside of USB
    ScsiTransparentCommandSet = 0x06,
    /// LSDFS specifies how host has to negotiate access before trying SCSI
    LsdFs = 0x07,
    /// Allocated by USB-IF for IEEE 1667. IEEE 1667 is defined outside of USB
    Ieee1667 = 0x08,
    /// Specific to device vendor. De facto use
    VendorSpecific = 0xFF,
}

/// Transport protocol of the MSC interface.
///
/// Reported in `bInterfaceProtocol` field.
///
/// Section 3 [USB Mass Storage Class Overview](https://www.usb.org/document-library/mass-storage-class-specification-overview-14)
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum MscProtocol {
    /// USB Mass Storage Class Control/Bulk/Interrupt (CBI) Transport (with command completion interrupt)
    CbiWithCCInterrupt = 0x00,
    /// USB Mass Storage Class Control/Bulk/Interrupt (CBI) Transport (with no command completion interrupt)
    CbiNoCCInterrupt = 0x01,
    /// USB Mass Storage Class Bulk-Only (BBB) Transport
    BulkOnlyTransport = 0x50,
    /// Allocated by USB-IF for UAS. UAS is defined outside of USB
    Uas = 0x62,
    /// Specific to device vendor. De facto use
    VendorSpecific = 0xFF,
}

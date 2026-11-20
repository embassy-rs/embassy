//! Media Transfer Protocol (MTP / PTP) constants, containers, and datasets.

use crate::driver::{EndpointDisabledError, EndpointError};

/// USB interface class code for Still Image Capture Devices (used by PTP/MTP).
pub const USB_CLASS_STILL_IMAGE: u8 = 0x06;
/// USB interface subclass code for Still Image Capture Devices.
pub const USB_SUBCLASS_STILL_IMAGE: u8 = 0x01;
/// USB interface protocol code for PTP/MTP Bulk-Only transport.
pub const USB_PROTOCOL_MTP: u8 = 0x01;

/// Class-specific control request: Cancel the current transaction (OUT, 6 bytes).
pub const REQ_CANCEL: u8 = 0x64;
/// Class-specific control request: Get extended event data (IN).
pub const REQ_GET_EXT_EVENT_DATA: u8 = 0x65;
/// Class-specific control request: Reset the MTP device transport state (OUT, 0 bytes).
pub const REQ_RESET: u8 = 0x66;
/// Class-specific control request: Get device status (IN, 4+ bytes).
pub const REQ_GET_DEVICE_STATUS: u8 = 0x67;

/// Cancellation code carried in the first 2 bytes of [`REQ_CANCEL`].
pub const CANCEL_CODE_CANCELLATION: u16 = 0x4001;

/// Length of an MTP/PTP container header in bytes.
pub const CONTAINER_HEADER_LEN: usize = 12;
/// Maximum length of an MTP Command container (12-byte header + up to five `u32` parameters).
pub const MAX_COMMAND_LEN: usize = CONTAINER_HEADER_LEN + 5 * 4;
/// Maximum length of an MTP Response container (12-byte header + up to five `u32` parameters).
pub const MAX_RESPONSE_LEN: usize = CONTAINER_HEADER_LEN + 5 * 4;
/// Maximum length of an MTP Event container (12-byte header + up to three `u32` parameters).
pub const MAX_EVENT_LEN: usize = CONTAINER_HEADER_LEN + 3 * 4;

/// Sentinel `ContainerLength` used when a Data container payload exceeds `u32::MAX - 12`
/// or its length is unknown upfront (streaming until short packet / ZLP).
pub const STREAMING_CONTAINER_LEN: u32 = 0xFFFF_FFFF;

/// Standard PTP/PIMA 15740 version reported in `DeviceInfo` (`100` = 1.00).
pub const STANDARD_VERSION_1_0: u16 = 0x0064;
/// Microsoft MTP vendor extension ID reported in `DeviceInfo`.
pub const VENDOR_EXT_ID_MICROSOFT_MTP: u32 = 0x0000_0006;
/// Standard Microsoft MTP vendor extension version (`100` = 1.00).
pub const VENDOR_EXT_VERSION_MTP_1_0: u16 = 0x0064;
/// Standard Microsoft MTP vendor extension description string.
pub const VENDOR_EXT_DESC_MICROSOFT_MTP: &str = "microsoft.com: 1.0;";

/// Special `ParentObject` handle indicating the root folder of a storage.
pub const PARENT_ROOT: u32 = 0x0000_0000;
/// Special `ParentObject` parameter indicating all objects regardless of hierarchy.
pub const PARENT_ALL: u32 = 0x0000_0000;
/// Special `ParentObject` parameter requesting only root-level objects in `GetObjectHandles`.
pub const ASSOC_ROOT_ONLY: u32 = 0xFFFF_FFFF;
/// Special `StorageID` matching all storages in `GetNumObjects` / `GetObjectHandles`.
pub const STORAGE_ALL: u32 = 0xFFFF_FFFF;
/// Special `ObjectFormat` matching all formats in `GetNumObjects` / `GetObjectHandles`.
pub const FORMAT_ALL: u16 = 0x0000;

/// MTP / PTP operation codes (`OperationCode`).
pub mod op {
    /// Query device capabilities, supported operations, and identification strings.
    pub const GET_DEVICE_INFO: u16 = 0x1001;
    /// Open a new MTP session.
    pub const OPEN_SESSION: u16 = 0x1002;
    /// Close the active MTP session.
    pub const CLOSE_SESSION: u16 = 0x1003;
    /// Retrieve the list of active storage IDs.
    pub const GET_STORAGE_IDS: u16 = 0x1004;
    /// Retrieve metadata about a storage volume.
    pub const GET_STORAGE_INFO: u16 = 0x1005;
    /// Count objects matching a storage/format/parent filter.
    pub const GET_NUM_OBJECTS: u16 = 0x1006;
    /// Retrieve handles of objects matching a storage/format/parent filter.
    pub const GET_OBJECT_HANDLES: u16 = 0x1007;
    /// Retrieve metadata (`ObjectInfo`) for an object handle.
    pub const GET_OBJECT_INFO: u16 = 0x1008;
    /// Read the full binary contents of an object.
    pub const GET_OBJECT: u16 = 0x1009;
    /// Read the thumbnail binary data of an object.
    pub const GET_THUMB: u16 = 0x100A;
    /// Delete an object (or association tree).
    pub const DELETE_OBJECT: u16 = 0x100B;
    /// Send metadata (`ObjectInfo`) before creating a new object.
    pub const SEND_OBJECT_INFO: u16 = 0x100C;
    /// Send binary data for the object previously declared by `SEND_OBJECT_INFO`.
    pub const SEND_OBJECT: u16 = 0x100D;
    /// Trigger image capture on imaging devices.
    pub const INITIATE_CAPTURE: u16 = 0x100E;
    /// Format a storage volume.
    pub const FORMAT_STORE: u16 = 0x100F;
    /// Reset the device.
    pub const RESET_DEVICE: u16 = 0x1010;
    /// Run a device self-test.
    pub const SELF_TEST: u16 = 0x1011;
    /// Set write-protection on an object.
    pub const SET_OBJECT_PROTECTION: u16 = 0x1012;
    /// Power down the device.
    pub const POWER_DOWN: u16 = 0x1013;
    /// Query a device property descriptor.
    pub const GET_DEVICE_PROP_DESC: u16 = 0x1014;
    /// Query a device property value.
    pub const GET_DEVICE_PROP_VALUE: u16 = 0x1015;
    /// Update a device property value.
    pub const SET_DEVICE_PROP_VALUE: u16 = 0x1016;
    /// Reset a device property to its default value.
    pub const RESET_DEVICE_PROP_VALUE: u16 = 0x1017;
    /// Terminate an open capture session.
    pub const TERMINATE_OPEN_CAPTURE: u16 = 0x1018;
    /// Move an object to another parent/storage.
    pub const MOVE_OBJECT: u16 = 0x1019;
    /// Copy an object to another parent/storage.
    pub const COPY_OBJECT: u16 = 0x101A;
    /// Read a byte range from an object (`offset`, `max_bytes`).
    pub const GET_PARTIAL_OBJECT: u16 = 0x101B;
    /// Start an open capture operation.
    pub const INITIATE_OPEN_CAPTURE: u16 = 0x101C;
    /// Read a 64-bit byte range from a large object (MTP extension).
    pub const GET_PARTIAL_OBJECT_64: u16 = 0x95C1;
    /// Write a 64-bit byte range into an object (MTP extension).
    pub const SEND_PARTIAL_OBJECT: u16 = 0x95C2;
    /// Truncate an object to a 64-bit length (MTP extension).
    pub const TRUNCATE_OBJECT: u16 = 0x95C3;
    /// Begin editing an existing object in-place (MTP extension).
    pub const BEGIN_EDIT_OBJECT: u16 = 0x95C4;
    /// Commit in-place edits to an object (MTP extension).
    pub const END_EDIT_OBJECT: u16 = 0x95C5;
    /// Query object property codes supported for a given object format.
    pub const GET_OBJECT_PROPS_SUPPORTED: u16 = 0x9801;
    /// Query descriptor for an object property.
    pub const GET_OBJECT_PROP_DESC: u16 = 0x9802;
    /// Read an object property value.
    pub const GET_OBJECT_PROP_VALUE: u16 = 0x9803;
    /// Write an object property value.
    pub const SET_OBJECT_PROP_VALUE: u16 = 0x9804;
    /// Query a list of object property values.
    pub const GET_OBJECT_PROP_LIST: u16 = 0x9805;
    /// Update a list of object property values.
    pub const SET_OBJECT_PROP_LIST: u16 = 0x9806;
    /// Query object references.
    pub const GET_OBJECT_REFERENCES: u16 = 0x9810;
    /// Update object references.
    pub const SET_OBJECT_REFERENCES: u16 = 0x9811;
}

/// MTP / PTP response codes (`ResponseCode`).
pub mod resp {
    /// Undefined response.
    pub const UNDEFINED: u16 = 0x2000;
    /// Operation completed successfully.
    pub const OK: u16 = 0x2001;
    /// Unspecified general error.
    pub const GENERAL_ERROR: u16 = 0x2002;
    /// No MTP session is currently open.
    pub const SESSION_NOT_OPEN: u16 = 0x2003;
    /// Transaction ID is invalid or out of sequence.
    pub const INVALID_TRANSACTION_ID: u16 = 0x2004;
    /// Operation is not supported by this device.
    pub const OPERATION_NOT_SUPPORTED: u16 = 0x2005;
    /// One or more parameters are not supported.
    pub const PARAMETER_NOT_SUPPORTED: u16 = 0x2006;
    /// Transfer ended before completion.
    pub const INCOMPLETE_TRANSFER: u16 = 0x2007;
    /// Specified `StorageID` does not exist.
    pub const INVALID_STORAGE_ID: u16 = 0x2008;
    /// Specified `ObjectHandle` does not exist.
    pub const INVALID_OBJECT_HANDLE: u16 = 0x2009;
    /// Device property code is not supported.
    pub const DEVICE_PROP_NOT_SUPPORTED: u16 = 0x200A;
    /// Object format code is invalid or unsupported.
    pub const INVALID_OBJECT_FORMAT_CODE: u16 = 0x200B;
    /// Storage volume does not have enough free space.
    pub const STORE_FULL: u16 = 0x200C;
    /// Object is write-protected.
    pub const OBJECT_WRITE_PROTECTED: u16 = 0x200D;
    /// Storage volume is read-only.
    pub const STORE_READ_ONLY: u16 = 0x200E;
    /// Access to the object or storage was denied.
    pub const ACCESS_DENIED: u16 = 0x200F;
    /// Object has no thumbnail.
    pub const NO_THUMBNAIL_PRESENT: u16 = 0x2010;
    /// Device self-test failed.
    pub const SELF_TEST_FAILED: u16 = 0x2011;
    /// Only part of an association/tree could be deleted.
    pub const PARTIAL_DELETION: u16 = 0x2012;
    /// Storage volume is currently unavailable.
    pub const STORE_NOT_AVAILABLE: u16 = 0x2013;
    /// Filtering by format is unsupported.
    pub const SPECIFICATION_BY_FORMAT_UNSUPPORTED: u16 = 0x2014;
    /// `SendObject` was issued without a preceding `SendObjectInfo`.
    pub const NO_VALID_OBJECT_INFO: u16 = 0x2015;
    /// Code format is invalid.
    pub const INVALID_CODE_FORMAT: u16 = 0x2016;
    /// Vendor code is unknown.
    pub const UNKNOWN_VENDOR_CODE: u16 = 0x2017;
    /// Capture operation was already terminated.
    pub const CAPTURE_ALREADY_TERMINATED: u16 = 0x2018;
    /// Device is busy processing another operation or cancellation.
    pub const DEVICE_BUSY: u16 = 0x2019;
    /// Specified `ParentObject` handle is invalid or not an association.
    pub const INVALID_PARENT_OBJECT: u16 = 0x201A;
    /// Device property format is invalid.
    pub const INVALID_DEVICE_PROP_FORMAT: u16 = 0x201B;
    /// Device property value is invalid.
    pub const INVALID_DEVICE_PROP_VALUE: u16 = 0x201C;
    /// Invalid parameter value.
    pub const INVALID_PARAMETER: u16 = 0x201D;
    /// A session is already open.
    pub const SESSION_ALREADY_OPEN: u16 = 0x201E;
    /// Transaction was cancelled by the host.
    pub const TRANSACTION_CANCELLED: u16 = 0x201F;
    /// Destination specification is unsupported.
    pub const SPECIFICATION_OF_DESTINATION_UNSUPPORTED: u16 = 0x2020;
    /// Object property code is invalid.
    pub const INVALID_OBJECT_PROP_CODE: u16 = 0xA801;
    /// Object property format is invalid.
    pub const INVALID_OBJECT_PROP_FORMAT: u16 = 0xA802;
    /// Object property value is invalid.
    pub const INVALID_OBJECT_PROP_VALUE: u16 = 0xA803;
    /// Object reference is invalid.
    pub const INVALID_OBJECT_REFERENCE: u16 = 0xA804;
    /// Object property is not supported for this format.
    pub const OBJECT_PROP_NOT_SUPPORTED: u16 = 0xA80A;
}

/// MTP / PTP asynchronous event codes (`EventCode`).
pub mod event {
    /// Undefined event.
    pub const UNDEFINED: u16 = 0x4000;
    /// Transaction cancelled.
    pub const CANCEL_TRANSACTION: u16 = 0x4001;
    /// A new object was added (`param1 = ObjectHandle`).
    pub const OBJECT_ADDED: u16 = 0x4002;
    /// An object was removed (`param1 = ObjectHandle`).
    pub const OBJECT_REMOVED: u16 = 0x4003;
    /// A storage volume was added (`param1 = StorageID`).
    pub const STORE_ADDED: u16 = 0x4004;
    /// A storage volume was removed (`param1 = StorageID`).
    pub const STORE_REMOVED: u16 = 0x4005;
    /// A device property changed (`param1 = DevicePropCode`).
    pub const DEVICE_PROP_CHANGED: u16 = 0x4006;
    /// An object's metadata changed (`param1 = ObjectHandle`).
    pub const OBJECT_INFO_CHANGED: u16 = 0x4007;
    /// Device capabilities changed.
    pub const DEVICE_INFO_CHANGED: u16 = 0x4008;
    /// Device requests host to transfer an object (`param1 = ObjectHandle`).
    pub const REQUEST_OBJECT_TRANSFER: u16 = 0x4009;
    /// Storage became full (`param1 = StorageID`).
    pub const STORE_FULL: u16 = 0x400A;
    /// Device was reset.
    pub const DEVICE_RESET: u16 = 0x400B;
    /// Storage metadata changed (`param1 = StorageID`).
    pub const STORAGE_INFO_CHANGED: u16 = 0x400C;
    /// Capture operation completed.
    pub const CAPTURE_COMPLETE: u16 = 0x400D;
    /// Unreported status event available.
    pub const UNREPORTED_STATUS: u16 = 0x400E;
    /// Object property value changed (`param1 = ObjectHandle`, `param2 = ObjectPropCode`).
    pub const OBJECT_PROP_CHANGED: u16 = 0xC801;
    /// Object property descriptor changed.
    pub const OBJECT_PROP_DESC_CHANGED: u16 = 0xC802;
    /// Object references changed.
    pub const OBJECT_REFERENCES_CHANGED: u16 = 0xC803;
}

/// MTP / PTP object format codes (`ObjectFormatCode`).
pub mod format {
    /// Undefined / generic binary object.
    pub const UNDEFINED: u16 = 0x3000;
    /// Folder / directory association.
    pub const ASSOCIATION: u16 = 0x3001;
    /// Executable / script file.
    pub const SCRIPT: u16 = 0x3002;
    /// Executable binary.
    pub const EXECUTABLE: u16 = 0x3003;
    /// Plain text file.
    pub const TEXT: u16 = 0x3004;
    /// HTML document.
    pub const HTML: u16 = 0x3005;
    /// Digital Print Order Format (DPOF).
    pub const DPOF: u16 = 0x3006;
    /// AIFF audio.
    pub const AIFF: u16 = 0x3007;
    /// WAV audio.
    pub const WAV: u16 = 0x3008;
    /// MP3 audio.
    pub const MP3: u16 = 0x3009;
    /// AVI video.
    pub const AVI: u16 = 0x300A;
    /// MPEG video.
    pub const MPEG: u16 = 0x300B;
    /// ASF media.
    pub const ASF: u16 = 0x300C;
    /// EXIF / JPEG image.
    pub const EXIF_JPEG: u16 = 0x3801;
    /// TIFF/EP image.
    pub const TIFF_EP: u16 = 0x3802;
    /// BMP image.
    pub const BMP: u16 = 0x3804;
    /// GIF image.
    pub const GIF: u16 = 0x3807;
    /// JFIF image.
    pub const JFIF: u16 = 0x3808;
    /// PNG image.
    pub const PNG: u16 = 0x380B;
    /// TIFF image.
    pub const TIFF: u16 = 0x380D;
    /// Firmware binary (MTP extension).
    pub const UNDEFINED_FIRMWARE: u16 = 0xB802;
    /// WMA audio.
    pub const WMA: u16 = 0xB901;
    /// OGG audio.
    pub const OGG: u16 = 0xB902;
    /// AAC audio.
    pub const AAC: u16 = 0xB903;
    /// FLAC audio.
    pub const FLAC: u16 = 0xB906;
    /// MP4 video/audio.
    pub const MP4: u16 = 0xB982;
    /// XML document.
    pub const XML: u16 = 0xBA82;
}

/// Storage media type codes for `StorageInfo`.
pub mod storage_type {
    /// Undefined storage type.
    pub const UNDEFINED: u16 = 0x0000;
    /// Fixed ROM.
    pub const FIXED_ROM: u16 = 0x0001;
    /// Removable ROM.
    pub const REMOVABLE_ROM: u16 = 0x0002;
    /// Fixed RAM / internal flash.
    pub const FIXED_RAM: u16 = 0x0003;
    /// Removable RAM / SD card.
    pub const REMOVABLE_RAM: u16 = 0x0004;
}

/// Filesystem hierarchy type codes for `StorageInfo`.
pub mod fs_type {
    /// Undefined filesystem.
    pub const UNDEFINED: u16 = 0x0000;
    /// Flat filesystem (no folders).
    pub const GENERIC_FLAT: u16 = 0x0001;
    /// Hierarchical filesystem (folders supported).
    pub const GENERIC_HIERARCHICAL: u16 = 0x0002;
    /// Design rule for Camera File system (DCF).
    pub const DCF: u16 = 0x0003;
}

/// Storage access capability codes for `StorageInfo`.
pub mod access {
    /// Read and write access.
    pub const READ_WRITE: u16 = 0x0000;
    /// Read-only without object deletion.
    pub const READ_ONLY_NO_DELETE: u16 = 0x0001;
    /// Read-only with object deletion allowed.
    pub const READ_ONLY_WITH_DELETE: u16 = 0x0002;
}

/// Association type codes for `ObjectInfo` when `object_format == format::ASSOCIATION`.
pub mod assoc {
    /// Undefined association.
    pub const UNDEFINED: u16 = 0x0000;
    /// Generic folder / directory.
    pub const GENERIC_FOLDER: u16 = 0x0001;
    /// Album association.
    pub const ALBUM: u16 = 0x0002;
    /// Time sequence association.
    pub const TIME_SEQUENCE: u16 = 0x0003;
}

/// Standard MTP object property codes.
pub mod obj_prop {
    /// Storage ID (`UINT32`).
    pub const STORAGE_ID: u16 = 0xDC01;
    /// Object format (`UINT16`).
    pub const OBJECT_FORMAT: u16 = 0xDC02;
    /// Protection status (`UINT16`).
    pub const PROTECTION_STATUS: u16 = 0xDC03;
    /// Object size in bytes (`UINT64`).
    pub const OBJECT_SIZE: u16 = 0xDC04;
    /// Association type (`UINT16`).
    pub const ASSOCIATION_TYPE: u16 = 0xDC05;
    /// Association description (`UINT32`).
    pub const ASSOCIATION_DESC: u16 = 0xDC06;
    /// Object filename (`STR`).
    pub const OBJECT_FILE_NAME: u16 = 0xDC07;
    /// Creation timestamp (`STR`).
    pub const DATE_CREATED: u16 = 0xDC08;
    /// Modification timestamp (`STR`).
    pub const DATE_MODIFIED: u16 = 0xDC09;
    /// Parent object handle (`UINT32`).
    pub const PARENT_OBJECT: u16 = 0xDC0B;
    /// Persistent unique object identifier (`UINT128`).
    pub const PERSISTENT_UID: u16 = 0xDC41;
    /// Display name (`STR`).
    pub const NAME: u16 = 0xDC44;
}

/// Standard MTP device property codes.
pub mod dev_prop {
    /// Undefined property.
    pub const UNDEFINED: u16 = 0x5000;
    /// Battery level (`UINT8`).
    pub const BATTERY_LEVEL: u16 = 0x5001;
    /// Date/time string (`STR`).
    pub const DATE_TIME: u16 = 0x5011;
    /// Friendly device name (`STR`).
    pub const DEVICE_FRIENDLY_NAME: u16 = 0xD402;
    /// Session initiator info (`STR`).
    pub const SESSION_INITIATOR_VERSION_INFO: u16 = 0xD406;
    /// Perceived device type (`UINT32`).
    pub const PERCEIVED_DEVICE_TYPE: u16 = 0xD407;
}

/// MTP / PTP container type.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u16)]
pub enum ContainerType {
    /// Undefined container type.
    Undefined = 0,
    /// Command block (Bulk OUT).
    Command = 1,
    /// Data block (Bulk IN or Bulk OUT).
    Data = 2,
    /// Response block (Bulk IN).
    Response = 3,
    /// Event block (Interrupt IN).
    Event = 4,
}

impl TryFrom<u16> for ContainerType {
    type Error = MtpError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Undefined),
            1 => Ok(Self::Command),
            2 => Ok(Self::Data),
            3 => Ok(Self::Response),
            4 => Ok(Self::Event),
            other => Err(MtpError::InvalidContainerType(other)),
        }
    }
}

/// Errors produced by MTP transport, container framing, and dataset codecs.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum MtpError {
    /// Underlying USB endpoint error.
    Endpoint(EndpointError),
    /// Unknown `ContainerType` value on the wire.
    InvalidContainerType(u16),
    /// Container type did not match the expected phase.
    UnexpectedContainerType {
        /// Expected container type.
        expected: ContainerType,
        /// Actual container type received.
        actual: ContainerType,
    },
    /// Container `length` field was invalid (e.g. smaller than 12 bytes).
    InvalidLength,
    /// USB packet was shorter than the 12-byte container header.
    PacketTooShort,
    /// Caller buffer was too small for the encoded or decoded data.
    BufferTooSmall,
    /// More than the allowed number of container parameters were present.
    TooManyParameters,
    /// String payload contained invalid UTF-16 or exceeded the output buffer.
    InvalidString,
    /// String exceeded the 254 UTF-16 code unit limit of PTP/MTP strings.
    StringTooLong,
    /// Payload length did not match the declared `ContainerLength`.
    LengthMismatch,
    /// Data-OUT transfer ended early with a short packet before `ContainerLength` bytes arrived.
    Truncated,
    /// Transaction was cancelled by the host via `REQ_CANCEL`.
    Cancelled(u32),
    /// Transport was reset by the host via `REQ_RESET`.
    Reset,
}

impl From<EndpointError> for MtpError {
    fn from(e: EndpointError) -> Self {
        Self::Endpoint(e)
    }
}

impl From<EndpointDisabledError> for MtpError {
    fn from(e: EndpointDisabledError) -> Self {
        Self::Endpoint(e.into())
    }
}

impl core::fmt::Display for MtpError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Endpoint(e) => write!(f, "Endpoint error: {}", e),
            Self::InvalidContainerType(t) => write!(f, "Invalid MTP container type: {:#06x}", t),
            Self::UnexpectedContainerType { expected, actual } => {
                write!(
                    f,
                    "Unexpected MTP container type: expected {:?}, got {:?}",
                    expected, actual
                )
            }
            Self::InvalidLength => write!(f, "Invalid MTP container length"),
            Self::PacketTooShort => write!(f, "MTP packet shorter than 12-byte header"),
            Self::BufferTooSmall => write!(f, "Buffer too small"),
            Self::TooManyParameters => write!(f, "Too many MTP container parameters"),
            Self::InvalidString => write!(f, "Invalid MTP UTF-16 string"),
            Self::StringTooLong => write!(f, "MTP string exceeds 254 UTF-16 code units"),
            Self::LengthMismatch => write!(f, "MTP payload length mismatch"),
            Self::Truncated => write!(f, "MTP transfer truncated"),
            Self::Cancelled(tx) => write!(f, "MTP transaction {:#010x} cancelled", tx),
            Self::Reset => write!(f, "MTP device reset"),
        }
    }
}

impl core::error::Error for MtpError {}

/// 12-byte MTP / PTP generic container header.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ContainerHeader {
    /// Total container length in bytes (including this 12-byte header), or [`STREAMING_CONTAINER_LEN`].
    pub length: u32,
    /// Container phase type.
    pub container_type: ContainerType,
    /// Operation, response, or event code.
    pub code: u16,
    /// Host-assigned transaction ID (or session ID for certain events).
    pub transaction_id: u32,
}

impl ContainerHeader {
    /// Encodes the 12-byte container header into `buf`.
    pub fn encode(&self, buf: &mut [u8]) -> Result<usize, MtpError> {
        if buf.len() < CONTAINER_HEADER_LEN {
            return Err(MtpError::BufferTooSmall);
        }
        buf[0..4].copy_from_slice(&self.length.to_le_bytes());
        buf[4..6].copy_from_slice(&(self.container_type as u16).to_le_bytes());
        buf[6..8].copy_from_slice(&self.code.to_le_bytes());
        buf[8..12].copy_from_slice(&self.transaction_id.to_le_bytes());
        Ok(CONTAINER_HEADER_LEN)
    }

    /// Decodes a 12-byte container header from `buf`.
    pub fn decode(buf: &[u8]) -> Result<Self, MtpError> {
        if buf.len() < CONTAINER_HEADER_LEN {
            return Err(MtpError::PacketTooShort);
        }
        let length = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
        let container_type = ContainerType::try_from(u16::from_le_bytes([buf[4], buf[5]]))?;
        let code = u16::from_le_bytes([buf[6], buf[7]]);
        let transaction_id = u32::from_le_bytes([buf[8], buf[9], buf[10], buf[11]]);
        Ok(Self {
            length,
            container_type,
            code,
            transaction_id,
        })
    }
}

/// Decoded MTP Command container (Bulk OUT).
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CommandBlock {
    /// Operation code (see [`op`]).
    pub operation: u16,
    /// Transaction ID assigned by the host.
    pub transaction_id: u32,
    /// Up to 5 operation parameters.
    pub params: [u32; 5],
    /// Number of valid entries in `params` (`0..=5`).
    pub param_count: u8,
}

impl CommandBlock {
    /// Creates a new `CommandBlock`.
    pub fn new(operation: u16, transaction_id: u32, params: &[u32]) -> Result<Self, MtpError> {
        if params.len() > 5 {
            return Err(MtpError::TooManyParameters);
        }
        let mut p = [0u32; 5];
        p[..params.len()].copy_from_slice(params);
        Ok(Self {
            operation,
            transaction_id,
            params: p,
            param_count: params.len() as u8,
        })
    }

    /// Returns the active parameters slice.
    pub fn params(&self) -> &[u32] {
        &self.params[..self.param_count as usize]
    }

    /// Returns parameter `idx` if present.
    pub fn param(&self, idx: usize) -> Option<u32> {
        if idx < self.param_count as usize {
            Some(self.params[idx])
        } else {
            None
        }
    }

    /// Returns parameter `idx`, or `0` if omitted by the host.
    pub fn param_or_zero(&self, idx: usize) -> u32 {
        self.param(idx).unwrap_or(0)
    }

    /// Decodes a Command container from `buf`.
    pub fn decode(buf: &[u8]) -> Result<Self, MtpError> {
        let header = ContainerHeader::decode(buf)?;
        if header.container_type != ContainerType::Command {
            return Err(MtpError::UnexpectedContainerType {
                expected: ContainerType::Command,
                actual: header.container_type,
            });
        }
        let total_len = header.length as usize;
        if total_len < CONTAINER_HEADER_LEN
            || total_len > MAX_COMMAND_LEN
            || (total_len - CONTAINER_HEADER_LEN) % 4 != 0
        {
            return Err(MtpError::InvalidLength);
        }
        if buf.len() < total_len {
            return Err(MtpError::PacketTooShort);
        }
        let param_count = (total_len - CONTAINER_HEADER_LEN) / 4;
        let mut params = [0u32; 5];
        for (i, slot) in params[..param_count].iter_mut().enumerate() {
            let off = CONTAINER_HEADER_LEN + i * 4;
            *slot = u32::from_le_bytes([buf[off], buf[off + 1], buf[off + 2], buf[off + 3]]);
        }
        Ok(Self {
            operation: header.code,
            transaction_id: header.transaction_id,
            params,
            param_count: param_count as u8,
        })
    }

    /// Encodes the Command container into `buf`, returning the number of bytes written.
    pub fn encode(&self, buf: &mut [u8]) -> Result<usize, MtpError> {
        let count = usize::from(self.param_count);
        if count > 5 {
            return Err(MtpError::TooManyParameters);
        }
        let total_len = CONTAINER_HEADER_LEN + count * 4;
        if buf.len() < total_len {
            return Err(MtpError::BufferTooSmall);
        }
        ContainerHeader {
            length: total_len as u32,
            container_type: ContainerType::Command,
            code: self.operation,
            transaction_id: self.transaction_id,
        }
        .encode(buf)?;
        for (i, &val) in self.params[..count].iter().enumerate() {
            let off = CONTAINER_HEADER_LEN + i * 4;
            buf[off..off + 4].copy_from_slice(&val.to_le_bytes());
        }
        Ok(total_len)
    }
}

/// MTP Response container (Bulk IN).
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ResponseBlock {
    /// Response code (see [`resp`]).
    pub code: u16,
    /// Transaction ID matching the corresponding Command container.
    pub transaction_id: u32,
    /// Up to 5 response parameters.
    pub params: [u32; 5],
    /// Number of valid entries in `params` (`0..=5`).
    pub param_count: u8,
}

impl ResponseBlock {
    /// Creates a `ResponseBlock` with no parameters.
    pub const fn new(code: u16, transaction_id: u32) -> Self {
        Self {
            code,
            transaction_id,
            params: [0; 5],
            param_count: 0,
        }
    }

    /// Creates an [`resp::OK`] response with no parameters.
    pub const fn ok(transaction_id: u32) -> Self {
        Self::new(resp::OK, transaction_id)
    }

    /// Creates a `ResponseBlock` with up to 5 parameters.
    pub fn with_params(code: u16, transaction_id: u32, params: &[u32]) -> Result<Self, MtpError> {
        if params.len() > 5 {
            return Err(MtpError::TooManyParameters);
        }
        let mut p = [0u32; 5];
        p[..params.len()].copy_from_slice(params);
        Ok(Self {
            code,
            transaction_id,
            params: p,
            param_count: params.len() as u8,
        })
    }

    /// Encodes the Response container into `buf`, returning the number of bytes written.
    pub fn encode(&self, buf: &mut [u8]) -> Result<usize, MtpError> {
        let count = usize::from(self.param_count);
        if count > 5 {
            return Err(MtpError::TooManyParameters);
        }
        let total_len = CONTAINER_HEADER_LEN + count * 4;
        if buf.len() < total_len {
            return Err(MtpError::BufferTooSmall);
        }
        ContainerHeader {
            length: total_len as u32,
            container_type: ContainerType::Response,
            code: self.code,
            transaction_id: self.transaction_id,
        }
        .encode(buf)?;
        for (i, &val) in self.params[..count].iter().enumerate() {
            let off = CONTAINER_HEADER_LEN + i * 4;
            buf[off..off + 4].copy_from_slice(&val.to_le_bytes());
        }
        Ok(total_len)
    }
}

/// MTP asynchronous Event container (Interrupt IN).
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct EventBlock {
    /// Event code (see [`event`]).
    pub code: u16,
    /// Transaction ID or session ID (`0xFFFF_FFFF` for session-wide events).
    pub transaction_id: u32,
    /// Up to 3 event parameters.
    pub params: [u32; 3],
    /// Number of valid entries in `params` (`0..=3`).
    pub param_count: u8,
}

impl EventBlock {
    /// Creates an `EventBlock` with no parameters.
    pub const fn new(code: u16, transaction_id: u32) -> Self {
        Self {
            code,
            transaction_id,
            params: [0; 3],
            param_count: 0,
        }
    }

    /// Creates an `EventBlock` with up to 3 parameters.
    pub fn with_params(code: u16, transaction_id: u32, params: &[u32]) -> Result<Self, MtpError> {
        if params.len() > 3 {
            return Err(MtpError::TooManyParameters);
        }
        let mut p = [0u32; 3];
        p[..params.len()].copy_from_slice(params);
        Ok(Self {
            code,
            transaction_id,
            params: p,
            param_count: params.len() as u8,
        })
    }

    /// Encodes the Event container into `buf`, returning the number of bytes written.
    pub fn encode(&self, buf: &mut [u8]) -> Result<usize, MtpError> {
        let count = usize::from(self.param_count);
        if count > 3 {
            return Err(MtpError::TooManyParameters);
        }
        let total_len = CONTAINER_HEADER_LEN + count * 4;
        if buf.len() < total_len {
            return Err(MtpError::BufferTooSmall);
        }
        ContainerHeader {
            length: total_len as u32,
            container_type: ContainerType::Event,
            code: self.code,
            transaction_id: self.transaction_id,
        }
        .encode(buf)?;
        for (i, &val) in self.params[..count].iter().enumerate() {
            let off = CONTAINER_HEADER_LEN + i * 4;
            buf[off..off + 4].copy_from_slice(&val.to_le_bytes());
        }
        Ok(total_len)
    }
}

/// Cursor-based serializer for little-endian MTP/PTP datasets and UTF-16LE strings.
pub struct MtpWriter<'a> {
    buf: &'a mut [u8],
    pos: usize,
}

impl<'a> MtpWriter<'a> {
    /// Creates a new `MtpWriter` backed by `buf`.
    pub fn new(buf: &'a mut [u8]) -> Self {
        Self { buf, pos: 0 }
    }

    /// Returns the number of bytes written so far.
    pub const fn len(&self) -> usize {
        self.pos
    }

    /// Returns `true` if no bytes have been written.
    pub const fn is_empty(&self) -> bool {
        self.pos == 0
    }

    /// Returns the remaining capacity in bytes.
    pub const fn remaining(&self) -> usize {
        self.buf.len() - self.pos
    }

    /// Returns the written bytes slice.
    pub fn as_slice(&self) -> &[u8] {
        &self.buf[..self.pos]
    }

    /// Writes raw bytes.
    pub fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), MtpError> {
        if self.remaining() < bytes.len() {
            return Err(MtpError::BufferTooSmall);
        }
        self.buf[self.pos..self.pos + bytes.len()].copy_from_slice(bytes);
        self.pos += bytes.len();
        Ok(())
    }

    /// Writes a `u8`.
    pub fn write_u8(&mut self, v: u8) -> Result<(), MtpError> {
        self.write_bytes(&[v])
    }

    /// Writes a little-endian `u16`.
    pub fn write_u16(&mut self, v: u16) -> Result<(), MtpError> {
        self.write_bytes(&v.to_le_bytes())
    }

    /// Writes a little-endian `u32`.
    pub fn write_u32(&mut self, v: u32) -> Result<(), MtpError> {
        self.write_bytes(&v.to_le_bytes())
    }

    /// Writes a little-endian `u64`.
    pub fn write_u64(&mut self, v: u64) -> Result<(), MtpError> {
        self.write_bytes(&v.to_le_bytes())
    }

    /// Writes a PTP/MTP `u16` array (`u32` element count followed by LE `u16` elements).
    pub fn write_u16_array(&mut self, items: &[u16]) -> Result<(), MtpError> {
        self.write_u32(items.len() as u32)?;
        for &item in items {
            self.write_u16(item)?;
        }
        Ok(())
    }

    /// Writes a PTP/MTP `u32` array (`u32` element count followed by LE `u32` elements).
    pub fn write_u32_array(&mut self, items: &[u32]) -> Result<(), MtpError> {
        self.write_u32(items.len() as u32)?;
        for &item in items {
            self.write_u32(item)?;
        }
        Ok(())
    }

    /// Writes a PTP/MTP UTF-16LE length-prefixed string.
    ///
    /// Empty strings are encoded as a single `0x00` byte; non-empty strings are
    /// encoded as `num_chars` (including trailing `NUL`, max 255) followed by
    /// UTF-16LE code units and `0x0000`.
    pub fn write_string(&mut self, s: &str) -> Result<(), MtpError> {
        if s.is_empty() {
            return self.write_u8(0);
        }
        let unit_count: usize = s.chars().map(char::len_utf16).sum();
        if unit_count > 254 {
            return Err(MtpError::StringTooLong);
        }
        let needed = 1 + (unit_count + 1) * 2;
        if self.remaining() < needed {
            return Err(MtpError::BufferTooSmall);
        }
        self.write_u8((unit_count + 1) as u8)?;
        let mut enc = [0u16; 2];
        for ch in s.chars() {
            for &unit in ch.encode_utf16(&mut enc).iter() {
                self.write_u16(unit)?;
            }
        }
        self.write_u16(0)
    }
}

/// Cursor-based deserializer for little-endian MTP/PTP datasets and UTF-16LE strings.
pub struct MtpReader<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> MtpReader<'a> {
    /// Creates a new `MtpReader` over `buf`.
    pub const fn new(buf: &'a [u8]) -> Self {
        Self { buf, pos: 0 }
    }

    /// Returns the number of unread bytes remaining.
    pub const fn remaining(&self) -> usize {
        self.buf.len() - self.pos
    }

    /// Reads an exact slice of `len` bytes.
    pub fn read_bytes(&mut self, len: usize) -> Result<&'a [u8], MtpError> {
        if self.remaining() < len {
            return Err(MtpError::PacketTooShort);
        }
        let slice = &self.buf[self.pos..self.pos + len];
        self.pos += len;
        Ok(slice)
    }

    /// Reads a `u8`.
    pub fn read_u8(&mut self) -> Result<u8, MtpError> {
        Ok(self.read_bytes(1)?[0])
    }

    /// Reads a little-endian `u16`.
    pub fn read_u16(&mut self) -> Result<u16, MtpError> {
        let b = self.read_bytes(2)?;
        Ok(u16::from_le_bytes([b[0], b[1]]))
    }

    /// Reads a little-endian `u32`.
    pub fn read_u32(&mut self) -> Result<u32, MtpError> {
        let b = self.read_bytes(4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    /// Reads a little-endian `u64`.
    pub fn read_u64(&mut self) -> Result<u64, MtpError> {
        let b = self.read_bytes(8)?;
        Ok(u64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]))
    }

    /// Skips a PTP/MTP string without decoding its contents.
    pub fn skip_string(&mut self) -> Result<(), MtpError> {
        let num_chars = usize::from(self.read_u8()?);
        self.read_bytes(num_chars * 2)?;
        Ok(())
    }

    /// Reads a PTP/MTP UTF-16LE string and encodes it as UTF-8 into `out`.
    pub fn read_string_utf8<'b>(&mut self, out: &'b mut [u8]) -> Result<&'b str, MtpError> {
        let num_chars = usize::from(self.read_u8()?);
        if num_chars == 0 {
            return Ok("");
        }
        let raw = self.read_bytes(num_chars * 2)?;
        let utf16_iter = raw
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .take_while(|&u| u != 0);

        let mut written = 0usize;
        for decoded in char::decode_utf16(utf16_iter) {
            let ch = decoded.map_err(|_| MtpError::InvalidString)?;
            let mut utf8 = [0u8; 4];
            let s = ch.encode_utf8(&mut utf8);
            if written + s.len() > out.len() {
                return Err(MtpError::BufferTooSmall);
            }
            out[written..written + s.len()].copy_from_slice(s.as_bytes());
            written += s.len();
        }
        let s = core::str::from_utf8(&out[..written]).map_err(|_| MtpError::InvalidString)?;
        Ok(s)
    }
}

/// MTP `DeviceInfo` dataset returned by [`op::GET_DEVICE_INFO`].
#[derive(Clone, Copy, Debug)]
pub struct DeviceInfo<'a> {
    /// PTP standard version (typically [`STANDARD_VERSION_1_0`]).
    pub standard_version: u16,
    /// Vendor extension ID (typically [`VENDOR_EXT_ID_MICROSOFT_MTP`]).
    pub vendor_extension_id: u32,
    /// Vendor extension version (typically [`VENDOR_EXT_VERSION_MTP_1_0`]).
    pub vendor_extension_version: u16,
    /// Vendor extension description (typically [`VENDOR_EXT_DESC_MICROSOFT_MTP`]).
    pub vendor_extension_desc: &'a str,
    /// Functional mode (`0x0000` for standard mode).
    pub functional_mode: u16,
    /// List of supported [`op`] operation codes.
    pub operations_supported: &'a [u16],
    /// List of supported [`event`] event codes.
    pub events_supported: &'a [u16],
    /// List of supported [`dev_prop`] device property codes.
    pub device_properties_supported: &'a [u16],
    /// List of [`format`] codes the device can capture.
    pub capture_formats: &'a [u16],
    /// List of [`format`] codes the device can store/play back.
    pub playback_formats: &'a [u16],
    /// Manufacturer string.
    pub manufacturer: &'a str,
    /// Model name string.
    pub model: &'a str,
    /// Firmware/device version string.
    pub device_version: &'a str,
    /// Serial number string.
    pub serial_number: &'a str,
}

impl<'a> Default for DeviceInfo<'a> {
    fn default() -> Self {
        Self {
            standard_version: STANDARD_VERSION_1_0,
            vendor_extension_id: VENDOR_EXT_ID_MICROSOFT_MTP,
            vendor_extension_version: VENDOR_EXT_VERSION_MTP_1_0,
            vendor_extension_desc: VENDOR_EXT_DESC_MICROSOFT_MTP,
            functional_mode: 0x0000,
            operations_supported: &[
                op::GET_DEVICE_INFO,
                op::OPEN_SESSION,
                op::CLOSE_SESSION,
                op::GET_STORAGE_IDS,
                op::GET_STORAGE_INFO,
                op::GET_NUM_OBJECTS,
                op::GET_OBJECT_HANDLES,
                op::GET_OBJECT_INFO,
                op::GET_OBJECT,
            ],
            events_supported: &[],
            device_properties_supported: &[],
            capture_formats: &[],
            playback_formats: &[format::UNDEFINED, format::ASSOCIATION],
            manufacturer: "Embassy",
            model: "MTP Device",
            device_version: "1.0",
            serial_number: "0001",
        }
    }
}

impl<'a> DeviceInfo<'a> {
    /// Serializes the `DeviceInfo` dataset using `writer`.
    pub fn write_to(&self, writer: &mut MtpWriter<'_>) -> Result<(), MtpError> {
        writer.write_u16(self.standard_version)?;
        writer.write_u32(self.vendor_extension_id)?;
        writer.write_u16(self.vendor_extension_version)?;
        writer.write_string(self.vendor_extension_desc)?;
        writer.write_u16(self.functional_mode)?;
        writer.write_u16_array(self.operations_supported)?;
        writer.write_u16_array(self.events_supported)?;
        writer.write_u16_array(self.device_properties_supported)?;
        writer.write_u16_array(self.capture_formats)?;
        writer.write_u16_array(self.playback_formats)?;
        writer.write_string(self.manufacturer)?;
        writer.write_string(self.model)?;
        writer.write_string(self.device_version)?;
        writer.write_string(self.serial_number)?;
        Ok(())
    }

    /// Serializes the `DeviceInfo` dataset into `buf`, returning the encoded byte length.
    pub fn encode(&self, buf: &mut [u8]) -> Result<usize, MtpError> {
        let mut writer = MtpWriter::new(buf);
        self.write_to(&mut writer)?;
        Ok(writer.len())
    }
}

/// MTP `StorageInfo` dataset returned by [`op::GET_STORAGE_INFO`].
#[derive(Clone, Copy, Debug)]
pub struct StorageInfo<'a> {
    /// Storage media type (see [`storage_type`]).
    pub storage_type: u16,
    /// Filesystem type (see [`fs_type`]).
    pub filesystem_type: u16,
    /// Access capability (see [`access`]).
    pub access_capability: u16,
    /// Maximum capacity in bytes.
    pub max_capacity: u64,
    /// Free space in bytes.
    pub free_space_in_bytes: u64,
    /// Approximate free space in images (`0xFFFF_FFFF` if unused).
    pub free_space_in_images: u32,
    /// Human-readable storage description.
    pub storage_description: &'a str,
    /// Volume label.
    pub volume_label: &'a str,
}

impl<'a> Default for StorageInfo<'a> {
    fn default() -> Self {
        Self {
            storage_type: storage_type::FIXED_RAM,
            filesystem_type: fs_type::GENERIC_HIERARCHICAL,
            access_capability: access::READ_WRITE,
            max_capacity: 0,
            free_space_in_bytes: 0,
            free_space_in_images: 0xFFFF_FFFF,
            storage_description: "Internal Storage",
            volume_label: "EMBASSY",
        }
    }
}

impl<'a> StorageInfo<'a> {
    /// Serializes the `StorageInfo` dataset using `writer`.
    pub fn write_to(&self, writer: &mut MtpWriter<'_>) -> Result<(), MtpError> {
        writer.write_u16(self.storage_type)?;
        writer.write_u16(self.filesystem_type)?;
        writer.write_u16(self.access_capability)?;
        writer.write_u64(self.max_capacity)?;
        writer.write_u64(self.free_space_in_bytes)?;
        writer.write_u32(self.free_space_in_images)?;
        writer.write_string(self.storage_description)?;
        writer.write_string(self.volume_label)?;
        Ok(())
    }

    /// Serializes the `StorageInfo` dataset into `buf`, returning the encoded byte length.
    pub fn encode(&self, buf: &mut [u8]) -> Result<usize, MtpError> {
        let mut writer = MtpWriter::new(buf);
        self.write_to(&mut writer)?;
        Ok(writer.len())
    }
}

/// MTP `ObjectInfo` dataset used by [`op::GET_OBJECT_INFO`] and [`op::SEND_OBJECT_INFO`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObjectInfo<'a> {
    /// Storage ID containing this object.
    pub storage_id: u32,
    /// Object format code (see [`format`]).
    pub object_format: u16,
    /// Protection status (`0` = none, `1` = read-only).
    pub protection_status: u16,
    /// Size of the object in bytes (`0xFFFF_FFFF` if >= 4 GiB).
    pub object_compressed_size: u32,
    /// Thumbnail format (`0` if no thumbnail).
    pub thumb_format: u16,
    /// Thumbnail compressed size in bytes.
    pub thumb_compressed_size: u32,
    /// Thumbnail width in pixels.
    pub thumb_pix_width: u32,
    /// Thumbnail height in pixels.
    pub thumb_pix_height: u32,
    /// Image width in pixels.
    pub image_pix_width: u32,
    /// Image height in pixels.
    pub image_pix_height: u32,
    /// Image bit depth.
    pub image_bit_depth: u32,
    /// Parent association handle (`0` = root).
    pub parent_object: u32,
    /// Association type (see [`assoc`]).
    pub association_type: u16,
    /// Association description.
    pub association_desc: u32,
    /// Sequence number.
    pub sequence_number: u32,
    /// File or folder name.
    pub filename: &'a str,
    /// Capture date string (`YYYYMMDDThhmmss`).
    pub capture_date: &'a str,
    /// Modification date string (`YYYYMMDDThhmmss`).
    pub modification_date: &'a str,
    /// Keywords string.
    pub keywords: &'a str,
}

impl<'a> Default for ObjectInfo<'a> {
    fn default() -> Self {
        Self {
            storage_id: 0,
            object_format: format::UNDEFINED,
            protection_status: 0,
            object_compressed_size: 0,
            thumb_format: 0,
            thumb_compressed_size: 0,
            thumb_pix_width: 0,
            thumb_pix_height: 0,
            image_pix_width: 0,
            image_pix_height: 0,
            image_bit_depth: 0,
            parent_object: PARENT_ROOT,
            association_type: assoc::UNDEFINED,
            association_desc: 0,
            sequence_number: 0,
            filename: "",
            capture_date: "",
            modification_date: "",
            keywords: "",
        }
    }
}

impl<'a> ObjectInfo<'a> {
    /// Creates an `ObjectInfo` for a file in `storage_id` under `parent_object`.
    pub const fn file(storage_id: u32, parent_object: u32, object_format: u16, size: u32, filename: &'a str) -> Self {
        Self {
            storage_id,
            object_format,
            protection_status: 0,
            object_compressed_size: size,
            thumb_format: 0,
            thumb_compressed_size: 0,
            thumb_pix_width: 0,
            thumb_pix_height: 0,
            image_pix_width: 0,
            image_pix_height: 0,
            image_bit_depth: 0,
            parent_object,
            association_type: assoc::UNDEFINED,
            association_desc: 0,
            sequence_number: 0,
            filename,
            capture_date: "",
            modification_date: "",
            keywords: "",
        }
    }

    /// Creates an `ObjectInfo` for a folder (`format::ASSOCIATION`, `assoc::GENERIC_FOLDER`).
    pub const fn folder(storage_id: u32, parent_object: u32, filename: &'a str) -> Self {
        Self {
            storage_id,
            object_format: format::ASSOCIATION,
            protection_status: 0,
            object_compressed_size: 0,
            thumb_format: 0,
            thumb_compressed_size: 0,
            thumb_pix_width: 0,
            thumb_pix_height: 0,
            image_pix_width: 0,
            image_pix_height: 0,
            image_bit_depth: 0,
            parent_object,
            association_type: assoc::GENERIC_FOLDER,
            association_desc: 0,
            sequence_number: 0,
            filename,
            capture_date: "",
            modification_date: "",
            keywords: "",
        }
    }

    /// Serializes the `ObjectInfo` dataset using `writer`.
    pub fn write_to(&self, writer: &mut MtpWriter<'_>) -> Result<(), MtpError> {
        writer.write_u32(self.storage_id)?;
        writer.write_u16(self.object_format)?;
        writer.write_u16(self.protection_status)?;
        writer.write_u32(self.object_compressed_size)?;
        writer.write_u16(self.thumb_format)?;
        writer.write_u32(self.thumb_compressed_size)?;
        writer.write_u32(self.thumb_pix_width)?;
        writer.write_u32(self.thumb_pix_height)?;
        writer.write_u32(self.image_pix_width)?;
        writer.write_u32(self.image_pix_height)?;
        writer.write_u32(self.image_bit_depth)?;
        writer.write_u32(self.parent_object)?;
        writer.write_u16(self.association_type)?;
        writer.write_u32(self.association_desc)?;
        writer.write_u32(self.sequence_number)?;
        writer.write_string(self.filename)?;
        writer.write_string(self.capture_date)?;
        writer.write_string(self.modification_date)?;
        writer.write_string(self.keywords)?;
        Ok(())
    }

    /// Serializes the `ObjectInfo` dataset into `buf`, returning the encoded byte length.
    pub fn encode(&self, buf: &mut [u8]) -> Result<usize, MtpError> {
        let mut writer = MtpWriter::new(buf);
        self.write_to(&mut writer)?;
        Ok(writer.len())
    }

    /// Parses an `ObjectInfo` dataset received from the host (such as `SendObjectInfo`),
    /// decoding `filename` into `filename_buf` and skipping trailing metadata strings.
    pub fn decode(data: &[u8], filename_buf: &'a mut [u8]) -> Result<Self, MtpError> {
        let mut reader = MtpReader::new(data);
        let storage_id = reader.read_u32()?;
        let object_format = reader.read_u16()?;
        let protection_status = reader.read_u16()?;
        let object_compressed_size = reader.read_u32()?;
        let thumb_format = reader.read_u16()?;
        let thumb_compressed_size = reader.read_u32()?;
        let thumb_pix_width = reader.read_u32()?;
        let thumb_pix_height = reader.read_u32()?;
        let image_pix_width = reader.read_u32()?;
        let image_pix_height = reader.read_u32()?;
        let image_bit_depth = reader.read_u32()?;
        let parent_object = reader.read_u32()?;
        let association_type = reader.read_u16()?;
        let association_desc = reader.read_u32()?;
        let sequence_number = reader.read_u32()?;
        let filename = reader.read_string_utf8(filename_buf)?;
        // Some initiators omit trailing optional strings; skip if present.
        if reader.remaining() > 0 {
            reader.skip_string()?;
        }
        if reader.remaining() > 0 {
            reader.skip_string()?;
        }
        if reader.remaining() > 0 {
            reader.skip_string()?;
        }
        Ok(Self {
            storage_id,
            object_format,
            protection_status,
            object_compressed_size,
            thumb_format,
            thumb_compressed_size,
            thumb_pix_width,
            thumb_pix_height,
            image_pix_width,
            image_pix_height,
            image_bit_depth,
            parent_object,
            association_type,
            association_desc,
            sequence_number,
            filename,
            capture_date: "",
            modification_date: "",
            keywords: "",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn container_header_round_trip() {
        let hdr = ContainerHeader {
            length: 0x1234_5678,
            container_type: ContainerType::Data,
            code: op::GET_OBJECT,
            transaction_id: 42,
        };
        let mut buf = [0u8; CONTAINER_HEADER_LEN];
        assert_eq!(hdr.encode(&mut buf), Ok(12));
        assert_eq!(ContainerHeader::decode(&buf), Ok(hdr));
    }

    #[test]
    fn command_response_event_round_trip() {
        let cmd = CommandBlock::new(op::GET_OBJECT_HANDLES, 7, &[0x0001_0001, 0, PARENT_ROOT]).unwrap();
        let mut buf = [0u8; MAX_COMMAND_LEN];
        let n = cmd.encode(&mut buf).unwrap();
        assert_eq!(n, 24);
        assert_eq!(CommandBlock::decode(&buf[..n]), Ok(cmd));

        let resp = ResponseBlock::with_params(resp::OK, 7, &[0x0001_0001, PARENT_ROOT, 99]).unwrap();
        let mut rbuf = [0u8; MAX_RESPONSE_LEN];
        let rn = resp.encode(&mut rbuf).unwrap();
        assert_eq!(rn, 24);
        let rhdr = ContainerHeader::decode(&rbuf[..rn]).unwrap();
        assert_eq!(rhdr.container_type, ContainerType::Response);
        assert_eq!(rhdr.code, resp::OK);
        assert_eq!(rhdr.transaction_id, 7);

        let evt = EventBlock::with_params(event::OBJECT_ADDED, 0xFFFF_FFFF, &[0x0000_0005]).unwrap();
        let mut ebuf = [0u8; MAX_EVENT_LEN];
        let en = evt.encode(&mut ebuf).unwrap();
        assert_eq!(en, 16);
        let ehdr = ContainerHeader::decode(&ebuf[..en]).unwrap();
        assert_eq!(ehdr.container_type, ContainerType::Event);
        assert_eq!(ehdr.code, event::OBJECT_ADDED);
    }

    #[test]
    fn ptp_strings_encode_and_decode_safely() {
        let mut buf = [0u8; 64];
        let mut w = MtpWriter::new(&mut buf);
        w.write_string("").unwrap();
        w.write_string("log.txt").unwrap();
        w.write_string("🦀").unwrap();
        let len = w.len();

        assert_eq!(buf[0], 0x00); // Empty string is 1 byte 0x00.
        assert_eq!(buf[1], 8); // "log.txt" (7 chars) + NUL = 8.

        let mut r = MtpReader::new(&buf[..len]);
        let mut sbuf = [0u8; 32];
        assert_eq!(r.read_string_utf8(&mut sbuf), Ok(""));
        assert_eq!(r.read_string_utf8(&mut sbuf), Ok("log.txt"));
        assert_eq!(r.read_string_utf8(&mut sbuf), Ok("🦀"));
        assert_eq!(r.remaining(), 0);
    }

    #[test]
    fn ptp_string_bounds_checks() {
        // Claim 10 UTF-16 chars when only 2 bytes remain.
        let bad = [10u8, 0x41, 0x00];
        let mut r = MtpReader::new(&bad);
        let mut out = [0u8; 16];
        assert_eq!(r.read_string_utf8(&mut out), Err(MtpError::PacketTooShort));
    }

    #[test]
    fn datasets_encode_and_object_info_round_trip() {
        let mut dbuf = [0u8; 512];
        let dlen = DeviceInfo::default().encode(&mut dbuf).unwrap();
        assert!(dlen > 40);

        let mut sbuf = [0u8; 256];
        let slen = StorageInfo {
            max_capacity: 1024 * 1024,
            free_space_in_bytes: 512 * 1024,
            ..Default::default()
        }
        .encode(&mut sbuf)
        .unwrap();
        assert!(slen > 30);

        let info = ObjectInfo::file(0x0001_0001, PARENT_ROOT, format::TEXT, 1234, "hello.txt");
        let mut obuf = [0u8; 256];
        let olen = info.encode(&mut obuf).unwrap();
        let mut fname = [0u8; 64];
        let decoded = ObjectInfo::decode(&obuf[..olen], &mut fname).unwrap();
        assert_eq!(decoded, info);
    }
}

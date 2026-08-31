use core::ffi::c_void;

use super::Status;

// KBoot (KB) ROM API

#[repr(C)]
struct KbRegion {
    // Region base address.
    address: u32,
    // Region length in bytes.
    length: u32,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum KbOperation {
    // Verify/authenticate image.
    AuthenticateImage = 1,
    // Load image.
    LoadImage = 2,
    // Number of KB operations.
    OperationCount = 3,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct KbLoadSb {
    // Profile selector (meaning per ROM header / implementation).
    profile: u32,
    // Minimum build number.
    minBuildNumber: u32,
    // Override SB boot section ID.
    overrideSBBootSectionID: u32,
    // User SB KEK pointer.
    userSBKEK: *mut u32,
    // Number of region descriptors.
    regionCount: u32,
    // Pointer to regions array.
    regions: *const KbRegion,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct KbAuthenticate {
    // Profile selector (meaning per ROM header / implementation).
    profile: u32,
    // Minimum build number.
    minBuildNumber: u32,
    // Maximum image length.
    maxImageLength: u32,
    // User RHK pointer.
    userRHK: *mut u32,
}

#[repr(C)]
union KbOptionsParams {
    authenticate: KbAuthenticate,
    loadSB: KbLoadSb,
}

#[repr(C)]
struct KbOptions {
    // Must be KB_API_VERSION.
    version: u32,
    // Caller-provided buffer used by ROM.
    buffer: *mut u8,
    // Length of buffer in bytes.
    bufferLength: u32,
    // Requested operation.
    op: KbOperation,
    // Operation-specific parameters.
    params: KbOptionsParams,
}

#[repr(C)]
struct KbBufferDesc {
    // Buffer pointer.
    buf: *mut u8,
    // Buffer length in bytes.
    len: u32,
    // Allocated size.
    allocated: u32,
}

#[repr(C)]
struct KbSessionRef {
    // Options used to create the session.
    options: KbOptions,
    // Internal buffer descriptor.
    buffer_desc: KbBufferDesc,
    // Opaque operation context.
    op_context: *mut c_void,
}

#[repr(C)]
pub struct KBApiDriver {
    // Initialize KB session.
    kb_init: unsafe extern "C" fn(session: *mut *mut KbSessionRef, options: *const KbOptions) -> Status,
    // Deinitialize KB session.
    kb_deinit: unsafe extern "C" fn(session: *mut KbSessionRef) -> Status,
    // Execute KB operation over a data buffer.
    kb_execute: unsafe extern "C" fn(session: *mut KbSessionRef, data: *const u8, dataLength: u32) -> Status,
}

impl KBApiDriver {
    fn kb_init(&self, session: *mut *mut KbSessionRef, options: *const KbOptions) -> KbStatus {
        unsafe { (self.kb_init)(session, options) }.into()
    }

    fn kb_deinit(&self, session: *mut KbSessionRef) -> KbStatus {
        unsafe { (self.kb_deinit)(session) }.into()
    }

    fn kb_execute(&self, session: *mut KbSessionRef, data: *const u8, dataLength: u32) -> KbStatus {
        unsafe { (self.kb_execute)(session, data, dataLength) }.into()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum KbStatus {
    Success,
    Fail,
    InvalidArgument,
    RomLdrDataUnderrun,
    RomLdrJumpReturned,
    RomLdrRollbackBlocked,
    RomLdrPendingJumpCommand,
    RomApiBufferSizeNotEnough,
    RomApiInvalidBuffer,
    Unknown(u32),
}

impl From<u32> for KbStatus {
    fn from(raw: u32) -> Self {
        match raw {
            super::KSTATUS_KB_SUCCESS => Self::Success,
            super::KSTATUS_KB_FAIL => Self::Fail,
            super::KSTATUS_KB_INVALID_ARGUMENT => Self::InvalidArgument,
            super::KSTATUS_KB_ROMLDR_DATA_UNDERRUN => Self::RomLdrDataUnderrun,
            super::KSTATUS_KB_ROMLDR_JUMP_RETURNED => Self::RomLdrJumpReturned,
            super::KSTATUS_KB_ROMLDR_ROLLBACK_BLOCKED => Self::RomLdrRollbackBlocked,
            super::KSTATUS_KB_ROMLDR_PENDING_JUMP_COMMAND => Self::RomLdrPendingJumpCommand,
            super::KSTATUS_KB_BUFFER_SIZE_NOT_ENOUGH => Self::RomApiBufferSizeNotEnough,
            super::KSTATUS_KB_INVALID_BUFFER => Self::RomApiInvalidBuffer,
            other => Self::Unknown(other),
        }
    }
}

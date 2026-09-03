use core::ffi::c_void;
use core::ptr::null_mut;
use core::sync::atomic::{AtomicBool, Ordering};

use super::Status;

#[repr(C)]
pub struct KbRegion {
    /// Region base address.
    pub address: u32,
    /// Region length in bytes.
    pub length: u32,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KbOperation {
    /// Verify/authenticate image.
    AuthenticateImage = 1,
    /// Load image.
    LoadImage = 2,
    /// Number of KB operations.
    OperationCount = 3,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct KbLoadSb {
    /// Profile selector (meaning per ROM header / implementation).
    pub profile: u32,
    /// Minimum build number.
    pub min_build_number: u32,
    /// Override SB boot section ID.
    pub override_sbboot_section_id: u32,
    /// User SB KEK pointer.
    pub user_sbkek: *mut u32,
    /// Number of region descriptors.
    pub region_count: u32,
    /// Pointer to regions array.
    pub regions: *const KbRegion,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct KbAuthenticate {
    /// Profile selector (meaning per ROM header / implementation).
    pub profile: u32,
    /// Minimum build number.
    pub min_build_number: u32,
    /// Maximum image length.
    pub max_image_length: u32,
    /// User RHK pointer.
    pub user_rhk: *mut u32,
}

#[repr(C)]
pub union KbOptionsParams {
    pub authenticate: KbAuthenticate,
    pub load_sb: KbLoadSb,
}

#[repr(C)]
pub struct KbOptions {
    /// Must be KB_API_VERSION.
    pub version: u32,
    /// Caller-provided buffer used by ROM.
    pub buffer: *mut u8,
    /// Length of buffer in bytes.
    pub buffer_length: u32,
    /// Requested operation.
    pub op: KbOperation,
    /// Operation-specific parameters.
    pub params: KbOptionsParams,
}

#[repr(C)]
struct KbBufferDesc {
    /// Buffer pointer.
    buf: *mut u8,
    /// Buffer length in bytes.
    len: u32,
    /// Allocated size.
    allocated: u32,
}

#[repr(C)]
struct KbSessionRef {
    /// Options used to create the session.
    options: KbOptions,
    /// Internal buffer descriptor.
    buffer_desc: KbBufferDesc,
    /// Opaque operation context.
    op_context: *mut c_void,
}

#[repr(C)]
pub struct KbVtable {
    /// Initialize KB session.
    kb_init: unsafe extern "C" fn(session: *mut *mut KbSessionRef, options: *const KbOptions) -> Status,
    /// Deinitialize KB session.
    kb_deinit: unsafe extern "C" fn(session: *mut KbSessionRef) -> Status,
    /// Execute KB operation over a data buffer.
    kb_execute: unsafe extern "C" fn(session: *mut KbSessionRef, data: *const u8, data_length: u32) -> Status,
}

pub struct Kb {
    vtable: &'static KbVtable,
    session_ptr: *mut KbSessionRef,
}

static TAKEN: AtomicBool = AtomicBool::new(false);

impl Kb {
    pub(super) fn new(vtable: &'static KbVtable, options: KbOptions) -> Result<Self, KbError> {
        if TAKEN
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return Err(KbError::Unavailable);
        }

        let mut session_ptr = null_mut();

        Result::from(unsafe { (vtable.kb_init)(&raw mut session_ptr, &raw const options) })?;
        Ok(Self { vtable, session_ptr })
    }

    pub fn kb_execute(&mut self, data: &[u8]) -> Result<(), KbError> {
        unsafe { (self.vtable.kb_execute)(self.session_ptr, data.as_ptr(), data.len() as u32) }.into()
    }
}

impl Drop for Kb {
    fn drop(&mut self) {
        if let Err(_e) = Result::<(), KbError>::from(unsafe { (self.vtable.kb_deinit)(self.session_ptr) }) {
            // Not sure what we can do...
        }

        TAKEN.store(false, Ordering::Relaxed);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KbError {
    Fail,
    InvalidArgument,
    RomLdrDataUnderrun,
    RomLdrJumpReturned,
    RomLdrRollbackBlocked,
    RomLdrPendingJumpCommand,
    RomApiBufferSizeNotEnough,
    RomApiInvalidBuffer,
    Unknown(u32),
    Unavailable,
}

impl From<Status> for Result<(), KbError> {
    fn from(raw: Status) -> Self {
        match raw.0 {
            KSTATUS_KB_SUCCESS => Ok(()),
            KSTATUS_KB_FAIL => Err(KbError::Fail),
            KSTATUS_KB_INVALID_ARGUMENT => Err(KbError::InvalidArgument),
            KSTATUS_KB_ROMLDR_DATA_UNDERRUN => Err(KbError::RomLdrDataUnderrun),
            KSTATUS_KB_ROMLDR_JUMP_RETURNED => Err(KbError::RomLdrJumpReturned),
            KSTATUS_KB_ROMLDR_ROLLBACK_BLOCKED => Err(KbError::RomLdrRollbackBlocked),
            KSTATUS_KB_ROMLDR_PENDING_JUMP_COMMAND => Err(KbError::RomLdrPendingJumpCommand),
            KSTATUS_KB_BUFFER_SIZE_NOT_ENOUGH => Err(KbError::RomApiBufferSizeNotEnough),
            KSTATUS_KB_INVALID_BUFFER => Err(KbError::RomApiInvalidBuffer),
            other => Err(KbError::Unknown(other)),
        }
    }
}

const KSTATUS_KB_SUCCESS: u32 = 0;
const KSTATUS_KB_FAIL: u32 = 1;
const KSTATUS_KB_INVALID_ARGUMENT: u32 = 4;

const KSTATUS_KB_ROMLDR_DATA_UNDERRUN: u32 = 10109;
const KSTATUS_KB_ROMLDR_JUMP_RETURNED: u32 = 10110;
const KSTATUS_KB_ROMLDR_ROLLBACK_BLOCKED: u32 = 10115;
const KSTATUS_KB_ROMLDR_PENDING_JUMP_COMMAND: u32 = 10119;

const KSTATUS_KB_BUFFER_SIZE_NOT_ENOUGH: u32 = 10802;
const KSTATUS_KB_INVALID_BUFFER: u32 = 10803;

use core::sync::atomic::{AtomicBool, Ordering};

#[repr(C)]
struct NbootCtx {
    // Opaque context buffer. Size must match NBOOT_CONTEXT_SIZE from the ROM header.
    opaque: [u8; 0xA94],
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NbootMemCryptRegion {
    Region0 = 0,
    Region1 = 1,
    Region2 = 2,
    Region3 = 3,
    Region4 = 4,
    Region5 = 5,
    Region6 = 6,
    Region7 = 7,
}

type NbootMemCryptOperation = u32; // Need to confirm exact typed values.

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NbootMemCryptIpedModeSelect {
    Rounds12 = 0x3C5A_C33C,
    Rounds22 = 0x5AA5_5AA5,
    FullyPipelined = 0x5A5A_5A5A,
    NotFullyPipelined = 0xA5A5_A5A5,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NbootMemCryptEngineType {
    Npx = 0x5959_5959,
    Iped = 0x9595_9595,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct NbootNpxRegionConfig {
    /// Must be NbootMemCryptEngineType::Npx.
    pub config_id: NbootMemCryptEngineType,
    /// Start address (inclusive).
    pub start_address: u32,
    /// End address (inclusive).
    pub end_address: u32,
    /// Subregion bitmap.
    pub subregion: u64,
    /// IV erase counter.
    pub iv_erase_counter: u32,
    /// Region lock flag.
    pub region_lock: u8,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct NbootIpedRegionConfig {
    /// Must be NbootMemCryptEngineType::Iped.
    pub config_id: NbootMemCryptEngineType,
    /// Start address (inclusive).
    pub start_address: u32,
    /// End address (inclusive).
    pub end_address: u32,
    /// IV erase counter.
    pub iv_erase_counter: u32,
    /// Region flags.
    pub region_flags: u8,
    /// Additional authenticated data.
    pub aad: u32,
}

#[repr(C)]
pub union NbootMemCryptRegionConfig {
    pub npx_config: NbootNpxRegionConfig,
    pub iped_config: NbootIpedRegionConfig,
}

#[repr(C)]
pub struct NbootRotAuthParms {
    pub soc_root_key_revocation: [u32; 4],
    pub soc_image_key_revocation: u32,
    pub soc_rkh: [u32; 12],
    pub soc_rkh_1: [u32; 12], // PQC_ROTKH (hash of hashes)
    pub soc_number_of_root_keys: u32,
    pub soc_root_key_usage: [u32; 4],
    pub soc_root_key_type_and_length: u32,
    pub soc_lifecycle: u32,
}

#[repr(C)]
pub struct NbootImgAuthParms {
    pub soc_ro_tnvm: NbootRotAuthParms,
    pub soc_trusted_firmware_version: u32,
}

#[repr(C)]
pub struct NbootSb4LoadManifestParms {
    /// Returned RoTNVM/auth parameters.
    pub soc_ro_tnvm: NbootRotAuthParms,
    /// Returned trusted firmware version.
    pub soc_trusted_firmware_version: u32,
    /// Returned maximum SB block size.
    pub max_block_size: u32,
    /// Returned PCK blob.
    pub pck_blob: [u8; 48],
}

// Forward declarations of subtables.
#[repr(C)]
pub struct NbootVtable {
    // Initialize NBOOT context (must be called before other NBOOT APIs).
    context_init: unsafe extern "C" fn(ctx: *mut NbootCtx) -> NbootStatus,
    // Deinitialize NBOOT context.
    context_deinit: unsafe extern "C" fn(ctx: *mut NbootCtx) -> NbootStatus,
    // Set context UUID (bytes; see NXP header for exact semantics).
    context_set_uuid: unsafe extern "C" fn(ctx: *mut NbootCtx, uuid: *const u8) -> NbootStatus,
    // SB4: load/parse manifest.
    sb4_load_manifest: unsafe extern "C" fn(
        ctx: *mut NbootCtx,
        manifest: *const u32,
        parms: *mut NbootSb4LoadManifestParms,
    ) -> NbootStatus,
    // SB4: load next block.
    sb4_load_block: unsafe extern "C" fn(ctx: *mut NbootCtx, block: *mut u32) -> NbootStatus,
    // SB4: authenticate & complete check (ROMAPI entry).
    sb4_check_authenticity_and_completeness_romapi: unsafe extern "C" fn(
        ctx: *mut NbootCtx,
        address: *const u32,
        parms: *mut NbootSb4LoadManifestParms,
    ) -> NbootStatus,
    // Authenticate image (signature verification result returned via is_signature_verified).
    img_authenticate_romapi: unsafe extern "C" fn(
        ctx: *mut NbootCtx,
        image_start: *const u8,
        is_signature_verified: *mut NbootBool,
        parms: *mut NbootImgAuthParms,
    ) -> NbootStatus,

    // Enable/Configure in-memory encryption for a given address range.
    mem_crypt_enable_encrypt_for_address_range: unsafe extern "C" fn(
        ctx: *mut NbootCtx,
        region_number: NbootMemCryptRegion,
        region_config: *mut NbootMemCryptRegionConfig,
        iped_mode_select: NbootMemCryptIpedModeSelect,
    ) -> NbootStatus,
    // Check whether an operation is allowed for an address range and return flags/IV counters.
    mem_crypt_range_checker: unsafe extern "C" fn(
        ctx: *mut NbootCtx,
        operation: NbootMemCryptOperation,
        address: u32,
        length: u32,
        flags: *mut u8,
        npx_iv_erase_cntr: *mut u32,
        iped_iv_erase_cntr: *mut u32,
        npx_erase_check_en: u32,
    ) -> NbootStatus,
    // Enable background hashing (ROM-managed hash engine / DMA channel selection).
    background_hash_enable: unsafe extern "C" fn(ctx: *mut NbootCtx, hash_dma_channel: u32) -> NbootStatus,
}

pub struct Nboot {
    vtable: &'static NbootVtable,
    context: NbootCtx,
}

static TAKEN: AtomicBool = AtomicBool::new(false);

impl Nboot {
    pub(super) fn new(vtable: &'static NbootVtable) -> Result<Self, NbootError> {
        let mut context = NbootCtx { opaque: [0; _] };
        if TAKEN
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return Err(NbootError::Unavailable);
        }

        Result::from(unsafe { (vtable.context_init)(&raw mut context) })?;
        Ok(Self { vtable, context })
    }

    pub fn nboot_context_set_uuid(&mut self, uuid: &[u8; 16]) -> Result<(), NbootError> {
        unsafe { (self.vtable.context_set_uuid)(&raw mut self.context, uuid.as_ptr()) }.into()
    }

    pub fn nboot_sb4_load_manifest(
        &mut self,
        manifest: u32,
        parms: &mut NbootSb4LoadManifestParms,
    ) -> Result<(), NbootError> {
        unsafe { (self.vtable.sb4_load_manifest)(&raw mut self.context, manifest as *const _, parms) }.into()
    }

    pub fn nboot_sb4_load_block(&mut self, block_addr: u32) -> Result<(), NbootError> {
        unsafe { (self.vtable.sb4_load_block)(&raw mut self.context, block_addr as *mut _) }.into()
    }

    pub fn nboot_sb4_check_authenticity_and_completeness_romapi(
        &mut self,
        address: u32,
        parms: &mut NbootSb4LoadManifestParms,
    ) -> Result<(), NbootError> {
        unsafe {
            (self.vtable.sb4_check_authenticity_and_completeness_romapi)(
                &raw mut self.context,
                address as *const _,
                parms,
            )
        }
        .into()
    }

    pub fn nboot_img_authenticate_romapi(
        &mut self,
        image_start: u32,
        parms: &mut NbootImgAuthParms,
    ) -> Result<NbootBoolValue, NbootError> {
        let mut is_signature_verified = NbootBool(0);
        Result::from(unsafe {
            (self.vtable.img_authenticate_romapi)(
                &raw mut self.context,
                image_start as *const _,
                &raw mut is_signature_verified,
                parms,
            )
        })
        .map(|()| is_signature_verified.value().unwrap_or(NbootBoolValue::False))
    }

    pub fn nboot_mem_crypt_enable_encrypt_for_address_range(
        &mut self,
        region_number: NbootMemCryptRegion,
        region_config: &mut NbootMemCryptRegionConfig,
        iped_mode_select: NbootMemCryptIpedModeSelect,
    ) -> Result<(), NbootError> {
        unsafe {
            (self.vtable.mem_crypt_enable_encrypt_for_address_range)(
                &raw mut self.context,
                region_number,
                region_config,
                iped_mode_select,
            )
        }
        .into()
    }

    #[allow(clippy::too_many_arguments)]
    pub fn nboot_mem_crypt_range_checker(
        &mut self,
        operation: NbootMemCryptOperation,
        address: u32,
        length: u32,
        npx_erase_check_en: u32,
    ) -> Result<CryptRangeCheckData, NbootError> {
        let mut data = CryptRangeCheckData {
            flags: 0,
            npx_iv_erase_cntr: 0,
            iped_iv_erase_cntr: 0,
        };

        Result::from(unsafe {
            (self.vtable.mem_crypt_range_checker)(
                &raw mut self.context,
                operation,
                address,
                length,
                &raw mut data.flags,
                &raw mut data.npx_iv_erase_cntr,
                &raw mut data.iped_iv_erase_cntr,
                npx_erase_check_en,
            )
        })
        .map(|()| data)
    }

    pub fn nboot_background_hash_enable(&mut self, hash_dma_channel: u32) -> Result<(), NbootError> {
        unsafe { (self.vtable.background_hash_enable)(&raw mut self.context, hash_dma_channel) }.into()
    }
}

impl Drop for Nboot {
    fn drop(&mut self) {
        if let Err(_e) = Result::<(), NbootError>::from(unsafe { (self.vtable.context_deinit)(&raw mut self.context) })
        {
            // Not sure what we can do...
        }

        TAKEN.store(false, Ordering::Relaxed);
    }
}

pub struct CryptRangeCheckData {
    pub flags: u8,
    pub npx_iv_erase_cntr: u32,
    pub iped_iv_erase_cntr: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum NbootError {
    Fail,
    InvalidArgument,
    OperationAllowed,
    OperationDisallowed,
    KeyNotAvailable,
    Unknown(u64),
    Unavailable,
}

impl From<NbootStatus> for Result<(), NbootError> {
    fn from(raw: NbootStatus) -> Self {
        // The ROM returns a usable 32-bit value; upper 32 bits are likely security related metadata/ fault attack protection, need to mask it out.
        let raw = raw.0 & 0xFFFF_FFFF;
        match raw {
            KSTATUS_NBOOT_SUCCESS => Ok(()),
            KSTATUS_NBOOT_FAIL => Err(NbootError::Fail),
            KSTATUS_NBOOT_INVALID_ARGUMENT => Err(NbootError::InvalidArgument),
            KNBOOT_OPERATION_ALLOWED => Err(NbootError::OperationAllowed),
            KNBOOT_OPERATION_DISALLOWED => Err(NbootError::OperationDisallowed),
            KSTATUS_NBOOT_KEY_NOT_AVAILABLE => Err(NbootError::KeyNotAvailable),
            other => Err(NbootError::Unknown(other)),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(transparent)]
struct NbootBool(u32);

impl NbootBool {
    pub fn value(&self) -> Option<NbootBoolValue> {
        NbootBoolValue::try_from(*self).ok()
    }
}

/// Boolean/result markers returned via nboot_bool_t out-parameters.
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NbootBoolValue {
    True = 0x3C5A_C33C,
    True256 = 0x3C5A_C35A,
    True384 = 0x3C5A_C3A5,
    False = 0x5AA5_5AA5,
}

impl TryFrom<NbootBool> for NbootBoolValue {
    type Error = NbootError;
    fn try_from(raw: NbootBool) -> Result<Self, Self::Error> {
        match raw.0 {
            0x3C5A_C33C => Ok(Self::True),
            0x3C5A_C35A => Ok(Self::True256),
            0x3C5A_C3A5 => Ok(Self::True384),
            0x5AA5_5AA5 => Ok(Self::False),
            _ => Err(NbootError::InvalidArgument),
        }
    }
}

#[repr(transparent)]
struct NbootStatus(u64);

const KSTATUS_NBOOT_SUCCESS: u64 = 0x5A5A_5A5A;
const KSTATUS_NBOOT_FAIL: u64 = 0x5A5A_A5A5;
const KSTATUS_NBOOT_INVALID_ARGUMENT: u64 = 0x5A5A_A5F0;

// NBOOT API status codes (MCXA ROM, Table 46 / 9.2.5.11)
// These are returned by APIs such as `nboot_mem_crypt_range_checker`.
const KNBOOT_OPERATION_ALLOWED: u64 = 0x3C5A_33CC;
const KNBOOT_OPERATION_DISALLOWED: u64 = 0x5AA5_CC33;
const KSTATUS_NBOOT_KEY_NOT_AVAILABLE: u64 = 0x5A5A_A5E6;

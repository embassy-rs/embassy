use super::{NbootBool, NbootStatusProtected};

// Boolean/result markers returned via nboot_bool_t out-parameters.
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NbootBoolValue {
    True = 0x3C5A_C33C,
    True256 = 0x3C5A_C35A,
    True384 = 0x3C5A_C3A5,
    False = 0x5AA5_5AA5,
}

impl NbootBoolValue {
    const fn from_raw(raw: NbootBool) -> Option<Self> {
        match raw {
            0x3C5A_C33C => Some(Self::True),
            0x3C5A_C35A => Some(Self::True256),
            0x3C5A_C3A5 => Some(Self::True384),
            0x5AA5_5AA5 => Some(Self::False),
            _ => None,
        }
    }
}

#[repr(C)]
struct NbootCtx {
    // Opaque context buffer. Size must match NBOOT_CONTEXT_SIZE from the ROM header.
    opaque: [u8; 0xA94],
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NbootMemCryptRegion {
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
enum NbootMemCryptIpedModeSelect {
    Rounds12 = 0x3C5A_C33C,
    Rounds22 = 0x5AA5_5AA5,
    FullyPipelined = 0x5A5A_5A5A,
    NotFullyPipelined = 0xA5A5_A5A5,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NbootMemCryptEngineType {
    Npx = 0x5959_5959,
    Iped = 0x9595_9595,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct NbootNpxRegionConfig {
    // Must be NbootMemCryptEngineType::Npx.
    configId: NbootMemCryptEngineType,
    // Start address (inclusive).
    startAddress: u32,
    // End address (inclusive).
    endAddress: u32,
    // Subregion bitmap.
    subregion: u64,
    // IV erase counter.
    ivEraseCounter: u32,
    // Region lock flag.
    regionLock: u8,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct NbootIpedRegionConfig {
    // Must be NbootMemCryptEngineType::Iped.
    configId: NbootMemCryptEngineType,
    // Start address (inclusive).
    startAddress: u32,
    // End address (inclusive).
    endAddress: u32,
    // IV erase counter.
    ivEraseCounter: u32,
    // Region flags.
    regionFlags: u8,
    // Additional authenticated data.
    aad: u32,
}

#[repr(C)]
union NbootMemCryptRegionConfig {
    npxConfig: NbootNpxRegionConfig,
    ipedConfig: NbootIpedRegionConfig,
}

#[repr(C)]
struct NbootRotAuthParms {
    soc_rootKeyRevocation: [u32; 4],
    soc_imageKeyRevocation: u32,
    soc_rkh: [u32; 12],
    soc_rkh_1: [u32; 12], // PQC_ROTKH (hash of hashes)
    soc_numberOfRootKeys: u32,
    soc_rootKeyUsage: [u32; 4],
    soc_rootKeyTypeAndLength: u32,
    soc_lifecycle: u32,
}

#[repr(C)]
struct NbootImgAuthParms {
    soc_RoTNVM: NbootRotAuthParms,
    soc_trustedFirmwareVersion: u32,
}

#[repr(C)]
struct NbootImgAuthenticateCmacParms {
    expectedMAC: [u32; 4],
}

#[repr(C)]
struct NbootSb4LoadManifestParms {
    // Returned RoTNVM/auth parameters.
    soc_RoTNVM: NbootRotAuthParms,
    // Returned trusted firmware version.
    soc_trustedFirmwareVersion: u32,
    // Returned maximum SB block size.
    maxBlockSize: u32,
    // Returned PCK blob.
    pckBlob: [u8; 48],
}

// Root key configuration constants for NBOOT
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NbootRootKeyRevocation {
    Enabled = 0xAA,
    Revoked = 0xBB,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NbootRootKeyUsage {
    All = 0x0,
    DebugCa = 0x1,
    ImageCaFwCa = 0x2,
    DebugCaImageCaFwCa = 0x3,
    ImageKeyFwKey = 0x4,
    ImageKey = 0x5,
    FwKey = 0x6,
    Unused = 0x7,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NbootRootKeyType {
    EcdsaP384Mldsa87 = 0x0000_FD04, // Hybrid root key type: ECDSA P-384 + ML-DSA-87
}

// Lifecycle state codes (low-byte discriminators) and full CFPA LC_STATE values.
// Per Table 18 (Life Cycle States): LC_STATE is a u32 like 0x9635_FC03.
// Some call sites only carry the low-byte discriminator (e.g. 0x03 for Develop),
// so we keep both representations.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NbootLifecycleDiscriminator {
    Develop = 0x03,
    Develop2 = 0x07,
    InField = 0x0F,
    InFieldLocked = 0xCF,
    OemFieldReturn = 0x1F,
    FailureAnalysis = 0x3F,
    Bricked = 0x5A,
}

impl NbootLifecycleDiscriminator {
    const fn from_raw(raw: u8) -> Option<Self> {
        match raw {
            0x03 => Some(Self::Develop),
            0x07 => Some(Self::Develop2),
            0x0F => Some(Self::InField),
            0xCF => Some(Self::InFieldLocked),
            0x1F => Some(Self::OemFieldReturn),
            0x3F => Some(Self::FailureAnalysis),
            0x5A => Some(Self::Bricked),
            _ => None,
        }
    }

    const fn state(self) -> NbootLifecycleState {
        match self {
            Self::Develop => NbootLifecycleState::Develop,
            Self::Develop2 => NbootLifecycleState::Develop2,
            Self::InField => NbootLifecycleState::InField,
            Self::InFieldLocked => NbootLifecycleState::InFieldLocked,
            Self::OemFieldReturn => NbootLifecycleState::OemFieldReturn,
            Self::FailureAnalysis => NbootLifecycleState::FailureAnalysis,
            Self::Bricked => NbootLifecycleState::Bricked,
        }
    }
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NbootLifecycleState {
    Develop = 0x9635_FC03,
    Develop2 = 0x9635_F807,
    InField = 0x9635_F00F,
    InFieldLocked = 0x9635_30CF,
    OemFieldReturn = 0x9635_E01F,
    FailureAnalysis = 0x9635_C03F,
    Bricked = 0x9635_A55A,
}

impl NbootLifecycleState {
    const fn from_raw(raw: u32) -> Option<Self> {
        match raw {
            0x9635_FC03 => Some(Self::Develop),
            0x9635_F807 => Some(Self::Develop2),
            0x9635_F00F => Some(Self::InField),
            0x9635_30CF => Some(Self::InFieldLocked),
            0x9635_E01F => Some(Self::OemFieldReturn),
            0x9635_C03F => Some(Self::FailureAnalysis),
            0x9635_A55A => Some(Self::Bricked),
            _ => None,
        }
    }

    const fn discriminator(self) -> NbootLifecycleDiscriminator {
        match self {
            Self::Develop => NbootLifecycleDiscriminator::Develop,
            Self::Develop2 => NbootLifecycleDiscriminator::Develop2,
            Self::InField => NbootLifecycleDiscriminator::InField,
            Self::InFieldLocked => NbootLifecycleDiscriminator::InFieldLocked,
            Self::OemFieldReturn => NbootLifecycleDiscriminator::OemFieldReturn,
            Self::FailureAnalysis => NbootLifecycleDiscriminator::FailureAnalysis,
            Self::Bricked => NbootLifecycleDiscriminator::Bricked,
        }
    }

    const fn nboot_soc_lifecycle(self) -> u32 {
        let discriminator = self.discriminator() as u16;
        (((!discriminator) as u32) << 16) | (discriminator as u32)
    }

    const fn from_any_raw(raw: u32) -> Option<Self> {
        if let Some(state) = Self::from_raw(raw) {
            return Some(state);
        }
        // Short-form input: bare discriminator byte only. A long-form value that
        // failed the full-word match above is corrupt; do not guess from its low byte.
        if raw <= 0xFF
            && let Some(d) = NbootLifecycleDiscriminator::from_raw(raw as u8) {
                return Some(d.state());
            }
        None
    }

    /// Returns a monotonic rank for forward-only progression checks.
    /// Higher rank = further along the lifecycle.
    const fn rank(self) -> u8 {
        match self {
            Self::Develop => 0,
            Self::Develop2 => 1,
            Self::InField => 2,
            Self::InFieldLocked => 2, // Same rank as InField since locking is not a lifecycle progression.
            Self::OemFieldReturn => 3,
            Self::FailureAnalysis => 4,
            Self::Bricked => 5,
        }
    }

    /// Returns true if advancing to `next` is a valid forward progression (no regressions, no same state).
    const fn can_advance_to(self, next: Self) -> bool {
        next.rank() >= self.rank() && (self as u32 != next as u32)
    }
}

// Sealed trait: CanAdvanceTo<Next>

// Only the combinations listed below exist. Attempting to call
// `cfpa_stage_lifecycle_advance_and_reset` with any other (From, To) pair
// is a *compile* error.

mod sealed {
    pub trait Sealed {}
    impl Sealed for super::Develop {}
    impl Sealed for super::Develop2 {}
    impl Sealed for super::InField {}
    impl Sealed for super::InFieldLocked {}
    impl Sealed for super::OemFieldReturn {}
    impl Sealed for super::FailureAnalysis {}
    impl Sealed for super::Bricked {}
}

// Zero-sized types, one per lifecycle state, used as type parameters to enforce valid lifecycle transitions at compile time.
struct Develop;
struct Develop2;
struct InField;
struct InFieldLocked;
struct OemFieldReturn;
struct FailureAnalysis;
struct Bricked;

/// Compile-time proof that advancing from `Self` to `Next` is a valid transition.
/// Only the explicit `impl` blocks below are permitted.
trait CanAdvanceTo<Next>: sealed::Sealed {}

impl CanAdvanceTo<Develop2> for Develop {}
impl CanAdvanceTo<InField> for Develop {}
impl CanAdvanceTo<InField> for Develop2 {}
impl CanAdvanceTo<InFieldLocked> for Develop2 {}
impl CanAdvanceTo<InFieldLocked> for InField {}
impl CanAdvanceTo<InField> for InFieldLocked {}
impl CanAdvanceTo<OemFieldReturn> for InField {}
impl CanAdvanceTo<OemFieldReturn> for InFieldLocked {}
impl CanAdvanceTo<FailureAnalysis> for OemFieldReturn {}
impl CanAdvanceTo<Bricked> for Develop2 {}
impl CanAdvanceTo<Bricked> for InFieldLocked {}
impl CanAdvanceTo<Bricked> for InField {}
impl CanAdvanceTo<Bricked> for OemFieldReturn {}
impl CanAdvanceTo<Bricked> for FailureAnalysis {}

/// Maps a zero-sized lifecycle type to its runtime `NbootLifecycleState` value.
/// Used by `verify_lifecycle_transition` to assert the device is actually in
/// the expected `From` state before committing a transition.
trait ActualLifecycleState: sealed::Sealed {
    const STATE: NbootLifecycleState;
}

impl ActualLifecycleState for Develop {
    const STATE: NbootLifecycleState = NbootLifecycleState::Develop;
}
impl ActualLifecycleState for Develop2 {
    const STATE: NbootLifecycleState = NbootLifecycleState::Develop2;
}
impl ActualLifecycleState for InField {
    const STATE: NbootLifecycleState = NbootLifecycleState::InField;
}
impl ActualLifecycleState for InFieldLocked {
    const STATE: NbootLifecycleState = NbootLifecycleState::InFieldLocked;
}
impl ActualLifecycleState for OemFieldReturn {
    const STATE: NbootLifecycleState = NbootLifecycleState::OemFieldReturn;
}
impl ActualLifecycleState for FailureAnalysis {
    const STATE: NbootLifecycleState = NbootLifecycleState::FailureAnalysis;
}
impl ActualLifecycleState for Bricked {
    const STATE: NbootLifecycleState = NbootLifecycleState::Bricked;
}

// Forward declarations of subtables.
#[repr(C)]
pub struct NbootDriver {
    // Initialize NBOOT context (must be called before other NBOOT APIs).
    nboot_context_init: unsafe extern "C" fn(ctx: *mut NbootCtx) -> NbootStatusProtected,
    // Deinitialize NBOOT context.
    nboot_context_deinit: unsafe extern "C" fn(ctx: *mut NbootCtx) -> NbootStatusProtected,
    // Set context UUID (bytes; see NXP header for exact semantics).
    nboot_context_set_uuid: unsafe extern "C" fn(ctx: *mut NbootCtx, uuid: *const u8) -> NbootStatusProtected,
    // SB4: load/parse manifest.
    nboot_sb4_load_manifest: unsafe extern "C" fn(
        ctx: *mut NbootCtx,
        manifest: *const u32,
        parms: *mut NbootSb4LoadManifestParms,
    ) -> NbootStatusProtected,
    // SB4: load next block.
    nboot_sb4_load_block: unsafe extern "C" fn(ctx: *mut NbootCtx, block: *mut u32) -> NbootStatusProtected,
    // SB4: authenticate & complete check (ROMAPI entry).
    nboot_sb4_check_authenticity_and_completeness_romapi: unsafe extern "C" fn(
        ctx: *mut NbootCtx,
        address: *const u32,
        parms: *mut NbootSb4LoadManifestParms,
    ) -> NbootStatusProtected,
    // Authenticate image (signature verification result returned via is_signature_verified).
    nboot_img_authenticate_romapi: unsafe extern "C" fn(
        ctx: *mut NbootCtx,
        image_start: *const u8,
        is_signature_verified: *mut NbootBool,
        parms: *mut NbootImgAuthParms,
    ) -> NbootStatusProtected,

    // Enable/Configure in-memory encryption for a given address range.
    nboot_mem_crypt_enable_encrypt_for_address_range: unsafe extern "C" fn(
        ctx: *mut NbootCtx,
        region_number: NbootMemCryptRegion,
        region_config: *mut NbootMemCryptRegionConfig,
        iped_mode_select: NbootMemCryptIpedModeSelect,
    ) -> NbootStatusProtected,
    // Check whether an operation is allowed for an address range and return flags/IV counters.
    nboot_mem_crypt_range_checker: unsafe extern "C" fn(
        ctx: *mut NbootCtx,
        operation: NbootMemCryptOperation,
        address: u32,
        length: u32,
        flags: *mut u8,
        npx_iv_erase_cntr: *mut u32,
        iped_iv_erase_cntr: *mut u32,
        npx_erase_check_en: u32,
    ) -> NbootStatusProtected,
    // Enable background hashing (ROM-managed hash engine / DMA channel selection).
    nboot_background_hash_enable:
        unsafe extern "C" fn(ctx: *mut NbootCtx, hash_dma_channel: u32) -> NbootStatusProtected,
}

impl NbootDriver {
    fn nboot_context_init(&self, ctx: *mut NbootCtx) -> NbootStatus {
        unsafe { (self.nboot_context_init)(ctx) }.into()
    }

    fn nboot_context_deinit(&self, ctx: *mut NbootCtx) -> NbootStatus {
        unsafe { (self.nboot_context_deinit)(ctx) }.into()
    }

    fn nboot_context_set_uuid(&self, ctx: *mut NbootCtx, uuid: *const u8) -> NbootStatus {
        unsafe { (self.nboot_context_set_uuid)(ctx, uuid) }.into()
    }

    fn nboot_sb4_load_manifest(
        &self,
        ctx: *mut NbootCtx,
        manifest: *const u32,
        parms: *mut NbootSb4LoadManifestParms,
    ) -> NbootStatus {
        unsafe { (self.nboot_sb4_load_manifest)(ctx, manifest, parms) }.into()
    }

    fn nboot_sb4_load_block(&self, ctx: *mut NbootCtx, block: *mut u32) -> NbootStatus {
        unsafe { (self.nboot_sb4_load_block)(ctx, block) }.into()
    }

    fn nboot_sb4_check_authenticity_and_completeness_romapi(
        &self,
        ctx: *mut NbootCtx,
        address: *const u32,
        parms: *mut NbootSb4LoadManifestParms,
    ) -> NbootStatus {
        unsafe { (self.nboot_sb4_check_authenticity_and_completeness_romapi)(ctx, address, parms) }.into()
    }

    fn nboot_img_authenticate_romapi(
        &self,
        ctx: *mut NbootCtx,
        image_start: *const u8,
        is_signature_verified: *mut NbootBool,
        parms: *mut NbootImgAuthParms,
    ) -> NbootStatus {
        unsafe { (self.nboot_img_authenticate_romapi)(ctx, image_start, is_signature_verified, parms) }.into()
    }

    fn nboot_mem_crypt_enable_encrypt_for_address_range(
        &self,
        ctx: *mut NbootCtx,
        region_number: NbootMemCryptRegion,
        region_config: *mut NbootMemCryptRegionConfig,
        iped_mode_select: NbootMemCryptIpedModeSelect,
    ) -> NbootStatus {
        unsafe {
            (self.nboot_mem_crypt_enable_encrypt_for_address_range)(ctx, region_number, region_config, iped_mode_select)
        }
        .into()
    }

    #[allow(clippy::too_many_arguments)]
    fn nboot_mem_crypt_range_checker(
        &self,
        ctx: *mut NbootCtx,
        operation: NbootMemCryptOperation,
        address: u32,
        length: u32,
        flags: *mut u8,
        npx_iv_erase_cntr: *mut u32,
        iped_iv_erase_cntr: *mut u32,
        npx_erase_check_en: u32,
    ) -> NbootStatus {
        unsafe {
            (self.nboot_mem_crypt_range_checker)(
                ctx,
                operation,
                address,
                length,
                flags,
                npx_iv_erase_cntr,
                iped_iv_erase_cntr,
                npx_erase_check_en,
            )
        }
        .into()
    }

    fn nboot_background_hash_enable(&self, ctx: *mut NbootCtx, hash_dma_channel: u32) -> NbootStatus {
        // should enum for DMA channel selection be added? (Answer is yes but which channels??) For now just pass 0 for default channel. By doing an enum,
        // we can impose limitations on valid values (e.g. if only 2 channels are supported, etc.)
        unsafe { (self.nboot_background_hash_enable)(ctx, hash_dma_channel) }.into()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
enum NbootStatus {
    Success,
    Fail,
    InvalidArgument,
    OperationAllowed,
    OperationDisallowed,
    KeyNotAvailable,
    Unknown(u64),
}

impl From<u64> for NbootStatus {
    fn from(raw: u64) -> Self {
        // The ROM returns a usable 32-bit value; upper 32 bits are likely security related metadata/ fault attack protection, need to mask it out.
        let raw = raw & 0xFFFF_FFFF;
        match raw {
            super::KSTATUS_NBOOT_SUCCESS => Self::Success,
            super::KSTATUS_NBOOT_FAIL => Self::Fail,
            super::KSTATUS_NBOOT_INVALID_ARGUMENT => Self::InvalidArgument,
            super::KNBOOT_OPERATION_ALLOWED => Self::OperationAllowed,
            super::KNBOOT_OPERATION_DISALLOWED => Self::OperationDisallowed,
            super::KSTATUS_NBOOT_KEY_NOT_AVAILABLE => Self::KeyNotAvailable,
            other => Self::Unknown(other),
        }
    }
}

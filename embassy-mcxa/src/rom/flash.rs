use core::sync::atomic::{AtomicBool, Ordering};

use embedded_storage::nor_flash::{ErrorType, NorFlash, NorFlashError, NorFlashErrorKind, ReadNorFlash};
use nxp_pac::syscon::{ClrLpcac, DisDataSpec, DisFlashSpec, DisLpcac, DisMbeccErrData, DisMbeccErrInst};

use super::{StandardVersion, Status};

#[repr(C)]
#[derive(Debug, Default)]
pub struct FlashFfrConfig {
    /// FFR block base address.
    pub ffr_block_base: u32,
    /// FFR total size in bytes.
    pub ffr_total_size: u32,
    /// FFR page size in bytes.
    pub ffr_page_size: u32,
    /// Sector size in bytes.
    pub sector_size: u32,
    /// CFPA page version.
    pub cfpa_page_version: u32,
    /// CFPA page offset within FFR.
    pub cfpa_page_offset: u32,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlashReadEccOption {
    On = 0,
    Off = 1,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlashReadMarginOption {
    Normal = 0,
    VsProgram = 1,
    VsErase = 2,
    IllegalBitCombination = 3,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlashReadDmaccOption {
    Disabled = 0,
    Enabled = 1,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum FlashRampControlOption {
    #[default]
    Reserved = 0,
    DivisionFactor256 = 1,
    DivisionFactor128 = 2,
    DivisionFactor64 = 3,
}

#[cfg(feature = "mcxa5xx")]
#[repr(C)]
#[derive(Debug, Default)]
pub struct FlashReadSingleWordConfig {
    pub packed_options: u8,
    pub reserved1: [u8; 3],
}

impl FlashReadSingleWordConfig {
    pub const fn new(ecc: FlashReadEccOption, margin: FlashReadMarginOption, dmacc: FlashReadDmaccOption) -> Self {
        Self {
            packed_options: (ecc as u8) | ((margin as u8) << 1) | ((dmacc as u8) << 3),
            reserved1: [0; 3],
        }
    }
}

#[cfg(feature = "mcxa5xx")]
#[repr(C)]
#[derive(Debug, Default)]
pub struct FlashSetWriteModeConfig {
    pub program_ramp_control: FlashRampControlOption,
    pub erase_ramp_control: FlashRampControlOption,
    reserved: [u8; 2],
}

#[cfg(feature = "mcxa5xx")]
#[repr(C)]
#[derive(Debug, Default)]
pub struct FlashSetReadModeConfig {
    pub read_interface_timing_trim: u16,
    pub read_controller_timing_trim: u16,
    pub read_wait_states: u8,
    pub reserved: [u8; 3],
}

#[cfg(feature = "mcxa5xx")]
#[repr(C)]
#[derive(Debug, Default)]
pub struct FlashModeConfig {
    pub sys_freq_in_m_hz: u32,
    pub read_single_word: FlashReadSingleWordConfig,
    pub set_write_mode: FlashSetWriteModeConfig,
    pub set_read_mode: FlashSetReadModeConfig,
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct FlashConfig {
    /// P-Flash block base address.
    pub pflash_block_base: u32,
    /// P-Flash total size in bytes.
    pub pflash_total_size: u32,
    /// P-Flash block count.
    pub pflash_block_count: u32,
    /// P-Flash page size in bytes.
    pub pflash_page_size: u32,
    /// P-Flash sector size in bytes.
    pub pflash_sector_size: u32,
    /// FFR configuration.
    pub ffr_config: FlashFfrConfig,
    #[cfg(feature = "mcxa5xx")]
    /// Flash controller parameter configuration.
    pub mode_config: FlashModeConfig,
    #[cfg(feature = "mcxa5xx")]
    /// ROM NBOOT context pointer.
    pub nboot_ctx: *mut u32,
    #[cfg(feature = "mcxa5xx")]
    /// Use AHB read (true) or alternative read path (false), per ROM.
    pub use_ahb_read: bool,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlashProperty {
    PflashSectorSize = 0x00,
    PflashTotalSize = 0x01,
    PflashBlockSize = 0x02,
    PflashBlockCount = 0x03,
    PflashBlockBaseAddr = 0x04,
    PflashPageSize = 0x30,
    PflashSystemFreq = 0x31,
    FfrSectorSize = 0x40,
    FfrTotalSize = 0x41,
    FfrBlockBaseAddr = 0x42,
    FfrPageSize = 0x43,
}

#[repr(C)]
pub(super) struct FlashVtable {
    // Initialize flash driver/config.
    flash_init: unsafe extern "C" fn(config: *mut FlashConfig) -> Status,
    // Erase sector(s). Requires FLASH_API_ERASE_KEY.
    flash_erase_sector:
        unsafe extern "C" fn(config: *mut FlashConfig, start: u32, length_in_bytes: u32, key: u32) -> Status,
    // Program phrase (alignment requirements apply).
    flash_program_phrase:
        unsafe extern "C" fn(config: *mut FlashConfig, start: u32, src: *const u8, length_in_bytes: u32) -> Status,
    // Program page.
    flash_program_page:
        unsafe extern "C" fn(config: *mut FlashConfig, start: u32, src: *const u8, length_in_bytes: u32) -> Status,
    // Verify programmed data.
    flash_verify_program: unsafe extern "C" fn(
        config: *mut FlashConfig,
        start: u32,
        length_in_bytes: u32,
        expected_data: *const u8,
        failed_address: *mut u32,
        failed_data: *mut u32,
    ) -> Status,
    // Verify phrase erase.
    flash_verify_erase_phrase:
        unsafe extern "C" fn(config: *mut FlashConfig, start: u32, length_in_bytes: u32) -> Status,
    // Verify page erase.
    flash_verify_erase_page: unsafe extern "C" fn(config: *mut FlashConfig, start: u32, length_in_bytes: u32) -> Status,
    // Verify sector erase.
    flash_verify_erase_sector:
        unsafe extern "C" fn(config: *mut FlashConfig, start: u32, length_in_bytes: u32) -> Status,
    // Get flash property.
    flash_get_property: unsafe extern "C" fn(config: *mut FlashConfig, which_property: u32, value: *mut u32) -> Status,
    #[cfg(feature = "mcxa5xx")]
    // Verify phrase erase in IFR.
    ifr_verify_erase_phrase: unsafe extern "C" fn(config: *mut FlashConfig, start: u32, length_in_bytes: u32) -> Status,
    #[cfg(feature = "mcxa5xx")]
    // Verify page erase in IFR.
    ifr_verify_erase_page: unsafe extern "C" fn(config: *mut FlashConfig, start: u32, length_in_bytes: u32) -> Status,
    #[cfg(feature = "mcxa5xx")]
    // Verify sector erase in IFR.
    ifr_verify_erase_sector: unsafe extern "C" fn(config: *mut FlashConfig, start: u32, length_in_bytes: u32) -> Status,
    // Read flash into dest.
    flash_read:
        unsafe extern "C" fn(config: *mut FlashConfig, start: u32, dest: *mut u8, length_in_bytes: u32) -> Status,
    // Flash API version.
    version: StandardVersion,
}

pub struct Flash {
    vtable: &'static FlashVtable,
    config: FlashConfig,
}

static TAKEN: AtomicBool = AtomicBool::new(false);

impl Flash {
    pub(super) fn new(vtable: &'static FlashVtable) -> Result<Self, FlashStatus> {
        let mut config = FlashConfig::default();
        if TAKEN
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return Err(FlashStatus::Unavailable);
        }

        Result::from(unsafe { (vtable.flash_init)(&raw mut config) })?;
        Ok(Self { vtable, config })
    }

    pub fn erase_sector(&mut self, start: u32, length_in_bytes: u32) -> Result<(), FlashStatus> {
        Result::from(unsafe {
            (self.vtable.flash_erase_sector)(&raw mut self.config, start, length_in_bytes, 0x6b65666c)
        })
        .map(|()| clear_caches())
    }

    pub fn program_phrase(&mut self, start: u32, src: &[u8]) -> Result<(), FlashStatus> {
        Result::from(unsafe {
            (self.vtable.flash_program_phrase)(&raw mut self.config, start, src.as_ptr(), src.len() as u32)
        })
        .map(|()| clear_caches())
    }

    pub fn program_page(&mut self, start: u32, src: &[u8]) -> Result<(), FlashStatus> {
        Result::from(unsafe {
            (self.vtable.flash_program_page)(&raw mut self.config, start, src.as_ptr(), src.len() as u32)
        })
        .map(|()| clear_caches())
    }

    /// Returns the [FlashStatus::VerifyError] error with the failed address in case the verification encountered unexpected values
    pub fn verify_program(&mut self, start: u32, expected_data: &[u8]) -> Result<(), FlashStatus> {
        let mut failed_address: u32 = 0;
        let mut failed_data: u32 = 0;

        Result::from(unsafe {
            (self.vtable.flash_verify_program)(
                &raw mut self.config,
                start,
                expected_data.len() as u32,
                expected_data.as_ptr(),
                &raw mut failed_address,
                &raw mut failed_data,
            )
        })
        .map_err(|e| {
            if let FlashStatus::CompareError = e {
                FlashStatus::VerifyError(failed_address)
            } else {
                e
            }
        })
    }

    pub fn verify_erase_phrase(&mut self, start: u32, length_in_bytes: u32) -> Result<(), FlashStatus> {
        unsafe { (self.vtable.flash_verify_erase_phrase)(&raw mut self.config, start, length_in_bytes) }.into()
    }

    pub fn verify_erase_page(&mut self, start: u32, length_in_bytes: u32) -> Result<(), FlashStatus> {
        unsafe { (self.vtable.flash_verify_erase_page)(&raw mut self.config, start, length_in_bytes) }.into()
    }

    pub fn verify_erase_sector(&mut self, start: u32, length_in_bytes: u32) -> Result<(), FlashStatus> {
        unsafe { (self.vtable.flash_verify_erase_sector)(&raw mut self.config, start, length_in_bytes) }.into()
    }

    pub fn get_property(&mut self, which_property: FlashProperty) -> Result<u32, FlashStatus> {
        let mut value = 0;
        Result::from(unsafe {
            (self.vtable.flash_get_property)(&raw mut self.config, which_property as u32, &raw mut value)
        })
        .map(|()| value)
    }

    #[cfg(feature = "mcxa5xx")]
    pub fn ifr_verify_erase_phrase(&mut self, start: u32, length_in_bytes: u32) -> Result<(), FlashStatus> {
        unsafe { (self.vtable.ifr_verify_erase_phrase)(&raw mut self.config, start, length_in_bytes) }.into()
    }

    #[cfg(feature = "mcxa5xx")]
    pub fn ifr_verify_erase_page(&mut self, start: u32, length_in_bytes: u32) -> Result<(), FlashStatus> {
        unsafe { (self.vtable.ifr_verify_erase_page)(&raw mut self.config, start, length_in_bytes) }.into()
    }

    #[cfg(feature = "mcxa5xx")]
    pub fn ifr_verify_erase_sector(&mut self, start: u32, length_in_bytes: u32) -> Result<(), FlashStatus> {
        unsafe { (self.vtable.ifr_verify_erase_sector)(&raw mut self.config, start, length_in_bytes) }.into()
    }

    pub fn read(&mut self, start: u32, dest: &mut [u8]) -> Result<(), FlashStatus> {
        unsafe { (self.vtable.flash_read)(&raw mut self.config, start, dest.as_mut_ptr(), dest.len() as u32) }.into()
    }

    pub fn version(&self) -> StandardVersion {
        self.vtable.version
    }
}

impl Drop for Flash {
    fn drop(&mut self) {
        TAKEN.store(false, Ordering::Relaxed);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum FlashStatus {
    InvalidArgument,
    AlignmentError,
    AddressError,
    SizeError,
    CommandFailure,
    UnknownProperty,
    EraseKeyError,
    RegionExecuteOnly,
    CommandNotSupported,
    ReadOnlyProperty,
    InvalidPropertyValue,
    EccError,
    CompareError,
    InvalidWaitStateCycles,
    Unknown(u32),
    /// The driver already exists and cannot be created a second time
    Unavailable,
    VerifyError(u32),
}

impl From<Status> for Result<(), FlashStatus> {
    fn from(raw: Status) -> Self {
        match raw.0 {
            super::KSTATUS_FLASH_SUCCESS => Ok(()),
            super::KSTATUS_FLASH_INVALID_ARGUMENT => Err(FlashStatus::InvalidArgument),
            super::KSTATUS_FLASH_ALIGNMENT_ERROR => Err(FlashStatus::AlignmentError),
            super::KSTATUS_FLASH_ADDRESS_ERROR => Err(FlashStatus::AddressError),
            super::KSTATUS_FLASH_SIZE_ERROR => Err(FlashStatus::SizeError),
            super::KSTATUS_FLASH_COMMAND_FAILURE => Err(FlashStatus::CommandFailure),
            super::KSTATUS_FLASH_UNKNOWN_PROPERTY => Err(FlashStatus::UnknownProperty),
            super::KSTATUS_FLASH_ERASE_KEY_ERROR => Err(FlashStatus::EraseKeyError),
            super::KSTATUS_FLASH_REGION_EXECUTE_ONLY => Err(FlashStatus::RegionExecuteOnly),
            super::KSTATUS_FLASH_COMMAND_NOT_SUPPORTED => Err(FlashStatus::CommandNotSupported),
            super::KSTATUS_FLASH_READ_ONLY_PROPERTY => Err(FlashStatus::ReadOnlyProperty),
            super::KSTATUS_FLASH_INVALID_PROPERTY_VALUE => Err(FlashStatus::InvalidPropertyValue),
            super::KSTATUS_FLASH_ECC_ERROR => Err(FlashStatus::EccError),
            super::KSTATUS_FLASH_COMPARE_ERROR => Err(FlashStatus::CompareError),
            super::KSTATUS_FLASH_INVALID_WAIT_STATE_CYCLES => Err(FlashStatus::InvalidWaitStateCycles),
            other => Err(FlashStatus::Unknown(other)),
        }
    }
}

/// Clear flash and data speculation buffers by toggling the disable bits
/// in SYSCON->NVM_CTRL, matching the C `speculation_buffer_clear()`.
fn speculation_buffer_clear() {
    let nvm = nxp_pac::SYSCON.nvm_ctrl();
    let val = nvm.read();

    // Only proceed if MBECC error reporting is enabled for both inst and data
    if val.dis_mbecc_err_inst() == DisMbeccErrInst::Enable && val.dis_mbecc_err_data() == DisMbeccErrData::Enable {
        // Toggle flash speculation disable
        if val.dis_flash_spec() == DisFlashSpec::Enable {
            nvm.modify(|w| w.set_dis_flash_spec(DisFlashSpec::Disable));
            nvm.modify(|w| w.set_dis_flash_spec(DisFlashSpec::Enable));
        }
        // Toggle data speculation disable
        if nvm.read().dis_data_spec() == DisDataSpec::Enable {
            nvm.modify(|w| w.set_dis_data_spec(DisDataSpec::Disable));
            nvm.modify(|w| w.set_dis_data_spec(DisDataSpec::Enable));
        }
    }
}

/// Clear LPCAC by setting the CLR_LPCAC bit in SYSCON->LPCAC_CTRL,
/// matching the C `lpcac_clear()`.
fn lpcac_clear() {
    let lpcac = nxp_pac::SYSCON.lpcac_ctrl();
    if lpcac.read().dis_lpcac() == DisLpcac::Enable {
        lpcac.modify(|w| w.set_clr_lpcac(ClrLpcac::Disable));
    }
}

/// Combined cache clearing: speculation buffers + LPCAC.
fn clear_caches() {
    speculation_buffer_clear();
    lpcac_clear();
}

impl ErrorType for Flash {
    type Error = FlashStatus;
}

impl ReadNorFlash for Flash {
    const READ_SIZE: usize = 1;

    fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        self.read(offset, bytes)
    }

    fn capacity(&self) -> usize {
        FLASH_SIZE
    }
}

impl NorFlash for Flash {
    const WRITE_SIZE: usize = PHRASE_SIZE;
    const ERASE_SIZE: usize = SECTOR_SIZE;

    fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
        if to < from || to as usize > FLASH_SIZE {
            return Err(FlashStatus::AddressError);
        }
        if !(from as usize).is_multiple_of(Self::ERASE_SIZE) || !(to as usize).is_multiple_of(Self::ERASE_SIZE) {
            return Err(FlashStatus::AlignmentError);
        }

        // Erase one sector at a time
        for sector_addr in (from..to).step_by(Self::ERASE_SIZE) {
            self.erase_sector(FLASH_BASE + sector_addr, SECTOR_SIZE as u32)?;
        }
        Ok(())
    }

    fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        if offset as usize + bytes.len() > FLASH_SIZE {
            return Err(FlashStatus::AddressError);
        }
        if !(offset as usize).is_multiple_of(Self::WRITE_SIZE) || !bytes.len().is_multiple_of(Self::WRITE_SIZE) {
            return Err(FlashStatus::AlignmentError);
        }

        // Program one phrase at a time (16 bytes — the smallest write unit)
        for (i, chunk) in bytes.chunks(PHRASE_SIZE).enumerate() {
            let addr = FLASH_BASE + offset + (i * PHRASE_SIZE) as u32;
            self.program_phrase(addr, chunk)?;
        }
        Ok(())
    }
}

/// Base address of the internal program flash.
pub const FLASH_BASE: u32 = 0x0000_0000;

#[cfg(feature = "mcxa2xx")]
/// Total size of the internal program flash in bytes (1 MB).
pub const FLASH_SIZE: usize = 0x10_0000;
#[cfg(feature = "mcxa5xx")]
/// Total size of the internal program flash in bytes (2 MB).
pub const FLASH_SIZE: usize = 0x20_0000;

/// Sector size in bytes (8 KB) — erase granularity.
pub const SECTOR_SIZE: usize = 0x2000;

/// Page size in bytes (128) — program-page granularity.
pub const PAGE_SIZE: usize = 128;

/// Phrase size in bytes (16) — minimum program unit.
pub const PHRASE_SIZE: usize = 16;

impl NorFlashError for FlashStatus {
    fn kind(&self) -> NorFlashErrorKind {
        match self {
            Self::AddressError => NorFlashErrorKind::OutOfBounds,
            Self::AlignmentError => NorFlashErrorKind::NotAligned,
            _ => NorFlashErrorKind::Other,
        }
    }
}

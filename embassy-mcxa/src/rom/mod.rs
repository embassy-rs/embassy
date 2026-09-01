use core::ffi::c_char;

mod flash;
#[cfg(feature = "mcxa5xx")]
mod flexspi_nor;
#[cfg(feature = "mcxa5xx")]
mod kb;
// #[cfg(feature = "mcxa5xx")]
// mod nboot;
// #[cfg(feature = "mcxa5xx")]
// mod spi_flash;

pub use flash::*;
#[cfg(feature = "mcxa5xx")]
pub use flexspi_nor::*;
#[cfg(feature = "mcxa5xx")]
pub use kb::*;
// #[cfg(feature = "mcxa5xx")]
// pub use nboot::*;
// #[cfg(feature = "mcxa5xx")]
// pub use spi_flash::*;

#[repr(transparent)]
struct Status(u32);
pub type NbootBool = u32;
pub type NbootStatusProtected = u64;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct StandardVersion {
    pub bugfix: u8,
    pub minor: u8,
    pub major: u8,
    pub name: u8,
}

pub fn get() -> &'static RomApi {
    #[cfg(feature = "mcxa2xx")]
    /// Base address of the ROM API bootloader tree for MCXA276.
    const ROM_API_BASE: u32 = 0x0300_5FE0;
    #[cfg(feature = "mcxa5xx")]
    /// Base address of the ROM API bootloader tree for MCXA577.
    const ROM_API_BASE: u32 = 0x1303_D800;

    let ptr = ROM_API_BASE as *const RomApi;

    unsafe { &*ptr }
}

#[repr(C)]
pub struct RomApi {
    // NXP usage: uint32_t arg = ...; g_bootloaderTree->runBootloader(&arg);
    // The ROM API takes a pointer to the argument word (NULL is allowed for default behavior).
    run_bootloader: unsafe extern "C" fn(arg: *const u32),
    // Flash driver interface table.
    flash: *const FlashVtable,
    #[cfg(feature = "mcxa2xx")]
    jump: unsafe extern "C" fn(image_base: u32),
    #[cfg(feature = "mcxa5xx")]
    kb: *const KbVtable,
    // #[cfg(feature = "mcxa5xx")]
    // nboot_api: *const NbootVtable,
    #[cfg(feature = "mcxa5xx")]
    flex_spi_nor: *const FlexspiNorVtable,
    // #[cfg(feature = "mcxa5xx")]
    // spi_flash_api: *const SpiFlashVtable,
    #[cfg(feature = "mcxa5xx")]
    version: StandardVersion,
    #[cfg(feature = "mcxa5xx")]
    copyright: *const c_char,
}

impl RomApi {
    #[allow(clippy::not_unsafe_ptr_arg_deref, reason = "ROM will check the pointer for validity")]
    pub fn run_bootloader(&self, arg: *const u32) {
        unsafe { (self.run_bootloader)(arg) }
    }

    pub fn flash(&self) -> Result<Flash, FlashStatus> {
        Flash::new(unsafe { &*self.flash })
    }

    #[cfg(feature = "mcxa5xx")]
    pub fn kb(&self, options: KbOptions) -> Result<Kb, KbStatus> {
        Kb::new(unsafe { &*self.kb }, options)
    }

    // #[cfg(feature = "mcxa5xx")]
    // pub fn nboot(&self) -> &'static NbootDriver {
    //     unsafe { &*self.nboot_api }
    // }

    #[cfg(feature = "mcxa5xx")]
    pub fn flex_spi(&self, instance: u32, config: FlexspiNorConfig) -> Result<FlexspiNor, FlexspiStatus> {
        FlexspiNor::new(unsafe { &*self.flex_spi_nor }, instance, config)
    }

    // #[cfg(feature = "mcxa5xx")]
    // pub fn spi_flash(&self) -> &'static SpiFlashDriver {
    //     unsafe { &*self.spi_flash_api }
    // }

    #[cfg(feature = "mcxa5xx")]
    pub fn version(&self) -> StandardVersion {
        self.version
    }

    #[cfg(feature = "mcxa5xx")]
    pub fn copyright(&self) -> *const c_char {
        self.copyright
    }
}

// runBootloader API fields (Table 31)
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RunBootTag {
    EnterBoot = 0xEB << 24,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RunBootMode {
    PrimaryMasterBoot = 0x0 << 20,
    IspBoot = 0x1 << 20,
    ProvFwMode = 0x2 << 20,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RunBootIspInterface {
    AutoDetection = 0x0 << 16,
    Uart = 0x1 << 16,
    Spi = 0x2 << 16,
    I2c = 0x8 << 16,
    UsbHid = 0x10 << 16,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RunBootMasterFlashBootOption {
    InternalFlash = 0x0 << 16,
    FlexspiFlash = 0x2 << 16,
    OneBitSpiNorFlash = 0x3 << 16,
    AutoDetection = 0x1F << 16,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RunBootInterfaceInstance {
    FlexspiPortA = 0x0 << 12,
    FlexspiPortB = 0x1 << 12,
    FlexspiPortAAndB = 0x2 << 12,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RunBootImageIndex {
    Image0 = 0x0 << 8,
    Image1 = 0x1 << 8,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RunBootRecoveryBootCfg1 {
    SpiNorBaudRate0 = 0x0 << 6,
    SpiNorBaudRate1 = 0x1 << 6,
    SpiNorBaudRate2 = 0x2 << 6,
    SpiNorBaudRate3 = 0x3 << 6,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RunBootRecoveryBootCfg0 {
    SpiNorChipSelect0 = 0x0 << 4,
    SpiNorChipSelect1 = 0x1 << 4,
    SpiNorChipSelect2 = 0x2 << 4,
    SpiNorChipSelect3 = 0x3 << 4,
}

const KSTATUS_SUCCESS: u32 = 0;
const KSTATUS_FAIL: u32 = 1;
const KSTATUS_INVALID_ARGUMENT: u32 = 4;

const KSTATUS_FLASH_SUCCESS: u32 = 0;
const KSTATUS_FLASH_INVALID_ARGUMENT: u32 = 4;
const KSTATUS_FLASH_ALIGNMENT_ERROR: u32 = 101;
const KSTATUS_FLASH_ADDRESS_ERROR: u32 = 102;
const KSTATUS_FLASH_SIZE_ERROR: u32 = 100;
const KSTATUS_FLASH_COMMAND_FAILURE: u32 = 105;
const KSTATUS_FLASH_UNKNOWN_PROPERTY: u32 = 106;
const KSTATUS_FLASH_ERASE_KEY_ERROR: u32 = 107;
const KSTATUS_FLASH_REGION_EXECUTE_ONLY: u32 = 108;
const KSTATUS_FLASH_COMMAND_NOT_SUPPORTED: u32 = 111;
const KSTATUS_FLASH_READ_ONLY_PROPERTY: u32 = 112;
const KSTATUS_FLASH_INVALID_PROPERTY_VALUE: u32 = 113;
const KSTATUS_FLASH_ECC_ERROR: u32 = 116;
const KSTATUS_FLASH_COMPARE_ERROR: u32 = 117;
const KSTATUS_FLASH_INVALID_WAIT_STATE_CYCLES: u32 = 119;

// SPI flash driver status codes
const KSTATUS_SPIFLASH_SUCCESS: u32 = KSTATUS_SUCCESS;
const KSTATUS_SPIFLASH_FAIL: u32 = KSTATUS_FAIL;

// FlexSPI flash driver status codes
const KSTATUS_FLEXSPI_SUCCESS: u32 = KSTATUS_SUCCESS;
const KSTATUS_FLEXSPI_FAIL: u32 = KSTATUS_FAIL;
const KSTATUS_FLEXSPI_INVALID_ARGUMENT: u32 = KSTATUS_INVALID_ARGUMENT;
const KSTATUS_FLEXSPI_SEQUENCE_EXECUTION_TIMEOUT: u32 = 6000;
const KSTATUS_FLEXSPI_INVALID_SEQUENCE: u32 = 6001;
const KSTATUS_FLEXSPI_DEVICE_TIMEOUT: u32 = 6002;

const KSTATUS_FLEXSPINOR_PROGRAM_FAIL: u32 = 20100;
const KSTATUS_FLEXSPINOR_ERASE_SECTOR_FAIL: u32 = 20101;
const KSTATUS_FLEXSPINOR_ERASE_ALL_FAIL: u32 = 20102;
const KSTATUS_FLEXSPINOR_WAIT_TIMEOUT: u32 = 20103;
const KSTATUS_FLEXSPINOR_WRITE_ALIGNMENT_ERROR: u32 = 20105;
const KSTATUS_FLEXSPINOR_COMMAND_FAILURE: u32 = 20106;
const KSTATUS_FLEXSPINOR_SFDP_NOT_FOUND: u32 = 20107;
const KSTATUS_FLEXSPINOR_UNSUPPORTED_SFDP_VERSION: u32 = 20108;
const KSTATUS_FLEXSPINOR_FLASH_NOT_FOUND: u32 = 20109;
const KSTATUS_FLEXSPINOR_DTR_READ_DUMMY_PROBE_FAILED: u32 = 20110;

const KSTATUS_NBOOT_SUCCESS: u64 = 0x5A5A_5A5A;
const KSTATUS_NBOOT_FAIL: u64 = 0x5A5A_A5A5;
const KSTATUS_NBOOT_INVALID_ARGUMENT: u64 = 0x5A5A_A5F0;

// NBOOT API status codes (MCXA ROM, Table 46 / 9.2.5.11)
// These are returned by APIs such as `nboot_mem_crypt_range_checker`.
const KNBOOT_OPERATION_ALLOWED: u64 = 0x3C5A_33CC;
const KNBOOT_OPERATION_DISALLOWED: u64 = 0x5AA5_CC33;
const KSTATUS_NBOOT_KEY_NOT_AVAILABLE: u64 = 0x5A5A_A5E6;

const KSTATUS_ROMLDR_DATA_UNDERRUN: u32 = 10109;
const KSTATUS_ROMLDR_JUMP_RETURNED: u32 = 10110;
const KSTATUS_ROMLDR_ROLLBACK_BLOCKED: u32 = 10115;
const KSTATUS_ROMLDR_PENDING_JUMP_COMMAND: u32 = 10119;

// ROM API status codes
const KSTATUS_ROM_API_BUFFER_SIZE_NOT_ENOUGH: u32 = 10802;
const KSTATUS_ROM_API_INVALID_BUFFER: u32 = 10803;

// KBoot (KB) status codes (Table 35); KB reuses generic/ROM loader/ROM API status space; these aliases make callsites clearer.
const KSTATUS_KB_SUCCESS: u32 = KSTATUS_SUCCESS;
const KSTATUS_KB_FAIL: u32 = KSTATUS_FAIL;
const KSTATUS_KB_INVALID_ARGUMENT: u32 = KSTATUS_INVALID_ARGUMENT;

const KSTATUS_KB_ROMLDR_DATA_UNDERRUN: u32 = KSTATUS_ROMLDR_DATA_UNDERRUN;
const KSTATUS_KB_ROMLDR_JUMP_RETURNED: u32 = KSTATUS_ROMLDR_JUMP_RETURNED;
const KSTATUS_KB_ROMLDR_ROLLBACK_BLOCKED: u32 = KSTATUS_ROMLDR_ROLLBACK_BLOCKED;
const KSTATUS_KB_ROMLDR_PENDING_JUMP_COMMAND: u32 = KSTATUS_ROMLDR_PENDING_JUMP_COMMAND;

const KSTATUS_KB_BUFFER_SIZE_NOT_ENOUGH: u32 = KSTATUS_ROM_API_BUFFER_SIZE_NOT_ENOUGH;
const KSTATUS_KB_INVALID_BUFFER: u32 = KSTATUS_ROM_API_INVALID_BUFFER;

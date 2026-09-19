mod flash;
#[cfg(feature = "mcxa5xx")]
mod flexspi_nor;
#[cfg(feature = "mcxa5xx")]
mod kb;
#[cfg(feature = "mcxa5xx")]
mod nboot;
#[cfg(feature = "mcxa5xx")]
mod spi_flash;

pub use flash::*;
#[cfg(feature = "mcxa5xx")]
pub use flexspi_nor::*;
#[cfg(feature = "mcxa5xx")]
pub use kb::*;
#[cfg(feature = "mcxa5xx")]
pub use nboot::*;
#[cfg(feature = "mcxa5xx")]
pub use spi_flash::*;

#[repr(transparent)]
struct Status(u32);

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
    #[cfg(feature = "mcxa5xx")]
    nboot: *const NbootVtable,
    #[cfg(feature = "mcxa5xx")]
    flex_spi_nor: *const FlexspiNorVtable,
    #[cfg(feature = "mcxa5xx")]
    spi_flash: *const SpiFlashVtable,
    #[cfg(feature = "mcxa5xx")]
    version: StandardVersion,
    #[cfg(feature = "mcxa5xx")]
    copyright: *const core::ffi::c_char,
}

impl RomApi {
    pub fn run_bootloader(
        &self,
        mode: RunBootMode,
        isp_interface: RunBootIspInterface,
        master_flash_boot_option: RunBootMasterFlashBootOption,
        interface_instance: RunBootInterfaceInstance,
        image_index: RunBootImageIndex,
        recovery_boot_cfg1: RunBootRecoveryBootCfg1,
        recovery_boot_cfg0: RunBootRecoveryBootCfg0,
    ) {
        let arg = mode as u32
            | isp_interface as u32
            | master_flash_boot_option as u32
            | interface_instance as u32
            | image_index as u32
            | recovery_boot_cfg1 as u32
            | recovery_boot_cfg0 as u32;
        unsafe { (self.run_bootloader)(&raw const arg) }
    }

    pub fn flash(&self) -> Result<Flash, FlashError> {
        Flash::new(unsafe { &*self.flash })
    }

    #[cfg(feature = "mcxa5xx")]
    pub fn kb(&self, options: KbOptions) -> Result<Kb, KbError> {
        Kb::new(unsafe { &*self.kb }, options)
    }

    #[cfg(feature = "mcxa5xx")]
    pub fn nboot(&self) -> Result<Nboot, NbootError> {
        Nboot::new(unsafe { &*self.nboot })
    }

    #[cfg(feature = "mcxa5xx")]
    pub fn flex_spi(&self, instance: u32, config: FlexspiNorConfig) -> Result<FlexspiNor, FlexspiError> {
        FlexspiNor::new(unsafe { &*self.flex_spi_nor }, instance, config)
    }

    #[cfg(feature = "mcxa5xx")]
    pub fn spi_flash(&self, config: SpiFlashConfig) -> Result<SpiFlash, SpiFlashError> {
        SpiFlash::new(unsafe { &*self.spi_flash }, config)
    }

    #[cfg(feature = "mcxa5xx")]
    pub fn version(&self) -> StandardVersion {
        self.version
    }

    #[cfg(feature = "mcxa5xx")]
    pub fn copyright(&self) -> *const core::ffi::c_char {
        self.copyright
    }
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunBootTag {
    EnterBoot = 0xEB << 24,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunBootMode {
    PrimaryMasterBoot = 0x0 << 20,
    IspBoot = 0x1 << 20,
    ProvFwMode = 0x2 << 20,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunBootIspInterface {
    AutoDetection = 0x0 << 16,
    Uart = 0x1 << 16,
    Spi = 0x2 << 16,
    I2c = 0x8 << 16,
    UsbHid = 0x10 << 16,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunBootMasterFlashBootOption {
    InternalFlash = 0x0 << 16,
    FlexspiFlash = 0x2 << 16,
    OneBitSpiNorFlash = 0x3 << 16,
    AutoDetection = 0x1F << 16,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunBootInterfaceInstance {
    FlexspiPortA = 0x0 << 12,
    FlexspiPortB = 0x1 << 12,
    FlexspiPortAAndB = 0x2 << 12,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunBootImageIndex {
    Image0 = 0x0 << 8,
    Image1 = 0x1 << 8,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunBootRecoveryBootCfg1 {
    SpiNorBaudRate0 = 0x0 << 6,
    SpiNorBaudRate1 = 0x1 << 6,
    SpiNorBaudRate2 = 0x2 << 6,
    SpiNorBaudRate3 = 0x3 << 6,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunBootRecoveryBootCfg0 {
    SpiNorChipSelect0 = 0x0 << 4,
    SpiNorChipSelect1 = 0x1 << 4,
    SpiNorChipSelect2 = 0x2 << 4,
    SpiNorChipSelect3 = 0x3 << 4,
}

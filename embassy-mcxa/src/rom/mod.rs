//! On-chip boot ROM services for MCXA devices.
//!
//! Use [`get`] to access the chip family's [`RomApi`] table and construct a
//! driver for the required service. Both supported families provide internal
//! flash operations. MCXA5xx additionally provides FlexSPI NOR, SPI NOR/EEPROM,
//! KB image-loading and NBOOT security services.
//!
//! ROM calls are synchronous. Erase, program and image-loading operations can
//! change persistent contents; observe each service's buffer, alignment,
//! ownership and configuration requirements before invoking it.

mod flash;
#[cfg(feature = "_mcxa5xx")]
mod flexspi_nor;
#[cfg(feature = "_mcxa5xx")]
mod kb;
#[cfg(feature = "_mcxa5xx")]
mod nboot;
#[cfg(feature = "_mcxa5xx")]
mod spi_flash;

pub use flash::*;
#[cfg(feature = "_mcxa5xx")]
pub use flexspi_nor::*;
#[cfg(feature = "_mcxa5xx")]
pub use kb::*;
#[cfg(feature = "_mcxa5xx")]
pub use nboot::*;
#[cfg(feature = "_mcxa5xx")]
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
    #[cfg(feature = "_mcxa2xx")]
    /// Base address of the ROM API bootloader tree for MCXA276.
    const ROM_API_BASE: u32 = 0x0300_5FE0;
    #[cfg(feature = "_mcxa5xx")]
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
    #[cfg(feature = "_mcxa2xx")]
    jump: unsafe extern "C" fn(image_base: u32),
    #[cfg(feature = "_mcxa5xx")]
    kb: *const KbVtable,
    #[cfg(feature = "_mcxa5xx")]
    nboot: *const NbootVtable,
    #[cfg(feature = "_mcxa5xx")]
    flex_spi_nor: *const FlexspiNorVtable,
    #[cfg(feature = "_mcxa5xx")]
    spi_flash: *const SpiFlashVtable,
    #[cfg(feature = "_mcxa5xx")]
    version: StandardVersion,
    #[cfg(feature = "_mcxa5xx")]
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

    #[cfg(feature = "_mcxa5xx")]
    pub fn kb(&self, options: KbOptions) -> Result<Kb, KbError> {
        Kb::new(unsafe { &*self.kb }, options)
    }

    #[cfg(feature = "_mcxa5xx")]
    pub fn nboot(&self) -> Result<Nboot, NbootError> {
        Nboot::new(unsafe { &*self.nboot })
    }

    /// Initialize the ROM FlexSPI NOR driver from a complete configuration.
    ///
    /// The controller and selected pins are borrowed until the handle is dropped,
    /// excluding native FlexSPI and GPIO drivers using those same tokens. ROM
    /// always uses controller instance 0. An initialization error releases the
    /// ROM claim and borrows, but does not roll back controller or flash state.
    /// With `defmt`, failed ROM
    /// initialization phases are logged at debug level with their raw status.
    ///
    /// `config` must describe the attached device and the borrowed pins.
    /// The single-port CS0/quad footprint of [`FlexspiNorPins`] is checked before
    /// ROM entry; unsupported geometry returns [`FlexspiError::InvalidArgument`].
    /// Required upstream clocks must be running; invalid hardware settings may fail or
    /// stall ROM operations. Unsafe code creating mapped-flash references or XIP
    /// code remains responsible for ensuring that those objects are not
    /// invalidated by later ROM operations, even without concurrent dereferences.
    #[cfg(feature = "_mcxa5xx")]
    pub fn flex_spi<'d>(
        &self,
        peri: crate::Peri<'d, crate::peripherals::FLEXSPI0>,
        pins: FlexspiNorPins<'d>,
        config: FlexspiNorConfig,
    ) -> Result<FlexspiNor<'d>, FlexspiError> {
        FlexspiNor::new(unsafe { &*self.flex_spi_nor }, peri, pins, config)
    }

    /// Probe the serial NOR with `option`, run `custom_fcb` on the ROM-confirmed
    /// config, then init.
    ///
    /// # Destructive probing
    ///
    /// DDR/DTR dummy-cycle auto-probing can write test patterns at external-flash
    /// offset `0x200` (MCXA5xx RM Rev. 1, section 7.4.2). Arrange expendable flash
    /// space before calling: probing happens **before** `custom_fcb`, and an
    /// error does not guarantee unchanged contents. The write offset does not
    /// establish the complete erase/program footprint; reserving exactly 4 KiB
    /// is not a universal preservation guarantee.
    ///
    /// The controller and selected pins are borrowed for the handle's lifetime.
    /// ROM always uses instance 0; select port A/B through `option`, matching
    /// the borrowed pins. This variant uses the existing upstream clock for the
    /// probe. Prepare it before calling, or use
    /// [`Self::flex_spi_with_options_and_clock`] for selection inside the claim.
    /// Clock/probe/init errors release the claim, not hardware/flash state.
    /// With `defmt`, failed ROM phases are logged at debug level.
    /// Options selecting an unborrowed port, alternate pin group, or a wider
    /// device are rejected before probing. The final FCB is checked after
    /// `custom_fcb` and before init against the footprint of [`FlexspiNorPins`].
    ///
    /// The probe's upstream clock must already be prepared and running.
    /// `custom_fcb` should preserve the probed device parameters except for the
    /// intended board overrides; probing does not validate arbitrary later
    /// customization. The mapped-flash/XIP caveat of [`Self::flex_spi`] applies.
    #[cfg(feature = "_mcxa5xx")]
    pub fn flex_spi_with_options<'d>(
        &self,
        peri: crate::Peri<'d, crate::peripherals::FLEXSPI0>,
        pins: FlexspiNorPins<'d>,
        option: SerialNorConfigOption,
        custom_fcb: impl FnOnce(&mut FlexspiNorConfig),
    ) -> Result<FlexspiNor<'d>, FlexspiError> {
        FlexspiNor::new_with_options(unsafe { &*self.flex_spi_nor }, peri, pins, option, custom_fcb)
    }

    /// Select a prepared clock source **before** probing, then customize and init.
    ///
    /// This follows the pre-probe selection ordering in MCXA5xx RM Rev. 1,
    /// section 5.8.12, using the device-specific encodings in
    /// [`FlexspiClockSource`] rather than that section's incompatible Table 60.
    /// Selection only changes the clock mux; it does not start
    /// the source. The selection, probe, customization and init share one ROM
    /// claim, released on any error. `NoClock` is rejected
    /// before entering ROM. Failed ROM phases are logged at debug level with
    /// `defmt` enabled.
    ///
    /// # Destructive probing
    ///
    /// DDR/DTR auto-probing can write flash at offset `0x200` **before** the
    /// customization closure. Arrange expendable probe space before calling;
    /// neither an error nor a fixed 4 KiB reservation guarantees preservation.
    /// See [`Self::flex_spi_with_options`] for the full probing contract.
    ///
    /// The controller/pin borrowing and mapped-flash/XIP caveats of
    /// [`Self::flex_spi_with_options`] apply. Additionally, `clock_source` must
    /// already be running at a device-supported frequency, and the controller
    /// must be idle and prepared for the ROM clock-selector API before this
    /// call. Selecting a source is not a substitute for initializing its PLL/FRO
    /// or preparing the controller's clocks.
    #[cfg(feature = "_mcxa5xx")]
    pub fn flex_spi_with_options_and_clock<'d>(
        &self,
        peri: crate::Peri<'d, crate::peripherals::FLEXSPI0>,
        pins: FlexspiNorPins<'d>,
        option: SerialNorConfigOption,
        clock_source: FlexspiClockSource,
        custom_fcb: impl FnOnce(&mut FlexspiNorConfig),
    ) -> Result<FlexspiNor<'d>, FlexspiError> {
        FlexspiNor::new_with_options_and_clock(
            unsafe { &*self.flex_spi_nor },
            peri,
            pins,
            option,
            Some(clock_source),
            custom_fcb,
        )
    }

    #[cfg(feature = "_mcxa5xx")]
    pub fn spi_flash(&self, config: SpiFlashConfig) -> Result<SpiFlash, SpiFlashError> {
        SpiFlash::new(unsafe { &*self.spi_flash }, config)
    }

    #[cfg(feature = "_mcxa5xx")]
    pub fn version(&self) -> StandardVersion {
        self.version
    }

    #[cfg(feature = "_mcxa5xx")]
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

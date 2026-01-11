//! SysInfo
//!
//! As this is a read-only peripheral, this driver is designed to be free-standing for ease of use.
//! All functions can be called directly on [`SysInfo`] without needing to instantiate a singleton.

/// Processor boot mode.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum BootMode {
    /// Processor-internal BOOTROM as pre-initialized ROM.
    Bootloader,
    /// User-defined address.
    CustomAddress,
    /// Processor-internal IMEM as pre-initialized ROM.
    ImemImage,
    /// Unrecognized boot mode.
    Unknown,
}

impl From<u8> for BootMode {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Bootloader,
            1 => Self::CustomAddress,
            2 => Self::ImemImage,
            _ => Self::Unknown,
        }
    }
}

/// SoC configuration.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct SocConfig(u32);

impl SocConfig {
    #[inline(always)]
    fn is_supported(&self, i: u32) -> bool {
        self.0 & (1 << i) != 0
    }

    /// Returns raw 32-bit SoC config.
    pub fn raw(&self) -> u32 {
        self.0
    }

    /// Returns true if processor-internal bootloader is implemented.
    pub fn has_bootloader(&self) -> bool {
        self.is_supported(0)
    }

    /// Returns true if external bus interface (XBUS) is implemented.
    pub fn has_xbus(&self) -> bool {
        self.is_supported(1)
    }

    /// Returns true if processor-internal IMEM is implemented.
    pub fn has_imem(&self) -> bool {
        self.is_supported(2)
    }

    /// Returns true if processor-internal DMEM is implemented.
    pub fn has_dmem(&self) -> bool {
        self.is_supported(3)
    }

    /// Returns true if on-chip debugger is implemented.
    pub fn has_ocd(&self) -> bool {
        self.is_supported(4)
    }

    /// Returns true if processor-internal instruction cache is implemented.
    pub fn has_icache(&self) -> bool {
        self.is_supported(5)
    }

    /// Returns true if processor-internal data cache is implemented.
    pub fn has_dcache(&self) -> bool {
        self.is_supported(6)
    }

    /// Returns true if on-chip debugger authentication is implemented.
    pub fn has_ocd_auth(&self) -> bool {
        self.is_supported(11)
    }

    /// Returns true if processor-internal IMEM is implemented as pre-initialized ROM.
    pub fn has_imem_as_rom(&self) -> bool {
        self.is_supported(12)
    }

    /// Returns true if TWD is implemented.
    pub fn has_twd(&self) -> bool {
        self.is_supported(13)
    }

    /// Returns true if DMA is implemented.
    pub fn has_dma(&self) -> bool {
        self.is_supported(14)
    }

    /// Returns true if GPIO is implemented.
    pub fn has_gpio(&self) -> bool {
        self.is_supported(15)
    }

    /// Returns true if CLINT is implemented.
    pub fn has_clint(&self) -> bool {
        self.is_supported(16)
    }

    /// Returns true if UART0 is implemented.
    pub fn has_uart0(&self) -> bool {
        self.is_supported(17)
    }

    /// Returns true if SPI is implemented.
    pub fn has_spi(&self) -> bool {
        self.is_supported(18)
    }

    /// Returns true if TWI is implemented.
    pub fn has_twi(&self) -> bool {
        self.is_supported(19)
    }

    /// Returns true if PWM is implemented.
    pub fn has_pwm(&self) -> bool {
        self.is_supported(20)
    }

    /// Returns true if WDT is implemented.
    pub fn has_wdt(&self) -> bool {
        self.is_supported(21)
    }

    /// Returns true if CFS is implemented.
    pub fn has_cfs(&self) -> bool {
        self.is_supported(22)
    }

    /// Returns true if TRNG is implemented.
    pub fn has_trng(&self) -> bool {
        self.is_supported(23)
    }

    /// Returns true if SDI is implemented.
    pub fn has_sdi(&self) -> bool {
        self.is_supported(24)
    }

    /// Returns true if UART1 is implemented.
    pub fn has_uart1(&self) -> bool {
        self.is_supported(25)
    }

    /// Returns true if NEOLED is implemented.
    pub fn has_neoled(&self) -> bool {
        self.is_supported(26)
    }

    /// Returns true if TRACER is implemented.
    pub fn has_tracer(&self) -> bool {
        self.is_supported(27)
    }

    /// Returns true if GPTMR is implemented.
    pub fn has_gptmr(&self) -> bool {
        self.is_supported(28)
    }

    /// Returns true if SLINK is implemented.
    pub fn has_slink(&self) -> bool {
        self.is_supported(29)
    }

    /// Returns true if ONEWIRE is implemented.
    pub fn has_onewire(&self) -> bool {
        self.is_supported(30)
    }

    /// Returns true if NEORV32 is being simulated.
    pub fn is_simulation(&self) -> bool {
        self.is_supported(31)
    }
}

/// SysInfo driver
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct SysInfo;

impl SysInfo {
    /// Returns the main CPU clock frequency (Hz).
    pub fn clock_freq() -> u32 {
        reg().clk().read().bits()
    }

    /// Returns the IMEM size in bytes.
    pub fn imem_size() -> u32 {
        1 << reg().mem().read().sysinfo_misc_imem().bits()
    }

    /// Returns the DMEM size in bytes.
    pub fn dmem_size() -> u32 {
        1 << reg().mem().read().sysinfo_misc_dmem().bits()
    }

    /// Returns the number of harts (cores).
    pub fn num_harts() -> u8 {
        reg().mem().read().sysinfo_misc_hart().bits()
    }

    /// Returns the boot mode configuration.
    pub fn boot_mode() -> BootMode {
        let raw = reg().mem().read().sysinfo_misc_boot().bits();
        BootMode::from(raw)
    }

    /// Returns the number of internal bus timeout cycles.
    pub fn bus_itmo_cycles() -> u32 {
        1 << reg().mem().read().sysinfo_misc_itmo().bits()
    }

    /// Returns the number of external bus timeout cycles.
    pub fn bus_etmo_cycles() -> u32 {
        1 << reg().mem().read().sysinfo_misc_etmo().bits()
    }

    /// Returns the SoC config.
    ///
    /// Additional methods can be called to check if SoC features are implemented.
    pub fn soc_config() -> SocConfig {
        SocConfig(reg().soc().read().bits())
    }
}

fn reg() -> &'static crate::pac::sysinfo::RegisterBlock {
    // SAFETY: We only use this pointer internally and do so safely
    unsafe { &*crate::pac::Sysinfo::ptr() }
}

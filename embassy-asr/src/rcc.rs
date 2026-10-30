//! Reset and clock control for the ASR6601.
//!
//! The clock tree is deliberately configured before it is published through
//! [`clocks`]. Peripheral drivers may share RCC through the crate-private
//! helpers at the end of this module; their register updates are serialized by
//! a critical section.

#![allow(dead_code)]

use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use crate::{Peri, pac, peripherals};

pub(crate) const RCO48M_HZ: u32 = 48_000_000;
pub(crate) const RCO4M_HZ: u32 = 3_600_000;
pub(crate) const XO24M_HZ: u32 = 24_000_000;
pub(crate) const XO32M_HZ: u32 = 32_000_000;
pub(crate) const RCO32K_HZ: u32 = 32_000;
pub(crate) const XO32K_HZ: u32 = 32_768;

// NOTE: the `*_bat` LORAC accessors below address the same bits the vendor
// SDK names XO32M_ENABLE (bit 2), NRESET (5), POR (7), TCXO (0+1) and the
// READY status (SR bit 1) — positions verified against the PAC accessors:
// `clk_32m_en_bat`, `nreset_bat`, `por_bat`, `pwrtcxo_en_bat` +
// `tcxo_en_bat`, `clk_32m_rdy_bat_sr`.

const ASYNC_RESET_DELAY_US: u64 = 92;

static CLOCKS_INITIALIZED: AtomicBool = AtomicBool::new(false);
static SYSCLK_HZ: AtomicU32 = AtomicU32::new(0);
/// Readiness-poll budget published by the last [`init`] call.
///
/// Peripheral enable/disable/reset paths run outside `init` (including in
/// `Drop`), so they read the budget here instead of `Config::new()`: a board
/// that tunes `readiness_poll_limit` gets the same budget everywhere.
static POLL_LIMIT: AtomicU32 = AtomicU32::new(1_000_000);
static HCLK_HZ: AtomicU32 = AtomicU32::new(0);
static PCLK0_HZ: AtomicU32 = AtomicU32::new(0);
static PCLK1_HZ: AtomicU32 = AtomicU32::new(0);

/// Oscillators controlled by RCC initialization.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Oscillator {
    /// Internal 48 MHz RC oscillator.
    Rco48m,
    /// Internal 32.768 kHz RC oscillator.
    Rco32k,
    /// External 32.768 kHz crystal oscillator.
    Xo32k,
    /// External 24 MHz crystal oscillator.
    Xo24m,
    /// 32 MHz LoRa-radio crystal oscillator.
    Xo32m,
    /// Internal RC oscillator. Despite its name, the vendor reports 3.6 MHz.
    Rco4m,
}

/// System clock source.
///
/// The PAC also describes a PLL selector value. It is intentionally absent:
/// the vendor RCC API neither exposes PLL as a system-clock source nor
/// specifies its frequency or setup sequence.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum SystemClockSource {
    /// Internal 48 MHz RC oscillator divided by two.
    #[default]
    Rco48mDiv2 = 0,
    /// Internal 32 kHz RC oscillator.
    Rco32k = 1,
    /// External 32.768 kHz crystal oscillator.
    Xo32k = 2,
    /// External 24 MHz crystal oscillator.
    Xo24m = 4,
    /// 32 MHz LoRa-radio crystal oscillator.
    Xo32m = 5,
    /// Internal RC oscillator (3.6 MHz according to the vendor clock API).
    Rco4m = 6,
    /// Internal 48 MHz RC oscillator.
    Rco48m = 7,
}

impl SystemClockSource {
    /// Nominal source frequency in hertz.
    pub const fn frequency_hz(self) -> u32 {
        match self {
            Self::Rco48mDiv2 => RCO48M_HZ / 2,
            Self::Rco32k => RCO32K_HZ,
            Self::Xo32k => XO32K_HZ,
            Self::Xo24m => XO24M_HZ,
            Self::Xo32m => XO32M_HZ,
            Self::Rco4m => RCO4M_HZ,
            Self::Rco48m => RCO48M_HZ,
        }
    }

    const fn oscillator(self) -> Oscillator {
        match self {
            Self::Rco48mDiv2 | Self::Rco48m => Oscillator::Rco48m,
            Self::Rco32k => Oscillator::Rco32k,
            Self::Xo32k => Oscillator::Xo32k,
            Self::Xo24m => Oscillator::Xo24m,
            Self::Xo32m => Oscillator::Xo32m,
            Self::Rco4m => Oscillator::Rco4m,
        }
    }
}

/// HCLK divider.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum HclkDivider {
    /// Divide by 1.
    #[default]
    Div1 = 0,
    /// Divide by 2.
    Div2 = 1,
    /// Divide by 4.
    Div4 = 2,
    /// Divide by 8.
    Div8 = 3,
    /// Divide by 16.
    Div16 = 4,
    /// Divide by 32.
    Div32 = 5,
    /// Divide by 64.
    Div64 = 6,
    /// Divide by 128.
    Div128 = 7,
    /// Divide by 256.
    Div256 = 8,
    /// Divide by 512.
    Div512 = 9,
}

impl HclkDivider {
    /// Numeric divider.
    pub const fn divisor(self) -> u32 {
        1 << self as u8
    }
}

/// PCLK0 or PCLK1 divider.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum PclkDivider {
    /// Divide by 1.
    #[default]
    Div1 = 0,
    /// Divide by 2.
    Div2 = 1,
    /// Divide by 4.
    Div4 = 2,
    /// Divide by 8.
    Div8 = 3,
    /// Divide by 16.
    Div16 = 4,
}

impl PclkDivider {
    /// Numeric divider.
    pub const fn divisor(self) -> u32 {
        1 << self as u8
    }
}

/// RCC initialization configuration.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Config {
    /// System clock source.
    pub system_clock: SystemClockSource,
    /// HCLK divider.
    pub hclk_divider: HclkDivider,
    /// PCLK0 divider relative to HCLK.
    pub pclk0_divider: PclkDivider,
    /// PCLK1 divider relative to HCLK.
    pub pclk1_divider: PclkDivider,
    /// Set the vendor TCXO-enable bits when starting the LoRa 32 MHz source.
    pub xo32m_uses_tcxo: bool,
    /// Maximum number of status-register polls for each readiness transition.
    ///
    /// This is a polling budget, not a duration: the CPU frequency can change
    /// while RCC is being initialized.
    pub readiness_poll_limit: u32,
}

impl Config {
    /// Configuration matching the documented reset clock tree.
    pub const fn new() -> Self {
        Self {
            system_clock: SystemClockSource::Rco48mDiv2,
            hclk_divider: HclkDivider::Div1,
            pclk0_divider: PclkDivider::Div1,
            pclk1_divider: PclkDivider::Div1,
            xo32m_uses_tcxo: false,
            readiness_poll_limit: 1_000_000,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

/// Initialized core and bus clock frequencies.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub struct Clocks {
    /// System clock frequency in hertz.
    pub sysclk_hz: u32,
    /// CPU/AHB clock frequency in hertz.
    pub hclk_hz: u32,
    /// Peripheral bus 0 frequency in hertz.
    pub pclk0_hz: u32,
    /// Peripheral bus 1 frequency in hertz.
    pub pclk1_hz: u32,
}

impl Clocks {
    const fn from_config(config: Config) -> Self {
        let sysclk_hz = config.system_clock.frequency_hz();
        let hclk_hz = sysclk_hz / config.hclk_divider.divisor();
        Self {
            sysclk_hz,
            hclk_hz,
            pclk0_hz: hclk_hz / config.pclk0_divider.divisor(),
            pclk1_hz: hclk_hz / config.pclk1_divider.divisor(),
        }
    }
}

/// Hardware transition whose status did not become ready.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum WaitTarget {
    /// Internal 48 MHz RC oscillator.
    Rco48m,
    /// Internal 3.6 MHz RC oscillator.
    Rco4m,
    /// LoRa 32 MHz crystal oscillator.
    Xo32m,
    /// RCC clock-domain synchronization.
    ClockDomain,
    /// EFC flash read wait-state programming.
    FlashReadNum,
}

/// RCC configuration or peripheral-control error.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// A bounded readiness wait expired.
    Timeout(WaitTarget),
    /// A reset requiring a timed release was requested before clock setup.
    ClocksNotInitialized,
    /// The vendor RCC does not provide a reset bit for this peripheral.
    ResetUnsupported(Peripheral),
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Timeout(_) => f.write_str("RCC readiness wait expired"),
            Self::ClocksNotInitialized => f.write_str("RCC clocks are not initialized"),
            Self::ResetUnsupported(_) => f.write_str("peripheral has no RCC reset bit"),
        }
    }
}

impl core::error::Error for Error {}

/// Return the frequencies published by successful RCC initialization.
///
/// `None` is returned until [`init`] has completed all oscillator and clock
/// transitions.
pub fn clocks() -> Option<Clocks> {
    if !CLOCKS_INITIALIZED.load(Ordering::Acquire) {
        return None;
    }

    Some(Clocks {
        sysclk_hz: SYSCLK_HZ.load(Ordering::Relaxed),
        hclk_hz: HCLK_HZ.load(Ordering::Relaxed),
        pclk0_hz: PCLK0_HZ.load(Ordering::Relaxed),
        pclk1_hz: PCLK1_HZ.load(Ordering::Relaxed),
    })
}

/// Initialize the ASR6601 oscillator and core/bus clock tree.
///
/// The RCC singleton makes normal callers perform this operation once. The
/// selected oscillator is enabled before it is selected. PCLKs are temporarily
/// divided by 16, and HCLK/source ordering is chosen to avoid a transient clock
/// faster than either the existing or requested clock.
///
/// Matches vendor `system_init()` by enabling the AFEC clock before any
/// analog-window or `RAW_SR` access. Without that gate, oscillator readiness
/// bits read as clear and [`Error::Timeout`] is reported.
pub fn init(_rcc: Peri<'_, peripherals::RCC>, config: Config) -> Result<Clocks, Error> {
    CLOCKS_INITIALIZED.store(false, Ordering::Release);

    // Vendor `system_cm4.c` enables AFEC before oscillator or clock work.
    crate::afec::analog::enable_clock();

    enable_oscillator(
        config.system_clock.oscillator(),
        config.xo32m_uses_tcxo,
        config.readiness_poll_limit,
    )?;
    configure_clock_tree(config)?;

    let clocks = Clocks::from_config(config);
    SYSCLK_HZ.store(clocks.sysclk_hz, Ordering::Relaxed);
    HCLK_HZ.store(clocks.hclk_hz, Ordering::Relaxed);
    PCLK0_HZ.store(clocks.pclk0_hz, Ordering::Relaxed);
    PCLK1_HZ.store(clocks.pclk1_hz, Ordering::Relaxed);
    POLL_LIMIT.store(config.readiness_poll_limit, Ordering::Relaxed);
    CLOCKS_INITIALIZED.store(true, Ordering::Release);
    Ok(clocks)
}

fn rcc() -> pac::Rcc {
    // RCC is a shared hardware service. Every read-modify-write performed by
    // this module is protected by a critical section.
    unsafe { pac::Rcc::steal() }
}

fn afec() -> pac::Afec {
    unsafe { pac::Afec::steal() }
}

fn lorac() -> pac::Lorac {
    unsafe { pac::Lorac::steal() }
}

fn current_system_frequency() -> Option<u32> {
    use pac::rcc::cr0::SysclkSel;

    // The hardware field is fully specified, so every bit pattern decodes;
    // only PLL has no vendor frequency to report. Reuse
    // `SystemClockSource::frequency_hz` so the frequency table has one owner.
    let source = match rcc().cr0().read().sysclk_sel().variant() {
        SysclkSel::Rco48mDiv2 => SystemClockSource::Rco48mDiv2,
        SysclkSel::Rco32k => SystemClockSource::Rco32k,
        SysclkSel::Xo32k => SystemClockSource::Xo32k,
        SysclkSel::Pll => return None,
        SysclkSel::Xo24m => SystemClockSource::Xo24m,
        SysclkSel::Xo32m => SystemClockSource::Xo32m,
        SysclkSel::Rco4m => SystemClockSource::Rco4m,
        SysclkSel::Rco48m => SystemClockSource::Rco48m,
    };
    Some(source.frequency_hz())
}

fn set_hclk_divider(divider: HclkDivider) {
    use pac::rcc::cr0::HclkDiv;
    let variant = match divider {
        HclkDivider::Div1 => HclkDiv::Value1,
        HclkDivider::Div2 => HclkDiv::Value2,
        HclkDivider::Div4 => HclkDiv::Value4,
        HclkDivider::Div8 => HclkDiv::Value8,
        HclkDivider::Div16 => HclkDiv::Value16,
        HclkDivider::Div32 => HclkDiv::Value32,
        HclkDivider::Div64 => HclkDiv::Value64,
        HclkDivider::Div128 => HclkDiv::Value128,
        HclkDivider::Div256 => HclkDiv::Value256,
        HclkDivider::Div512 => HclkDiv::Value512,
    };
    critical_section::with(|_| {
        rcc().cr0().modify(|_, w| w.hclk_div().variant(variant));
    });
}

fn set_pclk_dividers(pclk0: PclkDivider, pclk1: PclkDivider) {
    use pac::rcc::cr0::{Pclk0Div, Pclk1Div};
    fn map0(div: PclkDivider) -> Pclk0Div {
        match div {
            PclkDivider::Div1 => Pclk0Div::Value1,
            PclkDivider::Div2 => Pclk0Div::Value2,
            PclkDivider::Div4 => Pclk0Div::Value4,
            PclkDivider::Div8 => Pclk0Div::Value8,
            PclkDivider::Div16 => Pclk0Div::Value16,
        }
    }
    fn map1(div: PclkDivider) -> Pclk1Div {
        match div {
            PclkDivider::Div1 => Pclk1Div::Value1,
            PclkDivider::Div2 => Pclk1Div::Value2,
            PclkDivider::Div4 => Pclk1Div::Value4,
            PclkDivider::Div8 => Pclk1Div::Value8,
            PclkDivider::Div16 => Pclk1Div::Value16,
        }
    }
    critical_section::with(|_| {
        rcc().cr0().modify(|_, w| {
            w.pclk0_div().variant(map0(pclk0));
            w.pclk1_div().variant(map1(pclk1))
        });
    });
}

fn set_system_clock(source: SystemClockSource) {
    use pac::rcc::cr0::SysclkSel;
    let variant = match source {
        SystemClockSource::Rco48mDiv2 => SysclkSel::Rco48mDiv2,
        SystemClockSource::Rco32k => SysclkSel::Rco32k,
        SystemClockSource::Xo32k => SysclkSel::Xo32k,
        SystemClockSource::Xo24m => SysclkSel::Xo24m,
        SystemClockSource::Xo32m => SysclkSel::Xo32m,
        SystemClockSource::Rco4m => SysclkSel::Rco4m,
        SystemClockSource::Rco48m => SysclkSel::Rco48m,
    };
    critical_section::with(|_| {
        rcc().cr0().modify(|_, w| w.sysclk_sel().variant(variant));
    });
}

fn configure_clock_tree(config: Config) -> Result<(), Error> {
    let old_frequency = current_system_frequency();
    let new_frequency = config.system_clock.frequency_hz();
    let new_read_num = flash_read_num_for(new_frequency);

    // Protect both peripheral buses while HCLK and SYSCLK are in transition.
    set_pclk_dividers(PclkDivider::Div16, PclkDivider::Div16);

    match old_frequency {
        Some(old_frequency) if new_frequency >= old_frequency => {
            // RM 10.3.3: increase the flash wait states before raising SYS_CLK.
            set_flash_read_num(new_read_num, config.readiness_poll_limit)?;
            set_hclk_divider(config.hclk_divider);
            set_system_clock(config.system_clock);
        }
        Some(_) => {
            set_system_clock(config.system_clock);
            set_hclk_divider(config.hclk_divider);
            // RM 10.3.3: decrease the flash wait states after lowering SYS_CLK.
            set_flash_read_num(new_read_num, config.readiness_poll_limit)?;
        }
        None => {
            // PLL setup/frequency is not specified by the vendor RCC API. Use
            // the highest wait-state setting while switching away from it.
            set_flash_read_num(2, config.readiness_poll_limit)?;
            set_hclk_divider(HclkDivider::Div512);
            set_system_clock(config.system_clock);
            set_hclk_divider(config.hclk_divider);
            set_flash_read_num(new_read_num, config.readiness_poll_limit)?;
        }
    }

    set_pclk_dividers(config.pclk0_divider, config.pclk1_divider);
    Ok(())
}

/// Flash read wait states required for a given SYS_CLK frequency.
///
/// RM 10.3.3: 48 MHz needs 2, 32/24 MHz need 1, and 4 MHz/32 kHz need 0.
fn flash_read_num_for(frequency_hz: u32) -> u8 {
    if frequency_hz >= 48_000_000 {
        2
    } else if frequency_hz >= 24_000_000 {
        1
    } else {
        0
    }
}

fn set_flash_read_num(read_num: u8, poll_limit: u32) -> Result<(), Error> {
    let efc = unsafe { pac::Efc::steal() };
    // SAFETY: the field is 4 bits and `read_num` is limited to 0..=2 here.
    efc.timing_cfg().modify(|_, w| unsafe { w.read_num().bits(read_num) });
    wait_until(poll_limit, WaitTarget::FlashReadNum, || {
        efc.sr().read().read_num_done().bit_is_set()
    })
}

fn wait_until(poll_limit: u32, target: WaitTarget, mut ready: impl FnMut() -> bool) -> Result<(), Error> {
    for _ in 0..poll_limit {
        if ready() {
            return Ok(());
        }
        core::hint::spin_loop();
    }

    if ready() { Ok(()) } else { Err(Error::Timeout(target)) }
}

fn enable_oscillator(oscillator: Oscillator, xo32m_uses_tcxo: bool, poll_limit: u32) -> Result<(), Error> {
    use crate::afec::analog;

    match oscillator {
        Oscillator::Rco48m => {
            analog::with(|afec| {
                afec.analog_06().modify(|_, w| w.rco48m_power_down().clear_bit());
            });
            wait_until(poll_limit, WaitTarget::Rco48m, || {
                afec().raw_sr().read().rco24m_ready().bit_is_set()
            })
        }
        Oscillator::Rco32k => {
            analog::with(|afec| {
                afec.analog_02().modify(|_, w| w.rco32k_power_down().clear_bit());
            });
            Ok(())
        }
        Oscillator::Xo32k => {
            analog::with(|afec| {
                afec.analog_02().modify(|_, w| w.xo32k_power_down().enabled());
            });
            Ok(())
        }
        Oscillator::Xo24m => {
            analog::with(|afec| {
                afec.analog_06().modify(|_, w| {
                    w.xo24m_enable().set_bit();
                    w.xo24m_power_down().clear_bit();
                    w
                });
            });
            Ok(())
        }
        Oscillator::Xo32m => {
            set_peripheral_clock_raw(Peripheral::Lora, true, poll_limit)?;

            critical_section::with(|_| {
                let nreset_clear = !lorac().cr1().read().nreset_bat().bit_is_set();
                lorac().cr1().modify(|_, w| {
                    if nreset_clear {
                        w.nreset_bat().set_bit();
                        w.por_bat().clear_bit();
                    }
                    if xo32m_uses_tcxo {
                        w.pwrtcxo_en_bat().set_bit();
                        w.tcxo_en_bat().set_bit();
                    }
                    w.clk_32m_en_bat().set_bit()
                });
            });

            wait_until(poll_limit, WaitTarget::Xo32m, || {
                lorac().sr().read().clk_32m_rdy_bat_sr().bit_is_set()
            })
        }
        Oscillator::Rco4m => {
            analog::with(|afec| {
                afec.analog_06().modify(|_, w| w.rco4m_power_down().clear_bit());
            });
            wait_until(poll_limit, WaitTarget::Rco4m, || {
                afec().raw_sr().read().rco4m_ready().bit_is_set()
            })
        }
    }
}

/// RCC-controlled peripheral clock/reset identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Peripheral {
    /// Smart access controller.
    Sac,
    /// Security engine.
    Sec,
    /// CRC engine.
    Crc,
    /// Real-time clock.
    Rtc,
    /// Window watchdog.
    Wdg,
    /// Independent watchdog.
    Iwdg,
    /// Low-power timer 0.
    Lptimer0,
    /// Basic timer 1.
    Bstimer1,
    /// Basic timer 0.
    Bstimer0,
    /// General-purpose timer 3.
    Timer3,
    /// General-purpose timer 2.
    Timer2,
    /// General-purpose timer 1.
    Timer1,
    /// General-purpose timer 0.
    Timer0,
    /// GPIO port A.
    GpioA,
    /// GPIO port B.
    GpioB,
    /// GPIO port C.
    GpioC,
    /// GPIO port D.
    GpioD,
    /// LoRa controller.
    Lora,
    /// DAC controller.
    Dac,
    /// LCD controller.
    Lcd,
    /// Analog front-end controller.
    Afec,
    /// ADC controller.
    Adc,
    /// I2C bus 2.
    I2c2,
    /// I2C bus 1.
    I2c1,
    /// I2C bus 0.
    I2c0,
    /// Quad-SPI controller.
    Qspi,
    /// Synchronous serial port 2.
    Ssp2,
    /// Synchronous serial port 1.
    Ssp1,
    /// Synchronous serial port 0.
    Ssp0,
    /// Low-power UART.
    Lpuart,
    /// UART3.
    Uart3,
    /// UART2.
    Uart2,
    /// UART1.
    Uart1,
    /// UART0.
    Uart0,
    /// DMA controller 1.
    Dma1,
    /// DMA controller 0.
    Dma0,
    /// I2S controller.
    I2s,
    /// Random number generator.
    Rng,
    /// Low-power timer 1.
    Lptimer1,
    /// System configuration controller.
    Syscfg,
    /// Power controller.
    Pwr,
}

#[derive(Clone, Copy)]
enum DualDomain {
    Lpuart,
    Lcd,
    Rtc,
    Iwdg,
}

#[derive(Clone, Copy)]
enum LpTimer {
    Lptim0,
    Lptim1,
}

fn all_aon_clocks_done() -> bool {
    let sr = rcc().sr().read();
    sr.set_iwdg_aon_clk_en_done().bit_is_set()
        && sr.set_rtc_aon_clk_en_done().bit_is_set()
        && sr.set_lpuart_aon_clk_en_done().bit_is_set()
        && sr.set_lcdctrl_aon_clk_en_done().bit_is_set()
        && sr.set_lptim0_aon_clk_en_done().bit_is_set()
        && sr.set_lptim1_aon_clk_en_done().bit_is_set()
}

fn dual_aon_done(peripheral: DualDomain) -> bool {
    let sr = rcc().sr().read();
    match peripheral {
        DualDomain::Lpuart => sr.set_lpuart_aon_clk_en_done().bit_is_set(),
        DualDomain::Lcd => sr.set_lcdctrl_aon_clk_en_done().bit_is_set(),
        DualDomain::Rtc => sr.set_rtc_aon_clk_en_done().bit_is_set(),
        DualDomain::Iwdg => sr.set_iwdg_aon_clk_en_done().bit_is_set(),
    }
}

fn lptimer_aon_done(timer: LpTimer) -> bool {
    let sr = rcc().sr().read();
    match timer {
        LpTimer::Lptim0 => sr.set_lptim0_aon_clk_en_done().bit_is_set(),
        LpTimer::Lptim1 => sr.set_lptim1_aon_clk_en_done().bit_is_set(),
    }
}

/// Gate one basic-domain peripheral clock.
///
/// GPIOA..D gates are the PAC `IOM0..3_CLK_EN` fields (GpioA is IOM0, …,
/// GpioD is IOM3 — the GPIO ports double as I/O-matrix instances).
/// Dual-domain peripherals (RTC/IWDG/LPTIMER/LCD/LPUART) are not handled
/// here — see `set_peripheral_clock_raw`.
fn set_basic_clock_gate(peripheral: Peripheral, enable: bool) {
    critical_section::with(|_| {
        let rcc = rcc();
        match peripheral {
            Peripheral::Timer3 => rcc.cgr0().modify(|_, w| w.gptim3_clk_en().bit(enable)),
            Peripheral::Timer2 => rcc.cgr0().modify(|_, w| w.gptim2_clk_en().bit(enable)),
            Peripheral::Timer1 => rcc.cgr0().modify(|_, w| w.gptim1_clk_en().bit(enable)),
            Peripheral::Timer0 => rcc.cgr0().modify(|_, w| w.gptim0_clk_en().bit(enable)),
            Peripheral::Lora => rcc.cgr0().modify(|_, w| w.lorac_clk_en().bit(enable)),
            Peripheral::Dac => rcc.cgr0().modify(|_, w| w.dacctrl_clk_en().bit(enable)),
            Peripheral::Afec => rcc.cgr0().modify(|_, w| w.afec_clk_en().bit(enable)),
            Peripheral::Adc => rcc.cgr0().modify(|_, w| w.adc_clk_en().bit(enable)),
            Peripheral::I2c2 => rcc.cgr0().modify(|_, w| w.i2c2_clk_en().bit(enable)),
            Peripheral::I2c1 => rcc.cgr0().modify(|_, w| w.i2c1_clk_en().bit(enable)),
            Peripheral::I2c0 => rcc.cgr0().modify(|_, w| w.i2c0_clk_en().bit(enable)),
            Peripheral::Ssp2 => rcc.cgr0().modify(|_, w| w.ssp2_clk_en().bit(enable)),
            Peripheral::Ssp1 => rcc.cgr0().modify(|_, w| w.ssp1_clk_en().bit(enable)),
            Peripheral::Ssp0 => rcc.cgr0().modify(|_, w| w.ssp0_clk_en().bit(enable)),
            Peripheral::Uart3 => rcc.cgr0().modify(|_, w| w.uart3_clk_en().bit(enable)),
            Peripheral::Uart2 => rcc.cgr0().modify(|_, w| w.uart2_clk_en().bit(enable)),
            Peripheral::Uart1 => rcc.cgr0().modify(|_, w| w.uart1_clk_en().bit(enable)),
            Peripheral::Uart0 => rcc.cgr0().modify(|_, w| w.uart0_clk_en().bit(enable)),
            Peripheral::Syscfg => rcc.cgr0().modify(|_, w| w.syscfg_clk_en().bit(enable)),
            Peripheral::GpioA => rcc.cgr0().modify(|_, w| w.iom0_clk_en().bit(enable)),
            Peripheral::GpioB => rcc.cgr0().modify(|_, w| w.iom1_clk_en().bit(enable)),
            Peripheral::GpioC => rcc.cgr0().modify(|_, w| w.iom2_clk_en().bit(enable)),
            Peripheral::GpioD => rcc.cgr0().modify(|_, w| w.iom3_clk_en().bit(enable)),
            Peripheral::Bstimer1 => rcc.cgr0().modify(|_, w| w.basictim1_clk_en().bit(enable)),
            Peripheral::Bstimer0 => rcc.cgr0().modify(|_, w| w.basictim0_clk_en().bit(enable)),
            Peripheral::Crc => rcc.cgr0().modify(|_, w| w.crc_clk_en().bit(enable)),
            Peripheral::Dma1 => rcc.cgr0().modify(|_, w| w.dmac1_clk_en().bit(enable)),
            Peripheral::Dma0 => rcc.cgr0().modify(|_, w| w.dmac0_clk_en().bit(enable)),
            Peripheral::Pwr => rcc.cgr0().modify(|_, w| w.pwr_clk_en().bit(enable)),
            Peripheral::Sec => rcc.cgr1().modify(|_, w| w.sec_clk_en().bit(enable)),
            Peripheral::Qspi => rcc.cgr1().modify(|_, w| w.qspi_clk_en().bit(enable)),
            Peripheral::Sac => rcc.cgr1().modify(|_, w| w.sac_clk_en().bit(enable)),
            Peripheral::I2s => rcc.cgr1().modify(|_, w| w.i2s_clk_en().bit(enable)),
            Peripheral::Rng => rcc.cgr1().modify(|_, w| w.rngc_clk_en().bit(enable)),
            Peripheral::Wdg => rcc.cgr1().modify(|_, w| {
                w.wwdg_clk_en().bit(enable);
                w.wwdg_cnt_clk_en().bit(enable)
            }),
            Peripheral::Rtc
            | Peripheral::Iwdg
            | Peripheral::Lptimer0
            | Peripheral::Lptimer1
            | Peripheral::Lcd
            | Peripheral::Lpuart => unreachable!("dual-domain clocks use set_peripheral_clock_raw"),
        }
    });
}

fn set_dual_domain_clock(peripheral: DualDomain, enable: bool, poll_limit: u32) -> Result<(), Error> {
    critical_section::with(|_| {
        let rcc = rcc();
        match peripheral {
            DualDomain::Lpuart => rcc.cgr0().modify(|_, w| w.lpuart_clk_en().bit(enable)),
            DualDomain::Lcd => rcc.cgr0().modify(|_, w| w.lcd_clk_en().bit(enable)),
            DualDomain::Rtc => rcc.cgr1().modify(|_, w| w.rtc_clk_en().bit(enable)),
            DualDomain::Iwdg => rcc.cgr1().modify(|_, w| w.iwdg_clk_en().bit(enable)),
        };
    });
    wait_until(poll_limit, WaitTarget::ClockDomain, all_aon_clocks_done)?;

    critical_section::with(|_| {
        let rcc = rcc();
        match peripheral {
            DualDomain::Lpuart => rcc.cgr2().modify(|_, w| w.lpuart_aon_clk_en().bit(enable)),
            DualDomain::Lcd => rcc.cgr2().modify(|_, w| w.lcd_aon_clk_en().bit(enable)),
            DualDomain::Rtc => rcc.cgr2().modify(|_, w| w.rtc_aon_clk_en().bit(enable)),
            DualDomain::Iwdg => rcc.cgr2().modify(|_, w| w.iwdg_clk_en().bit(enable)),
        };
    });
    wait_until(poll_limit, WaitTarget::ClockDomain, || dual_aon_done(peripheral))
}

fn set_lptimer_clock(timer: LpTimer, enable: bool, poll_limit: u32) -> Result<(), Error> {
    if enable {
        critical_section::with(|_| {
            let rcc = rcc();
            match timer {
                LpTimer::Lptim0 => rcc.cgr1().modify(|_, w| w.lptim0_inf_clk_en().bit(true)),
                LpTimer::Lptim1 => rcc.cgr1().modify(|_, w| w.lptim1_inf_clk_en().bit(true)),
            };
        });
        let _ = wait_until(poll_limit, WaitTarget::ClockDomain, all_aon_clocks_done);

        critical_section::with(|_| {
            let rcc = rcc();
            match timer {
                LpTimer::Lptim0 => rcc.cgr2().modify(|_, w| w.lptim0_aon_clk_en().bit(true)),
                LpTimer::Lptim1 => rcc.cgr2().modify(|_, w| w.lptim1_aon_clk_en().bit(true)),
            };
        });
        // v1.6.2 SVD's cgr2/sr always-on sync bits are flaky and a bootloader
        // may leave them in a state that never asserts. The functional gate in
        // cgr1 is sufficient for polling `now()`; don't fail init if the sync
        // never asserts.
        let _ = wait_until(poll_limit, WaitTarget::ClockDomain, || lptimer_aon_done(timer));

        critical_section::with(|_| {
            let rcc = rcc();
            match timer {
                LpTimer::Lptim0 => rcc.cgr1().modify(|_, w| w.lptim0_clk_en().bit(true)),
                LpTimer::Lptim1 => rcc.cgr1().modify(|_, w| w.lptim1_clk_en().bit(true)),
            };
        });
    } else {
        critical_section::with(|_| {
            let rcc = rcc();
            match timer {
                LpTimer::Lptim0 => rcc.cgr1().modify(|_, w| w.lptim0_clk_en().bit(false)),
                LpTimer::Lptim1 => rcc.cgr1().modify(|_, w| w.lptim1_clk_en().bit(false)),
            };
        });
        let _ = wait_until(poll_limit, WaitTarget::ClockDomain, all_aon_clocks_done);

        critical_section::with(|_| {
            let rcc = rcc();
            match timer {
                LpTimer::Lptim0 => rcc.cgr2().modify(|_, w| w.lptim0_aon_clk_en().bit(false)),
                LpTimer::Lptim1 => rcc.cgr2().modify(|_, w| w.lptim1_aon_clk_en().bit(false)),
            };
        });
        let _ = wait_until(poll_limit, WaitTarget::ClockDomain, || lptimer_aon_done(timer));
        // The vendor intentionally leaves the PCLK gate enabled on disable.
    }
    Ok(())
}

fn set_peripheral_clock_raw(peripheral: Peripheral, enable: bool, poll_limit: u32) -> Result<(), Error> {
    match peripheral {
        Peripheral::Lpuart => set_dual_domain_clock(DualDomain::Lpuart, enable, poll_limit),
        Peripheral::Lcd => set_dual_domain_clock(DualDomain::Lcd, enable, poll_limit),
        Peripheral::Rtc => set_dual_domain_clock(DualDomain::Rtc, enable, poll_limit),
        Peripheral::Iwdg => set_dual_domain_clock(DualDomain::Iwdg, enable, poll_limit),
        Peripheral::Lptimer0 => set_lptimer_clock(LpTimer::Lptim0, enable, poll_limit),
        Peripheral::Lptimer1 => set_lptimer_clock(LpTimer::Lptim1, enable, poll_limit),
        _ => {
            set_basic_clock_gate(peripheral, enable);
            Ok(())
        }
    }
}

/// Enable a peripheral clock using the default bounded synchronization budget.
pub(crate) fn enable_peripheral(peripheral: Peripheral) -> Result<(), Error> {
    set_peripheral_clock_raw(peripheral, true, POLL_LIMIT.load(Ordering::Relaxed))
}

/// Disable a peripheral clock using the default bounded synchronization budget.
pub(crate) fn disable_peripheral(peripheral: Peripheral) -> Result<(), Error> {
    set_peripheral_clock_raw(peripheral, false, POLL_LIMIT.load(Ordering::Relaxed))
}

fn reset_needs_release_delay(peripheral: Peripheral) -> bool {
    matches!(
        peripheral,
        Peripheral::Lptimer0
            | Peripheral::Lptimer1
            | Peripheral::Lcd
            | Peripheral::Rtc
            | Peripheral::Iwdg
            | Peripheral::Lpuart
    )
}

/// Assert or release one peripheral's active-low reset line.
///
/// Every peripheral except Syscfg/Pwr has a generated reset accessor (GPIO
/// ports share `IOM_RST_N`); Syscfg/Pwr have no reset at all
/// (`Error::ResetUnsupported`).
fn set_reset_line(peripheral: Peripheral, asserted: bool) -> Result<(), Error> {
    // Active-low: asserting clears the bit.
    if matches!(peripheral, Peripheral::Syscfg | Peripheral::Pwr) {
        return Err(Error::ResetUnsupported(peripheral));
    }
    let release = !asserted;
    critical_section::with(|_| {
        let rcc = rcc();
        match peripheral {
            Peripheral::Sac => rcc.rst0().modify(|_, w| w.sac_rst_n().bit(release)),
            Peripheral::Sec => rcc.rst0().modify(|_, w| w.sec_rst_n().bit(release)),
            Peripheral::Crc => rcc.rst0().modify(|_, w| w.crc_rst_n().bit(release)),
            Peripheral::Rtc => rcc.rst0().modify(|_, w| w.rtc_rst_n().bit(release)),
            Peripheral::Wdg => rcc.rst0().modify(|_, w| w.wwdg_rst_n().bit(release)),
            Peripheral::Iwdg => rcc.rst0().modify(|_, w| w.iwdg_rst_n().bit(release)),
            Peripheral::Lptimer0 => rcc.rst0().modify(|_, w| w.lptim0_rst_n().bit(release)),
            Peripheral::Bstimer1 => rcc.rst0().modify(|_, w| w.basictim1_rst_n().bit(release)),
            Peripheral::Bstimer0 => rcc.rst0().modify(|_, w| w.basictim0_rst_n().bit(release)),
            Peripheral::Timer3 => rcc.rst0().modify(|_, w| w.gptim3_rst_n().bit(release)),
            Peripheral::Timer2 => rcc.rst0().modify(|_, w| w.gptim2_rst_n().bit(release)),
            Peripheral::Timer1 => rcc.rst0().modify(|_, w| w.gptim1_rst_n().bit(release)),
            Peripheral::Timer0 => rcc.rst0().modify(|_, w| w.gptim0_rst_n().bit(release)),
            // One shared IOM reset covers all four GPIO ports.
            Peripheral::GpioA | Peripheral::GpioB | Peripheral::GpioC | Peripheral::GpioD => {
                rcc.rst0().modify(|_, w| w.iom_rst_n().bit(release))
            }
            Peripheral::Lora => rcc.rst0().modify(|_, w| w.lorac_rst_n().bit(release)),
            Peripheral::Dac => rcc.rst0().modify(|_, w| w.dacctrl_rst_n().bit(release)),
            Peripheral::Lcd => rcc.rst0().modify(|_, w| w.lcd_rst_n().bit(release)),
            Peripheral::Afec => rcc.rst0().modify(|_, w| w.afec_rst_n().bit(release)),
            Peripheral::Adc => rcc.rst0().modify(|_, w| w.adc_rst_n().bit(release)),
            Peripheral::I2c2 => rcc.rst0().modify(|_, w| w.i2c2_rst_n().bit(release)),
            Peripheral::I2c1 => rcc.rst0().modify(|_, w| w.i2c1_rst_n().bit(release)),
            Peripheral::I2c0 => rcc.rst0().modify(|_, w| w.i2c0_rst_n().bit(release)),
            Peripheral::Qspi => rcc.rst0().modify(|_, w| w.qspi_rst_n().bit(release)),
            Peripheral::Ssp2 => rcc.rst0().modify(|_, w| w.ssp2_rst_n().bit(release)),
            Peripheral::Ssp1 => rcc.rst0().modify(|_, w| w.ssp1_rst_n().bit(release)),
            Peripheral::Ssp0 => rcc.rst0().modify(|_, w| w.ssp0_rst_n().bit(release)),
            Peripheral::Lpuart => rcc.rst0().modify(|_, w| w.lpuart_rst_n().bit(release)),
            Peripheral::Uart3 => rcc.rst0().modify(|_, w| w.uart3_rst_n().bit(release)),
            Peripheral::Uart2 => rcc.rst0().modify(|_, w| w.uart2_rst_n().bit(release)),
            Peripheral::Uart1 => rcc.rst0().modify(|_, w| w.uart1_rst_n().bit(release)),
            Peripheral::Uart0 => rcc.rst0().modify(|_, w| w.uart0_rst_n().bit(release)),
            Peripheral::Dma1 => rcc.rst1().modify(|_, w| w.dmac1_rst_n().bit(release)),
            Peripheral::Dma0 => rcc.rst1().modify(|_, w| w.dmac0_rst_n().bit(release)),
            Peripheral::I2s => rcc.rst1().modify(|_, w| w.i2s_rst_n().bit(release)),
            Peripheral::Rng => rcc.rst1().modify(|_, w| w.rngc_rst_n().bit(release)),
            Peripheral::Lptimer1 => rcc.rst1().modify(|_, w| w.lptim1_rst_n().bit(release)),
            // Syscfg/Pwr rejected above: the vendor provides no reset bits.
            _ => unreachable!(),
        };
    });
    Ok(())
}

fn asynchronous_release_delay() -> Result<(), Error> {
    let hclk_hz = clocks().ok_or(Error::ClocksNotInitialized)?.hclk_hz as u64;
    let cycles = hclk_hz
        .saturating_mul(ASYNC_RESET_DELAY_US)
        .div_ceil(1_000_000)
        .min(u32::MAX as u64) as u32;
    cortex_m::asm::delay(cycles);
    Ok(())
}

/// Assert a peripheral's active-low reset line.
pub(crate) fn assert_peripheral_reset(peripheral: Peripheral) -> Result<(), Error> {
    set_reset_line(peripheral, true)
}

/// Release a peripheral reset and honor the vendor's asynchronous-domain delay.
pub(crate) fn release_peripheral_reset(peripheral: Peripheral) -> Result<(), Error> {
    if reset_needs_release_delay(peripheral) && clocks().is_none() {
        return Err(Error::ClocksNotInitialized);
    }
    set_reset_line(peripheral, false)?;
    if reset_needs_release_delay(peripheral) {
        asynchronous_release_delay()?;
    }
    Ok(())
}

/// Pulse a peripheral reset line.
pub(crate) fn reset_peripheral(peripheral: Peripheral) -> Result<(), Error> {
    if reset_needs_release_delay(peripheral) && clocks().is_none() {
        return Err(Error::ClocksNotInitialized);
    }
    assert_peripheral_reset(peripheral)?;
    release_peripheral_reset(peripheral)
}

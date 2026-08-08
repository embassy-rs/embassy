//! Reset and clock control for the ASR6601.
//!
//! The clock tree is deliberately configured before it is published through
//! [`clocks`]. Peripheral drivers may share RCC through the crate-private
//! helpers at the end of this module; their register updates are serialized by
//! a critical section.

use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use crate::{Peri, pac, peripherals};

const RCO48M_HZ: u32 = 48_000_000;
const RCO4M_HZ: u32 = 3_600_000;
const XO24M_HZ: u32 = 24_000_000;
const XO32M_HZ: u32 = 32_000_000;
const LOW_SPEED_HZ: u32 = 32_768;

// The analog oscillator control registers occupy the otherwise-unmodelled
// first 0x200 bytes of the AFEC address range.
const AFEC_ANALOG_BASE: usize = 0x4000_8000;
const AFEC_ANALOG_OSC32K: usize = 0x02;
const AFEC_ANALOG_OSC_HIGH_SPEED: usize = 0x06;

const ANALOG_RCO32K_POWER_DOWN: u32 = 1 << 15;
const ANALOG_XO32K_POWER_DOWN: u32 = (1 << 13) | (1 << 14);
const ANALOG_XO24M_ENABLE: u32 = 1 << 3;
const ANALOG_XO24M_POWER_DOWN: u32 = 1 << 4;
const ANALOG_RCO48M_POWER_DOWN: u32 = 1 << 5;
const ANALOG_RCO4M_POWER_DOWN: u32 = 1 << 6;

const CR0_PCLK0_DIV_MASK: u32 = 0x0000_00e0;
const CR0_HCLK_DIV_MASK: u32 = 0x0000_0f00;
const CR0_SYSCLK_SEL_MASK: u32 = 0x0000_7000;
const CR0_PCLK1_DIV_MASK: u32 = 0x0003_8000;

const RCC_SR_ALL_DONE: u32 = 0x3f;

const LORAC_CR1_TCXO_ENABLE: u32 = 0x03;
const LORAC_CR1_XO32M_ENABLE: u32 = 1 << 2;
const LORAC_CR1_NRESET: u32 = 1 << 5;
const LORAC_CR1_POWER_ON_RESET: u32 = 1 << 7;
const LORAC_SR_XO32M_READY: u32 = 1 << 1;

const ASYNC_RESET_DELAY_US: u64 = 92;

static CLOCKS_INITIALIZED: AtomicBool = AtomicBool::new(false);
static SYSCLK_HZ: AtomicU32 = AtomicU32::new(0);
static HCLK_HZ: AtomicU32 = AtomicU32::new(0);
static PCLK0_HZ: AtomicU32 = AtomicU32::new(0);
static PCLK1_HZ: AtomicU32 = AtomicU32::new(0);

/// Oscillators controlled by RCC initialization.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
#[repr(u8)]
pub enum SystemClockSource {
    /// Internal 48 MHz RC oscillator divided by two.
    #[default]
    Rco48mDiv2 = 0,
    /// Internal 32.768 kHz RC oscillator.
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
            Self::Rco32k | Self::Xo32k => LOW_SPEED_HZ,
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
pub enum WaitTarget {
    /// Internal 48 MHz RC oscillator.
    Rco48m,
    /// Internal 3.6 MHz RC oscillator.
    Rco4m,
    /// LoRa 32 MHz crystal oscillator.
    Xo32m,
    /// RCC clock-domain synchronization.
    ClockDomain,
}

/// RCC configuration or peripheral-control error.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    /// A bounded readiness wait expired.
    Timeout(WaitTarget),
    /// A reset requiring a timed release was requested before clock setup.
    ClocksNotInitialized,
    /// The vendor RCC does not provide a reset bit for this peripheral.
    ResetUnsupported(Peripheral),
}

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
pub fn init(_rcc: Peri<'_, peripherals::RCC>, config: Config) -> Result<Clocks, Error> {
    CLOCKS_INITIALIZED.store(false, Ordering::Release);

    enable_oscillator(
        config.system_clock.oscillator(),
        config.xo32m_uses_tcxo,
        config.readiness_poll_limit,
    )?;
    configure_clock_tree(config);

    let clocks = Clocks::from_config(config);
    SYSCLK_HZ.store(clocks.sysclk_hz, Ordering::Relaxed);
    HCLK_HZ.store(clocks.hclk_hz, Ordering::Relaxed);
    PCLK0_HZ.store(clocks.pclk0_hz, Ordering::Relaxed);
    PCLK1_HZ.store(clocks.pclk1_hz, Ordering::Relaxed);
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

fn update_bits(value: u32, mask: u32, set: bool) -> u32 {
    if set { value | mask } else { value & !mask }
}

fn modify_cr0(mask: u32, value: u32) {
    critical_section::with(|_| {
        rcc()
            .cr0()
            .modify(|r, w| unsafe { w.bits((r.bits() & !mask) | (value & mask)) });
    });
}

fn current_system_frequency() -> Option<u32> {
    match (rcc().cr0().read().bits() & CR0_SYSCLK_SEL_MASK) >> 12 {
        0 => Some(RCO48M_HZ / 2),
        1 | 2 => Some(LOW_SPEED_HZ),
        3 => None,
        4 => Some(XO24M_HZ),
        5 => Some(XO32M_HZ),
        6 => Some(RCO4M_HZ),
        7 => Some(RCO48M_HZ),
        _ => unreachable!(),
    }
}

fn set_hclk_divider(divider: HclkDivider) {
    modify_cr0(CR0_HCLK_DIV_MASK, (divider as u32) << 8);
}

fn set_pclk_dividers(pclk0: PclkDivider, pclk1: PclkDivider) {
    modify_cr0(
        CR0_PCLK0_DIV_MASK | CR0_PCLK1_DIV_MASK,
        ((pclk0 as u32) << 5) | ((pclk1 as u32) << 15),
    );
}

fn set_system_clock(source: SystemClockSource) {
    modify_cr0(CR0_SYSCLK_SEL_MASK, (source as u32) << 12);
}

fn configure_clock_tree(config: Config) {
    let old_frequency = current_system_frequency();
    let new_frequency = config.system_clock.frequency_hz();

    // Protect both peripheral buses while HCLK and SYSCLK are in transition.
    set_pclk_dividers(PclkDivider::Div16, PclkDivider::Div16);

    match old_frequency {
        Some(old_frequency) if new_frequency >= old_frequency => {
            set_hclk_divider(config.hclk_divider);
            set_system_clock(config.system_clock);
        }
        Some(_) => {
            set_system_clock(config.system_clock);
            set_hclk_divider(config.hclk_divider);
        }
        None => {
            // PLL setup/frequency is not specified by the vendor RCC API.
            set_hclk_divider(HclkDivider::Div512);
            set_system_clock(config.system_clock);
            set_hclk_divider(config.hclk_divider);
        }
    }

    set_pclk_dividers(config.pclk0_divider, config.pclk1_divider);
}

fn modify_analog(register: usize, f: impl FnOnce(u32) -> u32) {
    critical_section::with(|_| unsafe {
        let address = (AFEC_ANALOG_BASE + register * size_of::<u32>()) as *mut u32;
        let value = core::ptr::read_volatile(address);
        core::ptr::write_volatile(address, f(value));
    });
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
    match oscillator {
        Oscillator::Rco48m => {
            modify_analog(AFEC_ANALOG_OSC_HIGH_SPEED, |value| value & !ANALOG_RCO48M_POWER_DOWN);
            wait_until(poll_limit, WaitTarget::Rco48m, || {
                afec().raw_sr().read().rco24m_ready().bit_is_set()
            })
        }
        Oscillator::Rco32k => {
            modify_analog(AFEC_ANALOG_OSC32K, |value| value & !ANALOG_RCO32K_POWER_DOWN);
            Ok(())
        }
        Oscillator::Xo32k => {
            modify_analog(AFEC_ANALOG_OSC32K, |value| value & !ANALOG_XO32K_POWER_DOWN);
            Ok(())
        }
        Oscillator::Xo24m => {
            modify_analog(AFEC_ANALOG_OSC_HIGH_SPEED, |value| {
                (value | ANALOG_XO24M_ENABLE) & !ANALOG_XO24M_POWER_DOWN
            });
            Ok(())
        }
        Oscillator::Xo32m => {
            set_peripheral_clock_raw(Peripheral::Lora, true, poll_limit)?;

            critical_section::with(|_| {
                lorac().cr1().modify(|r, w| {
                    let mut value = r.bits();
                    if value & LORAC_CR1_NRESET == 0 {
                        value |= LORAC_CR1_NRESET;
                        value &= !LORAC_CR1_POWER_ON_RESET;
                    }
                    if xo32m_uses_tcxo {
                        value |= LORAC_CR1_TCXO_ENABLE;
                    }
                    value |= LORAC_CR1_XO32M_ENABLE;
                    unsafe { w.bits(value) }
                });
            });

            wait_until(poll_limit, WaitTarget::Xo32m, || {
                lorac().sr().read().bits() & LORAC_SR_XO32M_READY != 0
            })
        }
        Oscillator::Rco4m => {
            modify_analog(AFEC_ANALOG_OSC_HIGH_SPEED, |value| value & !ANALOG_RCO4M_POWER_DOWN);
            wait_until(poll_limit, WaitTarget::Rco4m, || {
                afec().raw_sr().read().rco4m_ready().bit_is_set()
            })
        }
    }
}

/// RCC-controlled peripheral clock/reset identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Peripheral {
    Sac,
    Sec,
    Crc,
    Rtc,
    Wdg,
    Iwdg,
    Lptimer0,
    Bstimer1,
    Bstimer0,
    Timer3,
    Timer2,
    Timer1,
    Timer0,
    GpioA,
    GpioB,
    GpioC,
    GpioD,
    Lora,
    Dac,
    Lcd,
    Afec,
    Adc,
    I2c2,
    I2c1,
    I2c0,
    Qspi,
    Ssp2,
    Ssp1,
    Ssp0,
    Lpuart,
    Uart3,
    Uart2,
    Uart1,
    Uart0,
    Dma1,
    Dma0,
    I2s,
    Rng,
    Lptimer1,
    Syscfg,
    Pwr,
}

#[derive(Clone, Copy)]
enum ClockRegister {
    Cgr0,
    Cgr1,
    Cgr2,
}

fn modify_clock_register(register: ClockRegister, mask: u32, enable: bool) {
    critical_section::with(|_| {
        let rcc = rcc();
        match register {
            ClockRegister::Cgr0 => rcc
                .cgr0()
                .modify(|r, w| unsafe { w.bits(update_bits(r.bits(), mask, enable)) }),
            ClockRegister::Cgr1 => rcc
                .cgr1()
                .modify(|r, w| unsafe { w.bits(update_bits(r.bits(), mask, enable)) }),
            ClockRegister::Cgr2 => rcc
                .cgr2()
                .modify(|r, w| unsafe { w.bits(update_bits(r.bits(), mask, enable)) }),
        };
    });
}

fn basic_clock(peripheral: Peripheral) -> Option<(ClockRegister, u32)> {
    let value = match peripheral {
        Peripheral::Timer3 => (ClockRegister::Cgr0, 1 << 0),
        Peripheral::Timer2 => (ClockRegister::Cgr0, 1 << 1),
        Peripheral::Timer1 => (ClockRegister::Cgr0, 1 << 2),
        Peripheral::Timer0 => (ClockRegister::Cgr0, 1 << 3),
        Peripheral::Lora => (ClockRegister::Cgr0, 1 << 4),
        Peripheral::Dac => (ClockRegister::Cgr0, 1 << 5),
        Peripheral::Afec => (ClockRegister::Cgr0, 1 << 7),
        Peripheral::Adc => (ClockRegister::Cgr0, 1 << 8),
        Peripheral::I2c2 => (ClockRegister::Cgr0, 1 << 10),
        Peripheral::I2c1 => (ClockRegister::Cgr0, 1 << 11),
        Peripheral::I2c0 => (ClockRegister::Cgr0, 1 << 12),
        Peripheral::Ssp2 => (ClockRegister::Cgr0, 1 << 13),
        Peripheral::Ssp1 => (ClockRegister::Cgr0, 1 << 14),
        Peripheral::Ssp0 => (ClockRegister::Cgr0, 1 << 15),
        Peripheral::Uart3 => (ClockRegister::Cgr0, 1 << 17),
        Peripheral::Uart2 => (ClockRegister::Cgr0, 1 << 18),
        Peripheral::Uart1 => (ClockRegister::Cgr0, 1 << 19),
        Peripheral::Uart0 => (ClockRegister::Cgr0, 1 << 20),
        Peripheral::Syscfg => (ClockRegister::Cgr0, 1 << 21),
        Peripheral::GpioD => (ClockRegister::Cgr0, 1 << 22),
        Peripheral::GpioC => (ClockRegister::Cgr0, 1 << 23),
        Peripheral::GpioB => (ClockRegister::Cgr0, 1 << 24),
        Peripheral::GpioA => (ClockRegister::Cgr0, 1 << 25),
        Peripheral::Bstimer1 => (ClockRegister::Cgr0, 1 << 26),
        Peripheral::Bstimer0 => (ClockRegister::Cgr0, 1 << 27),
        Peripheral::Crc => (ClockRegister::Cgr0, 1 << 28),
        Peripheral::Dma1 => (ClockRegister::Cgr0, 1 << 29),
        Peripheral::Dma0 => (ClockRegister::Cgr0, 1 << 30),
        Peripheral::Pwr => (ClockRegister::Cgr0, 1 << 31),
        Peripheral::Sec => (ClockRegister::Cgr1, 1 << 0),
        Peripheral::Qspi => (ClockRegister::Cgr1, 1 << 5),
        Peripheral::Sac => (ClockRegister::Cgr1, 1 << 7),
        Peripheral::I2s => (ClockRegister::Cgr1, 1 << 8),
        Peripheral::Rng => (ClockRegister::Cgr1, 1 << 10),
        Peripheral::Wdg => (ClockRegister::Cgr1, (1 << 2) | (1 << 6)),
        Peripheral::Rtc
        | Peripheral::Iwdg
        | Peripheral::Lptimer0
        | Peripheral::Lptimer1
        | Peripheral::Lcd
        | Peripheral::Lpuart => return None,
    };
    Some(value)
}

fn wait_rcc_status(mask: u32, poll_limit: u32) -> Result<(), Error> {
    wait_until(poll_limit, WaitTarget::ClockDomain, || {
        rcc().sr().read().bits() & mask == mask
    })
}

fn set_dual_domain_clock(
    functional_register: ClockRegister,
    functional_mask: u32,
    aon_mask: u32,
    enable: bool,
    poll_limit: u32,
) -> Result<(), Error> {
    modify_clock_register(functional_register, functional_mask, enable);
    wait_rcc_status(RCC_SR_ALL_DONE, poll_limit)?;
    modify_clock_register(ClockRegister::Cgr2, aon_mask, enable);
    wait_rcc_status(aon_mask, poll_limit)
}

fn set_lptimer_clock(
    functional_mask: u32,
    pclk_mask: u32,
    aon_mask: u32,
    enable: bool,
    poll_limit: u32,
) -> Result<(), Error> {
    if enable {
        modify_clock_register(ClockRegister::Cgr1, pclk_mask, true);
        wait_rcc_status(RCC_SR_ALL_DONE, poll_limit)?;
        modify_clock_register(ClockRegister::Cgr2, aon_mask, true);
        wait_rcc_status(aon_mask, poll_limit)?;
        modify_clock_register(ClockRegister::Cgr1, functional_mask, true);
    } else {
        modify_clock_register(ClockRegister::Cgr1, functional_mask, false);
        wait_rcc_status(RCC_SR_ALL_DONE, poll_limit)?;
        modify_clock_register(ClockRegister::Cgr2, aon_mask, false);
        wait_rcc_status(aon_mask, poll_limit)?;
        // The vendor intentionally leaves the PCLK gate enabled on disable.
    }
    Ok(())
}

fn set_peripheral_clock_raw(peripheral: Peripheral, enable: bool, poll_limit: u32) -> Result<(), Error> {
    match peripheral {
        Peripheral::Lpuart => set_dual_domain_clock(ClockRegister::Cgr0, 1 << 16, 1 << 2, enable, poll_limit),
        Peripheral::Lcd => set_dual_domain_clock(ClockRegister::Cgr0, 1 << 6, 1 << 3, enable, poll_limit),
        Peripheral::Rtc => set_dual_domain_clock(ClockRegister::Cgr1, 1 << 1, 1 << 1, enable, poll_limit),
        Peripheral::Iwdg => set_dual_domain_clock(ClockRegister::Cgr1, 1 << 3, 1 << 0, enable, poll_limit),
        Peripheral::Lptimer0 => set_lptimer_clock(1 << 4, 1 << 9, 1 << 4, enable, poll_limit),
        Peripheral::Lptimer1 => set_lptimer_clock(1 << 11, 1 << 12, 1 << 5, enable, poll_limit),
        _ => {
            let (register, mask) = basic_clock(peripheral).unwrap();
            modify_clock_register(register, mask, enable);
            Ok(())
        }
    }
}

/// Enable a peripheral clock using the default bounded synchronization budget.
pub(crate) fn enable_peripheral(peripheral: Peripheral) -> Result<(), Error> {
    set_peripheral_clock_raw(peripheral, true, Config::new().readiness_poll_limit)
}

/// Disable a peripheral clock using the default bounded synchronization budget.
pub(crate) fn disable_peripheral(peripheral: Peripheral) -> Result<(), Error> {
    set_peripheral_clock_raw(peripheral, false, Config::new().readiness_poll_limit)
}

#[derive(Clone, Copy)]
enum ResetRegister {
    Rst0,
    Rst1,
}

fn reset_bit(peripheral: Peripheral) -> Result<(ResetRegister, u32), Error> {
    let bit = match peripheral {
        Peripheral::Sac => (ResetRegister::Rst0, 0),
        Peripheral::Sec => (ResetRegister::Rst0, 1),
        Peripheral::Crc => (ResetRegister::Rst0, 2),
        Peripheral::Rtc => (ResetRegister::Rst0, 3),
        Peripheral::Wdg => (ResetRegister::Rst0, 4),
        Peripheral::Iwdg => (ResetRegister::Rst0, 5),
        Peripheral::Lptimer0 => (ResetRegister::Rst0, 6),
        Peripheral::Bstimer1 => (ResetRegister::Rst0, 7),
        Peripheral::Bstimer0 => (ResetRegister::Rst0, 8),
        Peripheral::Timer3 => (ResetRegister::Rst0, 9),
        Peripheral::Timer2 => (ResetRegister::Rst0, 10),
        Peripheral::Timer1 => (ResetRegister::Rst0, 11),
        Peripheral::Timer0 => (ResetRegister::Rst0, 12),
        Peripheral::GpioA | Peripheral::GpioB | Peripheral::GpioC | Peripheral::GpioD => (ResetRegister::Rst0, 13),
        Peripheral::Lora => (ResetRegister::Rst0, 14),
        Peripheral::Dac => (ResetRegister::Rst0, 15),
        Peripheral::Lcd => (ResetRegister::Rst0, 16),
        Peripheral::Afec => (ResetRegister::Rst0, 17),
        Peripheral::Adc => (ResetRegister::Rst0, 18),
        Peripheral::I2c2 => (ResetRegister::Rst0, 20),
        Peripheral::I2c1 => (ResetRegister::Rst0, 21),
        Peripheral::I2c0 => (ResetRegister::Rst0, 22),
        Peripheral::Qspi => (ResetRegister::Rst0, 23),
        Peripheral::Ssp2 => (ResetRegister::Rst0, 24),
        Peripheral::Ssp1 => (ResetRegister::Rst0, 25),
        Peripheral::Ssp0 => (ResetRegister::Rst0, 26),
        Peripheral::Lpuart => (ResetRegister::Rst0, 27),
        Peripheral::Uart3 => (ResetRegister::Rst0, 28),
        Peripheral::Uart2 => (ResetRegister::Rst0, 29),
        Peripheral::Uart1 => (ResetRegister::Rst0, 30),
        Peripheral::Uart0 => (ResetRegister::Rst0, 31),
        Peripheral::Dma1 => (ResetRegister::Rst1, 0),
        Peripheral::Dma0 => (ResetRegister::Rst1, 1),
        Peripheral::I2s => (ResetRegister::Rst1, 2),
        Peripheral::Rng => (ResetRegister::Rst1, 3),
        Peripheral::Lptimer1 => (ResetRegister::Rst1, 4),
        Peripheral::Syscfg | Peripheral::Pwr => {
            return Err(Error::ResetUnsupported(peripheral));
        }
    };
    Ok((bit.0, 1 << bit.1))
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

fn set_reset_line(peripheral: Peripheral, asserted: bool) -> Result<(), Error> {
    let (register, mask) = reset_bit(peripheral)?;
    critical_section::with(|_| {
        let rcc = rcc();
        match register {
            ResetRegister::Rst0 => rcc
                .rst0()
                .modify(|r, w| unsafe { w.bits(update_bits(r.bits(), mask, !asserted)) }),
            ResetRegister::Rst1 => rcc
                .rst1()
                .modify(|r, w| unsafe { w.bits(update_bits(r.bits(), mask, !asserted)) }),
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

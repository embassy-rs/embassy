//! Clock configuration for the `RT6xx`
use core::sync::atomic::{AtomicU32, AtomicU8, Ordering};

#[cfg(feature = "defmt")]
use defmt;
use paste::paste;

use crate::pac;

/// Clock configuration;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Clocks {
    /// Low power oscillator
    Lposc,
    /// System Frequency Resonance Oscillator (SFRO)
    Sfro,
    /// Real Time Clock
    Rtc,
    /// Feed-forward Ring Oscillator
    Ffro, // This includes that div2 and div4 variations
    /// External Clock Input
    ClkIn,
    /// AHB Clock
    Hclk,
    /// Main Clock
    MainClk,
    /// Main PLL Clock
    MainPllClk, // also has aux0,aux1,dsp, and audio pll's downstream
    /// System Clock
    SysClk,
    /// System Oscillator
    SysOscClk,
    /// ADC Clock
    Adc,
}

/// Clock configuration.
pub struct ClockConfig {
    /// low-power oscillator config
    pub lposc: LposcConfig,
    /// 16Mhz internal oscillator config
    pub sfro: SfroConfig,
    /// Real Time Clock config
    pub rtc: RtcClkConfig,
    /// 48/60 Mhz internal oscillator config
    pub ffro: FfroConfig,
    // pub pll: Option<PllPfdConfig>, //potentially covered in main pll clk
    /// External Clock-In config
    pub clk_in: ClkInConfig,
    /// AHB bus clock config
    pub hclk: HclkConfig,
    /// Main Clock config
    pub main_clk: MainClkConfig,
    /// Main Pll clock config
    pub main_pll_clk: MainPllClkConfig,
    /// Software concept to be used with systick, doesn't map to a register
    pub sys_clk: SysClkConfig,
    /// System Oscillator Config
    pub sys_osc: SysOscConfig,
    // todo: move ADC here
}

impl ClockConfig {
    /// Clock configuration derived from external crystal.
    #[must_use]
    pub fn crystal() -> Self {
        const CORE_CPU_FREQ: u32 = 500_000_000;
        const PLL_CLK_FREQ: u32 = 528_000_000;
        const SYS_CLK_FREQ: u32 = CORE_CPU_FREQ / 2;
        Self {
            lposc: LposcConfig {
                state: State::Enabled,
                freq: AtomicU32::new(Into::into(LposcFreq::Lp1m)),
            },
            sfro: SfroConfig { state: State::Enabled },
            rtc: RtcClkConfig {
                state: State::Enabled,
                wake_alarm_state: State::Disabled,
                sub_second_state: State::Disabled,
                freq: AtomicU32::new(Into::into(RtcFreq::Default1Hz)),
                rtc_int: RtcInterrupts::None,
            },
            ffro: FfroConfig {
                state: State::Enabled,
                freq: AtomicU32::new(Into::into(FfroFreq::Ffro48m)),
            },
            //pll: Some(PllConfig {}),//includes aux0 and aux1 pll
            clk_in: ClkInConfig {
                state: State::Disabled,
                // This is an externally sourced clock
                // Don't give it an initial frequency
                freq: Some(AtomicU32::new(0)),
            },
            hclk: HclkConfig { state: State::Disabled },
            main_clk: MainClkConfig {
                state: State::Enabled,
                //FFRO divided by 4 is reset values of Main Clk Sel A, Sel B
                src: MainClkSrc::FFRO,
                div_int: AtomicU32::new(4),
                freq: AtomicU32::new(CORE_CPU_FREQ),
            },
            main_pll_clk: MainPllClkConfig {
                state: State::Enabled,
                src: MainPllClkSrc::SFRO,
                freq: AtomicU32::new(PLL_CLK_FREQ),
                mult: AtomicU8::new(16),
                pfd0: 19, //
                pfd1: 0,  // future field
                pfd2: 19, // 0x13
                pfd3: 0,  // future field
                aux0_div: 0,
                aux1_div: 0,
            },
            sys_clk: SysClkConfig {
                sysclkfreq: AtomicU32::new(SYS_CLK_FREQ),
            },
            sys_osc: SysOscConfig { state: State::Enabled },
            //adc: Some(AdcConfig {}), // TODO: add config
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// Clock state enum
pub enum State {
    /// Clock is enabled
    Enabled,
    /// Clock is disabled
    Disabled,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// Low Power Oscillator valid frequencies
pub enum LposcFreq {
    /// 1 `MHz` oscillator
    Lp1m,
    /// 32kHz oscillator
    Lp32k,
}

impl From<LposcFreq> for u32 {
    fn from(value: LposcFreq) -> Self {
        match value {
            LposcFreq::Lp1m => 1_000_000,
            LposcFreq::Lp32k => 32_768,
        }
    }
}

impl TryFrom<u32> for LposcFreq {
    type Error = ClockError;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            1_000_000 => Ok(LposcFreq::Lp1m),
            32_768 => Ok(LposcFreq::Lp32k),
            _ => Err(ClockError::InvalidFrequency),
        }
    }
}

/// Low power oscillator config
pub struct LposcConfig {
    state: State,
    // low power osc
    freq: AtomicU32,
}

const SFRO_FREQ: u32 = 16_000_000;
/// SFRO config
pub struct SfroConfig {
    state: State,
}

/// Valid RTC frequencies
pub enum RtcFreq {
    /// "Alarm" aka 1Hz clock
    Default1Hz,
    /// "Wake" aka 1kHz clock
    HighResolution1khz,
    /// 32kHz clock
    SubSecond32kHz,
}

impl From<RtcFreq> for u32 {
    fn from(value: RtcFreq) -> Self {
        match value {
            RtcFreq::Default1Hz => 1,
            RtcFreq::HighResolution1khz => 1_000,
            RtcFreq::SubSecond32kHz => 32_768,
        }
    }
}

impl TryFrom<u32> for RtcFreq {
    type Error = ClockError;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(RtcFreq::Default1Hz),
            1_000 => Ok(RtcFreq::HighResolution1khz),
            32_768 => Ok(RtcFreq::SubSecond32kHz),
            _ => Err(ClockError::InvalidFrequency),
        }
    }
}

/// RTC Interrupt options
pub enum RtcInterrupts {
    /// No interrupts are set
    None,
    /// 1Hz RTC clock aka Alarm interrupt set
    Alarm,
    /// 1kHz RTC clock aka Wake interrupt set
    Wake,
}

impl From<RtcInterrupts> for u8 {
    fn from(value: RtcInterrupts) -> Self {
        match value {
            RtcInterrupts::None => 0b00,
            RtcInterrupts::Alarm => 0b01,
            RtcInterrupts::Wake => 0b10,
        }
    }
}
/// RTC clock config.
pub struct RtcClkConfig {
    /// 1 Hz Clock state
    pub state: State,
    /// 1kHz Clock state
    pub wake_alarm_state: State,
    /// 32kHz Clock state
    pub sub_second_state: State,
    /// RTC clock source.
    pub freq: AtomicU32,
    /// RTC Interrupt
    pub rtc_int: RtcInterrupts,
}

/// Valid FFRO Frequencies
pub enum FfroFreq {
    /// 48 Mhz Internal Oscillator
    Ffro48m,
    /// 60 `MHz` Internal Oscillator
    Ffro60m,
}

/// FFRO Clock Config
pub struct FfroConfig {
    /// FFRO Clock state
    state: State,
    /// FFRO Frequency
    freq: AtomicU32,
}

impl From<FfroFreq> for u32 {
    fn from(value: FfroFreq) -> Self {
        match value {
            FfroFreq::Ffro48m => 48_000_000,
            FfroFreq::Ffro60m => 60_000_000,
        }
    }
}

impl TryFrom<u32> for FfroFreq {
    type Error = ClockError;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            48_000_000 => Ok(FfroFreq::Ffro48m),
            60_000_000 => Ok(FfroFreq::Ffro60m),
            _ => Err(ClockError::InvalidFrequency),
        }
    }
}

/// PLL clock source
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum MainPllClkSrc {
    /// SFRO
    SFRO,
    /// External Clock
    ClkIn,
    /// FFRO
    FFRO,
}

/// Transform from Source Clock enum to Clocks
impl From<MainPllClkSrc> for Clocks {
    fn from(value: MainPllClkSrc) -> Self {
        match value {
            MainPllClkSrc::SFRO => Clocks::Sfro,
            MainPllClkSrc::ClkIn => Clocks::ClkIn,
            MainPllClkSrc::FFRO => Clocks::Ffro,
        }
    }
}

impl TryFrom<Clocks> for MainPllClkSrc {
    type Error = ClockError;
    fn try_from(value: Clocks) -> Result<Self, Self::Error> {
        match value {
            Clocks::Sfro => Ok(MainPllClkSrc::SFRO),
            Clocks::Ffro => Ok(MainPllClkSrc::FFRO),
            Clocks::ClkIn => Ok(MainPllClkSrc::ClkIn),
            _ => Err(ClockError::ClockNotSupported),
        }
    }
}

/// PLL configuration.
pub struct MainPllClkConfig {
    /// Clock active state
    pub state: State,
    /// Main clock source.
    pub src: MainPllClkSrc,
    /// Main clock frequency
    pub freq: AtomicU32,
    //TODO: numerator and denominator not used but present in register
    /// Multiplication factor.
    pub mult: AtomicU8,
    // the following are actually 6-bits not 8
    /// Fractional divider 0, main pll clock
    pub pfd0: u8,
    /// Fractional divider 1
    pub pfd1: u8,
    /// Fractional divider 2
    pub pfd2: u8,
    /// Fractional divider 3
    pub pfd3: u8,
    // Aux dividers
    /// aux divider 0
    pub aux0_div: u8,
    /// aux divider 1
    pub aux1_div: u8,
}
/// External input clock config
pub struct ClkInConfig {
    /// External clock input state
    state: State,
    /// External clock input rate
    freq: Option<AtomicU32>,
}

/// AHB clock config
pub struct HclkConfig {
    /// divider to turn main clk into hclk for AHB bus
    pub state: State,
}

/// Main clock source.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum MainClkSrc {
    /// FFRO divided by 4
    FFROdiv4, // probably don't need since it'll be covered by div_int
    /// External Clock
    ClkIn,
    /// Low Power Oscillator
    Lposc,
    /// FFRO
    FFRO,
    /// SFRO
    SFRO,
    /// Main PLL Clock
    PllMain,
    /// RTC 32kHz oscillator.
    RTC32k,
}

impl From<MainClkSrc> for Clocks {
    fn from(value: MainClkSrc) -> Self {
        match value {
            MainClkSrc::ClkIn => Clocks::ClkIn,
            MainClkSrc::Lposc => Clocks::Lposc,
            MainClkSrc::FFRO => Clocks::Ffro,
            MainClkSrc::SFRO => Clocks::Sfro,
            MainClkSrc::PllMain => Clocks::MainPllClk,
            MainClkSrc::RTC32k => Clocks::Rtc,
            MainClkSrc::FFROdiv4 => Clocks::Ffro,
        }
    }
}

impl TryFrom<Clocks> for MainClkSrc {
    type Error = ClockError;
    fn try_from(value: Clocks) -> Result<Self, Self::Error> {
        match value {
            Clocks::ClkIn => Ok(MainClkSrc::ClkIn),
            Clocks::Lposc => Ok(MainClkSrc::Lposc),
            Clocks::Sfro => Ok(MainClkSrc::SFRO),
            Clocks::MainPllClk => Ok(MainClkSrc::PllMain),
            Clocks::Rtc => Ok(MainClkSrc::RTC32k),
            Clocks::Ffro => Ok(MainClkSrc::FFRO),
            _ => Err(ClockError::ClockNotSupported),
        }
    }
}

/// Main clock config.
pub struct MainClkConfig {
    /// Main clock state
    pub state: State,
    /// Main clock source.
    pub src: MainClkSrc,
    /// Main clock divider.
    pub div_int: AtomicU32,
    /// Clock Frequency
    pub freq: AtomicU32,
}

/// System Core Clock config, SW concept for systick
pub struct SysClkConfig {
    /// keeps track of the system core clock frequency
    /// future use with systick
    pub sysclkfreq: AtomicU32,
}

/// System Oscillator Config
pub struct SysOscConfig {
    /// Clock State
    pub state: State,
}
const SYS_OSC_DEFAULT_FREQ: u32 = 24_000_000;

/// Clock Errors
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ClockError {
    /// Error due to attempting to change a clock with the wrong config block
    ClockMismatch,
    /// Error due to attempting to modify a clock that's not yet been enabled
    ClockNotEnabled,
    /// Error due to attempting to set a clock source that's not a supported option
    ClockNotSupported,
    /// Error due to attempting to set a clock to an invalid frequency
    InvalidFrequency,
    /// Error due to attempting to modify a clock output with an invalid divider
    InvalidDiv,
    /// Error due to attempting to modify a clock output with an invalid multiplier
    InvalidMult,
}

/// Trait to configure one of the clocks
pub trait ConfigurableClock {
    /// Reset the clock, will enable it
    fn disable(&self) -> Result<(), ClockError>;
    /// Enable the clock
    fn enable_and_reset(&self) -> Result<(), ClockError>;
    /// Return the clock rate (Hz)
    fn get_clock_rate(&self) -> Result<u32, ClockError>;
    /// Set the desired clock rate (Hz)
    fn set_clock_rate(&mut self, div: u8, mult: u8, freq: u32) -> Result<(), ClockError>;
    /// Returns whether this clock is enabled
    fn is_enabled(&self) -> bool;
}

impl LposcConfig {
    /// Initializes low-power oscillator.
    fn init_lposc() {
        // Enable low power oscillator
        // SAFETY: unsafe needed to take pointer to Sysctl0, only happens once during init
        let sysctl0 = unsafe { crate::pac::Sysctl0::steal() };
        sysctl0.pdruncfg0_clr().write(|w| w.lposc_pd().clr_pdruncfg0());

        // Wait for low-power oscillator to be ready (typically 64 us)
        // Busy loop seems better here than trying to shoe-in an async delay
        // SAFETY: unsafe needed to take pointer to Clkctl0, needed to validate HW is ready
        let clkctl0 = unsafe { crate::pac::Clkctl0::steal() };
        while clkctl0.lposcctl0().read().clkrdy().bit_is_clear() {}
    }
}
impl ConfigurableClock for LposcConfig {
    fn enable_and_reset(&self) -> Result<(), ClockError> {
        LposcConfig::init_lposc();
        Ok(())
    }
    fn disable(&self) -> Result<(), ClockError> {
        // SAFETY: unsafe needed to take pointer to Sysctl0, needed to power down the LPOSC HW
        let sysctl0 = unsafe { crate::pac::Sysctl0::steal() };
        sysctl0.pdruncfg0_set().write(|w| w.lposc_pd().set_pdruncfg0());
        // Wait until LPOSC disabled
        while !sysctl0.pdruncfg0().read().lposc_pd().is_power_down() {}
        Ok(())
    }
    fn get_clock_rate(&self) -> Result<u32, ClockError> {
        Ok(self.freq.load(Ordering::Relaxed))
    }
    fn set_clock_rate(&mut self, _div: u8, _mult: u8, freq: u32) -> Result<(), ClockError> {
        if let Ok(r) = <u32 as TryInto<LposcFreq>>::try_into(freq) {
            match r {
                LposcFreq::Lp1m => {
                    self.freq
                        .store(LposcFreq::Lp1m as u32, core::sync::atomic::Ordering::Relaxed);
                    Ok(())
                }
                LposcFreq::Lp32k => {
                    self.freq
                        .store(LposcFreq::Lp1m as u32, core::sync::atomic::Ordering::Relaxed);
                    Ok(())
                }
            }
        } else {
            error!("failed to convert desired clock rate, {:#}, to LPOSC Freq", freq);
            Err(ClockError::InvalidFrequency)
        }
    }
    fn is_enabled(&self) -> bool {
        self.state == State::Enabled
    }
}

impl FfroConfig {
    /// Necessary register writes to initialize the FFRO clock
    pub fn init_ffro_clk() {
        // SAFETY: unsafe needed to take pointer to Sysctl0, only to power up FFRO
        let sysctl0 = unsafe { crate::pac::Sysctl0::steal() };

        /* Power on FFRO (48/60MHz) */
        sysctl0.pdruncfg0_clr().write(|w| w.ffro_pd().clr_pdruncfg0());

        // SAFETY: unsafe needed to take pointer to Clkctl0, only to set proper ffro update mode
        let clkctl0 = unsafe { crate::pac::Clkctl0::steal() };

        clkctl0.ffroctl1().write(|w| w.update().normal_mode());

        // No FFRO enable/disable control in CLKCTL.
        // Delay enough for FFRO to be stable in case it was just powered on
        delay_loop_clocks(50, 12_000_000);
    }
}

impl ConfigurableClock for FfroConfig {
    fn enable_and_reset(&self) -> Result<(), ClockError> {
        // SAFETY: should be called once
        FfroConfig::init_ffro_clk();
        // default is 48 MHz
        Ok(())
    }
    fn disable(&self) -> Result<(), ClockError> {
        // SAFETY: unsafe needed to take pointer to Sysctl0, only to power down FFRO
        let sysctl0 = unsafe { crate::pac::Sysctl0::steal() };
        sysctl0.pdruncfg0_set().write(|w| w.ffro_pd().set_pdruncfg0());
        delay_loop_clocks(30, 12_000_000);
        // Wait until FFRO disabled
        while !sysctl0.pdruncfg0().read().ffro_pd().is_power_down() {}
        Ok(())
    }
    fn get_clock_rate(&self) -> Result<u32, ClockError> {
        trace!("getting ffro clock rate");
        Ok(self.freq.load(Ordering::Relaxed))
    }
    fn set_clock_rate(&mut self, _div: u8, _mult: u8, freq: u32) -> Result<(), ClockError> {
        if let Ok(r) = <u32 as TryInto<FfroFreq>>::try_into(freq) {
            match r {
                FfroFreq::Ffro48m => {
                    // SAFETY: unsafe needed to take pointer to Clkctl0, needed to set the right HW frequency
                    let clkctl0 = unsafe { crate::pac::Clkctl0::steal() };
                    clkctl0.ffroctl1().write(|w| w.update().update_safe_mode());
                    clkctl0.ffroctl0().write(|w| w.trim_range().ffro_48mhz());
                    clkctl0.ffroctl1().write(|w| w.update().normal_mode());

                    self.freq
                        .store(FfroFreq::Ffro48m as u32, core::sync::atomic::Ordering::Relaxed);
                    Ok(())
                }
                FfroFreq::Ffro60m => {
                    // SAFETY: unsafe needed to take pointer to Clkctl0, needed to set the right HW frequency
                    let clkctl0 = unsafe { crate::pac::Clkctl0::steal() };
                    clkctl0.ffroctl1().write(|w| w.update().update_safe_mode());
                    clkctl0.ffroctl0().write(|w| w.trim_range().ffro_60mhz());
                    clkctl0.ffroctl1().write(|w| w.update().normal_mode());

                    self.freq
                        .store(FfroFreq::Ffro60m as u32, core::sync::atomic::Ordering::Relaxed);
                    Ok(())
                }
            }
        } else {
            error!("failed to convert desired clock rate, {:#}, to FFRO Freq", freq);
            Err(ClockError::InvalidFrequency)
        }
    }
    fn is_enabled(&self) -> bool {
        self.state == State::Enabled
    }
}

impl ConfigurableClock for SfroConfig {
    fn enable_and_reset(&self) -> Result<(), ClockError> {
        // SAFETY: unsafe needed to take pointer to Sysctl0, only to power up SFRO
        let sysctl0 = unsafe { crate::pac::Sysctl0::steal() };
        sysctl0.pdruncfg0_clr().write(|w| w.sfro_pd().clr_pdruncfg0());
        // wait until ready
        while !sysctl0.pdruncfg0().read().sfro_pd().is_enabled() {}
        Ok(())
    }
    fn disable(&self) -> Result<(), ClockError> {
        // SAFETY: unsafe needed to take pointer to Sysctl0, only to power down SFRO
        let sysctl0 = unsafe { crate::pac::Sysctl0::steal() };
        sysctl0.pdruncfg0_set().write(|w| w.sfro_pd().set_pdruncfg0());
        delay_loop_clocks(30, 12_000_000);
        // Wait until SFRO disabled
        while !sysctl0.pdruncfg0().read().sfro_pd().is_power_down() {}
        Ok(())
    }
    fn get_clock_rate(&self) -> Result<u32, ClockError> {
        if self.state == State::Enabled {
            Ok(SFRO_FREQ)
        } else {
            Err(ClockError::ClockNotEnabled)
        }
    }
    fn set_clock_rate(&mut self, _div: u8, _mult: u8, freq: u32) -> Result<(), ClockError> {
        if self.state == State::Enabled {
            if freq == SFRO_FREQ {
                trace!("Sfro frequency is already set at 16MHz");
                Ok(())
            } else {
                Err(ClockError::InvalidFrequency)
            }
        } else {
            Err(ClockError::ClockNotEnabled)
        }
    }
    fn is_enabled(&self) -> bool {
        self.state == State::Enabled
    }
}

/// A Clock with multiple options for clock source
pub trait MultiSourceClock {
    /// Returns which clock is being used as the clock source and its rate
    fn get_clock_source_and_rate(&self, clock: &Clocks) -> Result<(Clocks, u32), ClockError>;
    /// Sets a specific clock source and its associated rate
    fn set_clock_source_and_rate(
        &mut self,
        clock_src_config: &mut impl ConfigurableClock,
        clock_src: &Clocks,
        rate: u32,
    ) -> Result<(), ClockError>;
}

impl MultiSourceClock for MainPllClkConfig {
    fn get_clock_source_and_rate(&self, clock: &Clocks) -> Result<(Clocks, u32), ClockError> {
        match clock {
            Clocks::MainPllClk => {
                let converted_clock = Clocks::from(self.src);
                Ok((converted_clock, self.freq.load(Ordering::Relaxed)))
            }
            _ => Err(ClockError::ClockMismatch),
        }
    }
    fn set_clock_source_and_rate(
        &mut self,
        clock_src_config: &mut impl ConfigurableClock,
        clock_src: &Clocks,
        rate: u32,
    ) -> Result<(), ClockError> {
        if let Ok(c) = <Clocks as TryInto<MainPllClkSrc>>::try_into(*clock_src) {
            match c {
                MainPllClkSrc::ClkIn => {
                    self.src = MainPllClkSrc::ClkIn;
                    // div mult and rate don't matter since this is an external clock
                    self.set_clock_rate(1, 1, rate)
                }
                MainPllClkSrc::FFRO => {
                    // FFRO Clock is divided by 2
                    let r = clock_src_config.get_clock_rate()?;
                    let base_rate = r / 2;
                    let m = MainPllClkConfig::calc_mult(rate, base_rate)?;

                    self.src = MainPllClkSrc::FFRO;
                    self.set_clock_rate(2, m, rate)
                }
                MainPllClkSrc::SFRO => {
                    if !clock_src_config.is_enabled() {
                        error!("Can't set SFRO as source for MainPll as it's not enabled");
                        return Err(ClockError::ClockNotEnabled);
                    }
                    // check if desired frequency is a valid multiple of 16m SFRO clock
                    let m = MainPllClkConfig::calc_mult(rate, SFRO_FREQ)?;
                    self.src = MainPllClkSrc::SFRO;
                    self.set_clock_rate(1, m, rate)
                }
            }
        } else {
            Err(ClockError::ClockNotSupported)
        }
    }
}

impl ConfigurableClock for MainPllClkConfig {
    fn enable_and_reset(&self) -> Result<(), ClockError> {
        MainPllClkConfig::init_syspll();

        MainPllClkConfig::init_syspll_pfd0(self.pfd0);

        MainPllClkConfig::init_syspll_pfd2(self.pfd2);
        Ok(())
    }
    fn disable(&self) -> Result<(), ClockError> {
        if self.is_enabled() {
            error!("Attempting to reset the Main Pll Clock, should be resetting its source");
            Err(ClockError::ClockNotSupported)
        } else {
            Err(ClockError::ClockNotEnabled)
        }
    }
    fn get_clock_rate(&self) -> Result<u32, ClockError> {
        if self.is_enabled() {
            let (_c, rate) = self.get_clock_source_and_rate(&Clocks::MainPllClk)?;
            Ok(rate)
        } else {
            Err(ClockError::ClockNotEnabled)
        }
    }
    fn set_clock_rate(&mut self, div: u8, mult: u8, freq: u32) -> Result<(), ClockError> {
        if self.is_enabled() {
            trace!("attempting to set main pll clock rate");
            // SAFETY: unsafe needed to take pointers to Sysctl0 and Clkctl0
            let clkctl0 = unsafe { crate::pac::Clkctl0::steal() };
            let sysctl0 = unsafe { crate::pac::Sysctl0::steal() };

            // Power down pll before changes
            sysctl0
                .pdruncfg0_set()
                .write(|w| w.syspllldo_pd().set_pdruncfg0().syspllana_pd().set_pdruncfg0());

            let desired_freq: u64 = self.freq.load(Ordering::Relaxed).into();

            match self.src {
                c if c == MainPllClkSrc::ClkIn || c == MainPllClkSrc::FFRO || c == MainPllClkSrc::SFRO => {
                    let mut base_rate;
                    match c {
                        MainPllClkSrc::ClkIn => {
                            clkctl0.syspll0clksel().write(|w| w.sel().sysxtal_clk());
                            let r = self.get_clock_rate()?;
                            base_rate = r;
                        }
                        MainPllClkSrc::FFRO => {
                            trace!("found FFRO as source, wait a bit");
                            delay_loop_clocks(1000, desired_freq);
                            match clkctl0.ffroctl0().read().trim_range().is_ffro_48mhz() {
                                true => base_rate = Into::into(FfroFreq::Ffro48m),
                                false => base_rate = Into::into(FfroFreq::Ffro60m),
                            }
                            trace!("found ffro rate to be: {:#}", base_rate);
                            if div == 2 {
                                trace!("dividing FFRO rate by 2");
                                clkctl0.syspll0clksel().write(|w| w.sel().ffro_div_2());
                                delay_loop_clocks(150, desired_freq);
                                base_rate /= 2;
                            } else {
                                return Err(ClockError::InvalidDiv);
                            }
                        }
                        MainPllClkSrc::SFRO => {
                            base_rate = SFRO_FREQ;
                            clkctl0.syspll0clksel().write(|w| w.sel().sfro_clk());
                        }
                    };
                    base_rate *= u32::from(mult);
                    trace!("calculated base rate at: {:#}", base_rate);
                    if base_rate != freq {
                        // make sure to power syspll back up before returning the error
                        error!("invalid frequency found, powering syspll back up before returning error. Check div and mult");
                        // Clear System PLL reset
                        clkctl0.syspll0ctl0().write(|w| w.reset().normal());
                        // Power up SYSPLL
                        sysctl0
                            .pdruncfg0_clr()
                            .write(|w| w.syspllana_pd().clr_pdruncfg0().syspllldo_pd().clr_pdruncfg0());
                        return Err(ClockError::InvalidFrequency);
                    }
                    trace!("setting default num and denom");
                    // SAFETY: unsafe needed to write the bits for the num and demon fields
                    clkctl0.syspll0num().write(|w| unsafe { w.num().bits(0b0) });
                    clkctl0.syspll0denom().write(|w| unsafe { w.denom().bits(0b1) });
                    delay_loop_clocks(30, desired_freq);
                    self.mult.store(mult, Ordering::Relaxed);
                    trace!("setting self.mult as: {:#}", mult);
                    match mult {
                        16 => {
                            clkctl0.syspll0ctl0().modify(|_r, w| w.mult().div_16());
                        }
                        17 => {
                            clkctl0.syspll0ctl0().modify(|_r, w| w.mult().div_17());
                        }
                        20 => {
                            clkctl0.syspll0ctl0().modify(|_r, w| w.mult().div_20());
                        }
                        22 => {
                            clkctl0.syspll0ctl0().modify(|_r, w| w.mult().div_22());
                        }
                        27 => {
                            clkctl0.syspll0ctl0().modify(|_r, w| w.mult().div_27());
                        }
                        33 => {
                            clkctl0.syspll0ctl0().modify(|_r, w| w.mult().div_33());
                        }
                        _ => return Err(ClockError::InvalidMult),
                    }
                    trace!("clear syspll reset");
                    // Clear System PLL reset
                    clkctl0.syspll0ctl0().modify(|_r, w| w.reset().normal());
                    // Power up SYSPLL
                    sysctl0
                        .pdruncfg0_clr()
                        .write(|w| w.syspllana_pd().clr_pdruncfg0().syspllldo_pd().clr_pdruncfg0());

                    // Set System PLL HOLDRINGOFF_ENA
                    clkctl0.syspll0ctl0().modify(|_, w| w.holdringoff_ena().enable());
                    delay_loop_clocks(75, desired_freq);

                    // Clear System PLL HOLDRINGOFF_ENA
                    clkctl0.syspll0ctl0().modify(|_, w| w.holdringoff_ena().dsiable());
                    delay_loop_clocks(15, desired_freq);

                    trace!("setting new PFD0 bits");
                    // gate the output and clear bits.
                    // SAFETY: unsafe needed to write the bits for pfd0
                    clkctl0
                        .syspll0pfd()
                        .modify(|_, w| unsafe { w.pfd0_clkgate().gated().pfd0().bits(0x0) });
                    // set pfd bits and un-gate the clock output
                    // output is multiplied by syspll * 18/pfd0_bits
                    // SAFETY: unsafe needed to write the bits for pfd0
                    clkctl0
                        .syspll0pfd()
                        .modify(|_r, w| unsafe { w.pfd0_clkgate().not_gated().pfd0().bits(0x12) });
                    // wait for ready bit to be set
                    delay_loop_clocks(50, desired_freq);
                    trace!("waiting for mainpll clock to be ready");
                    while clkctl0.syspll0pfd().read().pfd0_clkrdy().bit_is_clear() {}
                    // clear by writing a 1
                    clkctl0.syspll0pfd().modify(|_, w| w.pfd0_clkrdy().set_bit());

                    Ok(())
                }
                _ => Err(ClockError::ClockNotSupported),
            }
        } else {
            Err(ClockError::ClockNotEnabled)
        }
    }
    fn is_enabled(&self) -> bool {
        self.state == State::Enabled
    }
}

impl MainPllClkConfig {
    /// Calculate the mult value of a desired frequency, return error if invalid
    pub(self) fn calc_mult(rate: u32, base_freq: u32) -> Result<u8, ClockError> {
        trace!("calculating mult for {:#} / {:#}", rate, base_freq);
        const VALIDMULTS: [u8; 6] = [16, 17, 20, 22, 27, 33];
        if rate > base_freq && rate % base_freq == 0 {
            let mult = (rate / base_freq) as u8;
            trace!("verifying that calculated mult {:#} is a valid one", mult);
            if VALIDMULTS.into_iter().any(|i| i == mult) {
                Ok(mult)
            } else {
                Err(ClockError::InvalidFrequency)
            }
        } else {
            Err(ClockError::InvalidFrequency)
        }
    }
    pub(self) fn init_syspll() {
        // SAFETY: unsafe needed to take pointers to Sysctl0 and Clkctl0
        let clkctl0 = unsafe { crate::pac::Clkctl0::steal() };
        let sysctl0 = unsafe { crate::pac::Sysctl0::steal() };

        // Power down SYSPLL before change fractional settings
        sysctl0
            .pdruncfg0_set()
            .write(|w| w.syspllldo_pd().set_pdruncfg0().syspllana_pd().set_pdruncfg0());

        clkctl0.syspll0clksel().write(|w| w.sel().ffro_div_2());
        // SAFETY: unsafe needed to write the bits for both num and denom
        clkctl0.syspll0num().write(|w| unsafe { w.num().bits(0x0) });
        clkctl0.syspll0denom().write(|w| unsafe { w.denom().bits(0x1) });

        // kCLOCK_SysPllMult22
        clkctl0.syspll0ctl0().modify(|_, w| w.mult().div_22());

        // Clear System PLL reset
        clkctl0.syspll0ctl0().modify(|_, w| w.reset().normal());

        // Power up SYSPLL
        sysctl0
            .pdruncfg0_clr()
            .write(|w| w.syspllldo_pd().clr_pdruncfg0().syspllana_pd().clr_pdruncfg0());
        delay_loop_clocks((150 & 0xFFFF) / 2, 12_000_000);

        // Set System PLL HOLDRINGOFF_ENA
        clkctl0.syspll0ctl0().modify(|_, w| w.holdringoff_ena().enable());
        delay_loop_clocks((150 & 0xFFFF) / 2, 12_000_000);

        // Clear System PLL HOLDRINGOFF_ENA
        clkctl0.syspll0ctl0().modify(|_, w| w.holdringoff_ena().dsiable());
        delay_loop_clocks((15 & 0xFFFF) / 2, 12_000_000);
    }
    /// enables default settings for pfd2 bits
    pub(self) fn init_syspll_pfd2(config_bits: u8) {
        // SAFETY: unsafe needed to take pointer to Clkctl0 and write specific bits
        // needed to change the output of pfd0
        unsafe {
            let clkctl0 = crate::pac::Clkctl0::steal();

            // Disable the clock output first.
            // SAFETY: unsafe needed to write the bits for pfd2
            clkctl0
                .syspll0pfd()
                .modify(|_, w| w.pfd2_clkgate().gated().pfd2().bits(0x0));

            // Set the new value and enable output.
            // SAFETY: unsafe needed to write the bits for pfd2
            clkctl0
                .syspll0pfd()
                .modify(|_, w| w.pfd2_clkgate().not_gated().pfd2().bits(config_bits));

            // Wait for output becomes stable.
            while clkctl0.syspll0pfd().read().pfd2_clkrdy().bit_is_clear() {}

            // Clear ready status flag.
            clkctl0.syspll0pfd().modify(|_, w| w.pfd2_clkrdy().clear_bit());
        }
    }
    /// Enables default settings for pfd0
    pub(self) fn init_syspll_pfd0(config_bits: u8) {
        // SAFETY: unsafe needed to take pointer to Clkctl0 and write specific bits
        // needed to change the output of pfd0
        unsafe {
            let clkctl0 = crate::pac::Clkctl0::steal();
            // Disable the clock output first
            clkctl0
                .syspll0pfd()
                .modify(|_, w| w.pfd0_clkgate().gated().pfd0().bits(0x0));

            // Set the new value and enable output
            clkctl0
                .syspll0pfd()
                .modify(|_, w| w.pfd0_clkgate().not_gated().pfd0().bits(config_bits));

            // Wait for output becomes stable
            while clkctl0.syspll0pfd().read().pfd0_clkrdy().bit_is_clear() {}

            // Clear ready status flag
            clkctl0.syspll0pfd().modify(|_, w| w.pfd0_clkrdy().clear_bit());
        }
    }
}

impl MainClkConfig {
    fn init_main_clk() {
        // SAFETY:: unsafe needed to take pointers to Clkctl0 and Clkctl1
        // used to set the right HW frequency
        let clkctl0 = unsafe { crate::pac::Clkctl0::steal() };
        let clkctl1 = unsafe { crate::pac::Clkctl1::steal() };

        clkctl0.mainclkselb().write(|w| w.sel().main_pll_clk());

        // Set PFC0DIV divider to value 2, Subtract 1 since 0-> 1, 1-> 2, etc...
        clkctl0.pfcdiv(0).modify(|_, w| w.reset().set_bit());
        // SAFETY: unsafe needed to write the bits for pfcdiv
        clkctl0
            .pfcdiv(0)
            .write(|w| unsafe { w.div().bits(2 - 1).halt().clear_bit() });
        while clkctl0.pfcdiv(0).read().reqflag().bit_is_set() {}

        // Set FRGPLLCLKDIV divider to value 12, Subtract 1 since 0-> 1, 1-> 2, etc...
        clkctl1.frgpllclkdiv().modify(|_, w| w.reset().set_bit());
        // SAFETY: unsafe needed to write the bits for frgpllclkdiv
        clkctl1
            .frgpllclkdiv()
            .write(|w| unsafe { w.div().bits(12 - 1).halt().clear_bit() });
        while clkctl1.frgpllclkdiv().read().reqflag().bit_is_set() {}
    }
}
impl MultiSourceClock for MainClkConfig {
    fn get_clock_source_and_rate(&self, clock: &Clocks) -> Result<(Clocks, u32), ClockError> {
        match clock {
            Clocks::MainClk => {
                let div: u32 = if self.src == MainClkSrc::FFROdiv4 { 4 } else { 1 };
                let converted_clock = Clocks::from(self.src);
                match ConfigurableClock::get_clock_rate(self) {
                    Ok(_rate) => {
                        // SAFETY: unsafe needed to take pointer to Clkctl0
                        // needed to calculate the clock rate from the bits written in the registers
                        let clkctl0 = unsafe { crate::pac::Clkctl0::steal() };
                        if self.src == MainClkSrc::PllMain && clkctl0.syspll0ctl0().read().bypass().is_programmed_clk()
                        {
                            let mut temp;
                            temp = self.freq.load(Ordering::Relaxed)
                                * u32::from(clkctl0.syspll0ctl0().read().mult().bits());
                            temp = (u64::from(temp) * 18 / u64::from(clkctl0.syspll0pfd().read().pfd0().bits())) as u32;
                            return Ok((converted_clock, temp));
                        }
                        Ok((converted_clock, self.freq.load(Ordering::Relaxed) / div))
                    }
                    Err(clk_err) => Err(clk_err),
                }
            }
            _ => Err(ClockError::ClockMismatch),
        }
    }
    fn set_clock_source_and_rate(
        &mut self,
        clock_src_config: &mut impl ConfigurableClock,
        clock_src: &Clocks,
        rate: u32,
    ) -> Result<(), ClockError> {
        if !clock_src_config.is_enabled() {
            return Err(ClockError::ClockNotEnabled);
        }
        if let Ok(c) = <Clocks as TryInto<MainClkSrc>>::try_into(*clock_src) {
            // SAFETY: unsafe needed to take pointer to Clkctl0
            // needed to change the clock source
            let clkctl0 = unsafe { crate::pac::Clkctl0::steal() };
            match c {
                MainClkSrc::ClkIn => {
                    self.src = MainClkSrc::ClkIn;

                    clkctl0.mainclksela().write(|w| w.sel().sysxtal_clk());
                    clkctl0.mainclkselb().write(|w| w.sel().main_1st_clk());
                    Ok(())
                }
                // the following will yield the same result as if compared to FFROdiv4
                MainClkSrc::FFRO | MainClkSrc::FFROdiv4 => match rate {
                    div4 if div4 == (FfroFreq::Ffro60m as u32) / 4 || div4 == (FfroFreq::Ffro48m as u32) / 4 => {
                        self.src = MainClkSrc::FFROdiv4;
                        self.freq.store(div4, Ordering::Relaxed);

                        clkctl0.mainclksela().write(|w| w.sel().ffro_div_4());
                        clkctl0.mainclkselb().write(|w| w.sel().main_1st_clk());
                        Ok(())
                    }
                    div1 if div1 == FfroFreq::Ffro60m as u32 || div1 == FfroFreq::Ffro48m as u32 => {
                        self.src = MainClkSrc::FFRO;
                        self.freq.store(div1, Ordering::Relaxed);

                        clkctl0.mainclksela().write(|w| w.sel().ffro_clk());
                        clkctl0.mainclkselb().write(|w| w.sel().main_1st_clk());
                        Ok(())
                    }
                    _ => Err(ClockError::InvalidFrequency),
                },
                MainClkSrc::Lposc => {
                    if let Ok(r) = <u32 as TryInto<LposcFreq>>::try_into(rate) {
                        match r {
                            LposcFreq::Lp1m => {
                                self.src = MainClkSrc::Lposc;
                                self.freq.store(rate, Ordering::Relaxed);

                                clkctl0.mainclksela().write(|w| w.sel().lposc());
                                clkctl0.mainclkselb().write(|w| w.sel().main_1st_clk());
                                Ok(())
                            }
                            LposcFreq::Lp32k => Err(ClockError::InvalidFrequency),
                        }
                    } else {
                        Err(ClockError::InvalidFrequency)
                    }
                }
                MainClkSrc::SFRO => {
                    if rate == SFRO_FREQ {
                        self.src = MainClkSrc::SFRO;
                        self.freq.store(rate, Ordering::Relaxed);
                        clkctl0.mainclkselb().write(|w| w.sel().sfro_clk());
                        Ok(())
                    } else {
                        Err(ClockError::InvalidFrequency)
                    }
                }
                MainClkSrc::PllMain => {
                    let r = rate;
                    // From Section 4.6.1.1 Pll Limitations of the RT6xx User manual
                    let pll_max = 572_000_000;
                    let pll_min = 80_000_000;
                    if pll_min <= r && r <= pll_max {
                        clkctl0.mainclkselb().write(|w| w.sel().main_pll_clk());
                        self.src = MainClkSrc::PllMain;
                        self.freq.store(r, Ordering::Relaxed);
                        Ok(())
                    } else {
                        Err(ClockError::InvalidFrequency)
                    }
                }
                MainClkSrc::RTC32k => {
                    if rate == RtcFreq::SubSecond32kHz as u32 {
                        self.src = MainClkSrc::RTC32k;
                        self.freq.store(rate, Ordering::Relaxed);
                        clkctl0.mainclkselb().write(|w| w.sel().rtc_32k_clk());
                        Ok(())
                    } else {
                        Err(ClockError::InvalidFrequency)
                    }
                }
            }
        } else {
            Err(ClockError::ClockNotSupported)
        }
    }
}

impl ConfigurableClock for MainClkConfig {
    fn enable_and_reset(&self) -> Result<(), ClockError> {
        MainClkConfig::init_main_clk();
        Ok(())
    }
    fn disable(&self) -> Result<(), ClockError> {
        error!("Attempting to reset the main clock, should NOT happen during runtime");
        Err(ClockError::ClockNotSupported)
    }
    fn get_clock_rate(&self) -> Result<u32, ClockError> {
        let (_c, rate) = MainClkConfig::get_clock_source_and_rate(self, &Clocks::MainClk)?;
        Ok(rate)
    }
    fn set_clock_rate(&mut self, _div: u8, _mult: u8, _freq: u32) -> Result<(), ClockError> {
        error!("The multi-source set_clock_rate_and_source method should be used instead of set_clock_rate");
        Err(ClockError::ClockNotSupported)
    }
    fn is_enabled(&self) -> bool {
        self.state == State::Enabled
    }
}

impl ConfigurableClock for ClkInConfig {
    fn enable_and_reset(&self) -> Result<(), ClockError> {
        // External Input, no hw writes needed
        Ok(())
    }
    fn disable(&self) -> Result<(), ClockError> {
        error!("Attempting to reset a clock input");
        Err(ClockError::ClockNotSupported)
    }
    fn get_clock_rate(&self) -> Result<u32, ClockError> {
        if self.freq.is_some() {
            Ok(self.freq.as_ref().unwrap().load(Ordering::Relaxed))
        } else {
            Err(ClockError::ClockNotEnabled)
        }
    }
    fn set_clock_rate(&mut self, _div: u8, _mult: u8, freq: u32) -> Result<(), ClockError> {
        trace!("Setting value of clk in config, this won't change the clock itself");
        self.freq.as_ref().unwrap().store(freq, Ordering::Relaxed);
        Ok(())
    }
    fn is_enabled(&self) -> bool {
        self.state == State::Enabled
    }
}

impl RtcClkConfig {
    /// Register writes to initialize the RTC Clock
    fn init_rtc_clk() {
        // SAFETY: unsafe needed to take pointer to Clkctl0, Clkctl1, and RTC
        // needed to enable the RTC HW
        let cc0 = unsafe { pac::Clkctl0::steal() };
        let cc1 = unsafe { pac::Clkctl1::steal() };
        let r = unsafe { pac::Rtc::steal() };
        // Enable the RTC peripheral clock
        cc1.pscctl2_set().write(|w| w.rtc_lite_clk_set().set_clock());
        // Make sure the reset bit is cleared amd RTC OSC is powered up
        r.ctrl().modify(|_, w| w.swreset().not_in_reset().rtc_osc_pd().enable());

        // set initial match value, note that with a 15 bit count-down timer this would
        // typically be 0x8000, but we are "doing some clever things" in time-driver.rs,
        // read more about it in the comments there
        // SAFETY: unsafe needed to write the bits
        r.wake().write(|w| unsafe { w.bits(0xA) });

        // Enable 32K OSC
        cc0.osc32khzctl0().write(|w| w.ena32khz().enabled());

        // enable rtc clk
        r.ctrl().modify(|_, w| w.rtc_en().enable());
    }
}

impl ConfigurableClock for RtcClkConfig {
    fn enable_and_reset(&self) -> Result<(), ClockError> {
        // should only be called once if previously disabled
        RtcClkConfig::init_rtc_clk();
        Ok(())
    }
    fn disable(&self) -> Result<(), ClockError> {
        error!("Resetting the RTC clock, this should NOT happen during runtime");
        Err(ClockError::ClockNotSupported)
    }
    fn set_clock_rate(&mut self, _div: u8, _mult: u8, freq: u32) -> Result<(), ClockError> {
        if let Ok(r) = <u32 as TryInto<RtcFreq>>::try_into(freq) {
            // SAFETY: unsafe needed to take pointer to RTC
            // needed to enable the HW for the different RTC frequencies, powered down by default
            let rtc = unsafe { crate::pac::Rtc::steal() };
            match r {
                RtcFreq::Default1Hz => {
                    if rtc.ctrl().read().rtc_en().is_enable() {
                        trace!("Attempting to enable an already enabled clock, RTC 1Hz");
                    } else {
                        rtc.ctrl().modify(|_r, w| w.rtc_en().enable());
                    }
                    Ok(())
                }
                RtcFreq::HighResolution1khz => {
                    if rtc.ctrl().read().rtc1khz_en().is_enable() {
                        trace!("Attempting to enable an already enabled clock, RTC 1Hz");
                    } else {
                        rtc.ctrl().modify(|_r, w| w.rtc1khz_en().enable());
                    }
                    Ok(())
                }
                RtcFreq::SubSecond32kHz => {
                    if rtc.ctrl().read().rtc_subsec_ena().is_enable() {
                        trace!("Attempting to enable an already enabled clock, RTC 1Hz");
                    } else {
                        rtc.ctrl().modify(|_r, w| w.rtc_subsec_ena().enable());
                    }
                    Ok(())
                }
            }
        } else {
            Err(ClockError::InvalidFrequency)
        }
    }
    // unlike the others, since this provides multiple clocks, return the fastest one
    fn get_clock_rate(&self) -> Result<u32, ClockError> {
        if self.sub_second_state == State::Enabled {
            Ok(RtcFreq::SubSecond32kHz as u32)
        } else if self.wake_alarm_state == State::Enabled {
            Ok(RtcFreq::HighResolution1khz as u32)
        } else if self.state == State::Enabled {
            Ok(RtcFreq::Default1Hz as u32)
        } else {
            Err(ClockError::ClockNotEnabled)
        }
    }
    fn is_enabled(&self) -> bool {
        self.state == State::Enabled
    }
}

impl SysClkConfig {
    /// Updates the system core clock frequency, SW concept used for systick
    fn update_sys_core_clock(&self) {
        trace!(
            "System core clock has been updated to {:?}, this involves no HW reg writes",
            self.sysclkfreq.load(Ordering::Relaxed)
        );
    }
}

impl ConfigurableClock for SysOscConfig {
    fn enable_and_reset(&self) -> Result<(), ClockError> {
        if self.state == State::Enabled {
            trace!("SysOsc was already enabled");
            return Ok(());
        }

        // SAFETY: unsafe needed to take pointers to Sysctl0 and Clkctl0, needed to modify clock HW
        let clkctl0 = unsafe { crate::pac::Clkctl0::steal() };
        let sysctl0 = unsafe { crate::pac::Sysctl0::steal() };

        // Let CPU run on ffro for safe switching
        clkctl0.mainclksela().write(|w| w.sel().ffro_clk());
        clkctl0.mainclksela().write(|w| w.sel().ffro_div_4());

        // Power on SYSXTAL
        sysctl0.pdruncfg0_clr().write(|w| w.sysxtal_pd().clr_pdruncfg0());

        // Enable system OSC
        clkctl0
            .sysoscctl0()
            .write(|w| w.lp_enable().lp().bypass_enable().normal_mode());

        delay_loop_clocks(260, SYS_OSC_DEFAULT_FREQ.into());
        Ok(())
    }
    fn disable(&self) -> Result<(), ClockError> {
        // SAFETY: unsafe needed to take pointers to Sysctl0 and Clkctl0, needed to modify clock HW
        let clkctl0 = unsafe { crate::pac::Clkctl0::steal() };
        let sysctl0 = unsafe { crate::pac::Sysctl0::steal() };

        // Let CPU run on ffro for safe switching
        clkctl0.mainclksela().write(|w| w.sel().ffro_clk());
        clkctl0.mainclksela().write(|w| w.sel().ffro_div_4());

        // Power on SYSXTAL
        sysctl0.pdruncfg0_set().write(|w| w.sysxtal_pd().set_pdruncfg0());
        Ok(())
    }
    fn get_clock_rate(&self) -> Result<u32, ClockError> {
        if self.state == State::Enabled {
            Ok(SYS_OSC_DEFAULT_FREQ)
        } else {
            Err(ClockError::ClockNotEnabled)
        }
    }
    fn is_enabled(&self) -> bool {
        self.state == State::Enabled
    }
    fn set_clock_rate(&mut self, _div: u8, _mult: u8, _freq: u32) -> Result<(), ClockError> {
        Err(ClockError::ClockNotSupported)
    }
}

/// Method to delay for a certain number of microseconds given a clock rate
pub fn delay_loop_clocks(usec: u64, freq_mhz: u64) {
    let mut ticks = usec * freq_mhz / 1_000_000 / 4;
    if ticks > u64::from(u32::MAX) {
        ticks = u64::from(u32::MAX);
    }
    // won't panic since we check value above
    cortex_m::asm::delay(ticks as u32);
}

/// Configure the pad voltage pmc registers for all 3 vddio ranges
fn set_pad_voltage_range() {
    // SAFETY: unsafe needed to take pointer to PNC as well as to write specific bits
    unsafe {
        let pmc = crate::pac::Pmc::steal();
        // Set up IO voltages
        // all 3 ranges need to be 1.71-1.98V which is 01
        pmc.padvrange().write(|w| {
            w.vddio_0range()
                .bits(0b01)
                .vddio_1range()
                .bits(0b01)
                .vddio_2range()
                .bits(0b01)
        });
    }
}

/// Initialize AHB clock
fn init_syscpuahb_clk() {
    // SAFETY: unsafe needed to take pointer to Clkctl0
    let clkctl0 = unsafe { crate::pac::Clkctl0::steal() };
    // SAFETY: unsafe needed to write the bits
    // Set syscpuahbclkdiv to value 2, Subtract 1 since 0-> 1, 1-> 2, etc...
    clkctl0.syscpuahbclkdiv().write(|w| unsafe { w.div().bits(2 - 1) });

    while clkctl0.syscpuahbclkdiv().read().reqflag().bit_is_set() {}
}

/// `ClockOut` config
pub struct ClockOutConfig {
    src: ClkOutSrc,
    div: u8,
}

/// `ClockOut` sources
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// `ClockOut` sources
pub enum ClkOutSrc {
    /// No Source, reduce power consumption
    None,
    /// SFRO clock
    Sfro,
    /// External input clock
    ClkIn,
    /// Low-power oscillator
    Lposc,
    /// FFRO clock
    Ffro,
    /// Main clock
    MainClk,
    /// Main DSP clock
    DspMainClk,
    /// Main Pll clock
    MainPllClk,
    /// `SysPll` Aux0 clock
    Aux0PllClk,
    /// `SysPll` DSP clock
    DspPllClk,
    /// `SysPll` Aux1 clock
    Aux1PllClk,
    /// Audio Pll clock
    AudioPllClk,
    /// 32 `KHz` RTC
    RTC32k,
}

/// Initialize the `ClkOutConfig`
impl ClockOutConfig {
    /// Default configuration for Clock out
    #[must_use]
    pub fn default_config() -> Self {
        Self {
            src: ClkOutSrc::None,
            div: 0,
        }
    }

    /// Enable the Clock Out output
    pub fn enable_and_reset(&mut self) -> Result<(), ClockError> {
        self.set_clkout_source_and_div(self.src, self.div)?;
        Ok(())
    }

    /// Disable Clock Out output and select None as the source to conserve power
    pub fn disable(&mut self) -> Result<(), ClockError> {
        self.set_clkout_source_and_div(ClkOutSrc::None, 0)?;
        Ok(())
    }

    /// Set the source of the Clock Out pin
    fn set_clkout_source(&mut self, src: ClkOutSrc) -> Result<(), ClockError> {
        // SAFETY: unsafe needed to take pointers to Clkctl1, needed to set source in HW
        let cc1 = unsafe { pac::Clkctl1::steal() };
        match src {
            ClkOutSrc::None => {
                cc1.clkoutsel0().write(|w| w.sel().none());
                cc1.clkoutsel1().write(|w| w.sel().none());
            }
            ClkOutSrc::Sfro => {
                cc1.clkoutsel0().write(|w| w.sel().sfro_clk());
                cc1.clkoutsel1().write(|w| w.sel().clkoutsel0_output());
            }
            ClkOutSrc::ClkIn => {
                cc1.clkoutsel0().write(|w| w.sel().xtalin_clk());
                cc1.clkoutsel1().write(|w| w.sel().clkoutsel0_output());
            }
            ClkOutSrc::Lposc => {
                cc1.clkoutsel0().write(|w| w.sel().lposc());
                cc1.clkoutsel1().write(|w| w.sel().clkoutsel0_output());
            }
            ClkOutSrc::Ffro => {
                cc1.clkoutsel0().write(|w| w.sel().ffro_clk());
                cc1.clkoutsel1().write(|w| w.sel().clkoutsel0_output());
            }
            ClkOutSrc::MainClk => {
                cc1.clkoutsel0().write(|w| w.sel().main_clk());
                cc1.clkoutsel1().write(|w| w.sel().clkoutsel0_output());
            }
            ClkOutSrc::DspMainClk => {
                cc1.clkoutsel0().write(|w| w.sel().dsp_main_clk());
                cc1.clkoutsel1().write(|w| w.sel().clkoutsel0_output());
            }
            ClkOutSrc::MainPllClk => {
                cc1.clkoutsel0().write(|w| w.sel().none());
                cc1.clkoutsel1().write(|w| w.sel().main_pll_clk());
            }
            ClkOutSrc::Aux0PllClk => {
                cc1.clkoutsel0().write(|w| w.sel().none());
                cc1.clkoutsel1().write(|w| w.sel().syspll0_aux0_pll_clk());
            }
            ClkOutSrc::DspPllClk => {
                cc1.clkoutsel0().write(|w| w.sel().none());
                cc1.clkoutsel1().write(|w| w.sel().dsp_pll_clk());
            }
            ClkOutSrc::AudioPllClk => {
                cc1.clkoutsel0().write(|w| w.sel().none());
                cc1.clkoutsel1().write(|w| w.sel().audio_pll_clk());
            }
            ClkOutSrc::Aux1PllClk => {
                cc1.clkoutsel0().write(|w| w.sel().none());
                cc1.clkoutsel1().write(|w| w.sel().syspll0_aux1_pll_clk());
            }
            ClkOutSrc::RTC32k => {
                cc1.clkoutsel0().write(|w| w.sel().none());
                cc1.clkoutsel1().write(|w| w.sel().rtc_clk_32khz());
            }
        }
        self.src = src;
        Ok(())
    }
    /// set the clock out divider
    /// note that 1 will be added to div when mapping to the divider
    /// so bits(0) -> divide by 1
    /// ...
    /// bits(255)-> divide by 256
    pub fn set_clkout_divider(&self, div: u8) -> Result<(), ClockError> {
        // don't wait for clock to be ready if there's no source
        if self.src != ClkOutSrc::None {
            let cc1 = unsafe { pac::Clkctl1::steal() };

            cc1.clkoutdiv()
                .modify(|_, w| unsafe { w.div().bits(div).halt().clear_bit() });
            while cc1.clkoutdiv().read().reqflag().bit_is_set() {}
        }
        Ok(())
    }
    /// set the source and divider for the clockout pin
    pub fn set_clkout_source_and_div(&mut self, src: ClkOutSrc, div: u8) -> Result<(), ClockError> {
        self.set_clkout_source(src)?;

        self.set_clkout_divider(div)?;

        Ok(())
    }
}

/// Using the config, enables all desired clocks to desired clock rates
fn init_clock_hw(config: ClockConfig) -> Result<(), ClockError> {
    if let Err(e) = config.rtc.enable_and_reset() {
        error!("couldn't Power on OSC for RTC, result: {:?}", e);
        return Err(e);
    }

    if let Err(e) = config.lposc.enable_and_reset() {
        error!("couldn't Power on LPOSC, result: {:?}", e);
        return Err(e);
    }

    if let Err(e) = config.ffro.enable_and_reset() {
        error!("couldn't Power on FFRO, result: {:?}", e);
        return Err(e);
    }

    if let Err(e) = config.sfro.enable_and_reset() {
        error!("couldn't Power on SFRO, result: {:?}", e);
        return Err(e);
    }

    if let Err(e) = config.sys_osc.enable_and_reset() {
        error!("Couldn't enable sys oscillator {:?}", e);
        return Err(e);
    }

    if let Err(e) = config.main_pll_clk.enable_and_reset() {
        error!("Couldn't enable main pll clock {:?}", e);
        return Err(e);
    }

    // Move FLEXSPI clock source from main clock to FFRO to avoid instruction/data fetch issue in XIP when
    // updating PLL and main clock.
    // SAFETY: unsafe needed to take pointers to Clkctl0
    let cc0 = unsafe { pac::Clkctl0::steal() };
    cc0.flexspifclksel().write(|w| w.sel().ffro_clk());

    // Move ESPI clock source to FFRO
    #[cfg(feature = "_espi")]
    {
        cc0.espiclksel().write(|w| w.sel().use_48_60m());
    }

    init_syscpuahb_clk();

    if let Err(e) = config.main_clk.enable_and_reset() {
        error!("Couldn't enable main clock {:?}", e);
        return Err(e);
    }

    config.sys_clk.update_sys_core_clock();
    Ok(())
}

/// SAFETY: must be called exactly once at bootup
pub(crate) unsafe fn init(config: ClockConfig) -> Result<(), ClockError> {
    init_clock_hw(config)?;

    // set VDDIO ranges 0-2
    set_pad_voltage_range();
    Ok(())
}

///Trait to expose perph clocks
trait SealedSysconPeripheral {
    fn enable_and_reset_perph_clock();
    fn disable_perph_clock();
}

/// Clock and Reset control for peripherals
#[allow(private_bounds)]
pub trait SysconPeripheral: SealedSysconPeripheral + 'static {}
/// Enables and resets peripheral `T`.
///
/// # Safety
///
/// Peripheral must not be in use.
pub fn enable_and_reset<T: SysconPeripheral>() {
    T::enable_and_reset_perph_clock();
}

/// Disables peripheral `T`.
///
/// # Safety
///
/// Peripheral must not be in use.
pub fn disable<T: SysconPeripheral>() {
    T::disable_perph_clock();
}
macro_rules! impl_perph_clk {
    ($peripheral:ident, $clkctl:ident, $clkreg:ident, $rstctl:ident, $rstreg:ident, $bit:expr) => {
        impl SealedSysconPeripheral for crate::peripherals::$peripheral {
            fn enable_and_reset_perph_clock() {
                // SAFETY: unsafe needed to take pointers to Rstctl1 and Clkctl1
                let cc1 = unsafe { pac::$clkctl::steal() };
                let rc1 = unsafe { pac::$rstctl::steal() };

                paste! {
                    // SAFETY: unsafe due to the use of bits()
                    cc1.[<$clkreg _set>]().write(|w| unsafe { w.bits(1 << $bit) });

                    // SAFETY: unsafe due to the use of bits()
                    rc1.[<$rstreg _clr>]().write(|w| unsafe { w.bits(1 << $bit) });
                }
            }

            fn disable_perph_clock() {
                // SAFETY: unsafe needed to take pointers to Rstctl1 and Clkctl1
                let cc1 = unsafe { pac::$clkctl::steal() };
                let rc1 = unsafe { pac::$rstctl::steal() };

                paste! {
                    // SAFETY: unsafe due to the use of bits()
                    rc1.[<$rstreg _set>]().write(|w| unsafe { w.bits(1 << $bit) });

                    // SAFETY: unsafe due to the use of bits()
                    cc1.[<$clkreg _clr>]().write(|w| unsafe { w.bits(1 << $bit) });
                }
            }
        }

        impl SysconPeripheral for crate::peripherals::$peripheral {}
    };
}

// These should enabled once the relevant peripherals are implemented.
// impl_perph_clk!(GPIOINTCTL, Clkctl1, pscctl2, Rstctl1, prstctl2, 30);
// impl_perph_clk!(OTP, Clkctl0, pscctl0, Rstctl0, prstctl0, 17);

// impl_perph_clk!(ROM_CTL_128KB, Clkctl0, pscctl0, Rstctl0, prstctl0, 2);
// impl_perph_clk!(USBHS_SRAM, Clkctl0, pscctl0, Rstctl0, prstctl0, 23);

impl_perph_clk!(PIMCTL, Clkctl1, pscctl2, Rstctl1, prstctl2, 31);
impl_perph_clk!(ACMP, Clkctl0, pscctl1, Rstctl0, prstctl1, 15);
impl_perph_clk!(ADC0, Clkctl0, pscctl1, Rstctl0, prstctl1, 16);
impl_perph_clk!(CASPER, Clkctl0, pscctl0, Rstctl0, prstctl0, 9);
impl_perph_clk!(CRC, Clkctl1, pscctl1, Rstctl1, prstctl1, 16);
impl_perph_clk!(CTIMER0_COUNT_CHANNEL0, Clkctl1, pscctl2, Rstctl1, prstctl2, 0);
impl_perph_clk!(CTIMER1_COUNT_CHANNEL0, Clkctl1, pscctl2, Rstctl1, prstctl2, 1);
impl_perph_clk!(CTIMER2_COUNT_CHANNEL0, Clkctl1, pscctl2, Rstctl1, prstctl2, 2);
impl_perph_clk!(CTIMER3_COUNT_CHANNEL0, Clkctl1, pscctl2, Rstctl1, prstctl2, 3);
impl_perph_clk!(CTIMER4_COUNT_CHANNEL0, Clkctl1, pscctl2, Rstctl1, prstctl2, 4);
impl_perph_clk!(DMA0, Clkctl1, pscctl1, Rstctl1, prstctl1, 23);
impl_perph_clk!(DMA1, Clkctl1, pscctl1, Rstctl1, prstctl1, 24);
impl_perph_clk!(DMIC0, Clkctl1, pscctl0, Rstctl1, prstctl0, 24);

#[cfg(feature = "_espi")]
impl_perph_clk!(ESPI, Clkctl0, pscctl1, Rstctl0, prstctl1, 7);

impl_perph_clk!(FLEXCOMM0, Clkctl1, pscctl0, Rstctl1, prstctl0, 8);
impl_perph_clk!(FLEXCOMM1, Clkctl1, pscctl0, Rstctl1, prstctl0, 9);
impl_perph_clk!(FLEXCOMM14, Clkctl1, pscctl0, Rstctl1, prstctl0, 22);
impl_perph_clk!(FLEXCOMM15, Clkctl1, pscctl0, Rstctl1, prstctl0, 23);
impl_perph_clk!(FLEXCOMM2, Clkctl1, pscctl0, Rstctl1, prstctl0, 10);
impl_perph_clk!(FLEXCOMM3, Clkctl1, pscctl0, Rstctl1, prstctl0, 11);
impl_perph_clk!(FLEXCOMM4, Clkctl1, pscctl0, Rstctl1, prstctl0, 12);
impl_perph_clk!(FLEXCOMM5, Clkctl1, pscctl0, Rstctl1, prstctl0, 13);
impl_perph_clk!(FLEXCOMM6, Clkctl1, pscctl0, Rstctl1, prstctl0, 14);
impl_perph_clk!(FLEXCOMM7, Clkctl1, pscctl0, Rstctl1, prstctl0, 15);
impl_perph_clk!(FLEXSPI, Clkctl0, pscctl0, Rstctl0, prstctl0, 16);
impl_perph_clk!(FREQME, Clkctl1, pscctl1, Rstctl1, prstctl1, 31);
impl_perph_clk!(HASHCRYPT, Clkctl0, pscctl0, Rstctl0, prstctl0, 10);
impl_perph_clk!(HSGPIO0, Clkctl1, pscctl1, Rstctl1, prstctl1, 0);
impl_perph_clk!(HSGPIO1, Clkctl1, pscctl1, Rstctl1, prstctl1, 1);
impl_perph_clk!(HSGPIO2, Clkctl1, pscctl1, Rstctl1, prstctl1, 2);
impl_perph_clk!(HSGPIO3, Clkctl1, pscctl1, Rstctl1, prstctl1, 3);
impl_perph_clk!(HSGPIO4, Clkctl1, pscctl1, Rstctl1, prstctl1, 4);
impl_perph_clk!(HSGPIO5, Clkctl1, pscctl1, Rstctl1, prstctl1, 5);
impl_perph_clk!(HSGPIO6, Clkctl1, pscctl1, Rstctl1, prstctl1, 6);
impl_perph_clk!(HSGPIO7, Clkctl1, pscctl1, Rstctl1, prstctl1, 7);
impl_perph_clk!(I3C0, Clkctl1, pscctl2, Rstctl1, prstctl2, 16);
impl_perph_clk!(MRT0, Clkctl1, pscctl2, Rstctl1, prstctl2, 8);
impl_perph_clk!(MU_A, Clkctl1, pscctl1, Rstctl1, prstctl1, 28);
impl_perph_clk!(OS_EVENT, Clkctl1, pscctl0, Rstctl1, prstctl0, 27);
impl_perph_clk!(POWERQUAD, Clkctl0, pscctl0, Rstctl0, prstctl0, 8);
impl_perph_clk!(PUF, Clkctl0, pscctl0, Rstctl0, prstctl0, 11);
impl_perph_clk!(RNG, Clkctl0, pscctl0, Rstctl0, prstctl0, 12);
impl_perph_clk!(RTC, Clkctl1, pscctl2, Rstctl1, prstctl2, 7);
impl_perph_clk!(SCT0, Clkctl0, pscctl0, Rstctl0, prstctl0, 24);
impl_perph_clk!(SECGPIO, Clkctl0, pscctl1, Rstctl0, prstctl1, 24);
impl_perph_clk!(SEMA42, Clkctl1, pscctl1, Rstctl1, prstctl1, 29);
impl_perph_clk!(USBHSD, Clkctl0, pscctl0, Rstctl0, prstctl0, 21);
impl_perph_clk!(USBHSH, Clkctl0, pscctl0, Rstctl0, prstctl0, 22);
impl_perph_clk!(USBPHY, Clkctl0, pscctl0, Rstctl0, prstctl0, 20);
impl_perph_clk!(USDHC0, Clkctl0, pscctl1, Rstctl0, prstctl1, 2);
impl_perph_clk!(USDHC1, Clkctl0, pscctl1, Rstctl0, prstctl1, 3);
impl_perph_clk!(UTICK0, Clkctl0, pscctl2, Rstctl0, prstctl2, 0);
impl_perph_clk!(WDT0, Clkctl0, pscctl2, Rstctl0, prstctl2, 1);
impl_perph_clk!(WDT1, Clkctl1, pscctl2, Rstctl1, prstctl2, 10);

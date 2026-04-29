//! Clock configuration for the `RT6xx`
use core::sync::atomic::{AtomicU8, AtomicU32, Ordering};

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
        let sysctl0 = crate::pac::SYSCTL0;
        sysctl0
            .pdruncfg0_clr()
            .write(|w| w.set_lposc_pd(pac::sysctl0::vals::Pdruncfg0ClrLposcPd::CLR_PDRUNCFG0));

        // Wait for low-power oscillator to be ready (typically 64 us)
        // Busy loop seems better here than trying to shoe-in an async delay
        let clkctl0 = crate::pac::CLKCTL0;
        while !clkctl0.lposcctl0().read().clkrdy() {}
    }
}
impl ConfigurableClock for LposcConfig {
    fn enable_and_reset(&self) -> Result<(), ClockError> {
        LposcConfig::init_lposc();
        Ok(())
    }
    fn disable(&self) -> Result<(), ClockError> {
        let sysctl0 = crate::pac::SYSCTL0;
        sysctl0
            .pdruncfg0_set()
            .write(|w| w.set_lposc_pd(pac::sysctl0::vals::Pdruncfg0SetLposcPd::SET_PDRUNCFG0));
        // Wait until LPOSC disabled
        while sysctl0.pdruncfg0().read().lposc_pd() != pac::sysctl0::vals::Pdruncfg0LposcPd::POWER_DOWN {}
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
        let sysctl0 = crate::pac::SYSCTL0;

        /* Power on FFRO (48/60MHz) */
        sysctl0
            .pdruncfg0_clr()
            .write(|w| w.set_ffro_pd(pac::sysctl0::vals::Pdruncfg0ClrFfroPd::CLR_PDRUNCFG0));

        let clkctl0 = crate::pac::CLKCTL0;

        clkctl0
            .ffroctl1()
            .write(|w| w.set_update(pac::clkctl0::vals::Update::NORMAL_MODE));

        // No FFRO enable/disable control in CLKCTL.
        // Delay enough for FFRO to be stable in case it was just powered on
        delay_loop_clocks(50, 12_000_000);
    }
}

impl ConfigurableClock for FfroConfig {
    fn enable_and_reset(&self) -> Result<(), ClockError> {
        FfroConfig::init_ffro_clk();
        // default is 48 MHz
        Ok(())
    }
    fn disable(&self) -> Result<(), ClockError> {
        let sysctl0 = crate::pac::SYSCTL0;
        sysctl0
            .pdruncfg0_set()
            .write(|w| w.set_ffro_pd(pac::sysctl0::vals::Pdruncfg0SetFfroPd::SET_PDRUNCFG0));
        delay_loop_clocks(30, 12_000_000);
        // Wait until FFRO disabled
        while sysctl0.pdruncfg0().read().ffro_pd() != pac::sysctl0::vals::Pdruncfg0FfroPd::POWER_DOWN {}
        Ok(())
    }
    fn get_clock_rate(&self) -> Result<u32, ClockError> {
        Ok(self.freq.load(Ordering::Relaxed))
    }
    fn set_clock_rate(&mut self, _div: u8, _mult: u8, freq: u32) -> Result<(), ClockError> {
        if let Ok(r) = <u32 as TryInto<FfroFreq>>::try_into(freq) {
            match r {
                FfroFreq::Ffro48m => {
                    let clkctl0 = crate::pac::CLKCTL0;
                    clkctl0
                        .ffroctl1()
                        .write(|w| w.set_update(pac::clkctl0::vals::Update::UPDATE_SAFE_MODE));
                    clkctl0
                        .ffroctl0()
                        .write(|w| w.set_trim_range(pac::clkctl0::vals::TrimRange::FFRO_48MHZ));
                    clkctl0
                        .ffroctl1()
                        .write(|w| w.set_update(pac::clkctl0::vals::Update::NORMAL_MODE));

                    self.freq
                        .store(FfroFreq::Ffro48m as u32, core::sync::atomic::Ordering::Relaxed);
                    Ok(())
                }
                FfroFreq::Ffro60m => {
                    let clkctl0 = crate::pac::CLKCTL0;
                    clkctl0
                        .ffroctl1()
                        .write(|w| w.set_update(pac::clkctl0::vals::Update::UPDATE_SAFE_MODE));
                    clkctl0
                        .ffroctl0()
                        .write(|w| w.set_trim_range(pac::clkctl0::vals::TrimRange::FFRO_60MHZ));
                    clkctl0
                        .ffroctl1()
                        .write(|w| w.set_update(pac::clkctl0::vals::Update::NORMAL_MODE));

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
        let sysctl0 = crate::pac::SYSCTL0;
        sysctl0
            .pdruncfg0_clr()
            .write(|w| w.set_sfro_pd(pac::sysctl0::vals::Pdruncfg0ClrSfroPd::CLR_PDRUNCFG0));
        // wait until ready
        while sysctl0.pdruncfg0().read().sfro_pd() != pac::sysctl0::vals::Pdruncfg0SfroPd::ENABLED {}
        Ok(())
    }
    fn disable(&self) -> Result<(), ClockError> {
        let sysctl0 = crate::pac::SYSCTL0;
        sysctl0
            .pdruncfg0_set()
            .write(|w| w.set_sfro_pd(pac::sysctl0::vals::Pdruncfg0SetSfroPd::SET_PDRUNCFG0));
        delay_loop_clocks(30, 12_000_000);
        // Wait until SFRO disabled
        while sysctl0.pdruncfg0().read().sfro_pd() != pac::sysctl0::vals::Pdruncfg0SfroPd::POWER_DOWN {}
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
            let clkctl0 = crate::pac::CLKCTL0;
            let sysctl0 = crate::pac::SYSCTL0;

            // Power down pll before changes
            sysctl0.pdruncfg0_set().write(|w| {
                w.set_syspllldo_pd(pac::sysctl0::vals::Pdruncfg0SetSyspllldoPd::SET_PDRUNCFG0);
                w.set_syspllana_pd(pac::sysctl0::vals::Pdruncfg0SetSyspllanaPd::SET_PDRUNCFG0);
            });

            let desired_freq: u64 = self.freq.load(Ordering::Relaxed).into();

            match self.src {
                c if c == MainPllClkSrc::ClkIn || c == MainPllClkSrc::FFRO || c == MainPllClkSrc::SFRO => {
                    let mut base_rate;
                    match c {
                        MainPllClkSrc::ClkIn => {
                            clkctl0
                                .syspll0clksel()
                                .write(|w| w.set_sel(pac::clkctl0::vals::Syspll0clkselSel::SYSXTAL_CLK));
                            let r = self.get_clock_rate()?;
                            base_rate = r;
                        }
                        MainPllClkSrc::FFRO => {
                            delay_loop_clocks(1000, desired_freq);
                            match clkctl0.ffroctl0().read().trim_range() == pac::clkctl0::vals::TrimRange::FFRO_48MHZ {
                                true => base_rate = Into::into(FfroFreq::Ffro48m),
                                false => base_rate = Into::into(FfroFreq::Ffro60m),
                            }
                            if div == 2 {
                                clkctl0
                                    .syspll0clksel()
                                    .write(|w| w.set_sel(pac::clkctl0::vals::Syspll0clkselSel::FFRO_DIV_2));
                                delay_loop_clocks(150, desired_freq);
                                base_rate /= 2;
                            } else {
                                return Err(ClockError::InvalidDiv);
                            }
                        }
                        MainPllClkSrc::SFRO => {
                            base_rate = SFRO_FREQ;
                            clkctl0
                                .syspll0clksel()
                                .write(|w| w.set_sel(pac::clkctl0::vals::Syspll0clkselSel::SFRO_CLK));
                        }
                    };
                    base_rate *= u32::from(mult);
                    if base_rate != freq {
                        // make sure to power syspll back up before returning the error
                        // Clear System PLL reset
                        clkctl0
                            .syspll0ctl0()
                            .write(|w| w.set_reset(pac::clkctl0::vals::Reset::NORMAL));
                        // Power up SYSPLL
                        sysctl0.pdruncfg0_clr().write(|w| {
                            w.set_syspllana_pd(pac::sysctl0::vals::Pdruncfg0ClrSyspllanaPd::CLR_PDRUNCFG0);
                            w.set_syspllldo_pd(pac::sysctl0::vals::Pdruncfg0ClrSyspllldoPd::CLR_PDRUNCFG0);
                        });
                        return Err(ClockError::InvalidFrequency);
                    }
                    clkctl0.syspll0num().write(|w| w.set_num(0b0));
                    clkctl0.syspll0denom().write(|w| w.set_denom(0b1));
                    delay_loop_clocks(30, desired_freq);
                    self.mult.store(mult, Ordering::Relaxed);
                    match mult {
                        16 => {
                            clkctl0
                                .syspll0ctl0()
                                .modify(|w| w.set_mult(pac::clkctl0::vals::Mult::DIV_16));
                        }
                        17 => {
                            clkctl0
                                .syspll0ctl0()
                                .modify(|w| w.set_mult(pac::clkctl0::vals::Mult::DIV_17));
                        }
                        20 => {
                            clkctl0
                                .syspll0ctl0()
                                .modify(|w| w.set_mult(pac::clkctl0::vals::Mult::DIV_20));
                        }
                        22 => {
                            clkctl0
                                .syspll0ctl0()
                                .modify(|w| w.set_mult(pac::clkctl0::vals::Mult::DIV_22));
                        }
                        27 => {
                            clkctl0
                                .syspll0ctl0()
                                .modify(|w| w.set_mult(pac::clkctl0::vals::Mult::DIV_27));
                        }
                        33 => {
                            clkctl0
                                .syspll0ctl0()
                                .modify(|w| w.set_mult(pac::clkctl0::vals::Mult::DIV_33));
                        }
                        _ => return Err(ClockError::InvalidMult),
                    }
                    // Clear System PLL reset
                    clkctl0
                        .syspll0ctl0()
                        .modify(|w| w.set_reset(pac::clkctl0::vals::Reset::NORMAL));
                    // Power up SYSPLL
                    sysctl0.pdruncfg0_clr().write(|w| {
                        w.set_syspllana_pd(pac::sysctl0::vals::Pdruncfg0ClrSyspllanaPd::CLR_PDRUNCFG0);
                        w.set_syspllldo_pd(pac::sysctl0::vals::Pdruncfg0ClrSyspllldoPd::CLR_PDRUNCFG0);
                    });

                    // Set System PLL HOLDRINGOFF_ENA
                    clkctl0
                        .syspll0ctl0()
                        .modify(|w| w.set_holdringoff_ena(pac::clkctl0::vals::HoldringoffEna::ENABLE));
                    delay_loop_clocks(75, desired_freq);

                    // Clear System PLL HOLDRINGOFF_ENA
                    clkctl0
                        .syspll0ctl0()
                        .modify(|w| w.set_holdringoff_ena(pac::clkctl0::vals::HoldringoffEna::DSIABLE));
                    delay_loop_clocks(15, desired_freq);

                    // gate the output and clear bits.
                    clkctl0.syspll0pfd().modify(|w| {
                        w.set_pfd0_clkgate(true);
                        w.set_pfd0(0x0);
                    });
                    // set pfd bits and un-gate the clock output
                    // output is multiplied by syspll * 18/pfd0_bits
                    clkctl0.syspll0pfd().modify(|w| {
                        w.set_pfd0_clkgate(false);
                        w.set_pfd0(0x12);
                    });
                    // wait for ready bit to be set
                    delay_loop_clocks(50, desired_freq);
                    while !clkctl0.syspll0pfd().read().pfd0_clkrdy() {}
                    // clear by writing a 1
                    clkctl0.syspll0pfd().modify(|w| w.set_pfd0_clkrdy(true));

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
        const VALIDMULTS: [u8; 6] = [16, 17, 20, 22, 27, 33];
        if rate > base_freq && rate.is_multiple_of(base_freq) {
            let mult = (rate / base_freq) as u8;
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
        let clkctl0 = crate::pac::CLKCTL0;
        let sysctl0 = crate::pac::SYSCTL0;

        // Power down SYSPLL before change fractional settings
        sysctl0.pdruncfg0_set().write(|w| {
            w.set_syspllldo_pd(pac::sysctl0::vals::Pdruncfg0SetSyspllldoPd::SET_PDRUNCFG0);
            w.set_syspllana_pd(pac::sysctl0::vals::Pdruncfg0SetSyspllanaPd::SET_PDRUNCFG0);
        });

        clkctl0
            .syspll0clksel()
            .write(|w| w.set_sel(pac::clkctl0::vals::Syspll0clkselSel::FFRO_DIV_2));
        clkctl0.syspll0num().write(|w| w.set_num(0x0));
        clkctl0.syspll0denom().write(|w| w.set_denom(0x1));

        // kCLOCK_SysPllMult22
        clkctl0
            .syspll0ctl0()
            .modify(|w| w.set_mult(pac::clkctl0::vals::Mult::DIV_22));

        // Clear System PLL reset
        clkctl0
            .syspll0ctl0()
            .modify(|w| w.set_reset(pac::clkctl0::vals::Reset::NORMAL));

        // Power up SYSPLL
        sysctl0.pdruncfg0_clr().write(|w| {
            w.set_syspllldo_pd(pac::sysctl0::vals::Pdruncfg0ClrSyspllldoPd::CLR_PDRUNCFG0);
            w.set_syspllana_pd(pac::sysctl0::vals::Pdruncfg0ClrSyspllanaPd::CLR_PDRUNCFG0);
        });
        delay_loop_clocks((150 & 0xFFFF) / 2, 12_000_000);

        // Set System PLL HOLDRINGOFF_ENA
        clkctl0
            .syspll0ctl0()
            .modify(|w| w.set_holdringoff_ena(pac::clkctl0::vals::HoldringoffEna::ENABLE));
        delay_loop_clocks((150 & 0xFFFF) / 2, 12_000_000);

        // Clear System PLL HOLDRINGOFF_ENA
        clkctl0
            .syspll0ctl0()
            .modify(|w| w.set_holdringoff_ena(pac::clkctl0::vals::HoldringoffEna::DSIABLE));
        delay_loop_clocks((15 & 0xFFFF) / 2, 12_000_000);
    }
    /// enables default settings for pfd2 bits
    pub(self) fn init_syspll_pfd2(config_bits: u8) {
        let clkctl0 = crate::pac::CLKCTL0;

        // Disable the clock output first.
        clkctl0.syspll0pfd().modify(|w| {
            w.set_pfd2_clkgate(true);
            w.set_pfd2(0x0);
        });

        // Set the new value and enable output.
        clkctl0.syspll0pfd().modify(|w| {
            w.set_pfd2_clkgate(false);
            w.set_pfd2(config_bits);
        });

        // Wait for output becomes stable.
        while !clkctl0.syspll0pfd().read().pfd2_clkrdy() {}

        // Clear ready status flag.
        clkctl0.syspll0pfd().modify(|w| w.set_pfd2_clkrdy(false));
    }
    /// Enables default settings for pfd0
    pub(self) fn init_syspll_pfd0(config_bits: u8) {
        let clkctl0 = crate::pac::CLKCTL0;
        // Disable the clock output first
        clkctl0.syspll0pfd().modify(|w| {
            w.set_pfd0_clkgate(true);
            w.set_pfd0(0x0);
        });

        // Set the new value and enable output
        clkctl0.syspll0pfd().modify(|w| {
            w.set_pfd0_clkgate(false);
            w.set_pfd0(config_bits);
        });

        // Wait for output becomes stable
        while !clkctl0.syspll0pfd().read().pfd0_clkrdy() {}

        // Clear ready status flag
        clkctl0.syspll0pfd().modify(|w| w.set_pfd0_clkrdy(false));
    }
}

impl MainClkConfig {
    fn init_main_clk() {
        // used to set the right HW frequency
        let clkctl0 = crate::pac::CLKCTL0;
        let clkctl1 = crate::pac::CLKCTL1;

        clkctl0
            .mainclkselb()
            .write(|w| w.set_sel(pac::clkctl0::vals::MainclkselbSel::MAIN_PLL_CLK));

        // Set PFC0DIV divider to value 2, Subtract 1 since 0-> 1, 1-> 2, etc...
        clkctl0.pfcdiv(0).modify(|w| w.set_reset(true));
        clkctl0.pfcdiv(0).write(|w| {
            w.set_div(2 - 1);
            w.set_halt(false);
        });
        while clkctl0.pfcdiv(0).read().reqflag() {}

        // Set FRGPLLCLKDIV divider to value 12, Subtract 1 since 0-> 1, 1-> 2, etc...
        clkctl1.frgpllclkdiv().modify(|w| w.set_reset(true));
        clkctl1.frgpllclkdiv().write(|w| {
            w.set_div(12 - 1);
            w.set_halt(false);
        });
        while clkctl1.frgpllclkdiv().read().reqflag() {}
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
                        // needed to calculate the clock rate from the bits written in the registers
                        let clkctl0 = crate::pac::CLKCTL0;
                        if self.src == MainClkSrc::PllMain
                            && clkctl0.syspll0ctl0().read().bypass() == pac::clkctl0::vals::Bypass::PROGRAMMED_CLK
                        {
                            let mut temp;
                            temp = self.freq.load(Ordering::Relaxed)
                                * u32::from(clkctl0.syspll0ctl0().read().mult().to_bits());
                            temp = (u64::from(temp) * 18 / u64::from(clkctl0.syspll0pfd().read().pfd0())) as u32;
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
            // needed to change the clock source
            let clkctl0 = crate::pac::CLKCTL0;
            match c {
                MainClkSrc::ClkIn => {
                    self.src = MainClkSrc::ClkIn;

                    clkctl0
                        .mainclksela()
                        .write(|w| w.set_sel(pac::clkctl0::vals::MainclkselaSel::SYSXTAL_CLK));
                    clkctl0
                        .mainclkselb()
                        .write(|w| w.set_sel(pac::clkctl0::vals::MainclkselbSel::MAIN_1ST_CLK));
                    Ok(())
                }
                // the following will yield the same result as if compared to FFROdiv4
                MainClkSrc::FFRO | MainClkSrc::FFROdiv4 => match rate {
                    div4 if div4 == (FfroFreq::Ffro60m as u32) / 4 || div4 == (FfroFreq::Ffro48m as u32) / 4 => {
                        self.src = MainClkSrc::FFROdiv4;
                        self.freq.store(div4, Ordering::Relaxed);

                        clkctl0
                            .mainclksela()
                            .write(|w| w.set_sel(pac::clkctl0::vals::MainclkselaSel::FFRO_DIV_4));
                        clkctl0
                            .mainclkselb()
                            .write(|w| w.set_sel(pac::clkctl0::vals::MainclkselbSel::MAIN_1ST_CLK));
                        Ok(())
                    }
                    div1 if div1 == FfroFreq::Ffro60m as u32 || div1 == FfroFreq::Ffro48m as u32 => {
                        self.src = MainClkSrc::FFRO;
                        self.freq.store(div1, Ordering::Relaxed);

                        clkctl0
                            .mainclksela()
                            .write(|w| w.set_sel(pac::clkctl0::vals::MainclkselaSel::FFRO_CLK));
                        clkctl0
                            .mainclkselb()
                            .write(|w| w.set_sel(pac::clkctl0::vals::MainclkselbSel::MAIN_1ST_CLK));
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

                                clkctl0
                                    .mainclksela()
                                    .write(|w| w.set_sel(pac::clkctl0::vals::MainclkselaSel::LPOSC));
                                clkctl0
                                    .mainclkselb()
                                    .write(|w| w.set_sel(pac::clkctl0::vals::MainclkselbSel::MAIN_1ST_CLK));
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
                        clkctl0
                            .mainclkselb()
                            .write(|w| w.set_sel(pac::clkctl0::vals::MainclkselbSel::SFRO_CLK));
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
                        clkctl0
                            .mainclkselb()
                            .write(|w| w.set_sel(pac::clkctl0::vals::MainclkselbSel::MAIN_PLL_CLK));
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
                        clkctl0
                            .mainclkselb()
                            .write(|w| w.set_sel(pac::clkctl0::vals::MainclkselbSel::RTC_32K_CLK));
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
        Err(ClockError::ClockNotSupported)
    }
    fn get_clock_rate(&self) -> Result<u32, ClockError> {
        let (_c, rate) = MainClkConfig::get_clock_source_and_rate(self, &Clocks::MainClk)?;
        Ok(rate)
    }
    fn set_clock_rate(&mut self, _div: u8, _mult: u8, _freq: u32) -> Result<(), ClockError> {
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
        // needed to enable the RTC HW
        let cc0 = pac::CLKCTL0;
        let cc1 = pac::CLKCTL1;
        let r = pac::RTC;
        // Enable the RTC peripheral clock
        cc1.pscctl2_set()
            .write(|w| w.set_rtc_lite_clk_set(pac::clkctl1::vals::RtcLiteClkSet::SET_CLOCK));
        // Make sure the reset bit is cleared amd RTC OSC is powered up
        r.ctrl().modify(|w| {
            w.set_swreset(false);
            w.set_rtc_osc_pd(pac::rtc::vals::RtcOscPd::ENABLE);
        });

        // set initial match value, note that with a 15 bit count-down timer this would
        // typically be 0x8000, but we are "doing some clever things" in time-driver.rs,
        // read more about it in the comments there
        r.wake().write_value(pac::rtc::regs::Wake(0xA));

        // Enable 32K OSC
        cc0.osc32khzctl0().write(|w| w.set_ena32khz(true));

        // enable rtc clk
        r.ctrl().modify(|w| w.set_rtc_en(true));
    }
}

impl ConfigurableClock for RtcClkConfig {
    fn enable_and_reset(&self) -> Result<(), ClockError> {
        // should only be called once if previously disabled
        RtcClkConfig::init_rtc_clk();
        Ok(())
    }
    fn disable(&self) -> Result<(), ClockError> {
        Err(ClockError::ClockNotSupported)
    }
    fn set_clock_rate(&mut self, _div: u8, _mult: u8, freq: u32) -> Result<(), ClockError> {
        if let Ok(r) = <u32 as TryInto<RtcFreq>>::try_into(freq) {
            // needed to enable the HW for the different RTC frequencies, powered down by default
            let rtc = crate::pac::RTC;
            match r {
                RtcFreq::Default1Hz => {
                    if rtc.ctrl().read().rtc_en() {
                    } else {
                        rtc.ctrl().modify(|w| w.set_rtc_en(true));
                    }
                    Ok(())
                }
                RtcFreq::HighResolution1khz => {
                    if rtc.ctrl().read().rtc1khz_en() {
                    } else {
                        rtc.ctrl().modify(|w| w.set_rtc1khz_en(true));
                    }
                    Ok(())
                }
                RtcFreq::SubSecond32kHz => {
                    if rtc.ctrl().read().rtc_subsec_ena() {
                    } else {
                        rtc.ctrl().modify(|w| w.set_rtc_subsec_ena(true));
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
    fn update_sys_core_clock(&self) {}
}

impl ConfigurableClock for SysOscConfig {
    fn enable_and_reset(&self) -> Result<(), ClockError> {
        if self.state == State::Enabled {
            return Ok(());
        }

        let clkctl0 = crate::pac::CLKCTL0;
        let sysctl0 = crate::pac::SYSCTL0;

        // Let CPU run on ffro for safe switching
        clkctl0
            .mainclksela()
            .write(|w| w.set_sel(pac::clkctl0::vals::MainclkselaSel::FFRO_CLK));
        clkctl0
            .mainclksela()
            .write(|w| w.set_sel(pac::clkctl0::vals::MainclkselaSel::FFRO_DIV_4));

        // Power on SYSXTAL
        sysctl0
            .pdruncfg0_clr()
            .write(|w| w.set_sysxtal_pd(pac::sysctl0::vals::Pdruncfg0ClrSysxtalPd::CLR_PDRUNCFG0));

        // Enable system OSC
        clkctl0.sysoscctl0().write(|w| {
            w.set_lp_enable(pac::clkctl0::vals::LpEnable::LP);
            w.set_bypass_enable(pac::clkctl0::vals::BypassEnable::NORMAL_MODE);
        });

        delay_loop_clocks(260, SYS_OSC_DEFAULT_FREQ.into());
        Ok(())
    }
    fn disable(&self) -> Result<(), ClockError> {
        let clkctl0 = crate::pac::CLKCTL0;
        let sysctl0 = crate::pac::SYSCTL0;

        // Let CPU run on ffro for safe switching
        clkctl0
            .mainclksela()
            .write(|w| w.set_sel(pac::clkctl0::vals::MainclkselaSel::FFRO_CLK));
        clkctl0
            .mainclksela()
            .write(|w| w.set_sel(pac::clkctl0::vals::MainclkselaSel::FFRO_DIV_4));

        // Power on SYSXTAL
        sysctl0
            .pdruncfg0_set()
            .write(|w| w.set_sysxtal_pd(pac::sysctl0::vals::Pdruncfg0SetSysxtalPd::SET_PDRUNCFG0));
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
    let pmc = crate::pac::PMC;
    // Set up IO voltages
    // all 3 ranges need to be 1.71-1.98V which is 01
    pmc.padvrange().write(|w| {
        w.set_vddio_0range(pac::pmc::vals::Vddio0range::from_bits(0b01));
        w.set_vddio_1range(pac::pmc::vals::Vddio1range::from_bits(0b01));
        w.set_vddio_2range(pac::pmc::vals::Vddio2range::from_bits(0b01));
    });
}

/// Initialize AHB clock
fn init_syscpuahb_clk() {
    let clkctl0 = crate::pac::CLKCTL0;
    // Set syscpuahbclkdiv to value 2, Subtract 1 since 0-> 1, 1-> 2, etc...
    clkctl0.syscpuahbclkdiv().write(|w| w.set_div(2 - 1));

    while clkctl0.syscpuahbclkdiv().read().reqflag() {}
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
        let cc1 = pac::CLKCTL1;
        match src {
            ClkOutSrc::None => {
                cc1.clkoutsel0()
                    .write(|w| w.set_sel(pac::clkctl1::vals::Clkoutsel0Sel::NONE));
                cc1.clkoutsel1()
                    .write(|w| w.set_sel(pac::clkctl1::vals::Clkoutsel1Sel::NONE));
            }
            ClkOutSrc::Sfro => {
                cc1.clkoutsel0()
                    .write(|w| w.set_sel(pac::clkctl1::vals::Clkoutsel0Sel::SFRO_CLK));
                cc1.clkoutsel1()
                    .write(|w| w.set_sel(pac::clkctl1::vals::Clkoutsel1Sel::CLKOUTSEL0_OUTPUT));
            }
            ClkOutSrc::ClkIn => {
                cc1.clkoutsel0()
                    .write(|w| w.set_sel(pac::clkctl1::vals::Clkoutsel0Sel::XTALIN_CLK));
                cc1.clkoutsel1()
                    .write(|w| w.set_sel(pac::clkctl1::vals::Clkoutsel1Sel::CLKOUTSEL0_OUTPUT));
            }
            ClkOutSrc::Lposc => {
                cc1.clkoutsel0()
                    .write(|w| w.set_sel(pac::clkctl1::vals::Clkoutsel0Sel::LPOSC));
                cc1.clkoutsel1()
                    .write(|w| w.set_sel(pac::clkctl1::vals::Clkoutsel1Sel::CLKOUTSEL0_OUTPUT));
            }
            ClkOutSrc::Ffro => {
                cc1.clkoutsel0()
                    .write(|w| w.set_sel(pac::clkctl1::vals::Clkoutsel0Sel::FFRO_CLK));
                cc1.clkoutsel1()
                    .write(|w| w.set_sel(pac::clkctl1::vals::Clkoutsel1Sel::CLKOUTSEL0_OUTPUT));
            }
            ClkOutSrc::MainClk => {
                cc1.clkoutsel0()
                    .write(|w| w.set_sel(pac::clkctl1::vals::Clkoutsel0Sel::MAIN_CLK));
                cc1.clkoutsel1()
                    .write(|w| w.set_sel(pac::clkctl1::vals::Clkoutsel1Sel::CLKOUTSEL0_OUTPUT));
            }
            ClkOutSrc::DspMainClk => {
                cc1.clkoutsel0()
                    .write(|w| w.set_sel(pac::clkctl1::vals::Clkoutsel0Sel::DSP_MAIN_CLK));
                cc1.clkoutsel1()
                    .write(|w| w.set_sel(pac::clkctl1::vals::Clkoutsel1Sel::CLKOUTSEL0_OUTPUT));
            }
            ClkOutSrc::MainPllClk => {
                cc1.clkoutsel0()
                    .write(|w| w.set_sel(pac::clkctl1::vals::Clkoutsel0Sel::NONE));
                cc1.clkoutsel1()
                    .write(|w| w.set_sel(pac::clkctl1::vals::Clkoutsel1Sel::MAIN_PLL_CLK));
            }
            ClkOutSrc::Aux0PllClk => {
                cc1.clkoutsel0()
                    .write(|w| w.set_sel(pac::clkctl1::vals::Clkoutsel0Sel::NONE));
                cc1.clkoutsel1()
                    .write(|w| w.set_sel(pac::clkctl1::vals::Clkoutsel1Sel::SYSPLL0_AUX0_PLL_CLK));
            }
            ClkOutSrc::DspPllClk => {
                cc1.clkoutsel0()
                    .write(|w| w.set_sel(pac::clkctl1::vals::Clkoutsel0Sel::NONE));
                cc1.clkoutsel1()
                    .write(|w| w.set_sel(pac::clkctl1::vals::Clkoutsel1Sel::DSP_PLL_CLK));
            }
            ClkOutSrc::AudioPllClk => {
                cc1.clkoutsel0()
                    .write(|w| w.set_sel(pac::clkctl1::vals::Clkoutsel0Sel::NONE));
                cc1.clkoutsel1()
                    .write(|w| w.set_sel(pac::clkctl1::vals::Clkoutsel1Sel::AUDIO_PLL_CLK));
            }
            ClkOutSrc::Aux1PllClk => {
                cc1.clkoutsel0()
                    .write(|w| w.set_sel(pac::clkctl1::vals::Clkoutsel0Sel::NONE));
                cc1.clkoutsel1()
                    .write(|w| w.set_sel(pac::clkctl1::vals::Clkoutsel1Sel::SYSPLL0_AUX1_PLL_CLK));
            }
            ClkOutSrc::RTC32k => {
                cc1.clkoutsel0()
                    .write(|w| w.set_sel(pac::clkctl1::vals::Clkoutsel0Sel::NONE));
                cc1.clkoutsel1()
                    .write(|w| w.set_sel(pac::clkctl1::vals::Clkoutsel1Sel::RTC_CLK_32KHZ));
            }
        }
        self.src = src;
        Ok(())
    }
    /// set the clock out divider
    /// note that 1 will be added to div when mapping to the divider
    /// so set_div(0) -> divide by 1
    /// ...
    /// set_div(255)-> divide by 256
    pub fn set_clkout_divider(&self, div: u8) -> Result<(), ClockError> {
        // don't wait for clock to be ready if there's no source
        if self.src != ClkOutSrc::None {
            let cc1 = pac::CLKCTL1;

            cc1.clkoutdiv().modify(|w| {
                w.set_div(div);
                w.set_halt(false);
            });
            while cc1.clkoutdiv().read().reqflag() {}
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
    config.rtc.enable_and_reset()?;
    config.lposc.enable_and_reset()?;
    config.ffro.enable_and_reset()?;
    config.sfro.enable_and_reset()?;
    config.sys_osc.enable_and_reset()?;
    config.main_pll_clk.enable_and_reset()?;

    // Move FLEXSPI clock source from main clock to FFRO to avoid instruction/data fetch issue in XIP when
    // updating PLL and main clock.
    let cc0 = pac::CLKCTL0;
    cc0.flexspifclksel()
        .write(|w| w.set_sel(pac::clkctl0::vals::FlexspifclkselSel::FFRO_CLK));

    // Move ESPI clock source to FFRO
    #[cfg(feature = "_espi")]
    {
        cc0.espifclksel0()
            .write(|w| w.set_sel(pac::clkctl0::vals::Espifclksel0Sel::FFRO_CLK));
    }

    init_syscpuahb_clk();

    config.main_clk.enable_and_reset()?;

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
    fn enable_perph_clock();
    fn reset_perph();
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
    T::enable_perph_clock();
    T::reset_perph();
}

/// Enables peripheral `T`.
pub fn enable<T: SysconPeripheral>() {
    T::enable_perph_clock();
}

/// Reset peripheral `T`.
pub fn reset<T: SysconPeripheral>() {
    T::reset_perph();
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
            fn enable_perph_clock() {
                let cc1 = pac::$clkctl;

                paste! {
                    cc1.[<$clkreg _set>]().write(|w| w.0 = 1 << $bit);
                }
            }

            fn reset_perph() {
                let rc1 = pac::$rstctl;

                paste! {
                    rc1.[<$rstreg _clr>]().write(|w| w.0 = 1 << $bit);
                }
            }

            fn disable_perph_clock() {
                let cc1 = pac::$clkctl;

                paste! {
                    cc1.[<$clkreg _clr>]().write(|w| w.0 = 1 << $bit);
                }
            }
        }

        impl SysconPeripheral for crate::peripherals::$peripheral {}
    };
}

// These should enabled once the relevant peripherals are implemented.
// impl_perph_clk!(GPIOINTCTL, CLKCTL1, pscctl2, RSTCTL1, prstctl2, 30);
// impl_perph_clk!(OTP, CLKCTL0, pscctl0, RSTCTL0, prstctl0, 17);

// impl_perph_clk!(ROM_CTL_128KB, CLKCTL0, pscctl0, RSTCTL0, prstctl0, 2);
// impl_perph_clk!(USBHS_SRAM, CLKCTL0, pscctl0, RSTCTL0, prstctl0, 23);

impl_perph_clk!(PIMCTL, CLKCTL1, pscctl2, RSTCTL1, prstctl2, 31);
impl_perph_clk!(ACMP, CLKCTL0, pscctl1, RSTCTL0, prstctl1, 15);
impl_perph_clk!(ADC0, CLKCTL0, pscctl1, RSTCTL0, prstctl1, 16);
impl_perph_clk!(CASPER, CLKCTL0, pscctl0, RSTCTL0, prstctl0, 9);
impl_perph_clk!(CRC, CLKCTL1, pscctl1, RSTCTL1, prstctl1, 16);
impl_perph_clk!(CTIMER0_COUNT_CHANNEL0, CLKCTL1, pscctl2, RSTCTL1, prstctl2, 0);
impl_perph_clk!(CTIMER1_COUNT_CHANNEL0, CLKCTL1, pscctl2, RSTCTL1, prstctl2, 1);
impl_perph_clk!(CTIMER2_COUNT_CHANNEL0, CLKCTL1, pscctl2, RSTCTL1, prstctl2, 2);
impl_perph_clk!(CTIMER3_COUNT_CHANNEL0, CLKCTL1, pscctl2, RSTCTL1, prstctl2, 3);
impl_perph_clk!(CTIMER4_COUNT_CHANNEL0, CLKCTL1, pscctl2, RSTCTL1, prstctl2, 4);
impl_perph_clk!(DMA0, CLKCTL1, pscctl1, RSTCTL1, prstctl1, 23);
impl_perph_clk!(DMA1, CLKCTL1, pscctl1, RSTCTL1, prstctl1, 24);
impl_perph_clk!(DMIC0, CLKCTL1, pscctl0, RSTCTL1, prstctl0, 24);

#[cfg(feature = "_espi")]
impl_perph_clk!(ESPI, CLKCTL0, pscctl1, RSTCTL0, prstctl1, 7);

impl_perph_clk!(FLEXCOMM0, CLKCTL1, pscctl0, RSTCTL1, prstctl0, 8);
impl_perph_clk!(FLEXCOMM1, CLKCTL1, pscctl0, RSTCTL1, prstctl0, 9);
impl_perph_clk!(FLEXCOMM14, CLKCTL1, pscctl0, RSTCTL1, prstctl0, 22);
impl_perph_clk!(FLEXCOMM15, CLKCTL1, pscctl0, RSTCTL1, prstctl0, 23);
impl_perph_clk!(FLEXCOMM2, CLKCTL1, pscctl0, RSTCTL1, prstctl0, 10);
impl_perph_clk!(FLEXCOMM3, CLKCTL1, pscctl0, RSTCTL1, prstctl0, 11);
impl_perph_clk!(FLEXCOMM4, CLKCTL1, pscctl0, RSTCTL1, prstctl0, 12);
impl_perph_clk!(FLEXCOMM5, CLKCTL1, pscctl0, RSTCTL1, prstctl0, 13);
impl_perph_clk!(FLEXCOMM6, CLKCTL1, pscctl0, RSTCTL1, prstctl0, 14);
impl_perph_clk!(FLEXCOMM7, CLKCTL1, pscctl0, RSTCTL1, prstctl0, 15);
impl_perph_clk!(FLEXSPI, CLKCTL0, pscctl0, RSTCTL0, prstctl0, 16);
impl_perph_clk!(FREQME, CLKCTL1, pscctl1, RSTCTL1, prstctl1, 31);
impl_perph_clk!(HASHCRYPT, CLKCTL0, pscctl0, RSTCTL0, prstctl0, 10);
impl_perph_clk!(HSGPIO0, CLKCTL1, pscctl1, RSTCTL1, prstctl1, 0);
impl_perph_clk!(HSGPIO1, CLKCTL1, pscctl1, RSTCTL1, prstctl1, 1);
impl_perph_clk!(HSGPIO2, CLKCTL1, pscctl1, RSTCTL1, prstctl1, 2);
impl_perph_clk!(HSGPIO3, CLKCTL1, pscctl1, RSTCTL1, prstctl1, 3);
impl_perph_clk!(HSGPIO4, CLKCTL1, pscctl1, RSTCTL1, prstctl1, 4);
impl_perph_clk!(HSGPIO5, CLKCTL1, pscctl1, RSTCTL1, prstctl1, 5);
impl_perph_clk!(HSGPIO6, CLKCTL1, pscctl1, RSTCTL1, prstctl1, 6);
impl_perph_clk!(HSGPIO7, CLKCTL1, pscctl1, RSTCTL1, prstctl1, 7);
impl_perph_clk!(I3C0, CLKCTL1, pscctl2, RSTCTL1, prstctl2, 16);
impl_perph_clk!(MRT0, CLKCTL1, pscctl2, RSTCTL1, prstctl2, 8);
impl_perph_clk!(MU_A, CLKCTL1, pscctl1, RSTCTL1, prstctl1, 28);
impl_perph_clk!(OS_EVENT, CLKCTL1, pscctl0, RSTCTL1, prstctl0, 27);
impl_perph_clk!(POWERQUAD, CLKCTL0, pscctl0, RSTCTL0, prstctl0, 8);
impl_perph_clk!(PUF, CLKCTL0, pscctl0, RSTCTL0, prstctl0, 11);
impl_perph_clk!(RNG, CLKCTL0, pscctl0, RSTCTL0, prstctl0, 12);
impl_perph_clk!(RTC, CLKCTL1, pscctl2, RSTCTL1, prstctl2, 7);
impl_perph_clk!(SCT0, CLKCTL0, pscctl0, RSTCTL0, prstctl0, 24);
impl_perph_clk!(SECGPIO, CLKCTL0, pscctl1, RSTCTL0, prstctl1, 24);
impl_perph_clk!(SEMA42, CLKCTL1, pscctl1, RSTCTL1, prstctl1, 29);
impl_perph_clk!(USBHSD, CLKCTL0, pscctl0, RSTCTL0, prstctl0, 21);
impl_perph_clk!(USBHSH, CLKCTL0, pscctl0, RSTCTL0, prstctl0, 22);
impl_perph_clk!(USBPHY, CLKCTL0, pscctl0, RSTCTL0, prstctl0, 20);
impl_perph_clk!(USDHC0, CLKCTL0, pscctl1, RSTCTL0, prstctl1, 2);
impl_perph_clk!(USDHC1, CLKCTL0, pscctl1, RSTCTL0, prstctl1, 3);
impl_perph_clk!(UTICK0, CLKCTL0, pscctl2, RSTCTL0, prstctl2, 0);
impl_perph_clk!(WDT0, CLKCTL0, pscctl2, RSTCTL0, prstctl2, 1);
impl_perph_clk!(WDT1, CLKCTL1, pscctl2, RSTCTL1, prstctl2, 10);

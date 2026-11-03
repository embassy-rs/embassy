//! Continuous low-power comparison with optional FRO16K digital filtering.
//!
//! Construct [`Cmp`] with compatible owned [`PinInput`]s and/or its internal
//! [`DacInput`]. Construction leaves comparison disabled. Call `enable()` before
//! observing COUT; disable before changing configuration or the DAC code. A live
//! configuration request is rejected without stopping/restarting the comparator.
//! COUT includes inversion and filtering; it is not a measurement of voltage.
//!
//! Initialization is blanked for 50 us plus filter priming. This follows the
//! supplied A2 datasheet Rev.5 and A5 Rev.1DraftD (40 us initialization, up to 5 us
//! propagation under their electrical conditions). Filter timing allows a 10%
//! slow FRO16K, covering the supplied -9% limit. These are not new electrical
//! guarantees: meet the selected chip's supply, input, reference, oscillator-trim
//! and overdrive specifications. Noise or input changes can restart filtering.
//! Nonblocking reads return `NotReady` during priming; async waits yield through
//! it. A configured Embassy time driver is required, including in blocking mode.
//!
//! Edge waits start a fresh window after priming, discarding startup/stale flags.
//! Flags coalesce events: no lossless queue, ordering, count or timestamp is
//! promised. Level waits also accept the corresponding latched edge: a high/low
//! pulse still completes its wait if the signal switches back before polling,
//! as required by `embedded-hal-async`. Cancelling masks interrupts but leaves
//! comparison active
//! for hardware consumers. Quiesce those consumers before disabling, configuring
//! or dropping CMP. The driver retains an active-only wake guard until dropped.
//!
//! DAC output is `Vreference * (code + 1) / 256`; code zero is not zero volts.
//! Selecting VREFI does not enable a reference or own its external pin. Supply a
//! valid reference separately. External inputs have their digital buffers and
//! pulls disabled; do not leave analog inputs floating in a real application.
//!
//! # Not yet supported
//!
//! - Externally sampled/windowed modes, round-robin scanning and DMA.
//! - Other peripherals' analog outputs, including A2's internal channel4 sources.
//! - Physical comparator output-pin setup or filter clocks other than FRO16K.
//! - Deep-sleep operation/wakeup. Filtering requires the relevant FRO16K domain
//!   configured at HAL initialization (CMP0: VSYS; A2 CMP1: VDD_CORE).

use core::marker::PhantomData;
use core::num::NonZeroU8;

use embassy_hal_internal::drop::OnDrop;
use embassy_time::{Duration, Instant, Timer};
use maitake_sync::WaitCell;

use crate::clocks::periph_helpers::{CmpConfig, CmpInstance, SPConfHelper};
use crate::clocks::{ClockError, Gate, WakeGuard, enable_and_reset, with_clocks};
use crate::gpio::{AnyPin, GpioPin, Pull, SealedPin};
use crate::interrupt::typelevel::{Binding, Interrupt};
pub use crate::pac::cmp::Hystctr as Hysteresis;
use crate::pac::cmp::{CmpHpmd, CoutSel, FiltCnt, FuncClkSel, Vrsel};
use crate::{Peri, PeripheralType, interrupt, pac};

/// Analog comparison speed/power selection.
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Performance {
    /// Reset/normal low-power comparison.
    #[default]
    LowPower,
    /// High-speed comparison.
    HighSpeed,
    /// Lowest-power, slowest comparison.
    NanoPower,
}

/// Number of consecutive identical samples required by the digital filter.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SampleCount {
    /// Two samples.
    Two,
    /// Three samples.
    Three,
    /// Four samples.
    Four,
    /// Five samples.
    Five,
    /// Six samples.
    Six,
    /// Seven samples.
    Seven,
}

impl SampleCount {
    fn count(self) -> u64 {
        match self {
            Self::Two => 2,
            Self::Three => 3,
            Self::Four => 4,
            Self::Five => 5,
            Self::Six => 6,
            Self::Seven => 7,
        }
    }

    fn field(self) -> FiltCnt {
        match self {
            Self::Two => FiltCnt::Sample2,
            Self::Three => FiltCnt::Sample3,
            Self::Four => FiltCnt::Sample4,
            Self::Five => FiltCnt::Sample5,
            Self::Six => FiltCnt::Sample6,
            Self::Seven => FiltCnt::Sample7,
        }
    }
}

/// Continuous unfiltered comparison or internally timed low-pass filtering.
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Filter {
    /// Unclocked, unfiltered continuous output (reset configuration).
    #[default]
    Disabled,
    /// Filter using FRO16K; `period` is 1..=255 functional-clock cycles.
    Consecutive {
        /// Required consecutive identical samples.
        samples: SampleCount,
        /// Sample interval in FRO16K cycles.
        period: NonZeroU8,
    },
}

/// Comparison configuration; input ownership is fixed at construction.
#[derive(Debug, Copy, Clone)]
#[non_exhaustive]
pub struct Config {
    /// Analog speed/power tradeoff.
    pub performance: Performance,
    /// Nominal hysteresis level; see the electrical specifications.
    pub hysteresis: Hysteresis,
    /// Invert the analog output before filtering and edge detection.
    pub invert: bool,
    /// Digital filtering, disabled by default.
    pub filter: Filter,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            performance: Performance::LowPower,
            hysteresis: Hysteresis::Level0,
            invert: false,
            filter: Filter::Disabled,
        }
    }
}

/// Internal threshold DAC reference.
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum DacReference {
    /// VDD supply (VREFH0).
    #[default]
    Supply,
    /// VREFI (VREFH1), which the application must separately provide.
    VrefI,
}

/// Internal threshold DAC speed/power selection.
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum DacPower {
    /// Reset low-power DAC mode.
    #[default]
    Low,
    /// High-power DAC mode.
    High,
}

/// Comparator's internal DAC as an input; every `u8` code is legal.
#[derive(Debug, Copy, Clone)]
#[non_exhaustive]
pub struct DacInput {
    /// Output ratio `(code + 1) / 256` of the selected reference.
    pub code: u8,
    /// Reference supplied by the platform.
    pub reference: DacReference,
    /// DAC power selection.
    pub power: DacPower,
}

impl DacInput {
    /// Select a threshold relative to VDD in low-power DAC mode.
    pub const fn new(code: u8) -> Self {
        Self {
            code,
            reference: DacReference::Supply,
            power: DacPower::Low,
        }
    }
}

/// Owned external analog input; compatibility is checked by the constructor.
pub struct PinInput<'d, P: GpioPin> {
    pin: Peri<'d, P>,
}

impl<'d, P: GpioPin> PinInput<'d, P> {
    /// Reserve a pin for an instance-compatible positive or negative input.
    pub fn new(pin: Peri<'d, P>) -> Self {
        Self { pin }
    }
}

pub(crate) struct Input<'d> {
    channel: u8,
    pin: Option<Peri<'d, AnyPin>>,
    dac: Option<DacInput>,
}

pub(crate) mod sealed {
    pub(crate) trait Mode {}
    pub(crate) trait PositivePin<T> {
        const CHANNEL: u8;
    }
    pub(crate) trait NegativePin<T> {
        const CHANNEL: u8;
    }
    pub(crate) trait Positive<'d, T> {
        fn into_input(self) -> super::Input<'d>;
    }
    pub(crate) trait Negative<'d, T> {
        fn into_input(self) -> super::Input<'d>;
    }
}

/// Instance-compatible external positive input pin.
#[allow(private_bounds)]
pub trait PositivePin<T: Instance>: GpioPin + sealed::PositivePin<T> {}
/// Instance-compatible external negative input pin.
#[allow(private_bounds)]
pub trait NegativePin<T: Instance>: GpioPin + sealed::NegativePin<T> {}
/// Resource accepted as the positive input.
#[allow(private_bounds)]
pub trait PositiveInput<'d, T: Instance>: sealed::Positive<'d, T> {}
/// Resource accepted as the negative input.
#[allow(private_bounds)]
pub trait NegativeInput<'d, T: Instance>: sealed::Negative<'d, T> {}

impl<'d, T: Instance, P: PositivePin<T>> sealed::Positive<'d, T> for PinInput<'d, P> {
    fn into_input(self) -> Input<'d> {
        Input {
            channel: <P as sealed::PositivePin<T>>::CHANNEL,
            pin: Some(self.pin.into()),
            dac: None,
        }
    }
}
impl<'d, T: Instance, P: PositivePin<T>> PositiveInput<'d, T> for PinInput<'d, P> {}
impl<'d, T: Instance, P: NegativePin<T>> sealed::Negative<'d, T> for PinInput<'d, P> {
    fn into_input(self) -> Input<'d> {
        Input {
            channel: <P as sealed::NegativePin<T>>::CHANNEL,
            pin: Some(self.pin.into()),
            dac: None,
        }
    }
}
impl<'d, T: Instance, P: NegativePin<T>> NegativeInput<'d, T> for PinInput<'d, P> {}
impl<'d, T: Instance> sealed::Positive<'d, T> for DacInput {
    fn into_input(self) -> Input<'d> {
        Input {
            channel: 7,
            pin: None,
            dac: Some(self),
        }
    }
}
impl<'d, T: Instance> PositiveInput<'d, T> for DacInput {}
impl<'d, T: Instance> sealed::Negative<'d, T> for DacInput {
    fn into_input(self) -> Input<'d> {
        Input {
            channel: 7,
            pin: None,
            dac: Some(self),
        }
    }
}
impl<'d, T: Instance> NegativeInput<'d, T> for DacInput {}

/// Driver setup failure.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum SetupError {
    /// Upstream clock is unavailable.
    ClockSetup(ClockError),
    /// Hardware requires PSEL and MSEL to differ.
    SameInput,
}

/// Disabled-only configuration failure; rejected requests leave controls intact.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum ConfigError {
    /// Disable comparison before configuring it.
    Enabled,
    /// Selected filter clock is unavailable.
    ClockSetup(ClockError),
    /// Neither input uses the comparator's internal DAC.
    NoDac,
}

/// Output observation failure, separate from setup/configuration errors.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// Comparison has not been enabled.
    Disabled,
    /// Analog/filter initialization is still being blanked.
    NotReady,
    /// The interrupt notification cell was closed.
    WaitClosed,
}

/// Sealed blocking/async driver mode.
#[allow(private_bounds)]
pub trait Mode: sealed::Mode {}
/// Interrupt-driven modes.
#[allow(private_bounds)]
pub trait AsyncMode: Mode {}
/// Polling/blocking observation mode.
pub struct Blocking;
impl sealed::Mode for Blocking {}
impl Mode for Blocking {}
/// Interrupt-driven observation mode.
pub struct Async;
impl sealed::Mode for Async {}
impl Mode for Async {}
impl AsyncMode for Async {}

pub(crate) enum Registers {
    Cmp0(pac::cmp::Cmp),
    #[cfg(feature = "_mcxa2xx")]
    Cmp1(pac::cmp1::Cmp1),
}

// CMP1's generated fieldset types are distinct even where the layouts agree.
// Dispatch operations through each actual PAC type rather than casting handles.
macro_rules! with_regs {
    ($this:expr, $r:ident, $body:expr) => {
        match &$this {
            Registers::Cmp0($r) => $body,
            #[cfg(feature = "_mcxa2xx")]
            Registers::Cmp1($r) => $body,
        }
    };
}

impl Registers {
    fn disable_round_robin(&self) {
        match self {
            Self::Cmp0(r) => r.rrcr0().modify(|w| w.set_rr_en(false)),
            #[cfg(feature = "_mcxa2xx")]
            Self::Cmp1(r) => r.rrcr0().modify(|w| w.set_rr_en(false)),
        }
    }
}

pub(crate) struct Info {
    pub(crate) regs: Registers,
    pub(crate) wait_cell: WaitCell,
    pub(crate) instance: CmpInstance,
    pub(crate) disable_clock: unsafe fn(),
    pub(crate) disable_interrupt: fn(),
}
unsafe impl Sync for Info {}

impl Info {
    fn mask(&self) {
        with_regs!(self.regs, r, r.ier().write(|_| {}));
    }
    fn clear_edges(&self) {
        with_regs!(
            self.regs,
            r,
            r.csr().write(|w| {
                w.set_cfr(true);
                w.set_cff(true);
            })
        );
    }
}

pub(crate) trait SealedInstance: Gate<MrccPeriphConfig = CmpConfig> {
    fn info() -> &'static Info;
}
/// Comparator instance with sealed register/pin information.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static + Send {
    /// Interrupt for this instance.
    type Interrupt: Interrupt;
}

pub(crate) fn disable_interrupt<T: Instance>() {
    T::Interrupt::disable();
    T::Interrupt::unpend();
}

/// Peripheral-instance-erased continuous comparator.
pub struct Cmp<'d, M: Mode> {
    info: &'static Info,
    pins: [Option<Peri<'d, AnyPin>>; 2],
    dac: Option<DacInput>,
    config: Config,
    settling_us: u64,
    ready_at: Option<Instant>,
    _wg: Option<WakeGuard>,
    _mode: PhantomData<&'d mut M>,
}

impl<'d, M: Mode> Cmp<'d, M> {
    fn new_inner<T: Instance>(
        _peri: Peri<'d, T>,
        positive: impl PositiveInput<'d, T>,
        negative: impl NegativeInput<'d, T>,
        config: Config,
    ) -> Result<Self, SetupError> {
        let positive = sealed::Positive::into_input(positive);
        let negative = sealed::Negative::into_input(negative);
        if positive.channel == negative.channel {
            return Err(SetupError::SameInput);
        }
        let info = T::info();
        let clock = CmpConfig {
            instance: info.instance,
            filtered: config.filter != Filter::Disabled,
        };
        critical_section::with(|_| {
            T::Interrupt::disable();
            // SAFETY: the peripheral token proves exclusive ownership.
            let parts = unsafe { enable_and_reset::<T>(&clock) }.map_err(SetupError::ClockSetup)?;
            info.mask();
            with_regs!(info.regs, r, {
                r.ccr0().write(|_| {});
                r.dcr().write(|_| {});
            });
            info.regs.disable_round_robin();
            info.clear_edges();
            let mut inst = Self {
                info,
                pins: [positive.pin, negative.pin],
                dac: positive.dac.or(negative.dac),
                config,
                settling_us: 0,
                ready_at: None,
                _wg: parts.wake_guard,
                _mode: PhantomData,
            };
            for pin in inst.pins.iter().flatten() {
                pin.set_as_disabled();
                pin.set_pull(Pull::Disabled);
            }
            with_regs!(
                inst.info.regs,
                r,
                r.ccr2().write(|w| {
                    w.set_psel(positive.channel.into());
                    w.set_msel(negative.channel.into());
                })
            );
            inst.apply_configuration(config, parts.freq);
            Ok(inst)
        })
    }

    fn apply_configuration(&mut self, config: Config, frequency: u32) {
        self.info.mask();
        with_regs!(self.info.regs, r, {
            r.ccr1().write(|w| {
                w.set_cout_inv(config.invert);
                w.set_cout_sel((CoutSel::Cout as u8).into());
                match config.filter {
                    Filter::Disabled => {
                        w.set_filt_cnt((FiltCnt::Bypassed as u8).into());
                        w.set_func_clk_sel((FuncClkSel::Func0 as u8).into());
                    }
                    Filter::Consecutive { samples, period } => {
                        w.set_func_clk_sel((FuncClkSel::Func1 as u8).into());
                        w.set_filt_cnt((samples.field() as u8).into());
                        w.set_filt_per(period.get());
                    }
                }
            });
            r.ccr2().modify(|w| {
                w.set_cmp_hpmd(
                    (if config.performance == Performance::HighSpeed {
                        CmpHpmd::High
                    } else {
                        CmpHpmd::Low
                    } as u8)
                        .into(),
                );
                w.set_cmp_npmd(config.performance == Performance::NanoPower);
                w.set_hystctr((config.hysteresis as u8).into());
            });
            r.dcr().write(|w| {
                if let Some(dac) = self.dac {
                    w.set_dac_data(dac.code);
                    w.set_dac_hpmd(dac.power == DacPower::High);
                    w.set_vrsel(
                        (match dac.reference {
                            DacReference::Supply => Vrsel::Vref0,
                            DacReference::VrefI => Vrsel::Vref1,
                        } as u8)
                            .into(),
                    );
                }
            });
        });
        self.settling_us = 50
            + match config.filter {
                Filter::Disabled => 0,
                Filter::Consecutive { samples, period } => {
                    // RM mode4B: (count*period+1) functional cycles. Allow 10%
                    // below the configured FRO16K frequency; round duration up.
                    let min_hz = u64::from(frequency) * 90 / 100;
                    ((samples.count() * u64::from(period.get()) + 1) * 1_000_000).div_ceil(min_hz)
                }
            };
        self.config = config;
        self.ready_at = None;
        self.info.clear_edges();
    }

    /// Whether the hardware comparison-enable bit is set (including priming).
    pub fn is_enabled(&self) -> bool {
        with_regs!(self.info.regs, r, r.ccr0().read().cmp_en())
    }

    /// Enable comparison and start its blanking interval; idempotent if enabled.
    pub fn enable(&mut self) {
        critical_section::with(|_| {
            if self.is_enabled() {
                return;
            }
            self.info.mask();
            self.info.clear_edges();
            with_regs!(self.info.regs, r, {
                r.dcr().modify(|w| w.set_dac_en(self.dac.is_some()));
                r.ccr0().modify(|w| w.set_cmp_en(true));
            });
            self.ready_at = Some(Instant::now() + Duration::from_micros(self.settling_us));
        });
    }

    /// Disable comparison/DAC and mask/clear edge sources, retaining input pins.
    pub fn disable(&mut self) {
        critical_section::with(|_| {
            self.info.mask();
            with_regs!(self.info.regs, r, {
                r.ccr0().modify(|w| w.set_cmp_en(false));
                r.dcr().modify(|w| w.set_dac_en(false));
            });
            self.info.clear_edges();
            self.ready_at = None;
        });
    }

    /// Change controls only while disabled; successful configuration stays disabled.
    pub fn set_config(&mut self, config: &Config) -> Result<(), ConfigError> {
        critical_section::with(|_| {
            if self.is_enabled() {
                return Err(ConfigError::Enabled);
            }
            let clock = CmpConfig {
                instance: self.info.instance,
                filtered: config.filter != Filter::Disabled,
            };
            let parts = with_clocks(move |clocks| clock.pre_enable_config(clocks))
                .ok_or(ConfigError::ClockSetup(ClockError::NeverInitialized))?
                .map_err(ConfigError::ClockSetup)?;
            self.apply_configuration(*config, parts.freq);
            self._wg = parts.wake_guard;
            Ok(())
        })
    }

    /// Change the internal threshold only while disabled; all `u8` codes are legal.
    pub fn set_dac_code(&mut self, code: u8) -> Result<(), ConfigError> {
        critical_section::with(|_| {
            if self.is_enabled() {
                return Err(ConfigError::Enabled);
            }
            let dac = self.dac.as_mut().ok_or(ConfigError::NoDac)?;
            dac.code = code;
            with_regs!(self.info.regs, r, r.dcr().modify(|w| w.set_dac_data(code)));
            Ok(())
        })
    }

    /// Read COUT without consuming edge flags; returns `NotReady` during priming.
    pub fn is_high(&self) -> Result<bool, Error> {
        self.ready()?;
        Ok(with_regs!(self.info.regs, r, r.csr().read().cout()))
    }

    fn ready(&self) -> Result<(), Error> {
        if !self.is_enabled() {
            return Err(Error::Disabled);
        }
        if self.ready_at.is_none_or(|deadline| Instant::now() < deadline) {
            return Err(Error::NotReady);
        }
        Ok(())
    }

    fn begin_wait(&self) {
        critical_section::with(|_| {
            self.info.mask();
            self.info.clear_edges();
        });
    }

    fn predicate(&self, condition: Condition, interrupt: bool) -> bool {
        critical_section::with(|_| {
            if interrupt {
                with_regs!(
                    self.info.regs,
                    r,
                    r.ier().write(|w| {
                        w.set_cfr_ie(matches!(
                            condition,
                            Condition::High | Condition::Rising | Condition::Any
                        ));
                        w.set_cff_ie(matches!(
                            condition,
                            Condition::Low | Condition::Falling | Condition::Any
                        ));
                    })
                );
            }
            let (high, rising, falling) = with_regs!(self.info.regs, r, {
                let status = r.csr().read();
                (status.cout(), status.cfr(), status.cff())
            });
            match condition {
                Condition::High => high || rising,
                Condition::Low => !high || falling,
                Condition::Rising => rising,
                Condition::Falling => falling,
                Condition::Any => rising || falling,
            }
        })
    }
}

impl<'d> Cmp<'d, Blocking> {
    /// Construct a disabled comparator without binding its interrupt.
    pub fn new_blocking<T: Instance>(
        peri: Peri<'d, T>,
        positive: impl PositiveInput<'d, T>,
        negative: impl NegativeInput<'d, T>,
        config: Config,
    ) -> Result<Self, SetupError> {
        Self::new_inner(peri, positive, negative, config)
    }

    fn blocking_wait(&mut self, condition: Condition) -> Result<(), Error> {
        while let Err(error) = self.ready() {
            if error != Error::NotReady {
                return Err(error);
            }
            core::hint::spin_loop();
        }
        self.begin_wait();
        let _cleanup = OnDrop::new(|| self.begin_wait());
        while !self.predicate(condition, false) {
            core::hint::spin_loop();
        }
        Ok(())
    }

    /// Block for high COUT or a newly latched high pulse; no timeout.
    pub fn blocking_wait_for_high(&mut self) -> Result<(), Error> {
        self.blocking_wait(Condition::High)
    }
    /// Block for low COUT or a newly latched low pulse; no timeout.
    pub fn blocking_wait_for_low(&mut self) -> Result<(), Error> {
        self.blocking_wait(Condition::Low)
    }
    /// Block for a fresh rising COUT edge; no timeout.
    pub fn blocking_wait_for_rising_edge(&mut self) -> Result<(), Error> {
        self.blocking_wait(Condition::Rising)
    }
    /// Block for a fresh falling COUT edge; no timeout.
    pub fn blocking_wait_for_falling_edge(&mut self) -> Result<(), Error> {
        self.blocking_wait(Condition::Falling)
    }
    /// Block for either fresh COUT edge; no timeout.
    pub fn blocking_wait_for_any_edge(&mut self) -> Result<(), Error> {
        self.blocking_wait(Condition::Any)
    }
}

impl<'d> Cmp<'d, Async> {
    /// Construct a disabled comparator with an interrupt binding.
    pub fn new_async<T: Instance>(
        peri: Peri<'d, T>,
        positive: impl PositiveInput<'d, T>,
        negative: impl NegativeInput<'d, T>,
        _irq: impl Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        config: Config,
    ) -> Result<Self, SetupError> {
        let inst = Self::new_inner(peri, positive, negative, config)?;
        T::Interrupt::unpend();
        // SAFETY: the binding provides an appropriate handler; sources are masked.
        unsafe { T::Interrupt::enable() };
        Ok(inst)
    }
}

impl<M: AsyncMode> Cmp<'_, M> {
    async fn wait(&mut self, condition: Condition) -> Result<(), Error> {
        if !self.is_enabled() {
            return Err(Error::Disabled);
        }
        let deadline = self.ready_at.ok_or(Error::NotReady)?;
        // Timer deliberately yields even for a past deadline. Do not postpone
        // an already-ready observation window or an already-satisfied level.
        if Instant::now() < deadline {
            Timer::at(deadline).await;
        }
        self.begin_wait();
        let _cleanup = OnDrop::new(|| self.begin_wait());
        self.info
            .wait_cell
            .wait_for(|| self.predicate(condition, true))
            .await
            .map_err(|_| Error::WaitClosed)
    }

    /// Wait for high COUT or a new high pulse, even if it ends before repolling.
    /// Cancellation leaves comparison running.
    pub async fn wait_for_high(&mut self) -> Result<(), Error> {
        self.wait(Condition::High).await
    }
    /// Wait for low COUT or a new low pulse, even if it ends before repolling.
    /// Cancellation leaves comparison running.
    pub async fn wait_for_low(&mut self) -> Result<(), Error> {
        self.wait(Condition::Low).await
    }
    /// Observe a fresh rising edge after priming.
    pub async fn wait_for_rising_edge(&mut self) -> Result<(), Error> {
        self.wait(Condition::Rising).await
    }
    /// Observe a fresh falling edge after priming.
    pub async fn wait_for_falling_edge(&mut self) -> Result<(), Error> {
        self.wait(Condition::Falling).await
    }
    /// Observe either fresh edge; simultaneous flags do not imply an ordering.
    pub async fn wait_for_any_edge(&mut self) -> Result<(), Error> {
        self.wait(Condition::Any).await
    }
}

#[derive(Copy, Clone)]
enum Condition {
    High,
    Low,
    Rising,
    Falling,
    Any,
}

/// Comparator interrupt handler: mask and wake, preserving latched edge evidence.
pub struct InterruptHandler<T: Instance> {
    _instance: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        critical_section::with(|_| {
            // A higher-priority task may have dropped the owner before this
            // handler entered the critical section. Inspect NVIC, not the now
            // potentially unclocked peripheral, before accessing registers.
            if !T::Interrupt::is_enabled() {
                return;
            }
            let info = T::info();
            let pending = with_regs!(info.regs, r, {
                let enabled = r.ier().read();
                let status = r.csr().read();
                (enabled.cfr_ie() && status.cfr()) || (enabled.cff_ie() && status.cff())
            });
            if pending {
                info.mask();
                info.wait_cell.wake();
            }
        });
    }
}

impl<M: Mode> Drop for Cmp<'_, M> {
    fn drop(&mut self) {
        critical_section::with(|_| {
            self.disable();
            (self.info.disable_interrupt)();
            for pin in self.pins.iter().flatten() {
                pin.set_as_disabled();
            }
            cortex_m::asm::dsb();
            // SAFETY: sources and NVIC are disabled, pending state is cleared,
            // and the shared MRCC/GPIO RMW operations are serialized.
            unsafe { (self.info.disable_clock)() };
            cortex_m::asm::dsb();
        });
    }
}

impl<M: Mode> embassy_embedded_hal::SetConfig for Cmp<'_, M> {
    type Config = Config;
    type ConfigError = ConfigError;
    fn set_config(&mut self, config: &Config) -> Result<(), ConfigError> {
        Cmp::set_config(self, config)
    }
}

impl embedded_hal_1::digital::Error for Error {
    fn kind(&self) -> embedded_hal_1::digital::ErrorKind {
        embedded_hal_1::digital::ErrorKind::Other
    }
}
impl<M: Mode> embedded_hal_1::digital::ErrorType for Cmp<'_, M> {
    type Error = Error;
}
impl<M: Mode> embedded_hal_1::digital::InputPin for Cmp<'_, M> {
    fn is_high(&mut self) -> Result<bool, Error> {
        Cmp::is_high(self)
    }
    fn is_low(&mut self) -> Result<bool, Error> {
        Cmp::is_high(self).map(|high| !high)
    }
}
impl<M: AsyncMode> embedded_hal_async::digital::Wait for Cmp<'_, M> {
    async fn wait_for_high(&mut self) -> Result<(), Error> {
        Cmp::wait_for_high(self).await
    }
    async fn wait_for_low(&mut self) -> Result<(), Error> {
        Cmp::wait_for_low(self).await
    }
    async fn wait_for_rising_edge(&mut self) -> Result<(), Error> {
        Cmp::wait_for_rising_edge(self).await
    }
    async fn wait_for_falling_edge(&mut self) -> Result<(), Error> {
        Cmp::wait_for_falling_edge(self).await
    }
    async fn wait_for_any_edge(&mut self) -> Result<(), Error> {
        Cmp::wait_for_any_edge(self).await
    }
}

/// Implement one comparator from selected-chip metadata.
#[doc(hidden)]
#[macro_export]
macro_rules! impl_cmp_instance {
    ($n:literal) => {
        paste::paste! {
            impl $crate::cmp::SealedInstance for $crate::peripherals::[<CMP $n>] {
                fn info() -> &'static $crate::cmp::Info {
                    static INFO: $crate::cmp::Info = $crate::cmp::Info {
                        regs: $crate::cmp::Registers::[<Cmp $n>]($crate::pac::[<CMP $n>]),
                        wait_cell: maitake_sync::WaitCell::new(),
                        instance: $crate::clocks::periph_helpers::CmpInstance::[<Cmp $n>],
                        disable_clock: $crate::clocks::disable::<$crate::peripherals::[<CMP $n>]>,
                        disable_interrupt: $crate::cmp::disable_interrupt::<$crate::peripherals::[<CMP $n>]>,
                    };
                    &INFO
                }
            }
            impl $crate::cmp::Instance for $crate::peripherals::[<CMP $n>] {
                type Interrupt = $crate::interrupt::typelevel::[<CMP $n>];
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_cmp_positive_pin {
    ($pin:ident, $peri:ident, $channel:literal) => {
        impl $crate::cmp::sealed::PositivePin<$crate::peripherals::$peri> for $crate::peripherals::$pin {
            const CHANNEL: u8 = $channel;
        }
        impl $crate::cmp::PositivePin<$crate::peripherals::$peri> for $crate::peripherals::$pin {}
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_cmp_negative_pin {
    ($pin:ident, $peri:ident, $channel:literal) => {
        impl $crate::cmp::sealed::NegativePin<$crate::peripherals::$peri> for $crate::peripherals::$pin {
            const CHANNEL: u8 = $channel;
        }
        impl $crate::cmp::NegativePin<$crate::peripherals::$peri> for $crate::peripherals::$pin {}
    };
}

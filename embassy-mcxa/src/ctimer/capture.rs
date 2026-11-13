//! CTimer-based Capture driver

use core::fmt;
use core::marker::PhantomData;
use core::mem::DropGuard;
use core::ops::{Add, Sub};
use core::sync::atomic::Ordering;

use embassy_hal_internal::Peri;

use super::{AnyChannel, CTimer, CTimerChannel, Channel, Info, InputPin, Instance};
use crate::clocks::WakeGuard;
use crate::gpio::{AnyPin, SealedPin};
use crate::inputmux::{SealedValidInputMuxConfig, ValidInputMuxConfig};
use crate::interrupt;
use crate::interrupt::typelevel::Interrupt;

/// Capture error.
#[derive(Debug)]
#[non_exhaustive]
pub enum CaptureError {
    /// Internal input routing could not enable its register-interface clock.
    ClockSetup(crate::clocks::ClockError),
    /// Other
    Other,
}

/// Capture configuration
#[derive(Debug, Copy, Clone, Default)]
#[non_exhaustive]
pub struct Config {
    /// Edge capture
    pub edge: Edge,
}

/// Capture configuration
#[derive(Debug, Copy, Clone, Default)]
pub enum Edge {
    /// Rising edge
    RisingEdge,
    /// Falling edge
    FallingEdge,
    /// Both edges
    #[default]
    Both,
}

/// Timestamp capture
///
/// Timestamp value in ticks.
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Timestamp(u32);

impl Timestamp {
    #[inline]
    fn ticks(self) -> u32 {
        self.0
    }

    #[inline]
    fn with_ticks(self, ticks: u32) -> Self {
        Self(ticks)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, PartialOrd, Ord, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct TicksDiff(pub i32);

impl TicksDiff {
    #[inline]
    pub fn to_period(self, tick_hz: u32) -> f32 {
        assert!(tick_hz != 0);
        self.0 as f32 / tick_hz as f32
    }

    #[inline]
    pub fn to_frequency(self, tick_hz: u32) -> f32 {
        assert!(self.0 != 0);
        tick_hz as f32 / self.0 as f32
    }

    #[inline]
    pub fn abs(self) -> TicksDiff {
        TicksDiff(self.0.abs())
    }
}

impl fmt::Debug for TicksDiff {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ticks", self.0)
    }
}

impl Add for Timestamp {
    type Output = TicksDiff;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        let lhs = self.ticks() as i32;
        let rhs = rhs.ticks() as i32;
        let raw = lhs.wrapping_add(rhs);
        TicksDiff(raw)
    }
}

impl Add<u32> for Timestamp {
    type Output = Timestamp;

    #[inline]
    fn add(self, rhs: u32) -> Self::Output {
        self.with_ticks(self.ticks().wrapping_add(rhs))
    }
}

impl Add<TicksDiff> for Timestamp {
    type Output = Timestamp;

    #[inline]
    fn add(self, rhs: TicksDiff) -> Self::Output {
        let t = self.ticks().wrapping_add(rhs.0 as u32);
        self.with_ticks(t)
    }
}

impl Sub for Timestamp {
    type Output = TicksDiff;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        let lhs = self.ticks() as i32;
        let rhs = rhs.ticks() as i32;
        let raw = lhs.wrapping_sub(rhs);
        TicksDiff(raw)
    }
}

impl Sub<u32> for Timestamp {
    type Output = Timestamp;

    #[inline]
    fn sub(self, rhs: u32) -> Self::Output {
        self.with_ticks(self.ticks().wrapping_sub(rhs))
    }
}

impl Sub<TicksDiff> for Timestamp {
    type Output = Timestamp;

    #[inline]
    fn sub(self, rhs: TicksDiff) -> Self::Output {
        // Subtracting a signed diff == adding the negated diff
        let t = self.ticks().wrapping_sub(rhs.0 as u32);
        self.with_ticks(t)
    }
}

/// Capture driver
pub struct Capture<'d> {
    info: &'static Info,
    ch: Peri<'d, AnyChannel>,
    pin: Option<Peri<'d, AnyPin>>,
    #[cfg(mcxa_aoi)]
    _aoi: Option<crate::aoi::Output<'d>>,
    source_freq: u32,
    _wg: Option<WakeGuard>,
}

impl<'d> Capture<'d> {
    /// Create Capture driver
    ///
    /// Upon `Drop`, the external `pin` will be placed into `Disabled`
    /// state.
    pub fn new_with_input_pin<T: Instance, CH: CTimerChannel<T>, PIN: InputPin<T>>(
        ctimer: CTimer<'d>,
        ch: Peri<'d, CH>,
        pin: Peri<'d, PIN>,
        _irq: impl crate::interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        config: Config,
    ) -> Result<Self, CaptureError>
    where
        (T, CH, PIN): ValidInputMuxConfig,
    {
        pin.mux();
        <(T, CH, PIN) as SealedValidInputMuxConfig>::mux();

        let mut inst = Self {
            info: T::info(),
            ch: ch.into(),
            pin: Some(pin.into()),
            #[cfg(mcxa_aoi)]
            _aoi: None,
            source_freq: ctimer._freq,
            _wg: ctimer._wg.clone(),
        };

        inst.set_configuration(&config)?;

        T::Interrupt::unpend();

        // Safety: `_irq` ensures an Interrupt Handler exists.
        unsafe { T::Interrupt::enable() };

        // Enable CTimer
        inst.info.regs().tcr().modify(|w| w.set_cen(true));

        Ok(inst)
    }

    /// Capture an AOI0 event through the internal INPUTMUX route, without a pin.
    ///
    /// The borrowed output keeps its AOI owner alive until this capture driver
    /// is dropped. Dropping capture disables its channel before releasing that
    /// borrow. Configure AOI before constructing this downstream consumer.
    #[cfg(mcxa_aoi)]
    pub fn new_with_aoi<T: Instance, CH: CTimerChannel<T>>(
        ctimer: CTimer<'d>,
        ch: Peri<'d, CH>,
        source: crate::aoi::Output<'d>,
        _irq: impl crate::interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        config: Config,
    ) -> Result<Self, CaptureError> {
        let channel: Peri<'d, AnyChannel> = ch.into();
        crate::inputmux::configure_ctimer_aoi(T::NUMBER, channel.number().into(), source.event)
            .map_err(CaptureError::ClockSetup)?;
        let mut inst = Self {
            info: T::info(),
            ch: channel,
            pin: None,
            _aoi: Some(source),
            source_freq: ctimer._freq,
            _wg: ctimer._wg.clone(),
        };
        inst.set_configuration(&config)?;
        inst.clear_pending();
        T::Interrupt::unpend();
        // SAFETY: the binding provides this timer's capture interrupt handler.
        unsafe { T::Interrupt::enable() };
        inst.info.regs().tcr().modify(|w| w.set_cen(true));
        Ok(inst)
    }

    fn set_configuration(&mut self, config: &Config) -> Result<(), CaptureError> {
        let rising = matches!(config.edge, Edge::Both | Edge::RisingEdge);
        let falling = matches!(config.edge, Edge::Both | Edge::FallingEdge);
        critical_section::with(|_| {
            self.info.regs().ccr().modify(|w| match self.ch.number() {
                Channel::Zero => {
                    w.set_cap0re(rising);
                    w.set_cap0fe(falling);
                }
                Channel::One => {
                    w.set_cap1re(rising);
                    w.set_cap1fe(falling);
                }
                Channel::Two => {
                    w.set_cap2re(rising);
                    w.set_cap2fe(falling);
                }
                Channel::Three => {
                    w.set_cap3re(rising);
                    w.set_cap3fe(falling);
                }
            });
        });

        Ok(())
    }

    pub fn frequency(&self) -> u32 {
        self.source_freq
    }

    fn set_interrupt(&self, enabled: bool) {
        critical_section::with(|_| {
            self.info.regs().ccr().modify(|w| match self.ch.number() {
                Channel::Zero => w.set_cap0i(enabled),
                Channel::One => w.set_cap1i(enabled),
                Channel::Two => w.set_cap2i(enabled),
                Channel::Three => w.set_cap3i(enabled),
            });
        });
    }

    fn clear_pending(&self) {
        critical_section::with(|_| {
            self.info.regs().ir().write(|w| match self.ch.number() {
                Channel::Zero => w.set_cr0int(true),
                Channel::One => w.set_cr1int(true),
                Channel::Two => w.set_cr2int(true),
                Channel::Three => w.set_cr3int(true),
            });
            let n: usize = self.ch.number().into();
            self.info.irq_flags().fetch_and(!(1 << n), Ordering::AcqRel);
        });
    }

    /// Wait for a fresh capture; cancellation masks this channel's interrupt.
    ///
    /// Hardware retains only the latest timestamp, not a lossless edge queue.
    pub async fn capture(&mut self) -> Result<Timestamp, CaptureError> {
        self.set_interrupt(false);
        self.clear_pending();
        let _cleanup = DropGuard::new((), |()| {
            self.set_interrupt(false);
            self.clear_pending();
        });
        self.info
            .wait_cell(self.ch.number())
            .wait_for(|| {
                self.set_interrupt(true);
                let n: usize = self.ch.number().into();
                let mask = 1 << n;
                (self.info.irq_flags().fetch_and(!mask, Ordering::AcqRel) & mask) != 0
            })
            .await
            .map_err(|_| CaptureError::Other)?;

        let timestamp = self.info.regs().cr(self.ch.number().into()).read().cap();
        Ok(Timestamp(timestamp))
    }
}

impl<'d> Drop for Capture<'d> {
    fn drop(&mut self) {
        critical_section::with(|_| {
            self.info.regs().ccr().modify(|w| match self.ch.number() {
                Channel::Zero => {
                    w.set_cap0re(false);
                    w.set_cap0fe(false);
                    w.set_cap0i(false);
                }
                Channel::One => {
                    w.set_cap1re(false);
                    w.set_cap1fe(false);
                    w.set_cap1i(false);
                }
                Channel::Two => {
                    w.set_cap2re(false);
                    w.set_cap2fe(false);
                    w.set_cap2i(false);
                }
                Channel::Three => {
                    w.set_cap3re(false);
                    w.set_cap3fe(false);
                    w.set_cap3i(false);
                }
            });
            self.clear_pending();
            cortex_m::asm::dsb();
        });
        if let Some(pin) = &self.pin {
            pin.set_as_disabled();
        }
    }
}

/// CTimer interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        T::PERF_INT_INCR();
        // Keep status acknowledgement, CCR updates, and completion publication
        // atomic against higher-priority tasks cancelling/dropping a channel.
        // Otherwise a preempted CCR RMW could resurrect a released channel.
        critical_section::with(|_| {
            let ir = T::info().regs().ir().read();
            T::info().regs().ir().write(|w| {
                w.set_cr0int(ir.cr0int());
                w.set_cr1int(ir.cr1int());
                w.set_cr2int(ir.cr2int());
                w.set_cr3int(ir.cr3int());
            });

            let mut mask = 0;
            T::info().regs().ccr().modify(|w| {
                if ir.cr0int() {
                    w.set_cap0i(false);
                    mask |= 1 << 0;
                }

                if ir.cr1int() {
                    w.set_cap1i(false);
                    mask |= 1 << 1;
                }

                if ir.cr2int() {
                    w.set_cap2i(false);
                    mask |= 1 << 2;
                }

                if ir.cr3int() {
                    w.set_cap3i(false);
                    mask |= 1 << 3;
                }
            });
            T::info().irq_flags().fetch_or(mask, Ordering::Release);

            for channel in [Channel::Zero, Channel::One, Channel::Two, Channel::Three] {
                let index: usize = channel.into();
                if mask & (1 << index) != 0 {
                    T::PERF_INT_WAKE_INCR();
                    T::info().wait_cell(channel).wake();
                }
            }
        });
    }
}

impl<'d> embassy_embedded_hal::SetConfig for Capture<'d> {
    type Config = Config;
    type ConfigError = CaptureError;

    fn set_config(&mut self, config: &Self::Config) -> Result<(), Self::ConfigError> {
        self.set_configuration(config)
    }
}

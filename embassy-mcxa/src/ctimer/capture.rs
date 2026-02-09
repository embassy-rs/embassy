//! CTimer-based Capture driver

use core::fmt;
use core::marker::PhantomData;
use core::ops::{Add, Sub};
use core::sync::atomic::Ordering;

use embassy_hal_internal::Peri;
use nxp_pac::ctimer::vals::{
    Cap0fe, Cap0i, Cap0re, Cap1fe, Cap1i, Cap1re, Cap2fe, Cap2i, Cap2re, Cap3fe, Cap3i, Cap3re,
};

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
    pin: Peri<'d, AnyPin>,
    source_freq: u32,
    _wg: Option<WakeGuard>,
}

impl<'d> Capture<'d> {
    /// Create Capture driver
    ///
    /// Upon `Drop`, the external `pin` will be placed into `Disabled`
    /// state.
    pub fn new_with_input_pin<T: Instance, CH: CTimerChannel<T>, PIN: InputPin>(
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
            pin: pin.into(),
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

    fn set_configuration(&mut self, config: &Config) -> Result<(), CaptureError> {
        self.info.regs().ccr().modify(|w| {
            match self.ch.number() {
                Channel::Zero => match config.edge {
                    Edge::Both => {
                        w.set_cap0re(Cap0re::CAPORE_1);
                        w.set_cap0fe(Cap0fe::CAPOFE_1);
                    }
                    Edge::RisingEdge => {
                        w.set_cap0re(Cap0re::CAPORE_1);
                    }
                    Edge::FallingEdge => {
                        w.set_cap0fe(Cap0fe::CAPOFE_1);
                    }
                },
                Channel::One => match config.edge {
                    Edge::Both => {
                        w.set_cap1re(Cap1re::CAP1RE_1);
                        w.set_cap1fe(Cap1fe::CAP1FE_1);
                    }
                    Edge::RisingEdge => {
                        w.set_cap1re(Cap1re::CAP1RE_1);
                    }
                    Edge::FallingEdge => {
                        w.set_cap1fe(Cap1fe::CAP1FE_1);
                    }
                },
                Channel::Two => match config.edge {
                    Edge::Both => {
                        w.set_cap2re(Cap2re::CAP2RE_1);
                        w.set_cap2fe(Cap2fe::CAP2FE_1);
                    }
                    Edge::RisingEdge => {
                        w.set_cap2re(Cap2re::CAP2RE_1);
                    }
                    Edge::FallingEdge => {
                        w.set_cap2fe(Cap2fe::CAP2FE_1);
                    }
                },
                Channel::Three => match config.edge {
                    Edge::Both => {
                        w.set_cap3re(Cap3re::CAP3RE_1);
                        w.set_cap3fe(Cap3fe::CAP3FE_1);
                    }
                    Edge::RisingEdge => {
                        w.set_cap3re(Cap3re::CAP3RE_1);
                    }
                    Edge::FallingEdge => {
                        w.set_cap3fe(Cap3fe::CAP3FE_1);
                    }
                },
            };
        });

        Ok(())
    }

    pub fn frequency(&self) -> u32 {
        self.source_freq
    }

    pub async fn capture(&mut self) -> Result<Timestamp, CaptureError> {
        self.info
            .wait_cell()
            .wait_for(|| {
                self.info.regs().ccr().modify(|w| match self.ch.number() {
                    Channel::Zero => {
                        w.set_cap0i(Cap0i::CAPOI_1);
                    }
                    Channel::One => {
                        w.set_cap1i(Cap1i::CAP1I_1);
                    }
                    Channel::Two => {
                        w.set_cap2i(Cap2i::CAP2I_1);
                    }
                    Channel::Three => {
                        w.set_cap3i(Cap3i::CAP3I_1);
                    }
                });

                let n: usize = self.ch.number().into();
                let mask = 1 << n;
                (self.info.irq_flags().fetch_and(!mask, Ordering::AcqRel)) != 0
            })
            .await
            .map_err(|_| CaptureError::Other)?;

        let timestamp = self.info.regs().cr(self.ch.number().into()).read().cap();
        Ok(Timestamp(timestamp))
    }
}

impl<'d> Drop for Capture<'d> {
    fn drop(&mut self) {
        self.pin.set_as_disabled();
    }
}

/// CTimer interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        T::PERF_INT_INCR();
        // Clear interrupt status
        let ir = T::info().regs().ir().read();
        T::info().regs().ir().write(|w| w.0 = ir.0);

        let mut mask = 0;
        T::info().regs().ccr().modify(|w| {
            if ir.cr0int() {
                w.set_cap0i(Cap0i::CAP0I_0);
                mask |= 1 << 0;
            }

            if ir.cr1int() {
                w.set_cap1i(Cap1i::CAP1I_0);
                mask |= 1 << 1;
            }

            if ir.cr2int() {
                w.set_cap2i(Cap2i::CAP2I_0);
                mask |= 1 << 2;
            }

            if ir.cr3int() {
                w.set_cap3i(Cap3i::CAP3I_0);
                mask |= 1 << 3;
            }
        });
        T::info().irq_flags().fetch_or(mask, Ordering::Release);

        T::PERF_INT_WAKE_INCR();
        T::info().wait_cell().wake();
    }
}

impl<'d> embassy_embedded_hal::SetConfig for Capture<'d> {
    type Config = Config;
    type ConfigError = CaptureError;

    fn set_config(&mut self, config: &Self::Config) -> Result<(), Self::ConfigError> {
        self.set_configuration(config)
    }
}

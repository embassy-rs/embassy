//! CTimer-based Capture driver.
//!
//! Suspension disables this channel's edge detection and interrupt before
//! releasing its guard. A shared timer stops only after its last PWM/capture
//! user stops; captured timestamps do not include time spent with it stopped.

use core::fmt;
use core::marker::PhantomData;
use core::ops::{Add, Sub};
use core::sync::atomic::Ordering;

use embassy_hal_internal::Peri;

use super::{AnyChannel, CTimer, CTimerChannel, Channel, Info, InputPin, Instance};
use crate::clocks::{PoweredClock, WakeGuard};
use crate::gpio::{AnyPin, SealedPin};
use crate::inputmux::{SealedValidInputMuxConfig, ValidInputMuxConfig};
use crate::interrupt;
use crate::interrupt::typelevel::Interrupt;

/// Capture error.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum CaptureError {
    /// The channel does not belong to the supplied timer core.
    TimerMismatch,
    /// Resume the channel before waiting for a capture.
    Suspended,
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
    power: PoweredClock,
    config: Config,
    suspended: bool,
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
        if !core::ptr::eq(ctimer.info, T::info()) {
            return Err(CaptureError::TimerMismatch);
        }
        pin.mux();
        <(T, CH, PIN) as SealedValidInputMuxConfig>::mux();

        let mut inst = Self {
            info: T::info(),
            ch: ch.into(),
            pin: pin.into(),
            source_freq: ctimer._freq,
            _wg: ctimer._wg.clone(),
            power: ctimer.power,
            config,
            suspended: false,
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
        critical_section::with(|_| {
            self.info.regs().ccr().modify(|w| {
                w.0 &= !(0b11 << (usize::from(self.ch.number()) * 3));
                match self.ch.number() {
                    Channel::Zero => match config.edge {
                        Edge::Both => {
                            w.set_cap0re(true);
                            w.set_cap0fe(true);
                        }
                        Edge::RisingEdge => {
                            w.set_cap0re(true);
                        }
                        Edge::FallingEdge => {
                            w.set_cap0fe(true);
                        }
                    },
                    Channel::One => match config.edge {
                        Edge::Both => {
                            w.set_cap1re(true);
                            w.set_cap1fe(true);
                        }
                        Edge::RisingEdge => {
                            w.set_cap1re(true);
                        }
                        Edge::FallingEdge => {
                            w.set_cap1fe(true);
                        }
                    },
                    Channel::Two => match config.edge {
                        Edge::Both => {
                            w.set_cap2re(true);
                            w.set_cap2fe(true);
                        }
                        Edge::RisingEdge => {
                            w.set_cap2re(true);
                        }
                        Edge::FallingEdge => {
                            w.set_cap2fe(true);
                        }
                    },
                    Channel::Three => match config.edge {
                        Edge::Both => {
                            w.set_cap3re(true);
                            w.set_cap3fe(true);
                        }
                        Edge::RisingEdge => {
                            w.set_cap3re(true);
                        }
                        Edge::FallingEdge => {
                            w.set_cap3fe(true);
                        }
                    },
                };
            })
        });

        Ok(())
    }

    pub fn frequency(&self) -> u32 {
        self.source_freq
    }

    /// Stop this channel after cancelling any outstanding capture future.
    pub fn suspend(&mut self) {
        critical_section::with(|_| {
            let shift = usize::from(self.ch.number()) * 3;
            self.info.regs().ccr().modify(|w| w.0 &= !(0b111 << shift));
            let mask = 1 << usize::from(self.ch.number());
            self.info.regs().ir().write(|w| w.0 = mask << 4);
            self.info.irq_flags().fetch_and(!(mask as u8), Ordering::AcqRel);
            self.info.stop_if_unused();
            self.suspended = true;
            self._wg = None;
        });
    }

    /// Restore this channel's requirement and edge configuration before running.
    pub fn resume(&mut self) -> Result<(), CaptureError> {
        if self.suspended {
            critical_section::with(|_| {
                self._wg = WakeGuard::for_power(&self.power);
                let config = self.config;
                self.set_configuration(&config)?;
                self.info.regs().tcr().modify(|w| w.set_cen(true));
                self.suspended = false;
                Ok(())
            })?;
        }
        Ok(())
    }

    /// Whether edge capture is explicitly suspended.
    pub fn is_suspended(&self) -> bool {
        self.suspended
    }

    pub async fn capture(&mut self) -> Result<Timestamp, CaptureError> {
        if self.suspended {
            return Err(CaptureError::Suspended);
        }
        self.info
            .wait_cell()
            .wait_for(|| {
                self.info.regs().ccr().modify(|w| match self.ch.number() {
                    Channel::Zero => {
                        w.set_cap0i(true);
                    }
                    Channel::One => {
                        w.set_cap1i(true);
                    }
                    Channel::Two => {
                        w.set_cap2i(true);
                    }
                    Channel::Three => {
                        w.set_cap3i(true);
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
        self.suspend();
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
        T::PERF_INT_WAKE_INCR();
        T::info().wait_cell().wake();
    }
}

#[cfg(test)]
mod power_tests {
    extern crate std;

    use super::*;
    use core::sync::atomic::AtomicU8;
    use embassy_futures::block_on;
    use std::boxed::Box;

    fn test_capture() -> Capture<'static> {
        let memory = Box::leak(Box::new([0u32; 64]));
        // SAFETY: isolated aligned RAM covers the CTIMER registers.
        let regs = unsafe { crate::pac::ctimer::Ctimer::from_ptr(memory.as_mut_ptr().cast()) };
        let info = Box::leak(Box::new(Info {
            regs,
            wait_cell: maitake_sync::WaitCell::new(),
            irq_flags: AtomicU8::new(0),
        }));
        // SAFETY: the channel addresses only the isolated register fixture.
        let ch = unsafe { Peri::new_unchecked(AnyChannel::new(Channel::One)) };
        let mut capture = Capture {
            info,
            ch,
            pin: crate::test_support::pin(),
            source_freq: 12_000_000,
            _wg: Some(WakeGuard::new()),
            power: PoweredClock::NormalEnabledDeepSleepDisabled,
            config: Config { edge: Edge::RisingEdge },
            suspended: false,
        };
        let config = capture.config;
        capture.set_configuration(&config).unwrap();
        info.regs().tcr().modify(|w| w.set_cen(true));
        capture
    }

    #[test]
    fn capture_stops_edges_before_releasing_its_guard() {
        critical_section::with(|cs| {
            let mut capture = test_capture();
            capture.info.regs().ccr().modify(|w| w.set_cap1i(true));
            capture.info.irq_flags().store(1 << 1, Ordering::Release);
            capture.suspend();
            assert!(capture.is_suspended());
            assert!(matches!(
                embassy_embedded_hal::SetConfig::set_config(&mut capture, &Config::default()),
                Err(CaptureError::Suspended)
            ));
            assert!(!capture.info.regs().ccr().read().cap1re());
            assert!(!capture.info.regs().ccr().read().cap1i());
            assert_eq!(capture.info.irq_flags().load(Ordering::Acquire), 0);
            assert!(!capture.info.regs().tcr().read().cen());
            assert!(!crate::clocks::active_wake_guards(&cs));
            assert!(matches!(block_on(capture.capture()), Err(CaptureError::Suspended)));
            capture.resume().unwrap();
            assert!(capture.info.regs().ccr().read().cap1re());
            assert!(!capture.info.regs().ccr().read().cap1fe());
            assert!(crate::clocks::active_wake_guards(&cs));
            let info = capture.info;
            drop(capture);
            assert!(!info.regs().tcr().read().cen());
            assert!(!crate::clocks::active_wake_guards(&cs));
        });
    }

    #[test]
    fn dropping_capture_preserves_other_timer_channels() {
        let capture = test_capture();
        let info = capture.info;
        info.regs().ccr().modify(|w| w.set_cap2re(true));
        drop(capture);
        assert!(!info.regs().ccr().read().cap1re());
        assert!(info.regs().ccr().read().cap2re());
        assert!(info.regs().tcr().read().cen());
    }

    #[test]
    fn resume_restores_the_latest_edge_configuration() {
        let mut capture = test_capture();
        embassy_embedded_hal::SetConfig::set_config(
            &mut capture,
            &Config {
                edge: Edge::FallingEdge,
            },
        )
        .unwrap();
        assert!(!capture.info.regs().ccr().read().cap1re());
        capture.suspend();
        capture.resume().unwrap();
        assert!(!capture.info.regs().ccr().read().cap1re());
        assert!(capture.info.regs().ccr().read().cap1fe());
    }
}

impl<'d> embassy_embedded_hal::SetConfig for Capture<'d> {
    type Config = Config;
    type ConfigError = CaptureError;

    fn set_config(&mut self, config: &Self::Config) -> Result<(), Self::ConfigError> {
        if self.suspended {
            return Err(CaptureError::Suspended);
        }
        self.set_configuration(config)?;
        self.config = *config;
        Ok(())
    }
}

//! Digital Filter and Sigma-Delta Modulator (DFSDM)

#![macro_use]

pub mod config;
pub mod detector;
pub mod dma;
pub mod filter;
pub mod splits;
pub mod transceiver;
pub mod types;

use core::cell::RefCell;
use core::future::poll_fn;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicU8, Ordering};
use core::task::Poll;

use bit_field::BitField;
pub use detector::*;
pub use dma::*;
use embassy_hal_internal::PeripheralType;
use embassy_sync::waitqueue::AtomicWaker;
pub use filter::*;
use interrupt::typelevel::Interrupt;
pub use splits::*;
pub use transceiver::*;
pub use types::*;

pub use crate::_generated::dfsdm::*;
use crate::dfsdm::capability::HasDelay;
use crate::dfsdm::config::{BreakSignals, FilterParameters};
use crate::gpio::{AfType, Flex, OutputType, Pull, Speed};
use crate::{Peri, interrupt, rcc};

/// DFSDM error.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// Overrun error: the hardware generated data faster than we could read it.
    Overrun,
    /// No data available yet.
    NotReady,
    /// Invalid filter parameters: FOSR/IOSR out of range, or the resulting
    /// filter gain exceeds the allowed ceiling for the input width.
    InvalidFilterParameters,
    /// Invalid configuration: a requested value (e.g. a CKOUT divider) is
    /// outside the achievable range.
    InvalidConfig,
}

/// 24-bit signed data range shared by filter results (`RDATAR`/`JDATAR`),
/// analog watchdog thresholds (`AWHT`/`AWLT`) and the extremes detector
/// (`EXMAX`/`EXMIN`): literal RM-stated extremes.
pub(crate) const I24_MAX: i32 = 0x7F_FFFF;
pub(crate) const I24_MIN: i32 = -0x80_0000;

// =============================================================================
// Entrypoint to creating a DFSDM driver instance.
// =============================================================================

/// DFSDM driver entry point.
pub struct Dfsdm<'d, T: Instance, C: ClockOutputMode> {
    _instance_marker: PhantomData<T>,
    _clock_mode: PhantomData<C>,
    /// Keeps the peripheral clock on while the entry point may still be
    /// configured. Moved into [`DfsdmCommon`] by
    /// [`Dfsdm::configure_pins`], which takes over the obligation.
    _rcc: RccOff<T>,
    peri: Option<Peri<'d, T>>,
    ckout: Option<Flex<'d>>,
}

impl<'d, T, C> Dfsdm<'d, T, C>
where
    T: Instance,
    C: ClockOutputMode,
{
}

impl<'d, T, C> Dfsdm<'d, T, C>
where
    C: ClockOutputMode,
    T: Instance<Transceivers = capability::TcvCnt8, Filters = capability::FltCnt8>,
{
}

impl<'d, T> Dfsdm<'d, T, OutputEnabled>
where
    T: Instance,
{
    /// Create the driver with an output clock on `ckout`.
    pub fn new_ckout(
        peri: Peri<'d, T>,
        ckout: Peri<'d, if_afio!(impl CkoutPin<T, A>)>,
        ckout_source: config::CkoutSource,
        ckout_div: config::CkoutDivider,
    ) -> Self {
        let ckout = new_pin!(ckout, AfType::output(OutputType::PushPull, Speed::VeryHigh));

        let mut dfsdm = Self::new_inner(peri, ckout);

        dfsdm.set_ckout_src(ckout_source);
        dfsdm.set_ckout_div(ckout_div);

        dfsdm
    }
}

impl<'d, T> Dfsdm<'d, T, OutputDisabled>
where
    T: Instance,
{
    /// Create the driver without an output clock.
    pub fn new(peri: Peri<'d, T>) -> Self {
        let mut dfsdm = Self::new_inner(peri, None);

        dfsdm.set_ckout_div(config::CkoutDivider::DISABLED);

        dfsdm
    }
}

impl<'d, T, C> Dfsdm<'d, T, C>
where
    T: Instance,
    C: ClockOutputMode,
{
    fn new_inner(peri: Peri<'d, T>, ckout: Option<Flex<'d>>) -> Self {
        let _ = peri;

        rcc::enable_and_reset::<T>();

        Self {
            _instance_marker: PhantomData,
            _clock_mode: PhantomData,
            _rcc: RccOff(PhantomData),
            ckout,
            peri: Some(peri),
        }
    }

    // Sets the clock-output clock-divider
    fn set_ckout_div(&mut self, divider: config::CkoutDivider) {
        T::regs().ch(0).cfgr1().modify(|w| w.set_ckoutdiv(divider.into()));
    }

    /// Sets the clock-output clock-source
    fn set_ckout_src(&mut self, source: config::CkoutSource) {
        T::regs().ch(0).cfgr1().modify(|w| w.set_ckoutsrc(source.into()));
    }
}

impl<'d, T, C> Dfsdm<'d, T, C>
where
    T: Instance + capability::HasHwid,
    C: ClockOutputMode,
{
    /// Reads the DFSDM version/ID register cluster (HWCFGR, VERR, IPIDR, SIDR).
    ///
    /// `filter_count`/`transceiver_count` self-describe the silicon; embassy's
    /// compile-time capability tags remain the primary shape mechanism.
    /// This is informational, not a runtime capability probe.
    pub fn hwid() -> Hwid {
        let hwcfgr = T::regs().hwid().hwcfgr().read();
        let verr = T::regs().hwid().verr().read();

        Hwid {
            filter_count: hwcfgr.nbf(),
            transceiver_count: hwcfgr.nbt(),
            version: (verr.majrev(), verr.minrev()),
            ip_id: T::regs().hwid().ipidr().read().0,
            silicon_id: T::regs().hwid().sidr().read().0,
        }
    }
}

// =============================================================================
// DfsdmCommon
// =============================================================================

/// Holds references to the peripheral and the optional clock-output. Disables the RCC of the peripheral when dropped.
pub struct DfsdmCommon<'d, T: Instance, P: PowerState> {
    /// Drop glue for the peripheral clock: declared first so it drops before
    /// the `Peri`/`Flex` fields, matching the previous `Drop` ordering.
    _rcc: RccOff<T>,
    _peri: Peri<'d, T>,
    _ckout: Option<Flex<'d>>,
    _powerstate_marker: PhantomData<P>,
    datin_slots: [PinSlot<'d>; 8],
    ckin_slots: [PinSlot<'d>; 8],
}

/// Disables the peripheral clock on drop. A guard field instead of a `Drop`
/// impl on [`DfsdmCommon`], so the latter stays freely destructurable.
pub(crate) struct RccOff<T: Instance>(PhantomData<T>);

impl<T: Instance> Drop for RccOff<T> {
    fn drop(&mut self) {
        rcc::disable::<T>();
    }
}

/// Kinds a pin-set consumes on its channel: `(Datin, Ckin)` flags.
///
/// Shared by the builder (to disclaim reservations it does not keep) and the
/// transceiver drop guard (to release the ones it does), so the two sets can
/// never drift apart.
pub(crate) fn pinset_kinds<S: PinSet>() -> (bool, bool) {
    (S::HAS_DATA, S::HAS_CLK)
}

impl<'d, T: Instance, P: PowerState> DfsdmCommon<'d, T, P> {
    pub(crate) fn insert_pin(&mut self, ch: usize, kind: PinKind, flex: Option<Flex<'d>>) {
        if let Some(p) = flex {
            let slot = match kind {
                PinKind::Datin => &mut self.datin_slots[ch],
                PinKind::Ckin => &mut self.ckin_slots[ch],
            };
            critical_section::with(|cs| {
                *slot.inner.borrow_ref_mut(cs) = Some(p);
            });
            // Two potential users per slot: this channel's own transceiver and
            // its predecessor's (CHINSEL=1 takes the next channel's pins, and
            // `NextChannel` wraps). Both reservations are minted up front;
            // whichever user does not take one disclaims it at build time, and
            // the other releases it at transceiver drop.
            slot.rc.store(2, Ordering::Relaxed);
        }
    }

    fn get_slot(&self, ch: usize, kind: PinKind) -> &PinSlot<'d> {
        match kind {
            PinKind::Datin => &self.datin_slots[ch],
            PinKind::Ckin => &self.ckin_slots[ch],
        }
    }

    pub(crate) fn release_pin(&self, ch: usize, kind: PinKind) {
        let slot = self.get_slot(ch, kind);
        loop {
            let val = slot.rc.load(Ordering::Acquire);
            if val == 0 {
                return; // Prevent underflow
            }
            if slot
                .rc
                .compare_exchange(val, val - 1, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                if val - 1 == 0 {
                    critical_section::with(|cs| {
                        let _ = slot.inner.borrow_ref_mut(cs).take();
                    });
                }
                return;
            }
        }
    }

    /// Releases the reservations `S` holds on `ch`, if any.
    pub(crate) fn release_pinset<S: PinSet>(&self, ch: usize) {
        let (data, clk) = pinset_kinds::<S>();
        if data {
            self.release_pin(ch, PinKind::Datin);
        }
        if clk {
            self.release_pin(ch, PinKind::Ckin);
        }
    }
}

impl<'d, T> DfsdmCommon<'d, T, Disabled>
where
    T: Instance,
{
    pub(crate) fn new(rcc: RccOff<T>, peri: Peri<'d, T>, ckout: Option<Flex<'d>>) -> Self {
        Self {
            _rcc: rcc,
            _peri: peri,
            _ckout: ckout,
            _powerstate_marker: PhantomData,
            datin_slots: [
                PinSlot::new(),
                PinSlot::new(),
                PinSlot::new(),
                PinSlot::new(),
                PinSlot::new(),
                PinSlot::new(),
                PinSlot::new(),
                PinSlot::new(),
            ],
            ckin_slots: [
                PinSlot::new(),
                PinSlot::new(),
                PinSlot::new(),
                PinSlot::new(),
                PinSlot::new(),
                PinSlot::new(),
                PinSlot::new(),
                PinSlot::new(),
            ],
        }
    }

    /// Enables the peripheral.
    pub fn enable(self) -> DfsdmCommon<'d, T, Enabled> {
        T::regs().ch(0).cfgr1().modify(|w| w.set_dfsdmen(true));
        let Self {
            _rcc,
            _peri,
            _ckout,
            datin_slots,
            ckin_slots,
            ..
        } = self;
        DfsdmCommon {
            _rcc,
            _peri,
            _ckout,
            _powerstate_marker: PhantomData,
            datin_slots,
            ckin_slots,
        }
    }
}

impl<'d, T> DfsdmCommon<'d, T, Enabled>
where
    T: Instance,
{
    /// Disables the peripheral.
    ///
    /// Setting DFEN=0 stops any conversion in progress and resets the status
    /// registers (ISR) and the analog-watchdog status register (AWSR). The data
    /// registers (RDATAR/JDATAR) are not documented to be cleared.
    pub fn disable(self) -> DfsdmCommon<'d, T, Disabled> {
        T::regs().ch(0).cfgr1().modify(|w| w.set_dfsdmen(false));

        let Self {
            _rcc,
            _peri,
            _ckout,
            datin_slots,
            ckin_slots,
            ..
        } = self;
        DfsdmCommon {
            _rcc,
            _peri,
            _ckout,
            _powerstate_marker: PhantomData,
            datin_slots,
            ckin_slots,
        }
    }
}

impl<'d, T, C> Dfsdm<'d, T, C>
where
    T: Instance,
    C: ClockOutputMode,
{
    /// The closure receives one selector per transceiver the instance actually
    /// has, and must return one token per transceiver, as a tuple in the same
    /// order.
    ///
    /// // DFSDM instance with 8-transceiver capability:
    /// ```rust,ignore
    /// dfsdm1.configure_pins(|tb| {
    ///     (
    ///         tb.ch0.datin_ckin(p.PC1, p.PC0),
    ///         tb.ch1.datin(p.PC3),
    ///         tb.ch2.datin_ckin(p.PC5, p.PC4),
    ///         tb.ch3.none(),
    ///         tb.ch4.none(),
    ///         tb.ch5.none(),
    ///         tb.ch6.none(),
    ///         tb.ch7.none(),
    ///     )
    /// });
    /// ```
    ///
    /// // DFSDM instance with 2-transceiver capability:
    /// ```rust,ignore
    /// dfsdm1.configure_pins(|tb| {
    ///     (
    ///         tb.ch0.datin_ckin(p.PC1, p.PC0),
    ///         tb.ch1.datin(p.PC3),
    ///     )
    /// });
    /// ```
    pub fn configure_pins<F, OUT>(
        mut self,
        f: F,
    ) -> (DfsdmCommon<'d, T, Enabled>, <OUT as ChannelCfgTuple<'d, T, C>>::Split)
    where
        F: FnOnce(<T::Transceivers as Shape>::Selectors<T>) -> OUT,
        OUT: ChannelCfgTuple<'d, T, C>,
    {
        let out = f(<T::Transceivers as Shape>::selectors::<T>());

        let mut common = DfsdmCommon::new(self._rcc, self.peri.expect("taken once"), self.ckout.take()).enable();
        let split = out.split_parts(&mut common);

        (common, split)
    }
}

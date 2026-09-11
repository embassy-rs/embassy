//! Digital Filter and Sigma-Delta Modulator (DFSDM)

#![macro_use]

/// Connect pac and embassy infrastructure to our types
pub mod associations;
/// DMA functions
pub mod dma;
/// Channel/filter splits - generated from a per-shape table
pub mod splits;
/// Type-system
pub mod types;

use core::cell::RefCell;
use core::future::poll_fn;
use core::marker::PhantomData;
use core::mem::ManuallyDrop;
use core::ptr;
use core::sync::atomic::{AtomicU8, Ordering};
use core::task::Poll;

pub use dma::*;
use embassy_hal_internal::PeripheralType;
use embassy_sync::waitqueue::AtomicWaker;
use interrupt::typelevel::Interrupt;
pub use splits::*;
pub use types::*;

use crate::dfsdm::capability::HasDelay;
use crate::dfsdm::config_types::{BreakSignals, FilterParameters};
use crate::gpio::{AfType, Flex, OutputType, Pull, Speed};
use crate::{Peri, interrupt, rcc};

/// DFSDM error.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    //TODO
    /// Overrun error: the hardware generated data faster than we could read it.
    Overrun,
    /// Internal peripheral error.
    PeripheralError,
    /// Neighbor pin unavailable.
    NeighborPinUnavailable,
}

/// DFSDM configuration.
#[non_exhaustive]
pub struct Config {
    //TODO
}

impl Default for Config {
    fn default() -> Self {
        Self {}
    }
}

// =============================================================================
// Pin Reference Counting Storage
// =============================================================================

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PinKind {
    Datin,
    Ckin,
}

pub struct PinSlot<'d> {
    inner: critical_section::Mutex<RefCell<Option<Flex<'d>>>>,
    rc: AtomicU8,
}

impl<'d> PinSlot<'d> {
    const fn new() -> Self {
        Self {
            inner: critical_section::Mutex::new(RefCell::new(None)),
            rc: AtomicU8::new(0),
        }
    }
}

// =============================================================================
// Entrypoint to creating a DFSDM driver instance.
// =============================================================================

/// DFSDM driver.
pub struct Dfsdm<'d, T: Instance, C: ClockOutputMode> {
    _instance_marker: PhantomData<T>,
    _clock_mode: PhantomData<C>,
    peri: Option<Peri<'d, T>>,
    ckout: Option<Flex<'d>>,
}

impl<'d, T, C> Dfsdm<'d, T, C>
where
    T: Instance,
    C: ClockOutputMode,
{
}

#[allow(private_bounds)]
impl<'d, T, C> Dfsdm<'d, T, C>
where
    C: ClockOutputMode,
    T: Instance<Transceivers = capability::Tcv8, Filters = capability::Flt8>,
{
}

impl<'d, T> Dfsdm<'d, T, OutputEnabled>
where
    T: Instance,
{
    /// Configure DFSDM module with a clock output
    pub fn new_ckout(
        peri: Peri<'d, T>,
        ckout: Peri<'d, if_afio!(impl CkoutPin<T, A>)>,
        ckout_source: config_types::CkoutSource,
        ckout_div: config_types::CkoutDivider,
    ) -> Self {
        let ckout = new_pin!(ckout, AfType::output(OutputType::PushPull, Speed::VeryHigh));

        //         macro_rules! config_pins {
        //     ($($pin:ident),*) => {
        //                 critical_section::with(|_| {
        //             $(
        //                 set_as_af!($pin, AfType::input(Pull::None));
        //             )*
        //         })
        //     };
        // }
        // TODO MAYBE USE CRITICAL SECTION FOR AFS?!

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
    /// Configure DFSDM module without a clock output
    pub fn new(peri: Peri<'d, T>) -> Self {
        let mut dfsdm = Self::new_inner(peri, None);

        dfsdm.set_ckout_div(config_types::CkoutDivider::DISABLED);

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
            ckout: ckout,
            peri: Some(peri),
        }
    }

    // Set's the clock-output clock-divider
    fn set_ckout_div(&mut self, divider: config_types::CkoutDivider) {
        T::regs().ch(0).cfgr1().modify(|w| w.set_ckoutdiv(divider.into()));
    }

    /// Set's the clock-output clock-source
    fn set_ckout_src(&mut self, source: config_types::CkoutSource) {
        T::regs().ch(0).cfgr1().modify(|w| w.set_ckoutsrc(source.into()));
    }
}

// =============================================================================
// DfsdmCommon
// =============================================================================

/// Holds regerences to the peripheral and the optional clock-output. Disables the RCC of the peripheral when dropped.
pub struct DfsdmCommon<'d, T: Instance, P: PowerState> {
    _peri: Peri<'d, T>,
    _ckout: Option<Flex<'d>>,
    _powerstate_marker: PhantomData<P>,
    datin_slots: [PinSlot<'d>; 8],
    ckin_slots: [PinSlot<'d>; 8],
}
impl<'d, T: Instance, P: PowerState> DfsdmCommon<'d, T, P> {
    fn insert_pin(&mut self, ch: usize, kind: PinKind, flex: Option<Flex<'d>>) {
        if let Some(p) = flex {
            let slot = match kind {
                PinKind::Datin => &mut self.datin_slots[ch],
                PinKind::Ckin => &mut self.ckin_slots[ch],
            };
            critical_section::with(|cs| {
                *slot.inner.borrow_ref_mut(cs) = Some(p);
            });
            slot.rc.store(1, Ordering::Relaxed);
        }
    }

    fn get_slot(&self, ch: usize, kind: PinKind) -> &PinSlot<'d> {
        match kind {
            PinKind::Datin => &self.datin_slots[ch],
            PinKind::Ckin => &self.ckin_slots[ch],
        }
    }

    pub(crate) fn acquire_pin(&self, ch: usize, kind: PinKind) -> Result<(), Error> {
        let slot = self.get_slot(ch, kind);
        loop {
            let val = slot.rc.load(Ordering::Acquire);
            if val == 0 {
                return Err(Error::NeighborPinUnavailable);
            }
            if slot
                .rc
                .compare_exchange(val, val + 1, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                return Ok(());
            }
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

    pub(crate) fn acquire_pins<S: PinSet>(&self, ch: usize) -> Result<(), Error> {
        if S::HAS_DATA {
            self.acquire_pin(ch, PinKind::Datin)?;
        }
        if S::HAS_CLK {
            if let Err(e) = self.acquire_pin(ch, PinKind::Ckin) {
                if S::HAS_DATA {
                    self.release_pin(ch, PinKind::Datin);
                }
                return Err(e);
            }
        }
        Ok(())
    }

    fn into_raw_parts(self) -> (Peri<'d, T>, Option<Flex<'d>>, [PinSlot<'d>; 8], [PinSlot<'d>; 8]) {
        let this = ManuallyDrop::new(self);
        // SAFETY: `this` is wrapped in `ManuallyDrop`, so its destructor will not
        // run. We use `ptr::read` to bitwise-move each field out, transferring
        // ownership to the caller exactly once per field (the source value is
        // consumed and intentionally never dropped). Since we never drop `this`,
        // immediately return the extracted values, and nothing between the reads
        // can unwind, no double-free, use-after-free or leak of `Flex` drop-glue
        // can occur. (`Peri` is a ghost type carrying no real `&mut`, so copying
        // it cannot alias.)
        unsafe {
            (
                ptr::read(&this._peri),
                ptr::read(&this._ckout),
                ptr::read(&this.datin_slots),
                ptr::read(&this.ckin_slots),
            )
        }
    }
}

impl<'d, T, P> Drop for DfsdmCommon<'d, T, P>
where
    T: Instance,
    P: PowerState,
{
    fn drop(&mut self) {
        rcc::disable::<T>();
    }
}

impl<'d, T> DfsdmCommon<'d, T, Disabled>
where
    T: Instance,
{
    pub(crate) fn new(peri: Peri<'d, T>, ckout: Option<Flex<'d>>) -> Self {
        Self {
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

    /// Enables the peripheral
    pub fn enable(self) -> DfsdmCommon<'d, T, Enabled> {
        T::regs().ch(0).cfgr1().modify(|w| w.set_dfsdmen(true));
        let (_peri, _ckout, datin_slots, ckin_slots) = self.into_raw_parts();
        DfsdmCommon {
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
    /// Disables the peripheral
    pub fn disable(self) -> DfsdmCommon<'d, T, Disabled> {
        T::regs().ch(0).cfgr1().modify(|w| w.set_dfsdmen(false));

        let (_peri, _ckout, datin_slots, ckin_slots) = self.into_raw_parts();
        DfsdmCommon {
            _peri,
            _ckout,
            _powerstate_marker: PhantomData,
            datin_slots,
            ckin_slots,
        }
    }
}

// =============================================================================
// FilterConfig
// =============================================================================

/// Confgiguration for Filter
pub struct FilterConfig<T: Instance> {
    pub filter_params: FilterParameters,
    pub enable_continuous_regular: bool,
    pub enable_fast_regular: bool,
    pub enable_regular_sync: bool,
    pub enable_injected_sync: bool,
    pub enable_injected_scanning: bool,

    /// Configures the trigger for injected conversions.
    ///
    /// `Some` enables the trigger with the specified trigger source and edge.
    /// `None` disables the trigger.
    pub trigger: Option<InjectedDfsdmTrigger<T>>,
}

impl<T: Instance> Default for FilterConfig<T> {
    fn default() -> Self {
        Self {
            filter_params: FilterParameters::new(config_types::FilterOrder::Disabled, 1),
            enable_continuous_regular: false,
            enable_fast_regular: false,
            enable_injected_sync: false,
            enable_regular_sync: false,
            enable_injected_scanning: false,
            trigger: None,
        }
    }
}

/// Sign-extends a 24bit LSB number to a i32
fn sign_extend_24(x: u32) -> i32 {
    ((x << 8) as i32) >> 8
}

// =============================================================================
// Filter
// =============================================================================

pub(crate) struct FilterRegs<T, M>(PhantomData<(T, M)>);
pub struct FilterDisabled<'a, 'd, T, M>
where
    T: Instance + FilterInterrupt<M>,
    M: FilterMarker + InstanceEvents<T>,
{
    _marker: PhantomData<M>,
    common: &'a DfsdmCommon<'d, T, Enabled>,
}

pub struct Filter<'tr, 'ti, 'a, 'd, T, M, D>
where
    T: Instance + FilterInterrupt<M>,
    M: FilterMarker + InstanceEvents<T>,
    D: DmaMode,
{
    common: &'a DfsdmCommon<'d, T, Enabled>,
    pub reg: FilterRegular<'a, 'd, 'tr, T, M, D>,
    pub inj: FilterInjected<'a, 'd, 'ti, T, M, D>,
    pub awd: AnalogWatchdog<'a, 'd, T, M>,
    pub extremes: ExtremesDetector<'a, 'd, T, M>,
}

pub struct FilterRegular<'a, 'd, 't, T, M, D>
where
    T: Instance + FilterInterrupt<M>,
    M: FilterMarker + InstanceEvents<T>,
    D: DmaMode,
{
    _common: PhantomData<(&'a DfsdmCommon<'d, T, Enabled>, M, D)>,
    regular: &'t dyn TransceiverTrait<T, Enabled>,
}

pub struct FilterInjected<'a, 'd, 't, T, M, D>
where
    T: Instance + FilterInterrupt<M>,
    M: FilterMarker + InstanceEvents<T>,
    D: DmaMode,
{
    _common: PhantomData<(&'a DfsdmCommon<'d, T, Enabled>, M, D)>,
    injected: [Option<&'t dyn TransceiverTrait<T, Enabled>>; 8],
}

//filter is "on", "off" version needs own off struct/"DIsabledFilter" because of members
impl<'a, 'd, T, M> FilterDisabled<'a, 'd, T, M>
where
    T: Instance + FilterInterrupt<M>,
    M: FilterMarker + InstanceEvents<T>,
{
    pub(crate) fn new(common: &'a DfsdmCommon<'d, T, Enabled>) -> Self {
        Self {
            _marker: PhantomData,
            common,
        }
    }
    /// Activate filter with no DMA enabled.
    pub fn enable_no_dma<'tr, 'ti, const N: usize>(
        self,
        regular: &'tr dyn TransceiverTrait<T, Enabled>,
        injected: [&'ti dyn TransceiverTrait<T, Enabled>; N],
        config: &FilterConfig<T>,
    ) -> Filter<'tr, 'ti, 'a, 'd, T, M, NoDma>
    where
        [(); N]: NonEmpty,
    {
        self.enable_int(regular, injected, config)
    }

    /// Activate Filter with DMA enabled for regular conversions
    pub fn enable_reg_dma<'tr, 'ti, const N: usize>(
        self,
        regular: &'tr dyn TransceiverTrait<T, Enabled>,
        injected: [&'ti dyn TransceiverTrait<T, Enabled>; N],
        config: &FilterConfig<T>,
    ) -> Filter<'tr, 'ti, 'a, 'd, T, M, RegDma>
    where
        [(); N]: NonEmpty,
    {
        self.enable_int(regular, injected, config)
    }

    /// Activate Filter with DMA enabled for injected conversions
    pub fn enable_inj_dma<'tr, 'ti, const N: usize>(
        self,
        regular: &'tr dyn TransceiverTrait<T, Enabled>,
        injected: [&'ti dyn TransceiverTrait<T, Enabled>; N],
        config: &FilterConfig<T>,
    ) -> Filter<'tr, 'ti, 'a, 'd, T, M, InjDma>
    where
        [(); N]: NonEmpty,
    {
        self.enable_int(regular, injected, config)
    }

    fn enable_int<'tr, 'ti, const N: usize, D>(
        self,
        regular: &'tr dyn TransceiverTrait<T, Enabled>,
        injected: [&'ti dyn TransceiverTrait<T, Enabled>; N],
        config: &FilterConfig<T>,
    ) -> Filter<'tr, 'ti, 'a, 'd, T, M, D>
    where
        D: DmaMode,
        [(); N]: NonEmpty,
    {
        let filter = Filter {
            common: self.common,
            reg: FilterRegular::new(self.common, regular),
            inj: FilterInjected::new(self.common, injected),
            awd: AnalogWatchdog::new(self.common),
            extremes: ExtremesDetector::new(self.common),
        };

        // Enable appropriate DMA request
        T::regs().flt(M::CHANNEL.index()).cr1().modify(|w| {
            w.set_rdmaen(D::REG_ENABLED);
            w.set_jdmaen(D::INJ_ENABLED);
        });

        Self::configure(config);

        FilterRegs::<T, M>::set_enabled(true);

        filter
    }

    fn configure(config: &FilterConfig<T>) {
        Self::set_filter_parameters(config.filter_params);
        Self::set_continuous(config.enable_continuous_regular);
        Self::set_fastmode(config.enable_fast_regular);
        Self::set_regular_synchronization(config.enable_regular_sync);
        Self::set_injected_synchronization(config.enable_injected_sync);
        Self::set_injected_scanning(config.enable_injected_scanning);
        Self::configure_injected_trigger(config.trigger);
    }

    /// Writes the filterparameters
    fn set_filter_parameters(params: config_types::FilterParameters) {
        let (order, fosr, iosr) = params.register_values();
        T::regs().flt(M::CHANNEL.index()).fcr().modify(|w| {
            w.set_ford(order);
            w.set_fosr(fosr);
            w.set_iosr(iosr);
        });
    }

    /// Enables or disables fast conversion mode.
    ///
    /// In continuous mode, fast mode reduces the conversion time after the first
    /// conversion because the filter is already filled and does not need to be
    /// filled again. Subsequent conversions therefore take only `FOSR * IOSR / fCKIN`
    /// instead of the normal filter fill time. Has no effect outside continuous mode.
    fn set_fastmode(enabled: bool) {
        T::regs().flt(M::CHANNEL.index()).cr1().modify(|w| w.set_fast(enabled));
    }

    /// Enables or disables continuous conversion mode.
    ///
    /// When enabled, the regular channel is converted repeatedly after each
    /// conversion request. Disabling it while a continuous conversion is in
    /// progress stops the conversion immediately.
    fn set_continuous(enabled: bool) {
        T::regs().flt(M::CHANNEL.index()).cr1().modify(|w| w.set_rcont(enabled));
    }

    /// Configures the trigger for injected conversions.
    ///
    /// `Some` enables the trigger with the specified trigger source and edge.
    /// `None` disables the trigger.
    fn configure_injected_trigger(trigger: Option<InjectedDfsdmTrigger<T>>) {
        let (jextsel, jexten) = match trigger {
            Some(InjectedDfsdmTrigger { trigger, edge, .. }) => (trigger, edge as u8),
            None => (0, 0), // Disable
        };

        T::regs()
            .flt(M::CHANNEL.index())
            .cr1()
            .modify(|w: &mut stm32_metapac::dfsdm::regs::Cr1| {
                w.set_jextsel(jextsel);
                w.set_jexten(jexten);
            });
    }

    /// Enables or disables synchronization for regular conversions.
    fn set_regular_synchronization(enable: bool) {
        T::regs().flt(M::CHANNEL.index()).cr1().modify(|w| w.set_rsync(enable));
    }

    /// Enables or disables synchronization for injected conversions.
    fn set_injected_synchronization(enable: bool) {
        T::regs().flt(M::CHANNEL.index()).cr1().modify(|w| w.set_jsync(enable));
    }

    /// Enables or disables scanning mode for injected conversions.
    ///
    /// When enabled, injected conversions cycle through all selected channels,
    /// starting again at the lowest selected channel. When disabled, each
    /// conversion advances to the next selected channel.
    ///
    /// Changing the injected channel group while scanning is disabled resets the
    /// channel selection to the lowest selected channel.
    fn set_injected_scanning(enabled: bool) {
        T::regs().flt(M::CHANNEL.index()).cr1().modify(|w| w.set_jscan(enabled));
    }
}

impl<'tr, 'ti, 'a, 'd, T, M, D> Drop for Filter<'tr, 'ti, 'a, 'd, T, M, D>
where
    T: Instance + FilterInterrupt<M>,
    M: FilterMarker + InstanceEvents<T>,
    D: DmaMode,
{
    fn drop(&mut self) {
        FilterRegs::<T, M>::set_enabled(false);
    }
}

impl<'tr, 'ti, 'a, 'd, T, M, D> Filter<'tr, 'ti, 'a, 'd, T, M, D>
where
    T: Instance + FilterInterrupt<M>,
    M: FilterMarker + InstanceEvents<T>,
    D: DmaMode,
{
    /// Disable the Filter
    pub fn disable(self) -> FilterDisabled<'a, 'd, T, M> {
        FilterRegs::<T, M>::set_enabled(false);

        FilterDisabled {
            _marker: PhantomData,
            common: self.common,
        }
    }
    // Normal stuff,

    /// 28-bit timer counting conversion time t = CNVCNT[27:0] / fDFSDMCLK
    pub fn get_cnv_cnt(&self) -> u32 {
        T::regs().flt(M::CHANNEL.index()).cnvtimr().read().cnvcnt()
    }

    /// Replaces the regular transceiver, releasing the old borrow so the
    /// previous transceiver can be mutated afterwards. Since this may change
    /// the lifetime of the borrows, it consumes and returns a new `Filter`
    /// rather than mutating in place. This is pure borrow-checker bookkeeping,
    /// not a hardware requirement - see [`FilterRegular::assign_transceiver`]
    /// for the in-place alternative when the lifetime doesn't need to change.

    pub fn replace_regular_transceiver<'new_reg>(
        self,
        transceiver: &'new_reg dyn TransceiverTrait<T, Enabled>,
    ) -> Filter<'new_reg, 'ti, 'a, 'd, T, M, D> {
        FilterRegular::<'a, 'd, 'ti, T, M, D>::set_regular_transceiver(transceiver.index());

        let this = ManuallyDrop::new(self);
        // SAFETY: `this` is wrapped in `ManuallyDrop` to prevent the destructor from
        // running. We extract each field with `ptr::read`, which performs a bitwise
        // move without invoking drop. The original `Filter` is never dropped and all
        // extracted fields are moved into the new `Filter`, maintaining ownership
        // invariants. Skipping the original `Filter`'s Drop is intentional: it would
        // clear DFEN, but the returned `Filter` re-acquires that teardown obligation.
        let common = unsafe { ptr::read(&this.common) };
        let inj = unsafe { ptr::read(&this.inj) };
        let awd = unsafe { ptr::read(&this.awd) };
        let extremes = unsafe { ptr::read(&this.extremes) };

        Filter {
            common,
            reg: FilterRegular {
                _common: PhantomData,
                regular: transceiver,
            },
            inj,
            awd,
            extremes,
        }
    }

    /// Replaces the injected transceivers, releasing the old borrows so the
    /// previous transceivers can be mutated afterwards. Since this may change
    /// the lifetime of the borrows, it consumes and returns a new `Filter`
    /// rather than mutating in place. This is pure borrow-checker bookkeeping,
    /// not a hardware requirement - see [`FilterInjected::assign_transceivers`]
    /// for the in-place alternative when the lifetime doesn't need to change.
    pub fn replace_injected_transceivers<'new_inj, const N: usize>(
        self,
        transceivers: [&'new_inj dyn TransceiverTrait<T, Enabled>; N],
    ) -> Filter<'tr, 'new_inj, 'a, 'd, T, M, D>
    where
        [(); N]: NonEmpty,
    {
        let (slots, filterword) = FilterInjected::<'a, 'd, 'ti, T, M, D>::build_injected_slots(transceivers);
        FilterInjected::<'a, 'd, 'ti, T, M, D>::set_injected_channels(filterword);

        let this = ManuallyDrop::new(self);
        // SAFETY: `this` is wrapped in `ManuallyDrop` to prevent the destructor from
        // running. We extract each field with `ptr::read`, which performs a bitwise
        // move without invoking drop. The original `Filter` is never dropped and all
        // extracted fields are moved into the new `Filter`, maintaining ownership
        // invariants. Skipping the original `Filter`'s Drop is intentional: it would
        // clear DFEN, but the returned `Filter` re-acquires that teardown obligation.
        let common = unsafe { ptr::read(&this.common) };
        let reg = unsafe { ptr::read(&this.reg) };
        let awd = unsafe { ptr::read(&this.awd) };
        let extremes = unsafe { ptr::read(&this.extremes) };

        Filter {
            common,
            reg,
            inj: FilterInjected {
                injected: slots,
                _common: PhantomData,
            },
            awd,
            extremes,
        }
    }
}

impl<'a, 'd, 't, T, M, D> FilterRegular<'a, 'd, 't, T, M, D>
where
    T: Instance + FilterInterrupt<M>,
    M: FilterMarker + InstanceEvents<T>,
    D: DmaMode,
{
    pub(crate) fn new(
        _common: &'a DfsdmCommon<'d, T, Enabled>,
        transceiver: &'t dyn TransceiverTrait<T, Enabled>,
    ) -> Self {
        Self::set_regular_transceiver(transceiver.index());
        Self {
            _common: PhantomData,
            regular: transceiver,
        }
    }
    // Normal stuff
    /// Reassigns the transceiver for regular conversions in-place.
    ///
    /// The new transceiver must live at least as long as the previous one
    /// (`'t`), since this does not change the `Filter`'s lifetime parameter.
    /// Use [`Filter::replace_regular_transceiver`] if you need to assign a
    /// transceiver with a shorter/different lifetime and get the old one back
    /// for further mutation.
    pub fn assign_transceiver(&mut self, transceiver: &'t dyn TransceiverTrait<T, Enabled>) {
        Self::set_regular_transceiver(transceiver.index());
        self.regular = transceiver;
    }

    fn set_regular_transceiver(ch: usize) {
        T::regs().flt(M::CHANNEL.index()).cr1().modify(|w| w.set_rch(ch as u8));
    }

    /// Trigger a regular conversion
    pub fn start_regular_conversion(&mut self) {
        T::regs().flt(M::CHANNEL.index()).cr1().modify(|w| w.set_rswstart(true));
    }

    /// Trigger a regular conversion and read it asynchronously using interrupts
    pub async fn read_regular(&mut self) -> (i32, u8, bool) {
        self.start_regular_conversion();

        poll_fn(|cx| {
            FilterRegs::<T, M>::set_regular_end_of_conversion_interrupt(false);
            T::state().regular_waker.register(cx.waker());

            if let Some(result) = self.try_get_regular_result() {
                Poll::Ready(result)
            } else {
                FilterRegs::<T, M>::set_regular_end_of_conversion_interrupt(true);
                Poll::Pending
            }
        })
        .await
    }

    /// Attempts to read the current regular conversion result.
    ///
    /// Returns `Some((data, channel, rpend))` if `REOCF` is set, or `None` if no
    /// regular conversion result is available.
    ///
    /// The conversion result is sign-extended from 24 to 32 bits and is not scaled.
    ///
    /// `rpend` is set if the regular conversion was delayed by an injected
    /// conversion.
    ///
    /// Reading the result clears the corresponding data register.
    pub fn try_get_regular_result(&mut self) -> Option<(i32, u8, bool)> {
        if self.end_of_regular_conversion() {
            let result = T::regs().flt(M::CHANNEL.index()).rdatar().read();
            let data = sign_extend_24(result.rdata());
            let channel = result.rdatach();
            return Some((data, channel, result.rpend()));
        }
        None
    }

    /// Reads and clears the current regular conversion result without checking
    /// `REOCF`.
    ///
    /// The conversion result is sign-extended from 24 to 32 bits and is not scaled.
    ///
    /// `rpend` is set if the regular conversion was delayed by an injected
    /// conversion.
    ///
    /// The returned data is only valid if `REOCF` was set before reading.
    ///
    /// Returns `(data, channel, rpend)`.
    pub fn get_regular_result_unchecked(&mut self) -> (i32, u8, bool) {
        let result = T::regs().flt(M::CHANNEL.index()).rdatar().read();
        let data = sign_extend_24(result.rdata());
        let channel = result.rdatach();
        (data, channel, result.rpend())
    }

    /// Returns whether a regular conversion result is available.
    pub fn end_of_regular_conversion(&self) -> bool {
        FilterRegs::<T, M>::end_of_regular_conversion()
    }

    /// Returns whether a regular conversion is currently in progress or pending.
    pub fn regular_conversion_in_progress(&self) -> bool {
        FilterRegs::<T, M>::regular_conversion_in_progress()
    }

    /// Enables or disables continuous conversion mode.
    ///
    /// When enabled, the regular channel is converted repeatedly after each
    /// conversion request. Disabling it while a continuous conversion is in
    /// progress stops the conversion immediately.
    pub fn set_continuous(&mut self, enabled: bool) {
        FilterDisabled::<T, M>::set_continuous(enabled);
    }
}

impl<'a, 'd, 't, T, M, D> FilterInjected<'a, 'd, 't, T, M, D>
where
    T: Instance + FilterInterrupt<M>,
    M: FilterMarker + InstanceEvents<T>,
    D: DmaMode,
{
    pub(crate) fn new<const N: usize>(
        _common: &'a DfsdmCommon<'d, T, Enabled>,
        transceivers: [&'t dyn TransceiverTrait<T, Enabled>; N],
    ) -> Self
    where
        [(); N]: NonEmpty,
    {
        let (slots, filterword) = Self::build_injected_slots(transceivers);
        Self::set_injected_channels(filterword);

        Self {
            _common: PhantomData,
            injected: slots,
        }
    }

    /// Reassigns the transceivers for injected conversions in-place.
    ///
    /// The new transceiver must live at least as long as the previous one
    /// (`'t`), since this does not change the `FilterInjected`'s lifetime parameter.
    /// Use [`Filter::replace_injected_transceivers`] if you need to assign a
    /// transceiver with a shorter/different lifetime and get the old one back
    /// for further mutation.
    pub fn assign_transceivers<const N: usize>(&mut self, transceivers: [&'t dyn TransceiverTrait<T, Enabled>; N])
    where
        [(); N]: NonEmpty,
    {
        let (slots, filterword) = Self::build_injected_slots(transceivers);
        Self::set_injected_channels(filterword);
        self.injected = slots;
    }

    /// Builds the fixed-size injected-slot array plus the register bitmask
    /// from a caller-provided transceiver array of any lifetime.
    fn build_injected_slots<'tcv, const N: usize>(
        transceivers: [&'tcv dyn TransceiverTrait<T, Enabled>; N],
    ) -> ([Option<&'tcv dyn TransceiverTrait<T, Enabled>>; 8], u8)
    where
        [(); N]: NonEmpty,
    {
        let filterword = transceivers.iter().fold(0u8, |acc, tcv| acc | (1 << tcv.index()));

        let mut slots: [Option<&'tcv dyn TransceiverTrait<T, Enabled>>; 8] = [None; 8];
        for (i, tcv) in transceivers.iter().enumerate() {
            slots[i] = Some(*tcv);
        }

        (slots, filterword)
    }

    fn set_injected_channels(channels: u8) {
        T::regs()
            .flt(M::CHANNEL.index())
            .jchgr()
            .write(|w| w.set_jchg(channels));
    }

    /// Trigger a injected conversion
    pub fn start_injected_conversion(&mut self) {
        T::regs().flt(M::CHANNEL.index()).cr1().modify(|w| w.set_jswstart(true));
    }

    /// Trigger a injected conversion and read it asynchronously using interrupts
    pub async fn read_injected(&mut self) -> (i32, u8) {
        self.start_injected_conversion();

        poll_fn(|cx| {
            FilterRegs::<T, M>::set_injected_end_of_conversion_interrupt(false);
            T::state().injected_waker.register(cx.waker());

            if let Some(result) = self.try_get_injected_result() {
                Poll::Ready(result)
            } else {
                FilterRegs::<T, M>::set_injected_end_of_conversion_interrupt(true);
                Poll::Pending
            }
        })
        .await
    }

    /// Attempts to read the current injected conversion result.
    ///
    /// Returns `Some((data, channel))` if `JEOCF` is set, or `None` if no injected
    /// conversion result is available.
    ///
    /// The conversion result is sign-extended from 24 to 32 bits and is not scaled.
    ///
    /// Reading the result clears the corresponding data register.
    pub fn try_get_injected_result(&mut self) -> Option<(i32, u8)> {
        if self.end_of_injected_conversion() {
            let result = T::regs().flt(M::CHANNEL.index()).jdatar().read();
            let data = sign_extend_24(result.jdata());
            let channel = result.jdatach();
            return Some((data, channel));
        }
        None
    }

    /// Reads and clears the current injected conversion result without checking
    /// `JEOCF`.
    ///
    /// The conversion result is sign-extended from 24 to 32 bits and is not scaled.
    ///
    /// The returned data is only valid if `JEOCF` was set before reading.
    ///
    /// Returns `(data, channel)`.
    pub fn get_injected_result_unchecked(&mut self) -> (i32, u8) {
        let result = T::regs().flt(M::CHANNEL.index()).jdatar().read();
        let data = sign_extend_24(result.jdata());
        let channel = result.jdatach();
        (data, channel)
    }

    /// Returns whether an injected conversion result is available.
    pub fn end_of_injected_conversion(&self) -> bool {
        FilterRegs::<T, M>::end_of_injected_conversion()
    }

    /// Returns whether an injected conversion is currently in progress or pending.
    pub fn injected_conversion_in_progress(&self) -> bool {
        FilterRegs::<T, M>::injected_conversion_in_progress()
    }
}

impl<'a, 'd, 't, T, M> FilterDma<T, M> for FilterRegular<'a, 'd, 't, T, M, RegDma>
where
    T: Instance + FilterInterrupt<M>,
    M: FilterMarker + InstanceEvents<T>,
{
    fn data_register(&self) -> *mut u32 {
        T::regs().flt(M::CHANNEL.index()).rdatar().as_ptr() as *mut u32
    }
}

impl<'a, 'd, 't, T, M> FilterDma<T, M> for FilterInjected<'a, 'd, 't, T, M, InjDma>
where
    T: Instance + FilterInterrupt<M>,
    M: FilterMarker + InstanceEvents<T>,
{
    fn data_register(&self) -> *mut u32 {
        T::regs().flt(M::CHANNEL.index()).jdatar().as_ptr() as *mut u32
    }
}

impl<T, M> FilterRegs<T, M>
where
    T: Instance + FilterInterrupt<M>,
    M: FilterMarker + InstanceEvents<T>,
{
    /// Enable or disable the filter
    pub(crate) fn set_enabled(enabled: bool) {
        T::regs().flt(M::CHANNEL.index()).cr1().modify(|w| w.set_dfen(enabled));
    }

    /// Returns whether a regular conversion result is available.
    pub(crate) fn end_of_regular_conversion() -> bool {
        T::regs().flt(M::CHANNEL.index()).isr().read().reocf()
    }

    /// Returns whether an injected conversion result is available.
    pub(crate) fn end_of_injected_conversion() -> bool {
        T::regs().flt(M::CHANNEL.index()).isr().read().jeocf()
    }

    /// Enables or disables regular end-of-conversion interrupts.
    pub(crate) fn set_regular_end_of_conversion_interrupt(enabled: bool) {
        // RMW'd from both ISR and thread (the ISR clears its own IE here) - cs is load-bearing.
        critical_section::with(|_cs| {
            T::regs()
                .flt(M::CHANNEL.index())
                .cr2()
                .modify(|w| w.set_reocie(enabled));
        });
    }

    /// Enables or disables injected end-of-conversion interrupts.
    pub(crate) fn set_injected_end_of_conversion_interrupt(enabled: bool) {
        // RMW'd from both ISR and thread (the ISR clears its own IE here) - cs is load-bearing.
        critical_section::with(|_cs| {
            T::regs()
                .flt(M::CHANNEL.index())
                .cr2()
                .modify(|w| w.set_jeocie(enabled));
        });
    }

    pub(crate) fn injected_conversion_in_progress() -> bool {
        T::regs().flt(M::CHANNEL.index()).isr().read().jcip()
    }

    pub(crate) fn regular_conversion_in_progress() -> bool {
        T::regs().flt(M::CHANNEL.index()).isr().read().rcip()
    }
}

// =============================================================================
// Transceiver
// =============================================================================

/// Configured DFSDM data input transceiver.
pub struct Transceiver<'a, 'd, T, M, S, MODE, PS, P>
where
    T: Instance,
    M: TransceiverMarker + NextChannelForInstance<T>,
    S: PinSet,
    MODE: ChannelMode,
    PS: PinSource,
    P: PowerState,
{
    pub(crate) common: &'a DfsdmCommon<'d, T, Enabled>,
    _instance_marker: PhantomData<T>,
    _transceiver_marker: PhantomData<M>,
    _pinset_marker: PhantomData<S>,
    _channel_mode_marker: PhantomData<MODE>,
    _pin_source_marker: PhantomData<PS>,
    _powerstate_marker: PhantomData<P>,
}

impl<'a, 'd, T, M, S, MODE, PS, P> Drop for Transceiver<'a, 'd, T, M, S, MODE, PS, P>
where
    T: Instance,
    M: TransceiverMarker + NextChannelForInstance<T>,
    S: PinSet,
    MODE: ChannelMode,
    PS: PinSource,
    P: PowerState,
{
    fn drop(&mut self) {
        // "Drop pin references" as we "manually" reference count
        let ch = if PS::FROM_NEIGHBOR {
            <M::Next as TransceiverMarker>::CHANNEL.index()
        } else {
            M::CHANNEL.index()
        };

        if S::HAS_DATA {
            self.common.release_pin(ch, PinKind::Datin);
        }
        if S::HAS_CLK {
            self.common.release_pin(ch, PinKind::Ckin);
        }

        // Disabling will deactivate the detector flags,
        // so we need to remove them from the cached version
        ShortCircuitDetector::<T>::drop_transceiver(M::CHANNEL);
        ClockAbsenceDetector::<T>::drop_transceiver(M::CHANNEL);

        Self::set_enabled(false);
    }
}

/// Only when enabled
impl<'a, 'd, T, M, S, MODE, PS> Transceiver<'a, 'd, T, M, S, MODE, PS, Enabled>
where
    T: Instance,
    M: TransceiverMarker + NextChannelForInstance<T>,
    S: PinSet,
    MODE: ChannelMode,
    PS: PinSource,
{
    /// Disables the channel
    pub fn disable(self) -> Transceiver<'a, 'd, T, M, S, MODE, PS, Disabled> {
        Self::set_enabled(false);

        let common = self.common;
        core::mem::forget(self);
        Transceiver {
            common,
            _instance_marker: PhantomData,
            _transceiver_marker: PhantomData,
            _pinset_marker: PhantomData,
            _channel_mode_marker: PhantomData,
            _pin_source_marker: PhantomData,
            _powerstate_marker: PhantomData,
        }
    }
}

/// Only when disabled
impl<'a, 'd, T, M, S, MODE, PS> Transceiver<'a, 'd, T, M, S, MODE, PS, Disabled>
where
    T: Instance,
    M: TransceiverMarker + NextChannelForInstance<T>,
    S: PinSet,
    MODE: ChannelMode,
    PS: PinSource,
{
    /// Enables the channel
    pub fn enable(self) -> Transceiver<'a, 'd, T, M, S, MODE, PS, Enabled> {
        Self::set_enabled(true);

        let common = self.common;
        core::mem::forget(self);

        Transceiver {
            common,
            _instance_marker: PhantomData,
            _transceiver_marker: PhantomData,
            _pinset_marker: PhantomData,
            _channel_mode_marker: PhantomData,
            _pin_source_marker: PhantomData,
            _powerstate_marker: PhantomData,
        }
    }

    /// Set channel right shift factor
    pub fn set_data_right_shift(self, shift: config_types::DataRightShift) -> Self {
        T::regs()
            .ch(M::CHANNEL.index())
            .cfgr2()
            .modify(|w| w.set_dtrbs(shift.into()));
        self
    }

    /// Set the filterorder of the analog watchdog
    pub fn select_analog_watchdog_filter_order(self, filter_order: config_types::AnalogWatchdogFilterOrder) -> Self {
        T::regs()
            .ch(M::CHANNEL.index())
            .awscdr()
            .modify(|w| w.set_awford(filter_order as u8));
        self
    }

    /// Set the oversampling ratio of the analog watchdog filter
    pub fn select_analog_watchdog_osr(self, osr: config_types::AnalogWatchdogOsr) -> Self {
        T::regs()
            .ch(M::CHANNEL.index())
            .awscdr()
            .modify(|w| w.set_awfosr(osr.into()));
        self
    }
}

impl<'a, 'd, T, M, S, PS, P> Transceiver<'a, 'd, T, M, S, ParallelDmaMode, PS, P>
where
    T: Instance,
    M: TransceiverMarker + NextChannelForInstance<T>,
    S: PinSet,
    P: PowerState,
    PS: PinSource,
{
    /// Get direct pointer to the DATINR register for DMA mem2mem use
    pub fn get_datinr_as_ptr(&self) -> *mut u32 {
        T::regs().ch(M::CHANNEL.index()).datinr().as_ptr() as *mut u32
    }

    /// Manually write one sample into the DATINR register, used for standard mode
    pub fn write_sample_standard(&self, data: u16) {
        T::regs().ch(M::CHANNEL.index()).datinr().write(|w| w.set_indat0(data));
    }

    /// Manually write two subsequent samples into the DATINR register, used for interleaved mode
    pub fn write_indat1(&self, data: [u16; 2]) {
        T::regs().ch(M::CHANNEL.index()).datinr().write(|w| {
            w.set_indat0(data[0]);
            w.set_indat1(data[1]);
        });
    }
}
/// Any powerstate
impl<'a, 'd, T, M, S, MODE, PS, P> Transceiver<'a, 'd, T, M, S, MODE, PS, P>
where
    T: Instance,
    M: TransceiverMarker + NextChannelForInstance<T>,
    S: PinSet,
    MODE: ChannelMode,
    P: PowerState,
    PS: PinSource,
{
    /// Enable/Disable the transceiver
    pub(crate) fn set_enabled(enabled: bool) {
        T::regs().ch(M::CHANNEL.index()).cfgr1().modify(|w| w.set_chen(enabled));
    }

    /// Set channel offset
    pub fn set_offset(&mut self, offset: u32) {
        T::regs()
            .ch(M::CHANNEL.index())
            .cfgr2()
            .modify(|w| w.set_offset(offset));
    }

    /// Read input channel watchdog data.
    /// Data converted by the analog watchdog filter for input channel y.
    /// This data is continuously converted (no trigger) for this channel,
    /// with a limited resolution (OSR=1..32/sinc order = 1..3).
    pub fn get_analog_watchdog_data(&self) -> u16 {
        T::regs().ch(M::CHANNEL.index()).wdatr().read().wdata()
    }
}

impl<'a, 'd, T, M, S, MODE, PS, P> Transceiver<'a, 'd, T, M, S, MODE, PS, P>
where
    T: Instance + HasDelay,
    M: TransceiverMarker + NextChannelForInstance<T>,
    S: PinSet,
    MODE: ChannelMode,
    P: PowerState,
    PS: PinSource,
{
    /// Configure to skip the next `skips` pulses
    pub fn skip_pulses(&mut self, skips: config_types::PulsesToSkip) {
        self.set_pulseskips(skips);
    }

    /// Set pulse `skips`
    fn set_pulseskips(&mut self, skips: config_types::PulsesToSkip) {
        T::regs()
            .ch(M::CHANNEL.index())
            .dlyr()
            .modify(|w| w.set_plsskp(skips.into()));
    }
}

// =============================================================================
// InterruptHandler
// =============================================================================

// Implement properly only for Flt0 as Flt0 Handles instance-level events
impl<T> InstanceEvents<T> for Flt0
where
    T: Instance + FilterInterrupt<Flt0>,
{
    unsafe fn handle_instance_events() {
        if ShortCircuitDetector::<T>::short_circuit_detector_channel_flags_masked() != 0u8 {
            ShortCircuitDetector::<T>::set_short_circuit_detector_interrupt(false);
            T::instance_state().short_circuit_waker.wake();
        }
        if ClockAbsenceDetector::<T>::clock_absence_detector_channel_flags_masked() != 0u8 {
            ClockAbsenceDetector::<T>::set_clock_absence_interrupt(false);
            T::instance_state().clock_absence_waker.wake();
        }
    }
}

/// InterruptHandler for all DFSDM interrupts
pub struct InterruptHandler<T, F: FilterMarker>(PhantomData<(T, F)>);

impl<T, F> interrupt::typelevel::Handler<<T as FilterInterrupt<F>>::Interrupt> for InterruptHandler<T, F>
where
    T: Instance + FilterInterrupt<F>,
    F: FilterMarker + InstanceEvents<T>,
{
    unsafe fn on_interrupt() {
        // Per-filter common logic
        if FilterRegs::<T, F>::end_of_injected_conversion() {
            FilterRegs::<T, F>::set_injected_end_of_conversion_interrupt(false);
            <T as FilterInterrupt<F>>::state().injected_waker.wake();
        }
        if FilterRegs::<T, F>::end_of_regular_conversion() {
            FilterRegs::<T, F>::set_regular_end_of_conversion_interrupt(false);
            <T as FilterInterrupt<F>>::state().regular_waker.wake();
        }
        if AnalogWatchdog::<T, F>::analog_watchdog_triggered() {
            AnalogWatchdog::<T, F>::set_analog_watchdog_interrupt(false);
            <T as FilterInterrupt<F>>::state().watchdog_waker.wake();
        }

        // Instance logic (compiled out for Flt1..7)
        F::handle_instance_events();
    }
}

// ============================================================
// Builders
// ============================================================

/// Used to build a [`Detector`].
pub struct DetectorsBuilder<T>
where
    T: Instance + FilterInterrupt<Flt0>,
{
    _marker: PhantomData<T>,
}

impl<T> DetectorsBuilder<T>
where
    T: Instance + FilterInterrupt<Flt0>,
{
    /// Creates a new builder for a filter.
    pub(crate) fn new() -> Self {
        Self { _marker: PhantomData }
    }
    /// Build the actual Filter, binding it to the DfsdmCommon peripheral.
    /// This prevents DfsdmCommon from being dropped while the Filter exists.
    pub fn build<'a, 'd>(
        self,
        common: &'a DfsdmCommon<'d, T, Enabled>,
        _irqs: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T, Flt0>>,
    ) -> Detectors<'a, 'd, T> {
        <T as FilterInterrupt<Flt0>>::Interrupt::unpend();
        // SAFETY: Enabling the interrupt is safe here because:
        // 1. The `_irqs: impl Binding<…>` argument proves (at compile time) that
        //    `InterruptHandler<T, Flt0>::on_interrupt` is wired to this IRQ line.
        // 2. The waker is initialized in `State::new()` (const, in a static) before
        //    any interrupt can fire.
        // 3. The NVIC unmask here is independent of the peripheral IE bits:
        //    no filter-level IE (REOCIE/JEOCIE/AWDIE/…) is set at this call site
        //    (they are armed lazily by read_*/wait_for_event); if instance-level
        //    detector IEs (SCDIE/CKABIE) are already armed by waiting tasks, any
        //    pending event is handled safely by the same handler (flag clear +
        //    no-op wake). The stale-NVIC pending case was cleared by `unpend()`
        //    above - unpend discards only orphaned pending state; live sources
        //    re-pend because DFSDM's lines are level-asserted while `flag && IE`
        //    hold, and their events live in the ISR flags, not the pending bit.
        unsafe {
            <T as FilterInterrupt<Flt0>>::Interrupt::enable();
        }
        Detectors::new(common)
    }
}

/// Used to build a [`Filter`].
pub struct FilterBuilder<T, M>
where
    T: Instance,
    M: FilterMarker,
{
    _t: PhantomData<T>,
    _m: PhantomData<M>,
}

impl<T, M> FilterBuilder<T, M>
where
    T: Instance + FilterInterrupt<M>,
    M: FilterMarker + InstanceEvents<T>,
{
    /// Creates a new builder for a filter.
    pub(crate) fn new() -> Self {
        Self {
            _t: PhantomData,
            _m: PhantomData,
        }
    }
    /// Build the actual Filter, binding it to the DfsdmCommon peripheral.
    /// This prevents DfsdmCommon from being dropped while the Filter exists.
    pub fn build<'a, 'd>(
        self,
        common: &'a DfsdmCommon<'d, T, Enabled>,
        _irqs: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T, M>>,
    ) -> FilterDisabled<'a, 'd, T, M> {
        <T as FilterInterrupt<M>>::Interrupt::unpend();
        // SAFETY: Enabling the interrupt is safe here because:
        // 1. The `_irqs: impl Binding<…>` argument proves (at compile time) that
        //    `InterruptHandler<T, M>::on_interrupt` is wired to this IRQ line.
        // 2. The waker is initialized in `State::new()` (const, in a static) before
        //    any interrupt can fire.
        // 3. The NVIC unmask here is independent of the peripheral IE bits:
        //    no filter-level IE (REOCIE/JEOCIE/AWDIE/…) is set at this call site
        //    (they are armed lazily by read_*/wait_for_event); if instance-level
        //    detector IEs (SCDIE/CKABIE) are already armed by waiting tasks, any
        //    pending event is handled safely by the same handler (flag clear +
        //    no-op wake). The stale-NVIC pending case was cleared by `unpend()`
        //    above - unpend discards only orphaned pending state; live sources
        //    re-pend because DFSDM's lines are level-asserted while `flag && IE`
        //    hold, and their events live in the ISR flags, not the pending bit.
        unsafe {
            <T as FilterInterrupt<M>>::Interrupt::enable();
        }

        FilterDisabled::new(common)
    }
}

/// Used to build a [`Transceiver`].
pub struct TransceiverBuilder<T, M, C, S, SN>
where
    T: Instance,
    M: TransceiverMarker,
    C: ClockOutputMode,
    S: PinSet,  //Own pins
    SN: PinSet, //Neighbors pins
{
    _m: PhantomData<(T, M, C, S, SN)>,
}

impl<T, M, C, S, SN> TransceiverBuilder<T, M, C, S, SN>
where
    T: Instance,
    M: TransceiverMarker + NextChannelForInstance<T>,
    C: ClockOutputMode,
    S: PinSet,
    SN: PinSet,
{
    /// Creates a new builder for a transceiver.
    pub(crate) fn new() -> Self {
        Self { _m: PhantomData }
    }

    /// Parallel input from ADC writes to CHyDATINR (DATMPX=1).
    /// No CKOUT, no pins needed. Serial pins declared on this channel
    /// are disconnected (the builder's Flexes drop here - they're unused
    /// in this mode).
    pub fn build_parallel_adc<'a, 'd>(
        mut self,
        common: &'a DfsdmCommon<'d, T, Enabled>,
    ) -> Transceiver<'a, 'd, T, M, S, ParallelAdcMode, OwnPins, Disabled>
    where
        T: capability::AdcInput,
    {
        self.select_channel_input(config_types::ChannelInput::Same);
        self.select_data_mux_input(config_types::InputDataMux::InternalAdc);
        Transceiver {
            common,
            _instance_marker: PhantomData,
            _transceiver_marker: PhantomData,
            _pinset_marker: PhantomData,
            _channel_mode_marker: PhantomData,
            _pin_source_marker: PhantomData,
            _powerstate_marker: PhantomData,
        }
    }

    /// Parallel input from CPU/DMA writes to CHyDATINR (DATMPX=2).
    /// No CKOUT, no pins needed. Serial pins declared on this channel
    /// are disconnected (the builder's Flexes drop here - they're unused
    /// in this mode).
    pub fn build_parallel_dma<'a, 'd>(
        mut self,
        common: &'a DfsdmCommon<'d, T, Enabled>,
        packing_mode: config_types::DataPackingModeReduced,
    ) -> Transceiver<'a, 'd, T, M, S, ParallelDmaMode, OwnPins, Disabled> {
        self.select_channel_input(config_types::ChannelInput::Same);
        self.select_data_mux_input(config_types::InputDataMux::InternalRegisterWrite);
        self.set_data_packing_mode(packing_mode.into());
        Transceiver {
            common,
            _instance_marker: PhantomData,
            _transceiver_marker: PhantomData,
            _pinset_marker: PhantomData,
            _channel_mode_marker: PhantomData,
            _pin_source_marker: PhantomData,
            _powerstate_marker: PhantomData,
        }
    }

    /// Create dualmode DMA
    ///
    /// Returns transceivers for channel `M` (even, owns DATINR) and `MN` (odd,
    /// reads INDAT1 from M's DATINR). Two filters must be configured - one
    /// assigned to `M` (reads INDAT0, the lower word) and one to `MN`
    /// (reads INDAT1, the upper word) - or the register won't drain and
    /// you'll get overrun errors.
    pub fn build_parallel_dma_dual<'a, 'd, MN, SNN>(
        mut self,
        common: &'a DfsdmCommon<'d, T, Enabled>,
        mut neighbor: TransceiverBuilder<T, MN, C, SN, SNN>,
    ) -> (
        Transceiver<'a, 'd, T, M, S, ParallelDmaMode, OwnPins, Disabled>,
        Transceiver<'a, 'd, T, MN, SN, ParallelDmaMode, OwnPins, Disabled>,
    )
    where
        M: DualPackingAllowed + NextChannelForInstance<T, Next = MN>,
        MN: TransceiverMarker + NextChannelForInstance<T>,
        SNN: PinSet,
    {
        self.select_channel_input(config_types::ChannelInput::Same);
        neighbor.select_channel_input(config_types::ChannelInput::Same);
        self.select_data_mux_input(config_types::InputDataMux::InternalRegisterWrite);
        neighbor.select_data_mux_input(config_types::InputDataMux::InternalRegisterWrite);
        self.set_data_packing_mode(config_types::DataPackingMode::Dual);
        neighbor.set_data_packing_mode(config_types::DataPackingMode::Standard);
        (
            Transceiver {
                common,
                _instance_marker: PhantomData,
                _transceiver_marker: PhantomData,
                _pinset_marker: PhantomData,
                _channel_mode_marker: PhantomData,
                _pin_source_marker: PhantomData,
                _powerstate_marker: PhantomData,
            },
            Transceiver {
                common,
                _instance_marker: PhantomData,
                _transceiver_marker: PhantomData,
                _pinset_marker: PhantomData,
                _channel_mode_marker: PhantomData,
                _pin_source_marker: PhantomData,
                _powerstate_marker: PhantomData,
            },
        )
    }

    /// Manchester-coded input over this channel's own DATIN pin (SITP = 2/3,
    /// DATMPX = 0). The clock is recovered from the data line, so CKOUT/CKIN
    /// are not needed; the declared DATIN pin carries data *and* clock.
    /// `mode` chooses the Manchester polarity (rising edge = 0 or 1).
    pub fn build_manchester<'a, 'd>(
        mut self,
        common: &'a DfsdmCommon<'d, T, Enabled>,
        mode: config_types::ManchesterMode,
    ) -> Transceiver<'a, 'd, T, M, S, ManchesterMode, OwnPins, Disabled>
    where
        S: HasData,
    {
        self.select_channel_input(config_types::ChannelInput::Same);
        self.select_data_mux_input(config_types::InputDataMux::ExternalSerial);
        self.select_serial_interface_type(mode.into());
        Transceiver {
            common,
            _instance_marker: PhantomData,
            _transceiver_marker: PhantomData,
            _pinset_marker: PhantomData,
            _channel_mode_marker: PhantomData,
            _pin_source_marker: PhantomData,
            _powerstate_marker: PhantomData,
        }
    }

    ///Same as [`Self::build_manchester`] but using neighbors pins
    pub fn build_manchester_neighbor<'a, 'd>(
        mut self,
        common: &'a DfsdmCommon<'d, T, Enabled>,
        mode: config_types::ManchesterMode,
    ) -> Result<Transceiver<'a, 'd, T, M, DataOnly, ManchesterMode, NeighborPins, Disabled>, Error>
    where
        SN: HasData,
    {
        let next_ch = <M::Next as TransceiverMarker>::CHANNEL.index();
        common.acquire_pin(next_ch, PinKind::Datin)?;

        self.select_channel_input(config_types::ChannelInput::Neighbor);
        self.select_data_mux_input(config_types::InputDataMux::ExternalSerial);
        self.select_serial_interface_type(mode.into());
        Ok(Transceiver {
            common,
            _instance_marker: PhantomData,
            _transceiver_marker: PhantomData,
            _pinset_marker: PhantomData,
            _channel_mode_marker: PhantomData,
            _pin_source_marker: PhantomData,
            _powerstate_marker: PhantomData,
        })
    }

    /// SPI input over this channel's own pins (DATMPX=0, SPICKSEL=0): sampling
    /// clock comes from the *external* CKIN pin; requires a `DataClk` pinset
    /// (both lines). `mode` chooses rising/falling-edge sampling (SITP 0/1).
    pub fn build_spi_ext<'a, 'd>(
        mut self,
        common: &'a DfsdmCommon<'d, T, Enabled>,
        mode: config_types::SpiMode,
    ) -> Transceiver<'a, 'd, T, M, S, SpiExtMode, OwnPins, Disabled>
    where
        S: HasDataAndClk,
    {
        self.select_channel_input(config_types::ChannelInput::Same);
        self.select_data_mux_input(config_types::InputDataMux::ExternalSerial);
        self.select_serial_interface_type(mode.into());
        self.select_spi_clock(config_types::SpiClockSelect::ExternalCkin);
        Transceiver {
            common,
            _instance_marker: PhantomData,
            _transceiver_marker: PhantomData,
            _pinset_marker: PhantomData,
            _channel_mode_marker: PhantomData,
            _pin_source_marker: PhantomData,
            _powerstate_marker: PhantomData,
        }
    }

    ///Same as [`Self::build_spi_ext`] but using neighbors pins
    pub fn build_spi_ext_neighbor<'a, 'd>(
        mut self,
        common: &'a DfsdmCommon<'d, T, Enabled>,
        mode: config_types::SpiMode,
    ) -> Result<Transceiver<'a, 'd, T, M, DataClk, SpiExtMode, NeighborPins, Disabled>, Error>
    where
        SN: HasDataAndClk,
    {
        let next_ch = <M::Next as TransceiverMarker>::CHANNEL.index();
        common.acquire_pins::<DataClk>(next_ch)?;

        self.select_channel_input(config_types::ChannelInput::Neighbor);
        self.select_data_mux_input(config_types::InputDataMux::ExternalSerial);
        self.select_serial_interface_type(mode.into());
        self.select_spi_clock(config_types::SpiClockSelect::ExternalCkin);
        Ok(Transceiver {
            common,
            _instance_marker: PhantomData,
            _transceiver_marker: PhantomData,
            _pinset_marker: PhantomData,
            _channel_mode_marker: PhantomData,
            _pin_source_marker: PhantomData,
            _powerstate_marker: PhantomData,
        })
    }

    fn set_data_packing_mode(&mut self, mode: config_types::DataPackingMode) {
        // Dual mode is
        // available only on even channel numbers (y = 0, 2, 4, 6), for odd channel numbers (y = 1, 3, 5, 7)
        // DFSDM_CHyDATINR is write protected. If an even channel is set to dual mode then the following
        // odd channel must be set into standard mode (DATPACK[1:0]=0) for correct cooperation with even
        // channel.
        //  could make that explicit with a semantic constructor:
        // ch0.new_parallel_dma_dual()
        // meaning:
        // "ch0 and its paired successor are now configured as a dual-input pair."
        // then keeping the odd one for yourself, idk

        T::regs()
            .ch(M::CHANNEL.index())
            .cfgr1()
            .modify(|w| w.set_datpack(mode as u8));
    }

    fn select_data_mux_input(&mut self, input: config_types::InputDataMux) {
        T::regs()
            .ch(M::CHANNEL.index())
            .cfgr1()
            .modify(|w| w.set_datmpx(input as u8));
    }

    fn select_channel_input(&mut self, source: config_types::ChannelInput) {
        T::regs()
            .ch(M::CHANNEL.index())
            .cfgr1()
            .modify(|w| w.set_chinsel(source.into()));
    }

    fn select_spi_clock(&mut self, source: config_types::SpiClockSelect) {
        T::regs()
            .ch(M::CHANNEL.index())
            .cfgr1()
            .modify(|w| w.set_spicksel(source as u8));
    }

    fn select_serial_interface_type(&mut self, if_type: config_types::SerialInterfaceType) {
        T::regs()
            .ch(M::CHANNEL.index())
            .cfgr1()
            .modify(|w| w.set_sitp(if_type as u8));
    }
}

impl<T, M, S, SN> TransceiverBuilder<T, M, OutputEnabled, S, SN>
where
    T: Instance,
    M: TransceiverMarker + NextChannelForInstance<T>,
    S: PinSet,
    SN: PinSet,
{
    /// SPI input over this channel's own DATIN pin (DATMPX=0), clock supplied
    /// by our own CKOUT - only meaningful with `OutputEnabled`
    /// (`InternalSpiMode` picks rising/falling or the half-rate edges).
    pub fn build_spi_int<'a, 'd>(
        mut self,
        common: &'a DfsdmCommon<'d, T, Enabled>,
        mode: config_types::InternalSpiMode,
    ) -> Transceiver<'a, 'd, T, M, S, SpiCkoutMode, OwnPins, Disabled>
    where
        S: HasData,
    {
        self.select_channel_input(config_types::ChannelInput::Same);
        self.select_data_mux_input(config_types::InputDataMux::ExternalSerial);
        self.select_serial_interface_type(mode.into());
        self.select_spi_clock(mode.into());
        Transceiver {
            common,
            _instance_marker: PhantomData,
            _transceiver_marker: PhantomData,
            _pinset_marker: PhantomData,
            _channel_mode_marker: PhantomData,
            _pin_source_marker: PhantomData,
            _powerstate_marker: PhantomData,
        }
    }

    ///Same as [`Self::build_spi_int`] but using neighbors pins
    pub fn build_spi_int_neighbor<'a, 'd>(
        mut self,
        common: &'a DfsdmCommon<'d, T, Enabled>,
        mode: config_types::InternalSpiMode,
    ) -> Result<Transceiver<'a, 'd, T, M, DataOnly, SpiCkoutMode, NeighborPins, Disabled>, Error>
    where
        SN: HasData,
    {
        let next_ch = <M::Next as TransceiverMarker>::CHANNEL.index();
        common.acquire_pin(next_ch, PinKind::Datin)?;

        self.select_channel_input(config_types::ChannelInput::Neighbor);
        self.select_data_mux_input(config_types::InputDataMux::ExternalSerial);
        self.select_serial_interface_type(mode.into());
        self.select_spi_clock(mode.into());
        Ok(Transceiver {
            common,
            _instance_marker: PhantomData,
            _transceiver_marker: PhantomData,
            _pinset_marker: PhantomData,
            _channel_mode_marker: PhantomData,
            _pin_source_marker: PhantomData,
            _powerstate_marker: PhantomData,
        })
    }
}

// ============================================================
// Interrupt/Event accessors filter
// ============================================================

pub struct AnalogWatchdogConfig {
    pub fastmode: bool,
    pub low_break_signals: config_types::BreakSignals,
    pub high_break_signals: config_types::BreakSignals,
    pub low_threshold: i32,
    pub high_threshold: i32,
}

impl Default for AnalogWatchdogConfig {
    fn default() -> Self {
        Self {
            fastmode: false,
            low_break_signals: BreakSignals::empty(),
            high_break_signals: BreakSignals::empty(),
            low_threshold: i32::MAX,
            high_threshold: i32::MIN,
        }
    }
}

//TODO DOCSTRINGS, BITMAP TYPE, SPLIT
pub enum AnalogWatchdogEvent {
    /// AnalogWatchdog high threshold trigerred.
    HighThreshold { transceivers: u8 },
    /// AnalogWatchdog low threshold trigerred.
    LowThreshold { transceivers: u8 },
}
pub struct AnalogWatchdog<'a, 'd, T, M>
where
    T: Instance + FilterInterrupt<M>,
    M: FilterMarker + InstanceEvents<T>,
{
    _instance_marker: PhantomData<(T, M)>,
    common: &'a DfsdmCommon<'d, T, Enabled>,
}

impl<'a, 'd, T, M> AnalogWatchdog<'a, 'd, T, M>
where
    T: Instance + FilterInterrupt<M>,
    M: FilterMarker + InstanceEvents<T>,
{
    pub(crate) fn new(common: &'a DfsdmCommon<'d, T, Enabled>) -> Self {
        let mut new = Self {
            _instance_marker: PhantomData,
            common,
        };

        new.configure(AnalogWatchdogConfig::default());
        new
    }

    /// Wait for a analog watchdog event
    pub async fn wait_for_event(&mut self) -> AnalogWatchdogEvent {
        poll_fn(|cx| {
            Self::set_analog_watchdog_interrupt(false);
            T::state().watchdog_waker.register(cx.waker());

            let high = Self::analog_watchdog_high_channels();
            let low = Self::analog_watchdog_low_channels();

            if high != 0 {
                Self::clear_analog_watchdog_high(high);
                return Poll::Ready(AnalogWatchdogEvent::HighThreshold { transceivers: high });
            }
            if low != 0 {
                Self::clear_analog_watchdog_low(low);
                return Poll::Ready(AnalogWatchdogEvent::LowThreshold { transceivers: low });
            }

            Self::set_analog_watchdog_interrupt(true);
            Poll::Pending
        })
        .await
    }

    pub fn configure(&mut self, config: AnalogWatchdogConfig) {
        self.enable_analog_watchdog_fastmode(config.fastmode);
        self.assign_low_to_break_signals(config.low_break_signals);
        self.assign_high_to_break_signals(config.high_break_signals);
        self.set_low_threshold(config.low_threshold);
        self.set_high_threshold(config.high_threshold);
    }

    pub fn set_high_threshold(&mut self, threshold: i32) {
        T::regs()
            .flt(M::CHANNEL.index())
            .awhtr()
            .modify(|w| w.set_awht(threshold as u32));
    }

    pub fn set_low_threshold(&mut self, threshold: i32) {
        T::regs()
            .flt(M::CHANNEL.index())
            .awltr()
            .modify(|w| w.set_awlt(threshold as u32));
    }

    pub fn assign_high_to_break_signals(&mut self, break_signals: config_types::BreakSignals) {
        T::regs()
            .flt(M::CHANNEL.index())
            .awhtr()
            .modify(|w| w.set_bkawh(break_signals.bits()));
    }

    pub fn assign_low_to_break_signals(&mut self, break_signals: config_types::BreakSignals) {
        T::regs()
            .flt(M::CHANNEL.index())
            .awltr()
            .modify(|w| w.set_bkawl(break_signals.bits()));
    }

    pub fn enable_analog_watchdog_fastmode(&mut self, enabled: bool) {
        T::regs()
            .flt(M::CHANNEL.index())
            .cr1()
            .modify(|w| w.set_awfsel(enabled));
    }

    /// Assign provided transceivers to this analog watchdog
    pub fn assign_transceivers<const N: usize>(
        &mut self,
        // No borrow lifetime here as watchdog events arent awaited when channel is off.
        // They're "errors", not results that waiting for might stall your program.
        transceivers: [&dyn TransceiverTrait<T, Enabled>; N],
    ) where
        [(); N]: NonEmpty,
    {
        let filterword = transceivers.iter().fold(0u8, |acc, tcv| acc | (1 << tcv.index()));

        // thread-only writes, but full-register RMW on CR2 competes with the ISR's IE RMW - same cs discipline.
        critical_section::with(|_cs| {
            T::regs()
                .flt(M::CHANNEL.index())
                .cr2()
                .modify(|w| w.set_awdch(filterword));
        });
    }

    /// Enables or disables analog watchdog interrupts.
    pub(crate) fn set_analog_watchdog_interrupt(enabled: bool) {
        // RMW'd from both ISR and thread (the ISR clears its own IE here) - cs is load-bearing.
        critical_section::with(|_cs| {
            T::regs().flt(M::CHANNEL.index()).cr2().modify(|w| w.set_awdie(enabled));
        });
    }

    /// Returns whether the analog watchdog has been triggerd
    pub(crate) fn analog_watchdog_triggered() -> bool {
        T::regs().flt(M::CHANNEL.index()).isr().read().awdf()
    }

    /// Returns bitmap of channels who triggered the high threshold
    pub(crate) fn analog_watchdog_high_channels() -> u8 {
        T::regs().flt(M::CHANNEL.index()).awsr().read().awhtf()
    }

    /// Returns bitmap of channels who triggered the low threshold
    pub(crate) fn analog_watchdog_low_channels() -> u8 {
        T::regs().flt(M::CHANNEL.index()).awsr().read().awltf()
    }

    /// Clears the provided channels' analog watchdog flags
    pub(crate) fn clear_analog_watchdog_high(channels: u8) {
        T::regs()
            .flt(M::CHANNEL.index())
            .awsr()
            .modify(|w| w.set_awhtf(channels));
    }

    /// Clears the provided channels' analog watchdog flags
    pub(crate) fn clear_analog_watchdog_low(channels: u8) {
        T::regs()
            .flt(M::CHANNEL.index())
            .awsr()
            .modify(|w| w.set_awltf(channels));
    }
}
pub struct ExtremesDetector<'a, 'd, T, M>
where
    T: Instance,
    M: FilterMarker,
{
    _common: PhantomData<(&'a DfsdmCommon<'d, T, Enabled>, M)>,
}

impl<'a, 'd, T, M> ExtremesDetector<'a, 'd, T, M>
where
    T: Instance,
    M: FilterMarker,
{
    pub(crate) fn new(_common: &'a DfsdmCommon<'d, T, Enabled>) -> Self {
        Self { _common: PhantomData }
    }

    /// Assign provided transceivers to this extremes detector
    pub fn assign_transceivers<const N: usize>(
        &mut self,
        // No borrow lifetime here as watchdog events arent awaited when channel is off.
        // They're "errors", not results that waiting for might stall your program.
        transceivers: [&dyn TransceiverTrait<T, Enabled>; N],
    ) where
        [(); N]: NonEmpty,
    {
        let filterword = transceivers.iter().fold(0u8, |acc, tcv| acc | (1 << tcv.index()));

        // thread-only writes, but full-register RMW on CR2 competes with the ISR's IE RMW - same cs discipline.
        critical_section::with(|_cs| {
            T::regs()
                .flt(M::CHANNEL.index())
                .cr2()
                .modify(|w| w.set_exch(filterword));
        });
    }

    /// Reads the extremes detector maximum value and its corresponding channel.
    ///
    /// Returns a tuple (maximum, channel) containing:
    /// - `maximum`: The highest value converted by the filter (`EXMAX[23:0]`). Reading this
    ///   register resets the value to `0x800000`.
    /// - `channel`: The channel index on which the maximum data was stored (`EXMAXCH[2:0]`).
    ///   Reading this register clears the bits.
    pub fn read_maxima(&mut self) -> (u32, u8) {
        let exmax = T::regs().flt(M::CHANNEL.index()).exmax().read();
        (exmax.exmax(), exmax.exmaxch())
    }

    /// Reads the extremes detector minimum value and its corresponding channel.
    ///
    /// Returns a tuple (maximum, channel) containing:
    /// - `minimum`: The highest value converted by the filter (`EXMAX[23:0]`). Reading this
    ///   register resets the value to `0x7FFFFF`.
    /// - `channel`: The channel index on which the maximum data was stored (`EXMAXCH[2:0]`).
    ///   Reading this register clears the bits.
    pub fn read_minima(&mut self) -> (u32, u8) {
        let exmin = T::regs().flt(M::CHANNEL.index()).exmin().read();
        (exmin.exmin(), exmin.exminch())
    }
}

// ============================================================
// Interrupt/Event accessors instance
// ============================================================

pub struct Detectors<'a, 'd, T>
where
    T: Instance + FilterInterrupt<Flt0>,
{
    pub short_circuit: ShortCircuitDetector<'a, 'd, T>,
    pub clock_absence: ClockAbsenceDetector<'a, 'd, T>,
}

impl<'a, 'd, T> Detectors<'a, 'd, T>
where
    T: Instance + FilterInterrupt<Flt0>,
{
    pub(crate) fn new(common: &'a DfsdmCommon<'d, T, Enabled>) -> Self {
        Self {
            short_circuit: ShortCircuitDetector::new(common),
            clock_absence: ClockAbsenceDetector::new(common),
        }
    }
}

/// One channel's short-circuit assignment: the transceiver to guard,
/// and the saturation threshold to guard it with.
#[derive(Copy, Clone)]
pub struct ShortCircuitAssignment<'t, T: Instance> {
    transceiver: &'t dyn TransceiverTrait<T, Enabled>,
    threshold: u8,
}

impl<'t, T: Instance> ShortCircuitAssignment<'t, T> {
    pub const fn new(transceiver: &'t dyn TransceiverTrait<T, Enabled>, threshold: u8) -> Self {
        Self { transceiver, threshold }
    }
}

pub struct ShortCircuitDetector<'a, 'd, T>
where
    T: Instance,
{
    _common: PhantomData<&'a DfsdmCommon<'d, T, Enabled>>,
}

impl<'a, 'd, T> ShortCircuitDetector<'a, 'd, T>
where
    T: Instance + FilterInterrupt<Flt0>,
{
    pub(crate) fn new(_common: &'a DfsdmCommon<'d, T, Enabled>) -> Self {
        Self { _common: PhantomData }
    }

    /// Wait for a short-circuit-detector event
    pub async fn wait_for_event(&mut self) -> u8 {
        poll_fn(|cx| {
            Self::set_short_circuit_detector_interrupt(false);
            T::instance_state().short_circuit_waker.register(cx.waker());

            let channels = Self::short_circuit_detector_channel_flags_masked();
            if channels != 0 {
                Self::clear_short_circuit_detector_channel(channels);
                return Poll::Ready(channels);
            }

            Self::set_short_circuit_detector_interrupt(true);
            Poll::Pending
        })
        .await
    }
}

impl<'a, 'd, T> ShortCircuitDetector<'a, 'd, T>
where
    T: Instance,
{
    /// Assigns the transceivers to the short-circuit-detector (overwrites assignments)
    pub fn assign_transceivers<const N: usize>(&mut self, assignments: [ShortCircuitAssignment<T>; N])
    where
        [(); N]: NonEmpty,
    {
        for assignment in assignments {
            self.set_threshold(assignment.transceiver, assignment.threshold);
        }
        let tcv: [&dyn TransceiverTrait<T, Enabled>; N] = assignments.map(|a| a.transceiver);

        Self::set_short_circuit_channels(detector_common::filterword_of(&tcv));
    }

    /// Unassigns the transceivers from the short-circuit-detector
    pub fn unassign_transceivers<const N: usize>(&mut self, transceivers: [&dyn TransceiverTrait<T, Enabled>; N])
    where
        [(); N]: NonEmpty,
    {
        Self::set_short_circuit_channels(
            Self::short_circuit_channel_word() & !detector_common::filterword_of(&transceivers),
        );
    }

    /// Assign break-signals for short-circuit-event of transceiver
    pub fn assign_break_signals(
        &mut self,
        transceiver: &dyn TransceiverTrait<T, Enabled>,
        signals: config_types::BreakSignals,
    ) {
        T::regs()
            .ch(transceiver.index())
            .awscdr()
            .modify(|w| w.set_bkscd(signals.bits()));
    }

    /// Set short-circuit-detector-threshold for transceiver
    pub fn set_threshold(&mut self, transceiver: &dyn TransceiverTrait<T, Enabled>, threshold: u8) {
        T::regs()
            .ch(transceiver.index())
            .awscdr()
            .modify(|w| w.set_scdt(threshold));
    }

    pub(crate) fn drop_transceiver(channel: TransceiverChannel) {
        let ch = channel.index();
        Self::set_short_circuit_channels(Self::short_circuit_channel_word() & !(1 << ch));
    }

    /// Aggregate CFGR1 bit-word for one detector kind, over the channels this
    /// instance actually has.
    fn short_circuit_channel_word() -> u8 {
        let count = <T::Transceivers as capability::TransceiverCount>::COUNT;
        (0..count).fold(0u8, |acc, y| {
            acc | ((T::regs().ch(y as usize).cfgr1().read().scden() as u8) << y)
        })
    }

    /// Authority: make the registers match `mask` exactly, then refresh the armed cache.
    fn set_short_circuit_channels(mask: u8) {
        let mask = mask & detector_common::channel_count_mask::<T>();
        for y in 0..<T::Transceivers as capability::TransceiverCount>::COUNT {
            let want = (mask >> y) & 1 == 1;
            T::regs().ch(y as usize).cfgr1().modify(|w| w.set_scden(want));
        }
        T::instance_state().short_circuit_armed.store(mask, Ordering::Relaxed);
    }

    /// Enables or disables short-circuit detector interrupts.
    pub(crate) fn set_short_circuit_detector_interrupt(enabled: bool) {
        // RMW'd from both ISR and thread (the ISR clears its own IE here) - cs is load-bearing.
        critical_section::with(|_cs| {
            T::regs().flt(0).cr2().modify(|w| w.set_scdie(enabled));
        });
    }

    /// Returns bitmap of channels who triggered the short-circuit-detector
    pub(crate) fn short_circuit_detector_channel_flags() -> u8 {
        T::regs().flt(0).isr().read().scdf()
    }

    /// Returns bitmap of channels who triggered the short-circuit-detector and are armed
    pub(crate) fn short_circuit_detector_channel_flags_masked() -> u8 {
        Self::short_circuit_detector_channel_flags() & T::instance_state().short_circuit_armed.load(Ordering::Relaxed)
    }

    /// Clears the provided channel flags in the short-circuit-detector
    pub(crate) fn clear_short_circuit_detector_channel(channels: u8) {
        T::regs().flt(0).icr().modify(|w| w.set_clrscdf(channels));
    }
}

pub struct ClockAbsenceDetector<'a, 'd, T>
where
    T: Instance,
{
    _common: PhantomData<&'a DfsdmCommon<'d, T, Enabled>>,
}

impl<'a, 'd, T> ClockAbsenceDetector<'a, 'd, T>
where
    T: Instance + FilterInterrupt<Flt0>,
{
    pub(crate) fn new(_common: &'a DfsdmCommon<'d, T, Enabled>) -> Self {
        Self { _common: PhantomData }
    }

    /// Wait for a clock-absence-detector event
    pub async fn wait_for_event(&mut self) -> u8 {
        poll_fn(|cx| {
            Self::set_clock_absence_interrupt(false);
            T::instance_state().clock_absence_waker.register(cx.waker());

            let channels = Self::clock_absence_detector_channel_flags_masked();

            if channels != 0 {
                Self::clear_clock_absence_detector_channel(channels);
                return Poll::Ready(channels);
            }

            Self::set_clock_absence_interrupt(true);
            Poll::Pending
        })
        .await
    }
}

impl<'a, 'd, T> ClockAbsenceDetector<'a, 'd, T>
where
    T: Instance,
{
    /// Assigns the transceivers to the clock-absence-detector (overwrites assignments)
    pub fn assign_transceivers<const N: usize>(&mut self, transceivers: [&dyn TransceiverTrait<T, Enabled>; N])
    where
        [(); N]: NonEmpty,
    {
        Self::set_clock_absence_channels(detector_common::filterword_of(&transceivers));
    }

    /// Unassigns the transceivers from the clock-absence-detector
    pub fn unassign_transceivers<const N: usize>(&mut self, transceivers: [&dyn TransceiverTrait<T, Enabled>; N])
    where
        [(); N]: NonEmpty,
    {
        Self::set_clock_absence_channels(
            Self::clock_absence_channel_word() & !detector_common::filterword_of(&transceivers),
        );
    }

    pub(crate) fn drop_transceiver(channel: TransceiverChannel) {
        let ch = channel.index();
        Self::set_clock_absence_channels(Self::clock_absence_channel_word() & !(1 << ch));
    }

    /// Aggregate CFGR1 bit-word for one detector kind, over the channels this
    /// instance actually has.
    fn clock_absence_channel_word() -> u8 {
        let count = <T::Transceivers as capability::TransceiverCount>::COUNT;
        (0..count).fold(0u8, |acc, y| {
            acc | ((T::regs().ch(y as usize).cfgr1().read().ckaben() as u8) << y)
        })
    }

    /// Authority: make the registers match `mask` exactly, then refresh the armed cache.
    fn set_clock_absence_channels(mask: u8) {
        let mask = mask & detector_common::channel_count_mask::<T>();
        for y in 0..<T::Transceivers as capability::TransceiverCount>::COUNT {
            let want = (mask >> y) & 1 == 1;
            T::regs().ch(y as usize).cfgr1().modify(|w| w.set_ckaben(want));
        }
        T::instance_state().clock_absence_armed.store(mask, Ordering::Relaxed);
    }

    /// Enables or disables clock absence interrupts.
    pub(crate) fn set_clock_absence_interrupt(enabled: bool) {
        // RMW'd from both ISR and thread (the ISR clears its own IE here) - cs is load-bearing.
        critical_section::with(|_cs| {
            T::regs().flt(0).cr2().modify(|w| w.set_ckabie(enabled));
        });
    }

    /// Returns bitmap of channels who triggered the clock-absence-detector
    pub(crate) fn clock_absence_detector_channel_flags() -> u8 {
        T::regs().flt(0).isr().read().ckabf()
    }

    /// Returns bitmap of channels who triggered the clock-absence-detector and are armed
    pub(crate) fn clock_absence_detector_channel_flags_masked() -> u8 {
        Self::clock_absence_detector_channel_flags() & T::instance_state().clock_absence_armed.load(Ordering::Relaxed)
    }

    /// Clears the provided channel flags in the clock-absence-detector
    pub(crate) fn clear_clock_absence_detector_channel(channels: u8) {
        T::regs().flt(0).icr().modify(|w| w.set_clrckabf(channels));
    }
}

mod detector_common {
    use super::*;

    /// filterword fold over a transceiver slice
    pub(crate) fn filterword_of<T: Instance>(transceivers: &[&dyn TransceiverTrait<T, Enabled>]) -> u8 {
        transceivers.iter().fold(0u8, |acc, tcv| acc | (1 << tcv.index()))
    }

    /// Valid-bits mask for this shape; every mask user passes gets intersected with it.
    pub(crate) const fn channel_count_mask<T: Instance>() -> u8 {
        let count = <T::Transceivers as capability::TransceiverCount>::COUNT as u8;
        ((1u16 << count) - 1) as u8
    }
}

// ============================================================
// Single-use selectors
// ============================================================

/// Per-channel pin selector. Cannot be constructed outside this module
/// (private field); handed out only inside the `configure_pins` closure,
/// one per channel, **by value**. Every method consumes `self`, so each
/// channel's pins can be declared exactly once (E0382 otherwise).
pub struct Sel<T: Instance, M: TransceiverMarker> {
    _m: PhantomData<(T, M)>,
}

impl<'d, T, M> Sel<T, M>
where
    T: Instance,
    M: TransceiverMarker,
{
    /// Declare this channel with a DATIN pin (AF set here).
    pub fn datin(self, datin: Peri<'d, if_afio!(impl DatinPin<T, M, A>)>) -> DatinCfg<'d, T, M> {
        DatinCfg {
            datin: new_pin!(datin, AfType::input(Pull::None)).unwrap(),
            _m: PhantomData,
        }
    }

    /// Declare this channel with DATIN + CKIN.
    pub fn datin_ckin(
        self,
        datin: Peri<'d, if_afio!(impl DatinPin<T, M, A>)>,
        ckin: Peri<'d, if_afio!(impl CkinPin<T, M, A>)>,
    ) -> DckCfg<'d, T, M> {
        DckCfg {
            datin: new_pin!(datin, AfType::input(Pull::None)).unwrap(),
            ckin: new_pin!(ckin, AfType::input(Pull::None)).unwrap(),
            _m: PhantomData,
        }
    }

    /// Declare this channel as pinless (same as [`NoPinsCfg`]).
    pub fn none(self) -> NoPinsCfg {
        NoPinsCfg
    }
}

impl<'d, T, C> Dfsdm<'d, T, C>
where
    T: Instance,
    C: ClockOutputMode,
{
    /// The closure receives one selector per channel the instance actually has,
    /// and must return one token per channel, as a tuple in the same order.
    ///
    /// // DFSDM instance with 8-channel capabiltiy:
    /// ```
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
    /// // DFSDM instance with 2-channel capabiltiy:
    /// ```
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

        let mut common = DfsdmCommon::new(self.peri.expect("taken once"), self.ckout.take()).enable();
        let split = out.split_parts(&mut common);

        (common, split)
    }
}

//! Digital Filter and Sigma-Delta Modulator (DFSDM)

#![macro_use]

/// Connect pac and embassy infrastructure to our types
pub mod associations;
pub mod dma;
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
pub use types::*;

use crate::dfsdm::capability::HasDelay;
use crate::dfsdm::config_types::FilterParameters;
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

/// [`Transceiver`]
/// Configured DFSDM data input transceiver.
pub struct Transceiver<'a, 'd, T, M, S, MODE, P>
where
    T: Instance,
    M: TransceiverMarker + NextChannelForInstance<T>,
    S: PinSet,
    MODE: ChannelMode,
    P: PowerState,
{
    pub(crate) common: &'a DfsdmCommon<'d, T, Enabled>,
    _instance_marker: PhantomData<T>,
    _transceiver_marker: PhantomData<M>,
    _pinset_marker: PhantomData<S>,
    _datasource_marker: PhantomData<MODE>,
    _powerstate_marker: PhantomData<P>,
}

impl<'a, 'd, T, M, S, MODE, P> Drop for Transceiver<'a, 'd, T, M, S, MODE, P>
where
    T: Instance,
    M: TransceiverMarker + NextChannelForInstance<T>,
    S: PinSet,
    MODE: ChannelMode,
    P: PowerState,
{
    fn drop(&mut self) {
        let ch = if MODE::USES_NEIGHBOR_PINS {
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
    }
}

/// Confgiguration for Filter
pub struct FilterConfig {
    pub filter_params: FilterParameters,
    pub enable_injected_dma: bool,
    pub enable_regular_dma: bool,
    pub enable_continuous_regular: bool,
    pub enable_fast_regular: bool,
}

impl Default for FilterConfig {
    fn default() -> Self {
        Self {
            filter_params: FilterParameters::new(config_types::FilterOrder::Disabled, 1),
            enable_injected_dma: false,
            enable_regular_dma: false,
            enable_continuous_regular: false,
            enable_fast_regular: false,
        }
    }
}
pub struct Filter<'reg, 'inj, T, M, P>
where
    T: Instance + FilterInterrupt<M>,
    M: FilterMarker,
    P: PowerState,
{
    _instance_marker: PhantomData<T>,
    _filter_marker: PhantomData<M>,
    _powerstate_marker: PhantomData<P>,
    regular: Option<&'reg dyn TransceiverTrait<T, Enabled>>,
    injected: [Option<&'inj dyn TransceiverTrait<T, Enabled>>; 8],
}

impl<'reg, 'inj, T, M> Filter<'reg, 'inj, T, M, Disabled>
where
    T: Instance + FilterInterrupt<M>,
    M: FilterMarker,
{
    pub(crate) fn new_disabled() -> Filter<'reg, 'inj, T, M, Disabled> {
        unsafe {
            T::Interrupt::enable();
        }

        Filter {
            _instance_marker: PhantomData,
            _filter_marker: PhantomData,
            _powerstate_marker: PhantomData,
            regular: None,
            injected: [None; 8],
        }
    }

    /// Enable the Filter
    pub fn enable<'new_reg, 'new_inj, const N: usize>(
        mut self,
        reg_transceiver: &'new_reg dyn TransceiverTrait<T, Enabled>,
        inj_transceivers: [&'new_inj dyn TransceiverTrait<T, Enabled>; N],
    ) -> Filter<'new_reg, 'new_inj, T, M, Enabled>
    where
        [(); N]: NonEmpty,
    {
        T::regs().flt(M::CHANNEL.index()).cr1().modify(|w| w.set_dfen(true));

        let this = self
            .assign_regular_transceiver(reg_transceiver)
            .assign_injected_transceivers(inj_transceivers);

        Filter {
            _instance_marker: PhantomData,
            _filter_marker: PhantomData,
            _powerstate_marker: PhantomData,
            regular: this.regular,
            injected: this.injected,
        }
    }

    /// Configure the Filter
    pub fn configure(mut self, config: &FilterConfig) -> Filter<'reg, 'inj, T, M, Disabled> {
        self.set_filter_parameters(config.filter_params);
        Self::set_regular_dma_en(config.enable_regular_dma);
        Self::set_injected_dma_en(config.enable_injected_dma);
        Self::set_continuous(config.enable_continuous_regular);
        Self::set_fastmode(config.enable_fast_regular);
        self
    }

    /// Writes the filterparameters
    fn set_filter_parameters(&mut self, params: config_types::FilterParameters) {
        let (order, fosr, iosr) = params.register_values();
        T::regs().flt(M::CHANNEL.index()).fcr().modify(|w| {
            w.set_ford(order);
            w.set_fosr(fosr);
            w.set_iosr(iosr);
        });
    }

    /// Enables or disables DMA transfers for regular conversions.
    pub(crate) fn set_regular_dma_en(dma_enable: bool) {
        T::regs()
            .flt(M::CHANNEL.index())
            .cr1()
            .modify(|w| w.set_rdmaen(dma_enable));
    }

    /// Enables or disables DMA transfers for injected conversions.
    pub(crate) fn set_injected_dma_en(dma_enable: bool) {
        T::regs()
            .flt(M::CHANNEL.index())
            .cr1()
            .modify(|w| w.set_jdmaen(dma_enable));
    }

    /// Enables or disables synchronization for regular conversions.
    pub(crate) fn set_regular_synchronization(enable: bool) {
        T::regs().flt(M::CHANNEL.index()).cr1().modify(|w| w.set_rsync(enable));
    }

    /// Enables or disables synchronization for injected conversions.
    pub(crate) fn set_injected_synchronization(enable: bool) {
        T::regs().flt(M::CHANNEL.index()).cr1().modify(|w| w.set_jsync(enable));
    }

    /// Configures the trigger for injected conversions.
    ///
    /// `Some` enables the trigger with the specified trigger source and edge.
    /// `None` disables the trigger.
    pub(crate) fn configure_injected_trigger(trigger: Option<(InjectedDfsdmTrigger<T>, config_types::TriggerEdge)>) {
        let (jextsel, jexten) = match trigger {
            Some((trigger, edge)) => (trigger.id(), edge as u8),
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
}

impl<'reg, 'inj, T, M, P> Filter<'reg, 'inj, T, M, P>
where
    T: Instance + FilterInterrupt<M>,
    M: FilterMarker,
    P: PowerState,
{
    /// Trigger a regular conversion
    pub fn start_regular_conversion(&mut self) {
        T::regs().flt(M::CHANNEL.index()).cr1().modify(|w| w.set_rswstart(true));
    }

    /// Trigger a injected conversion
    pub fn start_injected_conversion(&mut self) {
        T::regs().flt(M::CHANNEL.index()).cr1().modify(|w| w.set_jswstart(true));
    }

    /// Trigger a regular conversion and read it asynchronously using interrupts
    pub async fn read_regular(
        &mut self,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T, M>>,
    ) -> (i32, u8, bool) {
        self.start_regular_conversion();

        poll_fn(|cx| {
            Self::set_regular_end_of_conversion_interrupt(false);
            T::state().regular_waker.register(cx.waker());

            if let Some(result) = self.try_get_regular_result() {
                Poll::Ready(result)
            } else {
                Self::set_regular_end_of_conversion_interrupt(true);
                Poll::Pending
            }
        })
        .await
    }

    /// Trigger a injected conversion and read it asynchronously using interrupts
    pub async fn read_injected(
        &mut self,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T, M>>,
    ) -> (i32, u8) {
        self.start_injected_conversion();

        poll_fn(|cx| {
            Self::set_injected_end_of_conversion_interrupt(false);
            T::state().injected_waker.register(cx.waker());

            if let Some(result) = self.try_get_injected_result() {
                Poll::Ready(result)
            } else {
                Self::set_injected_end_of_conversion_interrupt(true);
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
        if self.is_end_of_regular_conversion() {
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

    /// Attempts to read the current injected conversion result.
    ///
    /// Returns `Some((data, channel))` if `JEOCF` is set, or `None` if no injected
    /// conversion result is available.
    ///
    /// The conversion result is sign-extended from 24 to 32 bits and is not scaled.
    ///
    /// Reading the result clears the corresponding data register.
    pub fn try_get_injected_result(&mut self) -> Option<(i32, u8)> {
        if self.is_end_of_injected_conversion() {
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

    /// Returns whether a regular conversion result is available.
    pub(crate) fn end_of_regular_conversion() -> bool {
        T::regs().flt(M::CHANNEL.index()).isr().read().reocf()
    }

    /// Returns whether a regular conversion result is available.
    pub fn is_end_of_regular_conversion(&mut self) -> bool {
        Self::end_of_regular_conversion()
    }

    /// Returns whether an injected conversion result is available.
    pub(crate) fn end_of_injected_conversion() -> bool {
        T::regs().flt(M::CHANNEL.index()).isr().read().jeocf()
    }

    /// Returns whether an injected conversion result is available.
    pub fn is_end_of_injected_conversion(&mut self) -> bool {
        Self::end_of_injected_conversion()
    }

    /// Returns whether a regular conversion is currently in progress or pendiong.
    pub fn regular_conversion_in_progress() -> bool {
        T::regs().flt(M::CHANNEL.index()).isr().read().rcip()
    }

    /// Returns whether a regular conversion is currently in progress or pendiong.
    pub fn is_regular_conversion_in_progress(&mut self) -> bool {
        Self::regular_conversion_in_progress()
    }

    /// Returns whether an injected conversion is currently in progress or pendiong.
    pub fn injected_conversion_in_progress() -> bool {
        T::regs().flt(M::CHANNEL.index()).isr().read().jcip()
    }

    /// Returns whether an injected conversion is currently in progress or pendiong.
    pub fn is_injected_conversion_in_progress(&mut self) -> bool {
        Self::injected_conversion_in_progress()
    }

    fn set_regular_transceiver(&mut self, ch: u8) {
        T::regs().flt(M::CHANNEL.index()).cr1().modify(|w| w.set_rch(ch));
    }

    fn set_injected_channels(&mut self, channels: u8) {
        T::regs()
            .flt(M::CHANNEL.index())
            .jchgr()
            .write(|w| w.set_jchg(channels));
    }

    /// Enables/Disables analog watchdog fast mode.
    /// When enabled, the analog watchdog works on data directly from the transceiver.
    /// When disabled, the analog watchdog works on data filtered by the filter.
    pub(crate) fn set_analog_watchdog_fastmode(enabled: bool) {
        T::regs()
            .flt(M::CHANNEL.index())
            .cr1()
            .modify(|w| w.set_awfsel(enabled));
    }

    /// Enables or disables fast conversion mode.
    ///
    /// In continuous mode, fast mode reduces the conversion time after the first
    /// conversion because the filter is already filled and does not need to be
    /// filled again. Subsequent conversions therefore take only `FOSR * IOSR / fCKIN`
    /// instead of the normal filter fill time. Has no effect outside continuous mode.
    pub(crate) fn set_fastmode(enabled: bool) {
        T::regs().flt(M::CHANNEL.index()).cr1().modify(|w| w.set_fast(enabled));
    }

    /// Enables or disables continuous conversion mode.
    ///
    /// When enabled, the regular channel is converted repeatedly after each
    /// conversion request. Disabling it while a continuous conversion is in
    /// progress stops the conversion immediately.
    pub(crate) fn set_continuous(enabled: bool) {
        T::regs().flt(M::CHANNEL.index()).cr1().modify(|w| w.set_rcont(enabled));
    }

    /// Enables or disables scanning mode for injected conversions.
    ///
    /// When enabled, injected conversions cycle through all selected channels,
    /// starting again at the lowest selected channel. When disabled, each
    /// conversion advances to the next selected channel.
    ///
    /// Changing the injected channel group while scanning is disabled resets the
    /// channel selection to the lowest selected channel.
    pub(crate) fn set_injected_scanning(enabled: bool) {
        T::regs().flt(M::CHANNEL.index()).cr1().modify(|w| w.set_jscan(enabled));
    }

    /// Enables the analog watchdog for the specified channel.
    pub(crate) fn set_watchdog_channel(ch: u8) {
        T::regs()
            .flt(M::CHANNEL.index())
            .cr2()
            .modify(|w| w.set_awdch(w.awdch() | (1 << ch)));
    }

    /// Disables the analog watchdog for the specified channel.
    pub(crate) fn clear_watchdog_channel(ch: u8) {
        T::regs()
            .flt(M::CHANNEL.index())
            .cr2()
            .modify(|w| w.set_awdch(w.awdch() & !(1 << ch)));
    }

    /// Assign the provided `Transceiver` to the analog watchdog channel of this `Filter`
    pub fn assign_watchdog_channel<MT, S, MODE, PT>(mut self, channel: &Transceiver<'_, '_, T, MT, S, MODE, PT>) -> Self
    where
        MT: TransceiverMarker + NextChannelForInstance<T>,
        S: PinSet,
        MODE: ChannelMode,
        PT: PowerState,
    {
        Self::set_watchdog_channel(channel.index() as u8);
        self
    }

    /// Unassign the provided channel from the analog watchdog of this `Filter`
    pub fn unassign_watchdog_channel<MT, S, MODE, PT>(
        mut self,
        channel: &Transceiver<'_, '_, T, MT, S, MODE, PT>,
    ) -> Self
    where
        MT: TransceiverMarker + NextChannelForInstance<T>,
        S: PinSet,
        MODE: ChannelMode,
        PT: PowerState,
    {
        Self::clear_watchdog_channel(channel.index() as u8);
        self
    }

    /// Enables the extremes detector for the specified channel.
    pub(crate) fn set_extremes_detector_channel(ch: u8) {
        T::regs()
            .flt(M::CHANNEL.index())
            .cr2()
            .modify(|w| w.set_exch(w.exch() | (1 << ch)));
    }

    /// Disables the extremes detector for the specified channel.
    pub(crate) fn clear_extremes_detector_channel(ch: u8) {
        T::regs()
            .flt(M::CHANNEL.index())
            .cr2()
            .modify(|w| w.set_exch(w.exch() & !(1 << ch)));
    }

    /// Assign the provided `Transceiver` as the regular conversion input for this `Filter`
    pub(crate) fn assign_regular_transceiver<'new_reg>(
        mut self,
        channel: &'new_reg dyn TransceiverTrait<T, Enabled>,
    ) -> Filter<'new_reg, 'inj, T, M, P> {
        self.set_regular_transceiver(channel.index() as u8);

        Filter {
            _instance_marker: PhantomData,
            _filter_marker: PhantomData,
            _powerstate_marker: PhantomData,
            regular: Some(channel),
            injected: self.injected,
        }
    }

    /// Assign transceivers to the injected conversion group of this `Filter`
    ///
    /// Note: Overwrites all assignments.
    pub(crate) fn assign_injected_transceivers<'new_inj, const N: usize>(
        mut self,
        transceivers: [&'new_inj dyn TransceiverTrait<T, Enabled>; N],
    ) -> Filter<'reg, 'new_inj, T, M, P>
    where
        [(); N]: NonEmpty,
    {
        let filterword = transceivers.iter().fold(0u8, |acc, tcv| acc | (1 << tcv.index()));
        self.set_injected_channels(filterword);

        let mut injected = [None; 8];
        for (i, tcv) in transceivers.iter().enumerate() {
            injected[i] = Some(*tcv);
        }
        Filter {
            _instance_marker: PhantomData,
            _filter_marker: PhantomData,
            _powerstate_marker: PhantomData,
            regular: self.regular,
            injected: injected,
        }
    }

    /// Assign the provided `Transceiver` to the extremes detector of this `Filter`.
    pub fn assign_extremes_detector_channel<MT, S, MODE, PT>(
        mut self,
        channel: &Transceiver<'_, '_, T, MT, S, MODE, PT>,
    ) -> Self
    where
        MT: TransceiverMarker + NextChannelForInstance<T>,
        S: PinSet,
        MODE: ChannelMode,
        PT: PowerState,
    {
        Self::set_extremes_detector_channel(channel.index() as u8);
        self
    }

    /// Unassign the provided channel from the extremes detector of this `Filter`.
    pub fn unassign_extremes_detector_channel<MT, S, MODE, PT>(
        mut self,
        channel: &Transceiver<'_, '_, T, MT, S, MODE, PT>,
    ) -> Self
    where
        MT: TransceiverMarker + NextChannelForInstance<T>,
        S: PinSet,
        MODE: ChannelMode,
        PT: PowerState,
    {
        Self::clear_extremes_detector_channel(channel.index() as u8);
        self
    }
    /// Enables or disables regular data overrun interrupts.
    pub(crate) fn set_regular_overrun_interrupt(enabled: bool) {
        T::regs()
            .flt(M::CHANNEL.index())
            .cr2()
            .modify(|w| w.set_rovrie(enabled));
    }

    /// Enables or disables injected data overrun interrupts.
    pub(crate) fn set_injected_overrun_interrupt(enabled: bool) {
        T::regs()
            .flt(M::CHANNEL.index())
            .cr2()
            .modify(|w| w.set_jovrie(enabled));
    }

    /// Enables or disables regular end-of-conversion interrupts.
    pub(crate) fn set_regular_end_of_conversion_interrupt(enabled: bool) {
        T::regs()
            .flt(M::CHANNEL.index())
            .cr2()
            .modify(|w| w.set_reocie(enabled));
    }

    /// Enables or disables injected end-of-conversion interrupts.
    pub(crate) fn set_injected_end_of_conversion_interrupt(enabled: bool) {
        T::regs()
            .flt(M::CHANNEL.index())
            .cr2()
            .modify(|w| w.set_jeocie(enabled));
    }

    /// Enables or disables analog watchdog interrupts.
    fn set_analog_watchdog_interrupt(&mut self, enabled: bool) {
        T::regs().flt(M::CHANNEL.index()).cr2().modify(|w| w.set_awdie(enabled));
    }
}

/// Sign-extends a 24bit LSB number to a i32
fn sign_extend_24(x: u32) -> i32 {
    ((x << 8) as i32) >> 8
}

impl<'reg, 'inj, T, M> Filter<'reg, 'inj, T, M, Enabled>
where
    T: Instance + FilterInterrupt<M>,
    M: FilterMarker,
{
    /// Disable the filter
    pub fn disable(self) -> Filter<'reg, 'inj, T, M, Disabled> {
        T::regs().flt(M::CHANNEL.index()).cr1().modify(|w| w.set_dfen(false));
        Filter {
            _instance_marker: PhantomData,
            _filter_marker: PhantomData,
            _powerstate_marker: PhantomData,
            regular: self.regular,
            injected: self.injected,
        }
    }

    /// Reassign the provided `Transceiver` as the regular conversion input for this `Filter`
    pub fn reassign_regular_transceiver<'new_reg>(
        self,
        channel: &'new_reg dyn TransceiverTrait<T, Enabled>,
    ) -> Filter<'new_reg, 'inj, T, M, Enabled> {
        self.assign_regular_transceiver(channel)
    }

    /// Reassign transceivers to the injected conversion group of this `Filter`
    ///
    /// Note: Overwrites all assignments.
    pub fn reassign_injected_transceivers<'new_inj, const N: usize>(
        self,
        transceivers: [&'new_inj dyn TransceiverTrait<T, Enabled>; N],
    ) -> Filter<'reg, 'new_inj, T, M, Enabled>
    where
        [(); N]: NonEmpty,
    {
        self.assign_injected_transceivers(transceivers)
    }
}

impl<T, F> interrupt::typelevel::Handler<<T as FilterInterrupt<F>>::Interrupt> for InterruptHandler<T, F>
where
    T: Instance + FilterInterrupt<F>,
    F: FilterMarker,
{
    unsafe fn on_interrupt() {
        if Filter::<T, F, Enabled>::end_of_injected_conversion() {
            Filter::<T, F, Enabled>::set_injected_end_of_conversion_interrupt(false);
            <T as FilterInterrupt<F>>::state().injected_waker.wake();
        }
        if Filter::<T, F, Enabled>::end_of_regular_conversion() {
            Filter::<T, F, Enabled>::set_regular_end_of_conversion_interrupt(false);
            <T as FilterInterrupt<F>>::state().regular_waker.wake();
        }
    }
}

struct DfsdmInner<T>
where
    T: Instance,
{
    _instance_marker: PhantomData<T>,
}

/// Only when enabled
impl<'a, 'd, T, M, S, MODE> Transceiver<'a, 'd, T, M, S, MODE, Enabled>
where
    T: Instance,
    M: TransceiverMarker + NextChannelForInstance<T>,
    S: PinSet,
    MODE: ChannelMode,
{
    /// Disables the channel
    pub fn disable(self) -> Transceiver<'a, 'd, T, M, S, MODE, Disabled> {
        T::regs().ch(M::CHANNEL.index()).cfgr1().modify(|w| w.set_chen(false));

        Transceiver {
            common: self.common,
            _instance_marker: PhantomData,
            _transceiver_marker: PhantomData,
            _pinset_marker: PhantomData,
            _datasource_marker: PhantomData,
            _powerstate_marker: PhantomData,
        }
    }
}

/// Configuration for Transceiver
pub struct TransceiverConfig {
    /// Amount of right-shifts the data shall experience
    pub data_right_shift: config_types::UInt<5, u8>,
    pub analog_watchdog_filter_config: config_types::AnalogWatchdogFilterConfiguration,
}

impl Default for TransceiverConfig {
    fn default() -> Self {
        Self {
            data_right_shift: config_types::UInt::new(0),
            analog_watchdog_filter_config: config_types::AnalogWatchdogFilterConfiguration::Bypass,
        }
    }
}

/// Configuration for Transceiver, also changable when enables
pub struct TransceiverConfigOnline {
    /// Clock absende detection
    pub enable_clock_absence_detection: bool,
    /// Short-circuit detection + threshold
    pub short_circuit_detection_config: config_types::ShortCircuitDetectionConfig,
    /// Offset
    pub offset: u32,
    /// Breaksignal assignment
    pub break_signals: config_types::BreakSignals,
}

impl Default for TransceiverConfigOnline {
    fn default() -> Self {
        Self {
            enable_clock_absence_detection: false,
            short_circuit_detection_config: config_types::ShortCircuitDetectionConfig::Disabled,
            offset: 0,
            break_signals: config_types::BreakSignals::empty(),
        }
    }
}

/// Only when disabled
impl<'a, 'd, T, M, S, MODE> Transceiver<'a, 'd, T, M, S, MODE, Disabled>
where
    T: Instance,
    M: TransceiverMarker + NextChannelForInstance<T>,
    S: PinSet,
    MODE: ChannelMode,
{
    /// Enables the channel
    pub fn enable(self) -> Transceiver<'a, 'd, T, M, S, MODE, Enabled> {
        T::regs().ch(M::CHANNEL.index()).cfgr1().modify(|w| w.set_chen(true));

        Transceiver {
            common: self.common,
            _instance_marker: PhantomData,
            _transceiver_marker: PhantomData,
            _pinset_marker: PhantomData,
            _datasource_marker: PhantomData,
            _powerstate_marker: PhantomData,
        }
    }

    /// Configure the transceiver
    pub fn configure(
        mut self,
        config: &TransceiverConfig,
        online_config: &TransceiverConfigOnline,
    ) -> Transceiver<'a, 'd, T, M, S, MODE, Disabled> {
        self.set_data_right_shift(config.data_right_shift);

        self.select_analog_watchdog_filter_order(config.analog_watchdog_filter_config.into());
        self.select_analog_watchdog_osr(config.analog_watchdog_filter_config.into());

        self.configure_online(online_config);
        self
    }

    /// Set channel right shift factor
    fn set_data_right_shift(&mut self, shift: config_types::UInt<5, u8>) {
        T::regs()
            .ch(M::CHANNEL.index())
            .cfgr2()
            .modify(|w| w.set_dtrbs(shift.into()));
    }

    /// Set the filterorder of the analog watchdog
    fn select_analog_watchdog_filter_order(&mut self, filter_order: config_types::AnalogWatchdogFilterOrder) {
        T::regs()
            .ch(M::CHANNEL.index())
            .awscdr()
            .modify(|w| w.set_awford(filter_order as u8));
    }

    /// Set the oversampling ratio of the analog watchdog filter
    fn select_analog_watchdog_osr(&mut self, osr: config_types::AnalogWatchdogOsr) {
        T::regs()
            .ch(M::CHANNEL.index())
            .awscdr()
            .modify(|w| w.set_awfosr(osr.into()));
    }
}

impl<'a, 'd, T, M, S, P> Transceiver<'a, 'd, T, M, S, ParallelDmaMode, P>
where
    T: Instance,
    M: TransceiverMarker + NextChannelForInstance<T>,
    S: PinSet,
    P: PowerState,
{
    /// Get direct pointer tothe DATINR register for dma mem2mem use
    pub fn get_datinr_as_ptr(&self) -> *mut u32 {
        T::regs().ch(M::CHANNEL.index()).datinr().as_ptr() as *mut u32
    }

    //TODO REF SOURCE ME MTUABLE TO TO ACCESS RULES ISNT YET
    /// Get reference tothe DATINR register for dma mem2mem use
    pub fn get_datinr_as_ref(&self) -> &mut u32 {
        let ptr = T::regs().ch(M::CHANNEL.index()).datinr().as_ptr() as *mut u32;

        // Safety: The pointer points to a valid memory-mapped register
        // address, and we hold a &mut self, guaranteeing exclusive access
        // for the lifetime of the returned reference.
        unsafe { &mut *ptr }
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
impl<'a, 'd, T, M, S, MODE, P> Transceiver<'a, 'd, T, M, S, MODE, P>
where
    T: Instance,
    M: TransceiverMarker + NextChannelForInstance<T>,
    S: PinSet,
    MODE: ChannelMode,
    P: PowerState,
{
    /// Configure parameters changable in Enabled mode. May be used to reconfigure while running.
    pub fn configure_online(&mut self, config: &TransceiverConfigOnline) {
        self.set_clock_absence_detector(config.enable_clock_absence_detection);

        let (scd_en, scd_ths) = match config.short_circuit_detection_config {
            config_types::ShortCircuitDetectionConfig::Disabled => (false, 0),
            config_types::ShortCircuitDetectionConfig::Enabled(threshold) => (true, threshold),
        };

        self.set_short_circuit_detector(scd_en);
        self.set_shortcircuit_threshold(scd_ths);
        self.set_offset(config.offset);
        self.assign_breakn(config.break_signals);
    }
    /// TODO REMOVE
    /// Configures the neighbor channel.
    /// Notice: No extra arguments! Rust knows `T` from `self`.
    // pub fn do_something_with_neighbor(&self) {
    //     // The compiler knows that `M::Next` is valid for this specific instance `T`.
    //     let next_idx = <M::Next as TransceiverMarker>::CHANNEL.index();
    // }

    /// Enable/Disable clock-absence-detector
    fn set_clock_absence_detector(&mut self, enabled: bool) {
        T::regs()
            .ch(M::CHANNEL.index())
            .cfgr1()
            .modify(|w| w.set_ckaben(enabled));
    }

    /// Enable/Disable short-circuit-detector
    fn set_short_circuit_detector(&mut self, enabled: bool) {
        T::regs()
            .ch(M::CHANNEL.index())
            .cfgr1()
            .modify(|w| w.set_scden(enabled));
    }

    /// Set channel offset
    fn set_offset(&mut self, offset: u32) {
        T::regs()
            .ch(M::CHANNEL.index())
            .cfgr2()
            .modify(|w| w.set_offset(offset));
    }

    /// Assign to break-signals
    fn assign_breakn(&mut self, signals: config_types::BreakSignals) {
        T::regs()
            .ch(M::CHANNEL.index())
            .awscdr()
            .modify(|w| w.set_bkscd(signals.bits()));
    }

    /// Set short-circuit-detector-threshold
    fn set_shortcircuit_threshold(&mut self, threshold: u8) {
        T::regs()
            .ch(M::CHANNEL.index())
            .awscdr()
            .modify(|w| w.set_scdt(threshold));
    }
}

impl<'a, 'd, T, M, S, MODE, P> Transceiver<'a, 'd, T, M, S, MODE, P>
where
    T: Instance + HasDelay,
    M: TransceiverMarker + NextChannelForInstance<T>,
    S: PinSet,
    MODE: ChannelMode,
    P: PowerState,
{
    /// Configure to skip the next [`skips`] pulses
    pub fn skip_pulses(&mut self, skips: config_types::UInt<6, u8>) {
        self.set_pulseskips(skips);
    }

    /// Set pulse skips
    fn set_pulseskips(&mut self, skips: config_types::UInt<6, u8>) {
        T::regs()
            .ch(M::CHANNEL.index())
            .dlyr()
            .modify(|w| w.set_plsskp(skips.into()));
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

/// Selectors for a 2-channel DFSDM instance.
///
/// NOTE: must never gain a `Drop` impl - partial moves out of it must remain legal.
pub struct ChannelSelectors2<T: Instance> {
    /// Selector for channel 0
    pub ch0: Sel<T, Tcv0>,
    /// Selector for channel 1
    pub ch1: Sel<T, Tcv1>,
}

impl<T: Instance> ChannelSelectors2<T> {
    pub(crate) fn new() -> Self {
        Self {
            ch0: Sel { _m: PhantomData },
            ch1: Sel { _m: PhantomData },
        }
    }
}

/// Selectors for a 4-channel DFSDM instance.
///
/// NOTE: must never gain a `Drop` impl - partial moves out of it must remain legal.
pub struct ChannelSelectors4<T: Instance> {
    /// Selector for channel 0
    pub ch0: Sel<T, Tcv0>,
    /// Selector for channel 1
    pub ch1: Sel<T, Tcv1>,
    /// Selector for channel 2
    pub ch2: Sel<T, Tcv2>,
    /// Selector for channel 3
    pub ch3: Sel<T, Tcv3>,
}

impl<T: Instance> ChannelSelectors4<T> {
    pub(crate) fn new() -> Self {
        Self {
            ch0: Sel { _m: PhantomData },
            ch1: Sel { _m: PhantomData },
            ch2: Sel { _m: PhantomData },
            ch3: Sel { _m: PhantomData },
        }
    }
}

/// Selectors for all channels of an 8-channel DFSDM instance.
///
/// NOTE: must never gain a `Drop` impl - partial moves out of it
/// (`s.ch0.datin(..)`) must remain legal.
pub struct ChannelSelectors8<T: Instance> {
    /// Selector for channel 0
    pub ch0: Sel<T, Tcv0>,
    /// Selector for channel 1
    pub ch1: Sel<T, Tcv1>,
    /// Selector for channel 2
    pub ch2: Sel<T, Tcv2>,
    /// Selector for channel 3
    pub ch3: Sel<T, Tcv3>,
    /// Selector for channel 4
    pub ch4: Sel<T, Tcv4>,
    /// Selector for channel 5
    pub ch5: Sel<T, Tcv5>,
    /// Selector for channel 6
    pub ch6: Sel<T, Tcv6>,
    /// Selector for channel 7
    pub ch7: Sel<T, Tcv7>,
}

impl<T: Instance> ChannelSelectors8<T> {
    pub(crate) fn new() -> Self {
        Self {
            ch0: Sel { _m: PhantomData },
            ch1: Sel { _m: PhantomData },
            ch2: Sel { _m: PhantomData },
            ch3: Sel { _m: PhantomData },
            ch4: Sel { _m: PhantomData },
            ch5: Sel { _m: PhantomData },
            ch6: Sel { _m: PhantomData },
            ch7: Sel { _m: PhantomData },
        }
    }
}

/// [`TransceiverBuilder`]
///
///
///
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
    /// Parallel input from ADC writes to CHyDATINR (DATMPX=2).
    /// No CKOUT, no pins needed. Serial pins declared on this channel
    /// are disconnected (the builder's Flexes drop here - they're unused
    /// in this mode).
    pub fn new_parallel_adc<'a, 'd>(
        mut self,
        common: &'a DfsdmCommon<'d, T, Enabled>,
    ) -> Transceiver<'a, 'd, T, M, S, ParallelAdcMode, Disabled>
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
            _datasource_marker: PhantomData,
            _powerstate_marker: PhantomData,
        }
    }

    /// Parallel input from CPU/DMA writes to CHyDATINR (DATMPX=2).
    /// No CKOUT, no pins needed. Serial pins declared on this channel
    /// are disconnected (the builder's Flexes drop here - they're unused
    /// in this mode).
    pub fn new_parallel_dma<'a, 'd>(
        mut self,
        common: &'a DfsdmCommon<'d, T, Enabled>,
        packing_mode: config_types::DataPackingModeReduced,
    ) -> Transceiver<'a, 'd, T, M, S, ParallelDmaMode, Disabled> {
        self.select_channel_input(config_types::ChannelInput::Same);
        self.select_data_mux_input(config_types::InputDataMux::InternalRegisterWrite);
        self.set_data_packing_mode(packing_mode.into());
        Transceiver {
            common,
            _instance_marker: PhantomData,
            _transceiver_marker: PhantomData,
            _pinset_marker: PhantomData,
            _datasource_marker: PhantomData,
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
    pub fn new_parallel_dma_dual<'a, 'd, MN, SNN>(
        mut self,
        common: &'a DfsdmCommon<'d, T, Enabled>,
        mut neighbor: TransceiverBuilder<T, MN, C, SN, SNN>,
    ) -> (
        Transceiver<'a, 'd, T, M, S, ParallelDmaMode, Disabled>,
        Transceiver<'a, 'd, T, MN, SN, ParallelDmaMode, Disabled>,
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
                _datasource_marker: PhantomData,
                _powerstate_marker: PhantomData,
            },
            Transceiver {
                common,
                _instance_marker: PhantomData,
                _transceiver_marker: PhantomData,
                _pinset_marker: PhantomData,
                _datasource_marker: PhantomData,
                _powerstate_marker: PhantomData,
            },
        )
    }

    /// Parallel input from ADC writes to CHyDATINR (DATMPX=2).
    /// No CKOUT, no pins needed. Serial pins declared on this channel
    /// are disconnected (the builder's Flexes drop here - they're unused
    /// in this mode).
    pub fn new_manchester<'a, 'd>(
        mut self,
        common: &'a DfsdmCommon<'d, T, Enabled>,
        mode: config_types::ManchesterMode,
    ) -> Transceiver<'a, 'd, T, M, S, ManchesterMode, Disabled>
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
            _datasource_marker: PhantomData,
            _powerstate_marker: PhantomData,
        }
    }

    ///Same as [`Self::new_manchester`] but using neighbors pins
    pub fn new_manchester_neighbor<'a, 'd>(
        mut self,
        common: &'a DfsdmCommon<'d, T, Enabled>,
        mode: config_types::ManchesterMode,
    ) -> Result<Transceiver<'a, 'd, T, M, DataOnly, ManchesterNeighborMode, Disabled>, Error>
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
            _datasource_marker: PhantomData,
            _powerstate_marker: PhantomData,
        })
    }

    ///TODO new_spi_ext description
    pub fn new_spi_ext<'a, 'd>(
        mut self,
        common: &'a DfsdmCommon<'d, T, Enabled>,
        mode: config_types::SpiMode,
    ) -> Transceiver<'a, 'd, T, M, S, SpiExtMode, Disabled>
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
            _datasource_marker: PhantomData,
            _powerstate_marker: PhantomData,
        }
    }

    ///Same as [`Self::new_spi_ext`] but using neighbors pins
    pub fn new_spi_ext_neighbor<'a, 'd>(
        mut self,
        common: &'a DfsdmCommon<'d, T, Enabled>,
        mode: config_types::SpiMode,
    ) -> Result<Transceiver<'a, 'd, T, M, DataClk, SpiExtNeighborMode, Disabled>, Error>
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
            _datasource_marker: PhantomData,
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
    ///TODO new_spi_int description
    pub fn new_spi_int<'a, 'd>(
        mut self,
        common: &'a DfsdmCommon<'d, T, Enabled>,
        mode: config_types::InternalSpiMode,
    ) -> Transceiver<'a, 'd, T, M, S, SpiCkoutMode, Disabled>
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
            _datasource_marker: PhantomData,
            _powerstate_marker: PhantomData,
        }
    }

    ///Same as [`Self::new_spi_int`] but using neighbors pins
    pub fn new_spi_int_neighbor<'a, 'd>(
        mut self,
        common: &'a DfsdmCommon<'d, T, Enabled>,
        mode: config_types::InternalSpiMode,
    ) -> Result<Transceiver<'a, 'd, T, M, DataOnly, SpiCkoutNeighborMode, Disabled>, Error>
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
            _datasource_marker: PhantomData,
            _powerstate_marker: PhantomData,
        })
    }
}

/// Used to configure a [`Transceiver`].
// pub struct TransceiverBuilder<T, M, C>
// where
//     T: Instance,
//     M: TransceiverMarker,
//     C: ClockOutputMode,
// {
//     _dfsdm_marker: PhantomData<T>,
//     _clockmode_marker: PhantomData<C>,
//     _transceiver_marker: PhantomData<M>,
// }

// impl<T, M, C> TransceiverBuilder<T, M, C>
// where
//     T: Instance,
//     M: TransceiverMarker,
//     C: ClockOutputMode,
// {
//     pub(crate) fn new(_dfsdm: &Dfsdm<T, C>) -> Self {
//         //TODO Remove dfsdm reference maybe
//         Self {
//             _dfsdm_marker: PhantomData,
//             _clockmode_marker: PhantomData,
//             _transceiver_marker: PhantomData,
//         }
//     }

//     /// Configure [`Transceiver`]. as receiving data from its internal register
//     pub fn new_parallel(self) -> Transceiver<T, M, InternalSource> {
//         // TODO
//         Transceiver {
//             _instance_marker: PhantomData,
//             _transceiver_marker: PhantomData,
//             _datasource_marker: PhantomData,
//         }
//     }

//     /// Configure [`Transceiver`]. as receiving data from an externally clocked bus
//     pub fn new_clk_ext(
//         self,
//         ckin: Peri<if_afio!(impl CkinPin<T, M, A>)>,
//         datin: Peri<if_afio!(impl DatinPin<T, M, A>)>,
//     ) -> Transceiver<T, M, ExternalSource> {
//         set_as_af!(ckin, AfType::input(Pull::None));
//         set_as_af!(datin, AfType::input(Pull::None));

//         Transceiver {
//             _instance_marker: PhantomData,
//             _transceiver_marker: PhantomData,
//             _datasource_marker: PhantomData,
//         }
//     }

//     /// Configure [`Transceiver`]. as receiving data from a manchester-coded signal
//     pub fn new_manchester(self, datin: Peri<if_afio!(impl DatinPin<T, M, A>)>) -> Transceiver<T, M, ExternalSource> {
//         set_as_af!(datin, AfType::input(Pull::None));

//         Transceiver {
//             _instance_marker: PhantomData,
//             _transceiver_marker: PhantomData,
//             _datasource_marker: PhantomData,
//         }
//     }
// }

// impl<T, M> TransceiverBuilder<T, M, OutputEnabled>
// where
//     T: Instance,
//     M: TransceiverMarker,
// {
//     /// Configure transceiver as receiving data from a synchronously clocked bus clocked by the controller
//     pub fn new_clk_int(self, datin: Peri<if_afio!(impl DatinPin<T, M, A>)>) -> Transceiver<T, M, ExternalSource>
//     where
//         T: Instance,
//         M: TransceiverMarker,
//     {
//         set_as_af!(datin, AfType::input(Pull::None));

//         Transceiver {
//             _instance_marker: PhantomData,
//             _transceiver_marker: PhantomData,
//             _datasource_marker: PhantomData,
//         }
//     }
// }

// =============================================================================
// Splitting
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
        unsafe {
            (
                ptr::read(&this._peri),
                ptr::read(&this._ckout),
                ptr::read(&this.datin_slots),
                ptr::read(&this.ckin_slots),
            )
        }
    }

    /// Enables or disables clock absence interrupts.
    fn set_clock_absence_interrupt(&mut self, enabled: bool) {
        T::regs().flt(0).cr2().modify(|w| w.set_ckabie(enabled));
    }

    /// Enables or disables short-circuit detector interrupts.
    fn set_short_circuit_detector_interrupt(&mut self, enabled: bool) {
        T::regs().flt(0).cr2().modify(|w| w.set_scdie(enabled));
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

pub struct DfsdmSplit2Ch1Flt<'reg, 'inj, 'd, T, C, S0, S1>
where
    T: Instance + FilterInterrupt<Flt0>,
    C: ClockOutputMode,
    S0: PinSet,
    S1: PinSet,
{
    pub common: DfsdmCommon<'d, T, Enabled>,
    pub ch0: TransceiverBuilder<T, Tcv0, C, S0, S1>, // neighbor = ch1
    pub ch1: TransceiverBuilder<T, Tcv1, C, S1, S0>, // neighbor = ch0 (wrap!)
    pub flt0: Filter<'reg, 'inj, T, Flt0, Disabled>,
}

pub struct DfsdmSplit8Ch8Flt<'reg, 'inj, 'd, T, C, S0, S1, S2, S3, S4, S5, S6, S7>
where
    T: Instance + Flt8Ready,
    C: ClockOutputMode,
    S0: PinSet,
    S1: PinSet,
    S2: PinSet,
    S3: PinSet,
    S4: PinSet,
    S5: PinSet,
    S6: PinSet,
    S7: PinSet,
{
    pub common: DfsdmCommon<'d, T, Enabled>,
    pub ch0: TransceiverBuilder<T, Tcv0, C, S0, S1>,
    pub ch1: TransceiverBuilder<T, Tcv1, C, S1, S2>,
    pub ch2: TransceiverBuilder<T, Tcv2, C, S2, S3>,
    pub ch3: TransceiverBuilder<T, Tcv3, C, S3, S4>,
    pub ch4: TransceiverBuilder<T, Tcv4, C, S4, S5>,
    pub ch5: TransceiverBuilder<T, Tcv5, C, S5, S6>,
    pub ch6: TransceiverBuilder<T, Tcv6, C, S6, S7>,
    pub ch7: TransceiverBuilder<T, Tcv7, C, S7, S0>, // neighbor = ch0 (wrap!)
    pub flt0: Filter<'reg, 'inj, T, Flt0, Disabled>,
    pub flt1: Filter<'reg, 'inj, T, Flt1, Disabled>,
    pub flt2: Filter<'reg, 'inj, T, Flt2, Disabled>,
    pub flt3: Filter<'reg, 'inj, T, Flt3, Disabled>,
    pub flt4: Filter<'reg, 'inj, T, Flt4, Disabled>,
    pub flt5: Filter<'reg, 'inj, T, Flt5, Disabled>,
    pub flt6: Filter<'reg, 'inj, T, Flt6, Disabled>,
    pub flt7: Filter<'reg, 'inj, T, Flt7, Disabled>,
}

pub struct DfsdmSplit8Ch4Flt<'reg, 'inj, 'd, T, C, S0, S1, S2, S3, S4, S5, S6, S7>
where
    T: Instance + Flt4Ready,
    C: ClockOutputMode,
    S0: PinSet,
    S1: PinSet,
    S2: PinSet,
    S3: PinSet,
    S4: PinSet,
    S5: PinSet,
    S6: PinSet,
    S7: PinSet,
{
    pub common: DfsdmCommon<'d, T, Enabled>,
    pub ch0: TransceiverBuilder<T, Tcv0, C, S0, S1>,
    pub ch1: TransceiverBuilder<T, Tcv1, C, S1, S2>,
    pub ch2: TransceiverBuilder<T, Tcv2, C, S2, S3>,
    pub ch3: TransceiverBuilder<T, Tcv3, C, S3, S4>,
    pub ch4: TransceiverBuilder<T, Tcv4, C, S4, S5>,
    pub ch5: TransceiverBuilder<T, Tcv5, C, S5, S6>,
    pub ch6: TransceiverBuilder<T, Tcv6, C, S6, S7>,
    pub ch7: TransceiverBuilder<T, Tcv7, C, S7, S0>, // neighbor = ch0 (wrap!)
    pub flt0: Filter<'reg, 'inj, T, Flt0, Disabled>,
    pub flt1: Filter<'reg, 'inj, T, Flt1, Disabled>,
    pub flt2: Filter<'reg, 'inj, T, Flt2, Disabled>,
    pub flt3: Filter<'reg, 'inj, T, Flt3, Disabled>,
}

/// Dfsdm
///
///

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

// impl<'d, T: Instance, C: ClockOutputMode> Dfsdm<'d, T, C> {
//     pub fn split(self) -> T::Split<C> {
//         T::split(self)
//     }
// }

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

/// Implemented for the tuple a `configure_pins` closure returns.
/// The arity *is* the channel-count check: `(C0, C1)` only impls for
/// `Tcv2` instances, the 8-tuple only for `Tcv8`.
#[diagnostic::on_unimplemented(
    message = "the closure must return one pin token per channel of `{T}`",
    label = "tuple length doesn't match `{T}`'s transceiver count",
    note = "check `{T}`'s channel count and return a tuple of that length, one token per `creator.chN`"
)]
pub trait ChannelCfgTuple<'reg, 'inj, 'd, T: Instance, C: ClockOutputMode> {
    /// The fully-wired split (neighbor pin-sets already correct).
    type Split;

    /// Split helper-function
    fn split_parts(self, common: DfsdmCommon<'d, T, Enabled>) -> Self::Split;
}

impl<'reg, 'inj, 'd, T, C, C0, C1> ChannelCfgTuple<'reg, 'inj, 'd, T, C> for (C0, C1)
where
    T: Instance<Transceivers = capability::Tcv2, Filters = capability::Flt1> + FilterInterrupt<Flt0>,
    C: ClockOutputMode,
    C0: ChannelCfg<'d, T, Tcv0>,
    C1: ChannelCfg<'d, T, Tcv1>,
{
    type Split = DfsdmSplit2Ch1Flt<
        'reg,
        'inj,
        'd,
        T,
        C,
        <C0 as ChannelCfg<'d, T, Tcv0>>::Presence,
        <C1 as ChannelCfg<'d, T, Tcv1>>::Presence,
    >;

    fn split_parts(self, mut common: DfsdmCommon<'d, T, Enabled>) -> Self::Split {
        let (d0, k0) = self.0.into_parts();
        let (d1, k1) = self.1.into_parts();

        common.insert_pin(0, PinKind::Datin, C0::Presence::extract_datin(d0));
        common.insert_pin(0, PinKind::Ckin, C0::Presence::extract_ckin(k0));
        common.insert_pin(1, PinKind::Datin, C1::Presence::extract_datin(d1));
        common.insert_pin(1, PinKind::Ckin, C1::Presence::extract_ckin(k1));

        DfsdmSplit2Ch1Flt {
            common,
            ch0: TransceiverBuilder { _m: PhantomData },
            ch1: TransceiverBuilder { _m: PhantomData },
            flt0: Filter::new_disabled(),
        }
    }
}

/// Builds the actual split struct from 8 already-extracted pin pairs.
/// Implemented separately for each filter-count capability.
pub trait Flt8SplitBuild<
    'reg,
    'inj,
    'd,
    T: Instance,
    C: ClockOutputMode,
    S0: PinSet,
    S1: PinSet,
    S2: PinSet,
    S3: PinSet,
    S4: PinSet,
    S5: PinSet,
    S6: PinSet,
    S7: PinSet,
>
{
    type Out;
    fn build(
        common: DfsdmCommon<'d, T, Enabled>,
        d0: S0::Datin<'d>,
        k0: S0::Ckin<'d>,
        d1: S1::Datin<'d>,
        k1: S1::Ckin<'d>,
        d2: S2::Datin<'d>,
        k2: S2::Ckin<'d>,
        d3: S3::Datin<'d>,
        k3: S3::Ckin<'d>,
        d4: S4::Datin<'d>,
        k4: S4::Ckin<'d>,
        d5: S5::Datin<'d>,
        k5: S5::Ckin<'d>,
        d6: S6::Datin<'d>,
        k6: S6::Ckin<'d>,
        d7: S7::Datin<'d>,
        k7: S7::Ckin<'d>,
    ) -> Self::Out;
}

impl<'reg, 'inj, 'd, T, C, S0, S1, S2, S3, S4, S5, S6, S7>
    Flt8SplitBuild<'reg, 'inj, 'd, T, C, S0, S1, S2, S3, S4, S5, S6, S7> for capability::Flt8
where
    T: Instance<Transceivers = capability::Tcv8, Filters = capability::Flt8>
        + FilterInterrupt<Flt0>
        + FilterInterrupt<Flt1>
        + FilterInterrupt<Flt2>
        + FilterInterrupt<Flt3>
        + FilterInterrupt<Flt4>
        + FilterInterrupt<Flt5>
        + FilterInterrupt<Flt6>
        + FilterInterrupt<Flt7>,
    C: ClockOutputMode,
    S0: PinSet,
    S1: PinSet,
    S2: PinSet,
    S3: PinSet,
    S4: PinSet,
    S5: PinSet,
    S6: PinSet,
    S7: PinSet,
{
    type Out = DfsdmSplit8Ch8Flt<'reg, 'inj, 'd, T, C, S0, S1, S2, S3, S4, S5, S6, S7>;

    fn build(
        mut common: DfsdmCommon<'d, T, Enabled>,
        d0: S0::Datin<'d>,
        k0: S0::Ckin<'d>,
        d1: S1::Datin<'d>,
        k1: S1::Ckin<'d>,
        d2: S2::Datin<'d>,
        k2: S2::Ckin<'d>,
        d3: S3::Datin<'d>,
        k3: S3::Ckin<'d>,
        d4: S4::Datin<'d>,
        k4: S4::Ckin<'d>,
        d5: S5::Datin<'d>,
        k5: S5::Ckin<'d>,
        d6: S6::Datin<'d>,
        k6: S6::Ckin<'d>,
        d7: S7::Datin<'d>,
        k7: S7::Ckin<'d>,
    ) -> Self::Out {
        common.insert_pin(0, PinKind::Datin, S0::extract_datin(d0));
        common.insert_pin(0, PinKind::Ckin, S0::extract_ckin(k0));
        common.insert_pin(1, PinKind::Datin, S1::extract_datin(d1));
        common.insert_pin(1, PinKind::Ckin, S1::extract_ckin(k1));
        common.insert_pin(2, PinKind::Datin, S2::extract_datin(d2));
        common.insert_pin(2, PinKind::Ckin, S2::extract_ckin(k2));
        common.insert_pin(3, PinKind::Datin, S3::extract_datin(d3));
        common.insert_pin(3, PinKind::Ckin, S3::extract_ckin(k3));
        common.insert_pin(4, PinKind::Datin, S4::extract_datin(d4));
        common.insert_pin(4, PinKind::Ckin, S4::extract_ckin(k4));
        common.insert_pin(5, PinKind::Datin, S5::extract_datin(d5));
        common.insert_pin(5, PinKind::Ckin, S5::extract_ckin(k5));
        common.insert_pin(6, PinKind::Datin, S6::extract_datin(d6));
        common.insert_pin(6, PinKind::Ckin, S6::extract_ckin(k6));
        common.insert_pin(7, PinKind::Datin, S7::extract_datin(d7));
        common.insert_pin(7, PinKind::Ckin, S7::extract_ckin(k7));

        DfsdmSplit8Ch8Flt {
            common,
            ch0: TransceiverBuilder { _m: PhantomData },
            ch1: TransceiverBuilder { _m: PhantomData },
            ch2: TransceiverBuilder { _m: PhantomData },
            ch3: TransceiverBuilder { _m: PhantomData },
            ch4: TransceiverBuilder { _m: PhantomData },
            ch5: TransceiverBuilder { _m: PhantomData },
            ch6: TransceiverBuilder { _m: PhantomData },
            ch7: TransceiverBuilder { _m: PhantomData },
            flt0: Filter::new_disabled(),
            flt1: Filter::new_disabled(),
            flt2: Filter::new_disabled(),
            flt3: Filter::new_disabled(),
            flt4: Filter::new_disabled(),
            flt5: Filter::new_disabled(),
            flt6: Filter::new_disabled(),
            flt7: Filter::new_disabled(),
        }
    }
}

impl<'reg, 'inj, 'd, T, C, S0, S1, S2, S3, S4, S5, S6, S7>
    Flt8SplitBuild<'reg, 'inj, 'd, T, C, S0, S1, S2, S3, S4, S5, S6, S7> for capability::Flt4
where
    T: Instance<Transceivers = capability::Tcv8, Filters = capability::Flt4>
        + FilterInterrupt<Flt0>
        + FilterInterrupt<Flt1>
        + FilterInterrupt<Flt2>
        + FilterInterrupt<Flt3>,
    C: ClockOutputMode,
    S0: PinSet,
    S1: PinSet,
    S2: PinSet,
    S3: PinSet,
    S4: PinSet,
    S5: PinSet,
    S6: PinSet,
    S7: PinSet,
{
    type Out = DfsdmSplit8Ch4Flt<'reg, 'inj, 'd, T, C, S0, S1, S2, S3, S4, S5, S6, S7>;

    fn build(
        common: DfsdmCommon<'d, T, Enabled>,
        d0: S0::Datin<'d>,
        k0: S0::Ckin<'d>,
        d1: S1::Datin<'d>,
        k1: S1::Ckin<'d>,
        d2: S2::Datin<'d>,
        k2: S2::Ckin<'d>,
        d3: S3::Datin<'d>,
        k3: S3::Ckin<'d>,
        d4: S4::Datin<'d>,
        k4: S4::Ckin<'d>,
        d5: S5::Datin<'d>,
        k5: S5::Ckin<'d>,
        d6: S6::Datin<'d>,
        k6: S6::Ckin<'d>,
        d7: S7::Datin<'d>,
        k7: S7::Ckin<'d>,
    ) -> Self::Out {
        DfsdmSplit8Ch4Flt {
            common,
            ch0: TransceiverBuilder { _m: PhantomData },
            ch1: TransceiverBuilder { _m: PhantomData },
            ch2: TransceiverBuilder { _m: PhantomData },
            ch3: TransceiverBuilder { _m: PhantomData },
            ch4: TransceiverBuilder { _m: PhantomData },
            ch5: TransceiverBuilder { _m: PhantomData },
            ch6: TransceiverBuilder { _m: PhantomData },
            ch7: TransceiverBuilder { _m: PhantomData },
            flt0: Filter::new_disabled(),
            flt1: Filter::new_disabled(),
            flt2: Filter::new_disabled(),
            flt3: Filter::new_disabled(),
        }
    }
}

impl<'reg, 'inj, 'd, T, C, C0, C1, C2, C3, C4, C5, C6, C7> ChannelCfgTuple<'reg, 'inj, 'd, T, C>
    for (C0, C1, C2, C3, C4, C5, C6, C7)
where
    T: Instance<Transceivers = capability::Tcv8>,
    C: ClockOutputMode,
    C0: ChannelCfg<'d, T, Tcv0>,
    C1: ChannelCfg<'d, T, Tcv1>,
    C2: ChannelCfg<'d, T, Tcv2>,
    C3: ChannelCfg<'d, T, Tcv3>,
    C4: ChannelCfg<'d, T, Tcv4>,
    C5: ChannelCfg<'d, T, Tcv5>,
    C6: ChannelCfg<'d, T, Tcv6>,
    C7: ChannelCfg<'d, T, Tcv7>,
    T::Filters: Flt8SplitBuild<
            'reg,
            'inj,
            'd,
            T,
            C,
            <C0 as ChannelCfg<'d, T, Tcv0>>::Presence,
            <C1 as ChannelCfg<'d, T, Tcv1>>::Presence,
            <C2 as ChannelCfg<'d, T, Tcv2>>::Presence,
            <C3 as ChannelCfg<'d, T, Tcv3>>::Presence,
            <C4 as ChannelCfg<'d, T, Tcv4>>::Presence,
            <C5 as ChannelCfg<'d, T, Tcv5>>::Presence,
            <C6 as ChannelCfg<'d, T, Tcv6>>::Presence,
            <C7 as ChannelCfg<'d, T, Tcv7>>::Presence,
        >,
{
    type Split = <T::Filters as Flt8SplitBuild<
        'reg,
        'inj,
        'd,
        T,
        C,
        <C0 as ChannelCfg<'d, T, Tcv0>>::Presence,
        <C1 as ChannelCfg<'d, T, Tcv1>>::Presence,
        <C2 as ChannelCfg<'d, T, Tcv2>>::Presence,
        <C3 as ChannelCfg<'d, T, Tcv3>>::Presence,
        <C4 as ChannelCfg<'d, T, Tcv4>>::Presence,
        <C5 as ChannelCfg<'d, T, Tcv5>>::Presence,
        <C6 as ChannelCfg<'d, T, Tcv6>>::Presence,
        <C7 as ChannelCfg<'d, T, Tcv7>>::Presence,
    >>::Out;

    fn split_parts(self, common: DfsdmCommon<'d, T, Enabled>) -> Self::Split {
        let (d0, k0) = self.0.into_parts();
        let (d1, k1) = self.1.into_parts();
        let (d2, k2) = self.2.into_parts();
        let (d3, k3) = self.3.into_parts();
        let (d4, k4) = self.4.into_parts();
        let (d5, k5) = self.5.into_parts();
        let (d6, k6) = self.6.into_parts();
        let (d7, k7) = self.7.into_parts();
        <T::Filters as Flt8SplitBuild<
            'reg,
            'inj,
            'd,
            T,
            C,
            <C0 as ChannelCfg<'d, T, Tcv0>>::Presence,
            <C1 as ChannelCfg<'d, T, Tcv1>>::Presence,
            <C2 as ChannelCfg<'d, T, Tcv2>>::Presence,
            <C3 as ChannelCfg<'d, T, Tcv3>>::Presence,
            <C4 as ChannelCfg<'d, T, Tcv4>>::Presence,
            <C5 as ChannelCfg<'d, T, Tcv5>>::Presence,
            <C6 as ChannelCfg<'d, T, Tcv6>>::Presence,
            <C7 as ChannelCfg<'d, T, Tcv7>>::Presence,
        >>::build(common, d0, k0, d1, k1, d2, k2, d3, k3, d4, k4, d5, k5, d6, k6, d7, k7)
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
    pub fn configure_pins<'reg, 'inj, F, OUT>(mut self, f: F) -> <OUT as ChannelCfgTuple<'reg, 'inj, 'd, T, C>>::Split
    where
        F: FnOnce(<T::Transceivers as Shape>::Selectors<T>) -> OUT,
        OUT: ChannelCfgTuple<'reg, 'inj, 'd, T, C>,
    {
        let out = f(<T::Transceivers as Shape>::selectors::<T>());
        out.split_parts(DfsdmCommon::new(self.peri.expect("taken once"), self.ckout.take()).enable())
    }
}

/// Playground
pub fn test<T>(f: impl FnOnce(ChannelSelectors8<T>))
where
    T: Instance,
{
    f(ChannelSelectors8::<T>::new())
}

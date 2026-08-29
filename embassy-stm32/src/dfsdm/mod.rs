//! Digital Filter and Sigma-Delta Modulator (DFSDM)

#![macro_use]

/// Connect pac and embassy infrastructure to our types
pub mod associations;
/// Type-system
pub mod types;

use core::marker::PhantomData;
use core::mem::ManuallyDrop;
use core::ptr;

use embassy_hal_internal::PeripheralType;
use embassy_sync::waitqueue::AtomicWaker;
pub use types::*;

use crate::gpio::{AfType, Flex, OutputType, Pull, Speed};
use crate::{Peri, interrupt, rcc};

/// TODO
#[allow(missing_docs)]
#[derive(Clone, Copy, PartialEq)]
pub enum GenericConfigEnum {
    Low,
    High,
}

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
    _instance_marker: PhantomData<T>,
    _transceiver_marker: PhantomData<M>,
    _datasource_marker: PhantomData<MODE>,
    _powerstate_marker: PhantomData<P>,
    pub(crate) datin: S::Datin<'d>,
    pub(crate) ckin: S::Ckin<'d>,
    pub(crate) common: Option<&'a DfsdmCommon<'d, T, Enabled>>,
    // ckin_pin: Option<Flex<'d>>,
    // data_pin: Flex<'d>,
}

pub struct Filter<T, M>
where
    T: Instance + FilterInterrupt<M>,
    M: FilterMarker,
{
    _instance_marker: PhantomData<T>,
    _filter_marker: PhantomData<M>,
}

impl<T, M> Filter<T, M>
where
    T: Instance + FilterInterrupt<M>,
    M: FilterMarker,
{
    pub async fn wait_for_irq_test(&mut self) {
        use core::sync::atomic::{Ordering, compiler_fence};
        use core::task::Poll;

        use futures_util::future::poll_fn;

        // T::regs().start();

        poll_fn(|cx| {
            T::state().waker.register(cx.waker());

            compiler_fence(Ordering::SeqCst);
            if !false {
                //T::regs().wait_done() {
                Poll::Pending
            } else {
                Poll::Ready(())
            }
        })
        .await;

        // unsafe { core::ptr::read_volatile(T::regs().data()) }
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
            _instance_marker: PhantomData,
            _transceiver_marker: PhantomData,
            _datasource_marker: PhantomData,
            _powerstate_marker: PhantomData,
            datin: self.datin,
            ckin: self.ckin,
            common: self.common,
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
            _instance_marker: PhantomData,
            _transceiver_marker: PhantomData,
            _datasource_marker: PhantomData,
            _powerstate_marker: PhantomData,
            datin: self.datin,
            ckin: self.ckin,
            common: self.common,
        }
    }

    /// Set channel right shift factor
    fn set_data_right_shift(&mut self, shift: config_types::UInt<5>) {
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
    fn select_analog_watchdog_osr(&mut self, osr: config_types::UInt<5>) {
        T::regs()
            .ch(M::CHANNEL.index())
            .awscdr()
            .modify(|w| w.set_awfosr(osr.into()));
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

    /// Set pulse skips
    fn set_pulseskips(&mut self, skips: config_types::UInt<6>) {
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
pub struct TransceiverBuilder<'d, T, M, C, S, SN>
where
    T: Instance,
    M: TransceiverMarker,
    C: ClockOutputMode,
    S: PinSet,  //Own pins
    SN: PinSet, //Neighbors pins
{
    pub(crate) datin: S::Datin<'d>,
    pub(crate) ckin: S::Ckin<'d>,
    _m: PhantomData<(T, M, C, SN)>,
}

impl<'d, T, M, C, S, SN> TransceiverBuilder<'d, T, M, C, S, SN>
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
    pub fn new_parallel_adc(self) -> Transceiver<'static, 'd, T, M, S, ParallelAdcMode, Disabled>
    where
        T: capability::AdcInput,
    {
        todo!()
    }

    ///Same as [`Self::new_parallel_adc`] but using neighbors pins
    pub fn new_parallel_adc_neighbor(self) -> Transceiver<'static, 'd, T, M, SN, ParallelAdcMode, Disabled>
    where
        T: capability::AdcInput,
    {
        todo!()
    }

    /// Parallel input from CPU/DMA writes to CHyDATINR (DATMPX=2).
    /// No CKOUT, no pins needed. Serial pins declared on this channel
    /// are disconnected (the builder's Flexes drop here - they're unused
    /// in this mode).
    pub fn new_parallel_dma(self) -> Transceiver<'static, 'd, T, M, S, ParallelDmaMode, Disabled> {
        todo!()
    }

    ///Same as [`Self::new_parallel_dma`] but using neighbors pins
    pub fn new_parallel_dma_neighbor(self) -> Transceiver<'static, 'd, T, M, SN, ParallelDmaMode, Disabled> {
        todo!()
    }

    /// Create dualmode DMA
    ///
    /// Returns transceivers for channel `M` (even, owns DATINR) and `MN` (odd,
    /// reads INDAT1 from M's DATINR). Two filters must be configured - one
    /// assigned to `M` (reads INDAT0, the lower word) and one to `MN`
    /// (reads INDAT1, the upper word) - or the register won't drain and
    /// you'll get overrun errors.
    pub fn new_parallel_dma_dual<MN, SNN>(
        self,
        neighbor: TransceiverBuilder<'d, T, MN, C, SN, SNN>,
    ) -> (
        Transceiver<'static, 'd, T, M, S, ParallelDmaMode, Disabled>,
        Transceiver<'static, 'd, T, MN, SN, ParallelDmaMode, Disabled>,
    )
    where
        M: DualPackingAllowed + NextChannelForInstance<T, Next = MN>,
        MN: TransceiverMarker + NextChannelForInstance<T>,
        SNN: PinSet,
    {
        let _ = neighbor;
        todo!()
    }

    /// Parallel input from ADC writes to CHyDATINR (DATMPX=2).
    /// No CKOUT, no pins needed. Serial pins declared on this channel
    /// are disconnected (the builder's Flexes drop here - they're unused
    /// in this mode).
    pub fn new_manchester(self) -> Transceiver<'static, 'd, T, M, S, ManchesterMode, Disabled>
    where
        S: HasData,
    {
        todo!()
    }

    ///Same as [`Self::new_manchester`] but using neighbors pins
    pub fn new_manchester_neighbor(self) -> Transceiver<'static, 'd, T, M, SN, ManchesterMode, Disabled>
    where
        SN: HasData,
    {
        todo!()
    }

    ///TODO new_synchronous_int description
    pub fn new_synchronous_int(self) -> Transceiver<'static, 'd, T, M, S, SpiExtMode, Disabled>
    where
        S: HasDataAndClk,
    {
        todo!()
    }

    ///Same as [`Self::new_synchronous_int`] but using neighbors pins
    pub fn new_synchronous_int_neighbor(self) -> Transceiver<'static, 'd, T, M, SN, SpiExtMode, Disabled>
    where
        SN: HasDataAndClk,
    {
        todo!()
    }

    fn set_data_packing_mode(&mut self, mode: config_types::DataPackingMode)
    where
        M: DualPackingAllowed,
    {
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

impl<'d, T, M, S, SN> TransceiverBuilder<'d, T, M, OutputEnabled, S, SN>
where
    T: Instance,
    M: TransceiverMarker + NextChannelForInstance<T>,
    S: PinSet,
    SN: PinSet,
{
    ///TODO new_synchronous_ext description
    pub fn new_synchronous_ext(self) -> Transceiver<'static, 'd, T, M, S, SpiExtMode, Disabled>
    where
        S: HasData,
    {
        todo!();
        // Transceiver {
        //     _instance_marker: PhantomData,
        //     _transceiver_marker: PhantomData,
        //     _datasource_marker: PhantomData,
        //     _powerstate_marker: PhantomData,
        //     datin: self.datin,
        //     ckin: self.ckin,
        //     common: None,
        // }
    }

    ///Same as [`Self::new_synchronous_ext`] but using neighbors pins
    pub fn new_synchronous_ext_neighbor(self) -> Transceiver<'static, 'd, T, M, SN, SpiExtMode, Disabled>
    where
        SN: HasData,
    {
        todo!()
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

/// Splits
///
///

pub struct FilterBuilder<T: Instance, F: FilterMarker> {
    _m: PhantomData<(T, F)>,
}

/// Holds regerences to the peripheral and the optional clock-output. Disables the RCC of the peripheral when dropped.
pub struct DfsdmCommon<'d, T: Instance, P: PowerState> {
    pub(crate) _peri: Peri<'d, T>,
    /// `Some` iff the driver was created via [`Dfsdm::new_ckout`].
    pub(crate) _ckout: Option<Flex<'d>>,
    _powerstate_marker: PhantomData<P>,
}

impl<'d, T, P> Drop for DfsdmCommon<'d, T, P>
where
    T: Instance,
    P: PowerState,
{
    fn drop(&mut self) {
        T::RCC_INFO.disable();
    }
}

impl<'d, T> DfsdmCommon<'d, T, Disabled>
where
    T: Instance,
{
    /// Enables the channel
    pub fn disable(self) -> DfsdmCommon<'d, T, Enabled> {
        T::regs().ch(0).cfgr1().modify(|w| w.set_dfsdmen(true));

        let (_peri, _ckout) = self.into_raw_parts();
        DfsdmCommon {
            _peri,
            _ckout,
            _powerstate_marker: PhantomData,
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

impl<'d, T> DfsdmCommon<'d, T, Enabled>
where
    T: Instance,
{
    /// Disables the channel
    pub fn disable(self) -> DfsdmCommon<'d, T, Disabled> {
        T::regs().ch(0).cfgr1().modify(|w| w.set_dfsdmen(false));

        let (_peri, _ckout) = self.into_raw_parts();
        DfsdmCommon {
            _peri,
            _ckout,
            _powerstate_marker: PhantomData,
        }
    }
}

impl<'d, T, P> DfsdmCommon<'d, T, P>
where
    T: Instance,
    P: PowerState,
{
    fn into_raw_parts(self) -> (Peri<'d, T>, Option<Flex<'d>>) {
        let this = ManuallyDrop::new(self);
        unsafe { (ptr::read(&this._peri), ptr::read(&this._ckout)) }
    }
}

pub struct DfsdmSplit2Ch1Flt<'d, T, C, S0, S1>
where
    T: Instance,
    C: ClockOutputMode,
    S0: PinSet,
    S1: PinSet,
{
    pub common: DfsdmCommon<'d, T, Enabled>,
    pub ch0: TransceiverBuilder<'d, T, Tcv0, C, S0, S1>, // neighbor = ch1
    pub ch1: TransceiverBuilder<'d, T, Tcv1, C, S1, S0>, // neighbor = ch0 (wrap!)
    pub flt0: FilterBuilder<T, Flt0>,
}

pub struct DfsdmSplit8Ch8Flt<'d, T, C, S0, S1, S2, S3, S4, S5, S6, S7>
where
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
{
    pub common: DfsdmCommon<'d, T, Enabled>,
    pub ch0: TransceiverBuilder<'d, T, Tcv0, C, S0, S1>,
    pub ch1: TransceiverBuilder<'d, T, Tcv1, C, S1, S2>,
    pub ch2: TransceiverBuilder<'d, T, Tcv2, C, S2, S3>,
    pub ch3: TransceiverBuilder<'d, T, Tcv3, C, S3, S4>,
    pub ch4: TransceiverBuilder<'d, T, Tcv4, C, S4, S5>,
    pub ch5: TransceiverBuilder<'d, T, Tcv5, C, S5, S6>,
    pub ch6: TransceiverBuilder<'d, T, Tcv6, C, S6, S7>,
    pub ch7: TransceiverBuilder<'d, T, Tcv7, C, S7, S0>, // neighbor = ch0 (wrap!)
    pub flt0: FilterBuilder<T, Flt0>,
    pub flt1: FilterBuilder<T, Flt1>,
    pub flt2: FilterBuilder<T, Flt2>,
    pub flt3: FilterBuilder<T, Flt3>,
    pub flt4: FilterBuilder<T, Flt4>,
    pub flt5: FilterBuilder<T, Flt5>,
    pub flt6: FilterBuilder<T, Flt6>,
    pub flt7: FilterBuilder<T, Flt7>,
}

pub struct DfsdmSplit8Ch4Flt<'d, T, C, S0, S1, S2, S3, S4, S5, S6, S7>
where
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
{
    pub common: DfsdmCommon<'d, T, Enabled>,
    pub ch0: TransceiverBuilder<'d, T, Tcv0, C, S0, S1>,
    pub ch1: TransceiverBuilder<'d, T, Tcv1, C, S1, S2>,
    pub ch2: TransceiverBuilder<'d, T, Tcv2, C, S2, S3>,
    pub ch3: TransceiverBuilder<'d, T, Tcv3, C, S3, S4>,
    pub ch4: TransceiverBuilder<'d, T, Tcv4, C, S4, S5>,
    pub ch5: TransceiverBuilder<'d, T, Tcv5, C, S5, S6>,
    pub ch6: TransceiverBuilder<'d, T, Tcv6, C, S6, S7>,
    pub ch7: TransceiverBuilder<'d, T, Tcv7, C, S7, S0>, // neighbor = ch0 (wrap!)
    pub flt0: FilterBuilder<T, Flt0>,
    pub flt1: FilterBuilder<T, Flt1>,
    pub flt2: FilterBuilder<T, Flt2>,
    pub flt3: FilterBuilder<T, Flt3>,
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
    pub fn get_filter_test<F>(&self) -> Filter<T, F>
    where
        T: FilterInterrupt<F>,
        F: FilterMarker,
    {
        Filter {
            _instance_marker: PhantomData,
            _filter_marker: PhantomData,
        }
    }
}

impl<'d, T> Dfsdm<'d, T, OutputEnabled>
where
    T: Instance,
{
    /// Configure DFSDM module with a clock output
    pub fn new_ckout(peri: Peri<'d, T>, ckout: Peri<'d, if_afio!(impl CkoutPin<T, A>)>, config: Config) -> Self {
        let ckout = new_pin!(ckout, AfType::output(OutputType::PushPull, Speed::VeryHigh));

        Self::new_inner(peri, ckout, config)
    }
}

impl<'d, T> Dfsdm<'d, T, OutputDisabled>
where
    T: Instance,
{
    /// Configure DFSDM module without a clock output
    pub fn new(peri: Peri<'d, T>, config: Config) -> Self {
        Self::new_inner(peri, None, config)
    }
}

impl<'d, T, C> Dfsdm<'d, T, C>
where
    T: Instance,
    C: ClockOutputMode,
{
    fn new_inner(peri: Peri<'d, T>, ckout: Option<Flex<'d>>, config: Config) -> Self {
        let _ = config;
        let _ = peri;

        rcc::enable_and_reset::<T>();

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
        Self {
            _instance_marker: PhantomData,
            _clock_mode: PhantomData,
            ckout: ckout,
            peri: Some(peri),
        }
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
pub trait ChannelCfgTuple<'d, T: Instance, C: ClockOutputMode> {
    /// The fully-wired split (neighbor pin-sets already correct).
    type Split;

    /// Split helper-function
    fn split_parts(self, common: DfsdmCommon<'d, T, Enabled>) -> Self::Split;
}

impl<'d, T, C, C0, C1> ChannelCfgTuple<'d, T, C> for (C0, C1)
where
    T: Instance<Transceivers = capability::Tcv2, Filters = capability::Flt1>,
    C: ClockOutputMode,
    C0: ChannelCfg<'d, T, Tcv0>,
    C1: ChannelCfg<'d, T, Tcv1>,
{
    type Split = DfsdmSplit2Ch1Flt<
        'd,
        T,
        C,
        <C0 as ChannelCfg<'d, T, Tcv0>>::Presence,
        <C1 as ChannelCfg<'d, T, Tcv1>>::Presence,
    >;

    fn split_parts(self, common: DfsdmCommon<'d, T, Enabled>) -> Self::Split {
        let (d0, k0) = self.0.into_parts();
        let (d1, k1) = self.1.into_parts();
        DfsdmSplit2Ch1Flt {
            common,
            ch0: TransceiverBuilder {
                datin: d0,
                ckin: k0,
                _m: PhantomData,
            },
            ch1: TransceiverBuilder {
                datin: d1,
                ckin: k1,
                _m: PhantomData,
            },
            flt0: FilterBuilder { _m: PhantomData },
        }
    }
}

/// Builds the actual split struct from 8 already-extracted pin pairs.
/// Implemented separately for each filter-count capability.
pub trait Flt8SplitBuild<
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

impl<'d, T, C, S0, S1, S2, S3, S4, S5, S6, S7> Flt8SplitBuild<'d, T, C, S0, S1, S2, S3, S4, S5, S6, S7>
    for capability::Flt8
where
    T: Instance<Transceivers = capability::Tcv8, Filters = capability::Flt8>,
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
    type Out = DfsdmSplit8Ch8Flt<'d, T, C, S0, S1, S2, S3, S4, S5, S6, S7>;

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
        DfsdmSplit8Ch8Flt {
            common,
            ch0: TransceiverBuilder {
                datin: d0,
                ckin: k0,
                _m: PhantomData,
            },
            ch1: TransceiverBuilder {
                datin: d1,
                ckin: k1,
                _m: PhantomData,
            },
            ch2: TransceiverBuilder {
                datin: d2,
                ckin: k2,
                _m: PhantomData,
            },
            ch3: TransceiverBuilder {
                datin: d3,
                ckin: k3,
                _m: PhantomData,
            },
            ch4: TransceiverBuilder {
                datin: d4,
                ckin: k4,
                _m: PhantomData,
            },
            ch5: TransceiverBuilder {
                datin: d5,
                ckin: k5,
                _m: PhantomData,
            },
            ch6: TransceiverBuilder {
                datin: d6,
                ckin: k6,
                _m: PhantomData,
            },
            ch7: TransceiverBuilder {
                datin: d7,
                ckin: k7,
                _m: PhantomData,
            },
            flt0: FilterBuilder { _m: PhantomData },
            flt1: FilterBuilder { _m: PhantomData },
            flt2: FilterBuilder { _m: PhantomData },
            flt3: FilterBuilder { _m: PhantomData },
            flt4: FilterBuilder { _m: PhantomData },
            flt5: FilterBuilder { _m: PhantomData },
            flt6: FilterBuilder { _m: PhantomData },
            flt7: FilterBuilder { _m: PhantomData },
        }
    }
}

impl<'d, T, C, S0, S1, S2, S3, S4, S5, S6, S7> Flt8SplitBuild<'d, T, C, S0, S1, S2, S3, S4, S5, S6, S7>
    for capability::Flt4
where
    T: Instance<Transceivers = capability::Tcv8, Filters = capability::Flt4>,
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
    type Out = DfsdmSplit8Ch4Flt<'d, T, C, S0, S1, S2, S3, S4, S5, S6, S7>;

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
            ch0: TransceiverBuilder {
                datin: d0,
                ckin: k0,
                _m: PhantomData,
            },
            ch1: TransceiverBuilder {
                datin: d1,
                ckin: k1,
                _m: PhantomData,
            },
            ch2: TransceiverBuilder {
                datin: d2,
                ckin: k2,
                _m: PhantomData,
            },
            ch3: TransceiverBuilder {
                datin: d3,
                ckin: k3,
                _m: PhantomData,
            },
            ch4: TransceiverBuilder {
                datin: d4,
                ckin: k4,
                _m: PhantomData,
            },
            ch5: TransceiverBuilder {
                datin: d5,
                ckin: k5,
                _m: PhantomData,
            },
            ch6: TransceiverBuilder {
                datin: d6,
                ckin: k6,
                _m: PhantomData,
            },
            ch7: TransceiverBuilder {
                datin: d7,
                ckin: k7,
                _m: PhantomData,
            },
            flt0: FilterBuilder { _m: PhantomData },
            flt1: FilterBuilder { _m: PhantomData },
            flt2: FilterBuilder { _m: PhantomData },
            flt3: FilterBuilder { _m: PhantomData },
        }
    }
}

impl<'d, T, C, C0, C1, C2, C3, C4, C5, C6, C7> ChannelCfgTuple<'d, T, C> for (C0, C1, C2, C3, C4, C5, C6, C7)
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
    pub fn configure_pins<F, OUT>(mut self, f: F) -> <OUT as ChannelCfgTuple<'d, T, C>>::Split
    where
        F: FnOnce(<T::Transceivers as Shape>::Selectors<T>) -> OUT,
        OUT: ChannelCfgTuple<'d, T, C>,
    {
        let out = f(<T::Transceivers as Shape>::selectors::<T>());
        out.split_parts(DfsdmCommon {
            _peri: self.peri.take().expect("taken once"),
            _ckout: self.ckout.take(),
            _powerstate_marker: PhantomData,
        })
    }
}

/// Playground
pub fn test<T>(f: impl FnOnce(ChannelSelectors8<T>))
where
    T: Instance,
{
    f(ChannelSelectors8::<T>::new())
}

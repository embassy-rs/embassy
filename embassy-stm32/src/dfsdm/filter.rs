use super::*;

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

pub struct NoDma;
pub struct RegDma;
pub struct InjDma;

mod sealed {
    pub trait Sealed {}
}

pub trait DmaMode: sealed::Sealed {
    const REG_ENABLED: bool;
    const INJ_ENABLED: bool;
}

impl sealed::Sealed for NoDma {}
impl sealed::Sealed for RegDma {}
impl sealed::Sealed for InjDma {}

impl DmaMode for NoDma {
    const REG_ENABLED: bool = false;
    const INJ_ENABLED: bool = false;
}
impl DmaMode for RegDma {
    const REG_ENABLED: bool = true;
    const INJ_ENABLED: bool = false;
}
impl DmaMode for InjDma {
    const REG_ENABLED: bool = false;
    const INJ_ENABLED: bool = true;
}

pub trait FilterDma<T, M>
where
    T: Instance + FilterInterrupt<M>,
    M: FilterMarker + InstanceEvents<T>,
{
    fn data_register(&self) -> *mut u32;
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
        config: &FilterConfig,
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
        config: &FilterConfig,
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
        config: &FilterConfig,
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
        config: &FilterConfig,
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

    fn configure(config: &FilterConfig) {
        Self::set_filter_parameters(config.filter_params);
        Self::set_continuous(config.enable_continuous_regular);
        Self::set_fastmode(config.enable_fast_regular);
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

    /// Replaces the injected transceivers, releasing the old borrows so the
    /// previous transceivers can be mutated afterwards. Since this may change
    /// the lifetime of the borrows, it consumes and returns a new `Filter`
    /// rather than mutating in place. This is pure borrow-checker bookkeeping,
    /// not a hardware requirement — see [`FilterRegular::assign_transceiver`]
    /// for the in-place alternative when the lifetime doesn't need to change.
    pub fn replace_regular_transceiver<'new_reg>(
        self,
        transceiver: &'new_reg dyn TransceiverTrait<T, Enabled>,
    ) -> Filter<'new_reg, 'ti, 'a, 'd, T, M, D> {
        FilterRegular::<'a, 'd, 'ti, T, M, D>::set_regular_transceiver(transceiver.index());

        Filter {
            reg: FilterRegular {
                _common: PhantomData,
                regular: transceiver,
            },
            ..self
        }
    }

    /// Replaces the injected transceivers, releasing the old borrows so the
    /// previous transceivers can be mutated afterwards. Since this may change
    /// the lifetime of the borrows, it consumes and returns a new `Filter`
    /// rather than mutating in place. This is pure borrow-checker bookkeeping,
    /// not a hardware requirement — see [`FilterInjected::assign_transceivers`]
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

        Filter {
            inj: FilterInjected {
                injected: slots,
                _common: PhantomData,
            },
            ..self
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
    pub async fn read_regular(
        &mut self,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T, M>>,
    ) -> (i32, u8, bool) {
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

    /// Returns whether a regular conversion result is available.
    pub fn is_end_of_regular_conversion(&mut self) -> bool {
        FilterRegs::<T, M>::end_of_regular_conversion()
    }

    /// Returns whether a regular conversion is currently in progress or pendiong.
    pub fn regular_conversion_in_progress() -> bool {
        T::regs().flt(M::CHANNEL.index()).isr().read().rcip()
    }

    /// Returns whether a regular conversion is currently in progress or pendiong.
    pub fn is_regular_conversion_in_progress(&mut self) -> bool {
        Self::regular_conversion_in_progress()
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
    pub async fn read_injected(
        &mut self,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T, M>>,
    ) -> (i32, u8) {
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

    /// Returns whether an injected conversion result is available.
    pub fn is_end_of_injected_conversion(&mut self) -> bool {
        FilterRegs::<T, M>::end_of_injected_conversion()
    }

    /// Returns whether an injected conversion is currently in progress or pendiong.
    pub fn injected_conversion_in_progress() -> bool {
        T::regs().flt(M::CHANNEL.index()).isr().read().jcip()
    }

    /// Returns whether an injected conversion is currently in progress or pendiong.
    pub fn is_injected_conversion_in_progress(&mut self) -> bool {
        Self::injected_conversion_in_progress()
    }
}

impl<'a, 'd, 't, T, M> FilterRegular<'a, 'd, 't, T, M, RegDma>
where
    T: Instance + FilterInterrupt<M>,
    M: FilterMarker + InstanceEvents<T>,
{
    // DMA read function
}

impl<'a, 'd, 't, T, M> FilterInjected<'a, 'd, 't, T, M, InjDma>
where
    T: Instance + FilterInterrupt<M>,
    M: FilterMarker + InstanceEvents<T>,
{
    // DMA read function
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
    pub fn set_enabled(enabled: bool) {
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
}

use super::*;

struct FilterDisabled<'a, 'd, T, M>
where
    T: Instance + FilterInterrupt<M>,
    M: FilterMarker + InstanceEvents<T>,
{
    marker: PhantomData<(T, M)>,
    common: &'a DfsdmCommon<'d, T, Enabled>,
}

struct Filter<'tr, 'ti, 'a, 'd, T, M, D>
where
    T: Instance + FilterInterrupt<M>,
    M: FilterMarker + InstanceEvents<T>,
    D: DmaMode,
{
    pub reg: FilterRegular<'tr, T, M, D>,
    pub inj: FilterInjected<'ti, T, M, D>,
    pub awd: AnalogWatchdog<'a, 'd, T, M>,
}

struct FilterRegular<'t, T, M, D>
where
    T: Instance + FilterInterrupt<M>,
    M: FilterMarker + InstanceEvents<T>,
    D: DmaMode,
{
    marker: PhantomData<(T, M, D)>,
    regular: &'t dyn TransceiverTrait<T, Enabled>,
}

struct FilterInjected<'t, T, M, D>
where
    T: Instance + FilterInterrupt<M>,
    M: FilterMarker + InstanceEvents<T>,
    D: DmaMode,
{
    marker: PhantomData<(T, M, D)>,
    injected: [Option<&'t dyn TransceiverTrait<T, Enabled>>; 8],
}

struct NoDma;
struct RegDma;
struct InjDma;

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

//filter is "on", "off" version needs own off struct/"DIsabledFilter" because of members
impl<'a, 'd, T, M> FilterDisabled<'a, 'd, T, M>
where
    T: Instance + FilterInterrupt<M>,
    M: FilterMarker + InstanceEvents<T>,
{
    pub fn new_no_dma<'tr, 'ti, const N: usize>(
        self,
        regular: &'tr dyn TransceiverTrait<T, Enabled>,
        injected: [&'ti dyn TransceiverTrait<T, Enabled>; N],
    ) -> Filter<'tr, 'ti, 'a, 'd, T, M, NoDma>
    where
        [(); N]: NonEmpty,
    {
        self.new_int(regular, injected)
    }

    pub fn new_reg_dma<'tr, 'ti, const N: usize>(
        self,
        regular: &'tr dyn TransceiverTrait<T, Enabled>,
        injected: [&'ti dyn TransceiverTrait<T, Enabled>; N],
    ) -> Filter<'tr, 'ti, 'a, 'd, T, M, RegDma>
    where
        [(); N]: NonEmpty,
    {
        self.new_int(regular, injected)
    }

    pub fn new_inj_dma<'tr, 'ti, const N: usize>(
        self,
        regular: &'tr dyn TransceiverTrait<T, Enabled>,
        injected: [&'ti dyn TransceiverTrait<T, Enabled>; N],
    ) -> Filter<'tr, 'ti, 'a, 'd, T, M, InjDma>
    where
        [(); N]: NonEmpty,
    {
        self.new_int(regular, injected)
    }

    fn new_int<'tr, 'ti, const N: usize, D>(
        self,
        regular: &'tr dyn TransceiverTrait<T, Enabled>,
        injected: [&'ti dyn TransceiverTrait<T, Enabled>; N],
    ) -> Filter<'tr, 'ti, 'a, 'd, T, M, D>
    where
        D: DmaMode,
        [(); N]: NonEmpty,
    {
        let filter = Filter {
            reg: FilterRegular::new(regular),
            inj: FilterInjected::new(injected),
            awd: AnalogWatchdog::new(self.common),
        };

        Self::set_regular_dma_en(D::REG_ENABLED);
        Self::set_injected_dma_en(D::INJ_ENABLED);
        filter_functions::set_enabled::<T, M>(true);

        filter
    }

    /// Enables or disables DMA transfers for regular conversions.
    fn set_regular_dma_en(dma_enable: bool) {
        T::regs()
            .flt(M::CHANNEL.index())
            .cr1()
            .modify(|w| w.set_rdmaen(dma_enable));
    }

    /// Enables or disables DMA transfers for injected conversions.
    fn set_injected_dma_en(dma_enable: bool) {
        T::regs()
            .flt(M::CHANNEL.index())
            .cr1()
            .modify(|w| w.set_jdmaen(dma_enable));
    }
}

impl<'tr, 'ti, 'a, 'd, T, M, D> Filter<'tr, 'ti, 'a, 'd, T, M, D>
where
    T: Instance + FilterInterrupt<M>,
    M: FilterMarker + InstanceEvents<T>,
    D: DmaMode,
{
    pub fn disable(self) -> FilterDisabled<'a, 'd, T, M> {
        filter_functions::set_enabled::<T, M>(false);
        let common = self.awd.common;

        FilterDisabled {
            marker: PhantomData,
            common,
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
        FilterRegular::<'ti, T, M, D>::set_regular_transceiver(transceiver.index());

        Filter {
            reg: FilterRegular {
                regular: transceiver,
                marker: PhantomData,
            },
            inj: self.inj,
            awd: self.awd,
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
        let (slots, filterword) = FilterInjected::<'ti, T, M, D>::build_injected_slots(transceivers);
        FilterInjected::<'ti, T, M, D>::set_injected_channels(filterword);

        Filter {
            reg: self.reg,
            inj: FilterInjected {
                injected: slots,
                marker: PhantomData,
            },
            awd: self.awd,
        }
    }
}

impl<'t, T, M, D> FilterRegular<'t, T, M, D>
where
    T: Instance + FilterInterrupt<M>,
    M: FilterMarker + InstanceEvents<T>,
    D: DmaMode,
{
    pub(crate) fn new(transceiver: &'t dyn TransceiverTrait<T, Enabled>) -> Self {
        Self::set_regular_transceiver(transceiver.index());
        Self {
            marker: PhantomData,
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
    pub(crate) fn end_of_regular_conversion() -> bool {
        T::regs().flt(M::CHANNEL.index()).isr().read().reocf()
    }

    /// Returns whether a regular conversion result is available.
    pub fn is_end_of_regular_conversion(&mut self) -> bool {
        Self::end_of_regular_conversion()
    }

    /// Returns whether a regular conversion is currently in progress or pendiong.
    pub fn regular_conversion_in_progress() -> bool {
        T::regs().flt(M::CHANNEL.index()).isr().read().rcip()
    }

    /// Returns whether a regular conversion is currently in progress or pendiong.
    pub fn is_regular_conversion_in_progress(&mut self) -> bool {
        Self::regular_conversion_in_progress()
    }

    /// Enables or disables regular end-of-conversion interrupts.
    pub(crate) fn set_regular_end_of_conversion_interrupt(enabled: bool) {
        T::regs()
            .flt(M::CHANNEL.index())
            .cr2()
            .modify(|w| w.set_reocie(enabled));
    }
}

impl<'t, T, M, D> FilterInjected<'t, T, M, D>
where
    T: Instance + FilterInterrupt<M>,
    M: FilterMarker + InstanceEvents<T>,
    D: DmaMode,
{
    pub(crate) fn new<const N: usize>(transceivers: [&'t dyn TransceiverTrait<T, Enabled>; N]) -> Self
    where
        [(); N]: NonEmpty,
    {
        let (slots, filterword) = Self::build_injected_slots(transceivers);
        Self::set_injected_channels(filterword);

        Self {
            marker: PhantomData,
            injected: slots,
        }
    }
    // Normal stuff

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
    pub(crate) fn end_of_injected_conversion() -> bool {
        T::regs().flt(M::CHANNEL.index()).isr().read().jeocf()
    }

    /// Returns whether an injected conversion result is available.
    pub fn is_end_of_injected_conversion(&mut self) -> bool {
        Self::end_of_injected_conversion()
    }

    /// Returns whether an injected conversion is currently in progress or pendiong.
    pub fn injected_conversion_in_progress() -> bool {
        T::regs().flt(M::CHANNEL.index()).isr().read().jcip()
    }

    /// Returns whether an injected conversion is currently in progress or pendiong.
    pub fn is_injected_conversion_in_progress(&mut self) -> bool {
        Self::injected_conversion_in_progress()
    }

    /// Enables or disables injected end-of-conversion interrupts.
    pub(crate) fn set_injected_end_of_conversion_interrupt(enabled: bool) {
        T::regs()
            .flt(M::CHANNEL.index())
            .cr2()
            .modify(|w| w.set_jeocie(enabled));
    }
}

impl<'t, T, M> FilterRegular<'t, T, M, RegDma>
where
    T: Instance + FilterInterrupt<M>,
    M: FilterMarker + InstanceEvents<T>,
{
    // DMA read function
}

impl<'t, T, M> FilterInjected<'t, T, M, InjDma>
where
    T: Instance + FilterInterrupt<M>,
    M: FilterMarker + InstanceEvents<T>,
{
    // DMA read function
}

mod filter_functions {
    use crate::dfsdm::{FilterMarker, Instance, InstanceEvents};

    /// Enable or disable the filter
    pub fn set_enabled<T, M>(enabled: bool)
    where
        T: Instance,
        M: FilterMarker + InstanceEvents<T>,
    {
        T::regs().flt(M::CHANNEL.index()).cr1().modify(|w| w.set_dfen(enabled));
    }
}

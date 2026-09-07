pub struct Filter<'a, 'd, 'reg, 'inj, T, M, P>
where
    T: Instance + FilterInterrupt<M>,
    M: FilterMarker + InstanceEvents<T>,
    P: PowerState,
{
    _instance_marker: PhantomData<T>,
    _filter_marker: PhantomData<M>,
    _powerstate_marker: PhantomData<P>,
    pub(crate) common: &'a DfsdmCommon<'d, T, Enabled>, // <-- Added!
    regular: Option<&'reg dyn TransceiverTrait<T, Enabled>>,
    injected: [Option<&'inj dyn TransceiverTrait<T, Enabled>>; 8],
}

impl<'a, 'd, 'reg, 'inj, T, M, P> Drop for Filter<'a, 'd, 'reg, 'inj, T, M, P>
where
    T: Instance + FilterInterrupt<M>,
    M: FilterMarker + InstanceEvents<T>,
    P: PowerState,
{
    fn drop(&mut self) {
        Self::set_enabled(false);
    }
}

impl<'a, 'd, 'reg, 'inj, T, M> Filter<'a, 'd, 'reg, 'inj, T, M, Disabled>
where
    T: Instance + FilterInterrupt<M>,
    M: FilterMarker + InstanceEvents<T>,
{
    /// Enable the Filter
    pub fn enable<'new_reg, 'new_inj, const N: usize>(
        mut self,
        reg_transceiver: &'new_reg dyn TransceiverTrait<T, Enabled>,
        inj_transceivers: [&'new_inj dyn TransceiverTrait<T, Enabled>; N],
    ) -> Filter<'a, 'd, 'new_reg, 'new_inj, T, M, Enabled>
    where
        [(); N]: NonEmpty,
    {
        let this = self
            .assign_regular_transceiver(reg_transceiver)
            .assign_injected_transceivers(inj_transceivers);

        Self::set_enabled(true);
        Filter {
            _instance_marker: PhantomData,
            _filter_marker: PhantomData,
            _powerstate_marker: PhantomData,
            common: this.common,
            regular: this.regular,
            injected: this.injected,
        }
    }

    /// Configure the Filter
    pub fn configure(mut self, config: &FilterConfig) -> Filter<'a, 'd, 'reg, 'inj, T, M, Disabled> {
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

impl<'a, 'd, 'reg, 'inj, T, M> Filter<'a, 'd, 'reg, 'inj, T, M, Enabled>
where
    T: Instance + FilterInterrupt<M>,
    M: FilterMarker + InstanceEvents<T>,
{
    /// Disable the filter
    pub fn disable(self) -> Filter<'a, 'd, 'reg, 'inj, T, M, Disabled> {
        Self::set_enabled(false);
        Filter {
            _instance_marker: PhantomData,
            _filter_marker: PhantomData,
            _powerstate_marker: PhantomData,
            common: self.common,
            regular: self.regular,
            injected: self.injected,
        }
    }

    /// Reassign the provided `Transceiver` as the regular conversion input for this `Filter`
    pub fn reassign_regular_transceiver<'new_reg>(
        self,
        channel: &'new_reg dyn TransceiverTrait<T, Enabled>,
    ) -> Filter<'a, 'd, 'new_reg, 'inj, T, M, Enabled> {
        self.assign_regular_transceiver(channel)
    }

    /// Reassign transceivers to the injected conversion group of this `Filter`
    ///
    /// Note: Overwrites all assignments.
    pub fn reassign_injected_transceivers<'new_inj, const N: usize>(
        self,
        transceivers: [&'new_inj dyn TransceiverTrait<T, Enabled>; N],
    ) -> Filter<'a, 'd, 'reg, 'new_inj, T, M, Enabled>
    where
        [(); N]: NonEmpty,
    {
        self.assign_injected_transceivers(transceivers)
    }
}

impl<'a, 'd, 'reg, 'inj, T, M, P> Filter<'a, 'd, 'reg, 'inj, T, M, P>
where
    T: Instance + FilterInterrupt<M>,
    M: FilterMarker + InstanceEvents<T>,
    P: PowerState,
{
    /// 28-bit timer counting conversion time t = CNVCNT[27:0] / fDFSDMCLK
    pub fn get_cnv_cnt(&self) -> u32 {
        T::regs().flt(M::CHANNEL.index()).cnvtimr().read().cnvcnt()
    }

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

    /// Assign the provided `Transceiver` as the regular conversion input for this `Filter`
    pub(crate) fn assign_regular_transceiver<'new_reg>(
        mut self,
        channel: &'new_reg dyn TransceiverTrait<T, Enabled>,
    ) -> Filter<'a, 'd, 'new_reg, 'inj, T, M, P> {
        self.set_regular_transceiver(channel.index() as u8);

        Filter {
            _instance_marker: PhantomData,
            _filter_marker: PhantomData,
            _powerstate_marker: PhantomData,
            common: self.common,
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
    ) -> Filter<'a, 'd, 'reg, 'new_inj, T, M, P>
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
            common: self.common,
            regular: self.regular,
            injected: injected,
        }
    }

    /// Enable or disable the filter
    pub(crate) fn set_enabled(enabled: bool) {
        T::regs().flt(M::CHANNEL.index()).cr1().modify(|w| w.set_dfen(enabled));
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
}

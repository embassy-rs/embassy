//! Filter and its interrupt handling: filter configuration, the filter driver
//! type, the interrupt handler, and per-filter interrupt/event state.

use super::*;

// =============================================================================
// FilterConfig
// =============================================================================

/// Configuration for a filter, applied on enable.
pub struct FilterConfig<T: Instance, M: FilterMarker> {
    /// Filter order, OSR and input width.
    pub filter_params: FilterParameters,
    /// Run regular conversions continuously.
    pub enable_continuous_regular: bool,
    /// Use fast mode for continuous regular conversions: the filter is not
    /// refilled between conversions, so the OSR windows overlap and each
    /// conversion after the first is faster.
    pub enable_fast_regular: bool,
    /// Synchronize regular conversions to the clock.
    pub enable_regular_sync: bool,
    /// Synchronize injected conversions to the clock.
    pub enable_injected_sync: bool,
    /// Cycle injected conversions through the selected transceivers.
    pub enable_injected_scanning: bool,

    /// Configures the trigger for injected conversions.
    ///
    /// [`InjectedTrigger::Enabled`] enables the trigger with the specified trigger
    /// source and edge; [`InjectedTrigger::Disabled`] disables it.
    pub trigger: InjectedTrigger<T, M>,
}

impl<T: Instance, M: FilterMarker> Default for FilterConfig<T, M> {
    fn default() -> Self {
        // `new` cannot panic: `Disabled` is a bypass filter with unity gain
        // (FOSR=1, IOSR=1), so the accumulator value equals the raw input (0/1
        // serial, at most 2^16 for 16-bit parallel), always far below the
        // 2^31-1 ceiling; `iosr = 1` is also a valid register value.
        FilterConfig {
            filter_params: FilterParameters::new(config::FilterOrder::Disabled, 1),
            enable_continuous_regular: false,
            enable_fast_regular: false,
            enable_injected_sync: false,
            enable_regular_sync: false,
            enable_injected_scanning: false,
            trigger: InjectedTrigger::Disabled,
        }
    }
}

/// Sign-extends a 24bit LSB number to a i32
pub(crate) fn sign_extend_24(x: u32) -> i32 {
    ((x << 8) as i32) >> 8
}

// =============================================================================
// Filter
// =============================================================================

/// A filter that is disabled (not yet enabled).
pub struct FilterDisabled<'a, 'd, T, M>
where
    T: Instance + FilterInterrupt<M>,
    M: FilterMarker + InstanceEvents<T>,
{
    _marker: PhantomData<M>,
    common: &'a DfsdmCommon<'d, T, Enabled>,
}

/// An enabled filter, split into its regular, injected, watchdog and extremes
/// parts.
pub struct Filter<'tr, 'ti, 'a, 'd, T, M, D>
where
    T: Instance + FilterInterrupt<M>,
    M: FilterMarker + InstanceEvents<T>,
    D: DmaMode,
{
    common: &'a DfsdmCommon<'d, T, Enabled>,
    /// Regular-conversion half.
    pub regular: FilterRegular<'a, 'd, 'tr, T, M, D>,
    /// Injected-conversion half.
    pub injected: FilterInjected<'a, 'd, 'ti, T, M, D>,
    /// Analog watchdog.
    pub awd: AnalogWatchdog<'a, 'd, T, M>,
    /// Extremes detector.
    pub extremes: ExtremesDetector<'a, 'd, T, M>,
}

/// Regular-conversion half of a filter.
pub struct FilterRegular<'a, 'd, 't, T, M, D>
where
    T: Instance + FilterInterrupt<M>,
    M: FilterMarker + InstanceEvents<T>,
    D: DmaMode,
{
    _common: PhantomData<(&'a DfsdmCommon<'d, T, Enabled>, M, D)>,
    regular: &'t dyn TransceiverTrait<T, Enabled>,
}

/// Injected-conversion half of a filter.
pub struct FilterInjected<'a, 'd, 't, T, M, D>
where
    T: Instance + FilterInterrupt<M>,
    M: FilterMarker + InstanceEvents<T>,
    D: DmaMode,
{
    _common: PhantomData<(&'a DfsdmCommon<'d, T, Enabled>, M, D)>,
    injected: [Option<&'t dyn TransceiverTrait<T, Enabled>>; 8],
}

//filter is "on", "off" version needs own off struct/"DisabledFilter" because of members
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
    /// Enable the filter without a DMA request flag.
    pub fn enable_no_dma<'tr, 'ti, const N: usize>(
        self,
        regular: &'tr dyn TransceiverTrait<T, Enabled>,
        injected: [&'ti dyn TransceiverTrait<T, Enabled>; N],
        config: &FilterConfig<T, M>,
    ) -> Filter<'tr, 'ti, 'a, 'd, T, M, NoDma>
    where
        [(); N]: NonEmpty,
    {
        self.enable_int(regular, injected, config)
    }

    /// Enable the filter and set the regular-conversion DMA request flag (RDMAEN).
    pub fn enable_reg_dma<'tr, 'ti, const N: usize>(
        self,
        regular: &'tr dyn TransceiverTrait<T, Enabled>,
        injected: [&'ti dyn TransceiverTrait<T, Enabled>; N],
        config: &FilterConfig<T, M>,
    ) -> Filter<'tr, 'ti, 'a, 'd, T, M, RegDma>
    where
        [(); N]: NonEmpty,
    {
        self.enable_int(regular, injected, config)
    }

    /// Enable the filter and set the injected-conversion DMA request flag (JDMAEN).
    pub fn enable_inj_dma<'tr, 'ti, const N: usize>(
        self,
        regular: &'tr dyn TransceiverTrait<T, Enabled>,
        injected: [&'ti dyn TransceiverTrait<T, Enabled>; N],
        config: &FilterConfig<T, M>,
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
        config: &FilterConfig<T, M>,
    ) -> Filter<'tr, 'ti, 'a, 'd, T, M, D>
    where
        D: DmaMode,
        [(); N]: NonEmpty,
    {
        let filter = Filter {
            common: self.common,
            regular: FilterRegular::new(self.common, regular),
            injected: FilterInjected::new(self.common, injected),
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

    fn configure(config: &FilterConfig<T, M>) {
        Self::set_filter_parameters(config.filter_params);
        Self::set_continuous(config.enable_continuous_regular);
        Self::set_fastmode(config.enable_fast_regular);
        Self::set_regular_synchronization(config.enable_regular_sync);
        Self::set_injected_synchronization(config.enable_injected_sync);
        Self::set_injected_scanning(config.enable_injected_scanning);
        Self::configure_injected_trigger(&config.trigger);
    }

    /// Writes the filter order, FOSR and IOSR into the filter registers.
    fn set_filter_parameters(params: config::FilterParameters) {
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
    /// When enabled, the regular transceiver is converted repeatedly after each
    /// conversion request. Disabling it while a continuous conversion is in
    /// progress stops the conversion immediately.
    fn set_continuous(enabled: bool) {
        T::regs().flt(M::CHANNEL.index()).cr1().modify(|w| w.set_rcont(enabled));
    }

    /// Configures the trigger for injected conversions.
    ///
    /// [`InjectedTrigger::Enabled`] enables the trigger with the specified trigger
    /// source and edge; [`InjectedTrigger::Disabled`] disables it.
    fn configure_injected_trigger(trigger: &InjectedTrigger<T, M>) {
        let (jextsel, jexten) = match trigger {
            InjectedTrigger::Disabled => (0, 0), // disable
            InjectedTrigger::Enabled { jextsel, edge, _m } => (*jextsel, *edge as u8),
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
    /// When enabled, injected conversions cycle through all selected transceivers,
    /// starting again at the lowest selected transceiver. When disabled, each
    /// conversion advances to the next selected transceiver.
    ///
    /// Changing the injected transceiver group while scanning is disabled resets
    /// the selection to the lowest selected transceiver.
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
    /// Disable the filter.
    pub fn disable(self) -> FilterDisabled<'a, 'd, T, M> {
        FilterRegs::<T, M>::set_enabled(false);

        FilterDisabled {
            _marker: PhantomData,
            common: self.common,
        }
    }
    // Normal stuff,

    /// 28-bit conversion-time counter: `CNVCNT[27:0] / fDFSDMCLK` is the time
    /// of the current (or most recent) conversion.
    ///
    /// The timer runs on the DFSDM kernel clock (fDFSDMCLK), starts when a
    /// conversion starts and stops when it finishes, so it measures the interval
    /// between the first and last serial sample of one conversion. The value is
    /// proportional to `FOSR * IOSR / fCKIN`, where `fCKIN` is the channel input
    /// clock (or the parallel input data rate), and it changes with each
    /// completed conversion. A bypassed filter (FOSR = 1) yields 0.
    ///
    /// Not a reliable liveness signal: it only updates when a conversion
    /// completes, so sampling it slower than the conversion rate aliases and
    /// looks stuck even while the filter is healthy. Use [`ClockAbsenceDetector`]
    /// or an overrun/timeout on your reads to detect starvation instead.
    pub fn conversion_timer(&self) -> u32 {
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
        FilterRegular::<'a, 'd, 'ti, T, M, D>::set_transceiver(transceiver.index());

        let this = ManuallyDrop::new(self);
        // SAFETY: `this` is wrapped in `ManuallyDrop` to prevent the destructor from
        // running. We extract each field with `ptr::read`, which performs a bitwise
        // move without invoking drop. The original `Filter` is never dropped and all
        // extracted fields are moved into the new `Filter`, maintaining ownership
        // invariants. Skipping the original `Filter`'s Drop is intentional: it would
        // clear DFEN, but the returned `Filter` re-acquires that teardown obligation.
        let common = unsafe { ptr::read(&this.common) };
        let injected = unsafe { ptr::read(&this.injected) };
        let awd = unsafe { ptr::read(&this.awd) };
        let extremes = unsafe { ptr::read(&this.extremes) };

        Filter {
            common,
            regular: FilterRegular {
                _common: PhantomData,
                regular: transceiver,
            },
            injected,
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
        let (slots, filterword) = FilterInjected::<'a, 'd, 'ti, T, M, D>::build_slots(transceivers);
        FilterInjected::<'a, 'd, 'ti, T, M, D>::set_channels(filterword);

        let this = ManuallyDrop::new(self);
        // SAFETY: `this` is wrapped in `ManuallyDrop` to prevent the destructor from
        // running. We extract each field with `ptr::read`, which performs a bitwise
        // move without invoking drop. The original `Filter` is never dropped and all
        // extracted fields are moved into the new `Filter`, maintaining ownership
        // invariants. Skipping the original `Filter`'s Drop is intentional: it would
        // clear DFEN, but the returned `Filter` re-acquires that teardown obligation.
        let common = unsafe { ptr::read(&this.common) };
        let regular = unsafe { ptr::read(&this.regular) };
        let awd = unsafe { ptr::read(&this.awd) };
        let extremes = unsafe { ptr::read(&this.extremes) };

        Filter {
            common,
            regular,
            injected: FilterInjected {
                injected: slots,
                _common: PhantomData,
            },
            awd,
            extremes,
        }
    }
}

/// Regular conversion result.
pub struct ResultRegular {
    /// Sign-extended 24-bit sample.
    pub data: i32,
    /// Transceiver the sample came from.
    pub channel: u8,
    /// Set if the conversion was delayed by an injected conversion.
    pub pending: bool,
}

impl ResultRegular {
    /// Decode a raw `u32` RDATAR word, e.g. read from a DMA ring buffer.
    ///
    /// The word layout is `RDATA[23:8]` (24-bit data), `RPEND` (bit 4) and
    /// `RDATACH[2:0]` (channel).
    pub fn from_word(word: u32) -> Self {
        let reg = crate::pac::dfsdm::regs::Rdatar(word);
        ResultRegular {
            data: sign_extend_24(reg.rdata()),
            channel: reg.rdatach(),
            pending: reg.rpend(),
        }
    }
}

/// Injected conversion result.
pub struct ResultInjected {
    /// Sign-extended 24-bit sample.
    pub data: i32,
    /// Transceiver the sample came from.
    pub channel: u8,
}

impl ResultInjected {
    /// Decode a raw `u32` JDATAR word, e.g. read from a DMA ring buffer.
    ///
    /// The word layout is `JDATA[23:8]` (24-bit data) and `JDATACH[2:0]`
    /// (channel).
    pub fn from_word(word: u32) -> Self {
        let reg = crate::pac::dfsdm::regs::Jdatar(word);
        ResultInjected {
            data: sign_extend_24(reg.jdata()),
            channel: reg.jdatach(),
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
        Self::set_transceiver(transceiver.index());
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
    ///
    /// # Note
    /// The regular channel select is shadowed: it takes effect only at the next
    /// [`start_conversion`](Self::start_conversion) (RSWSTART), so an in-progress
    /// conversion keeps using the previously selected channel.
    pub fn assign_transceiver(&mut self, transceiver: &'t dyn TransceiverTrait<T, Enabled>) {
        Self::set_transceiver(transceiver.index());
        self.regular = transceiver;
    }

    fn set_transceiver(ch: usize) {
        T::regs().flt(M::CHANNEL.index()).cr1().modify(|w| w.set_rch(ch as u8));
    }

    /// Start a regular conversion.
    ///
    /// # Note
    /// The request is ignored while a regular conversion is in progress (RCIP).
    /// An injected conversion preempts a running regular conversion, which is
    /// restarted and flagged via [`ResultRegular::pending`].
    pub fn start_conversion(&mut self) {
        T::regs().flt(M::CHANNEL.index()).cr1().modify(|w| w.set_rswstart(true));
    }

    /// Await the next regular conversion result.
    ///
    /// Does not start a conversion: the conversion must already be running,
    /// started by an external trigger, or started via
    /// [`start_and_read`](Self::start_and_read). Resolves with the next
    /// [`ResultRegular`] once a conversion completes.
    ///
    /// # Note
    /// A starved filter hangs forever: if the assigned transceiver produces no
    /// data (no modulator, dead clock, stalled source, or no trigger), no
    /// conversion completes and this future stays pending indefinitely. Detect
    /// starvation in layers:
    ///
    /// - the transceiver is borrowed for the filter's lifetime, so the source
    ///   cannot be dropped from under you (type system);
    /// - [`ClockAbsenceDetector`] flags a missing or failed source clock;
    /// - [`Error::Overrun`] is returned when data *is* arriving, faster than it
    ///   is read.
    pub async fn read(&mut self) -> Result<ResultRegular, Error> {
        poll_fn(|cx| {
            FilterRegs::<T, M>::set_regular_end_of_conversion_interrupt(false);
            FilterRegs::<T, M>::set_regular_overrun_interrupt(false);
            T::state().regular_waker.register(cx.waker());
            match self.try_get_result() {
                Ok(result) => Poll::Ready(Ok(result)),
                Err(Error::Overrun) => Poll::Ready(Err(Error::Overrun)),
                Err(Error::NotReady) => {
                    FilterRegs::<T, M>::set_regular_end_of_conversion_interrupt(true);
                    FilterRegs::<T, M>::set_regular_overrun_interrupt(true);
                    Poll::Pending
                }
                Err(_) => unreachable!("Other errors invalid"),
            }
        })
        .await
    }

    /// Start a regular conversion and await its result.
    ///
    /// Equivalent to [`start_conversion`](Self::start_conversion) followed by
    /// [`read`](Self::read): the read future waits for the conversion it just
    /// launched.
    pub async fn start_and_read(&mut self) -> Result<ResultRegular, Error> {
        self.start_conversion();
        self.read().await
    }

    /// Attempts to read the current regular conversion result.
    ///
    /// Returns [`ResultRegular`] if `REOCF` is set, [`Error::Overrun`] if an
    /// overrun occurred, or [`Error::NotReady`] if no conversion result is
    /// available.
    ///
    /// The conversion result is sign-extended from 24 to 32 bits and is not scaled.
    ///
    /// [`ResultRegular::pending`] is set if the regular conversion was delayed
    /// by an injected conversion.
    ///
    /// Reading the result clears the corresponding data register.
    pub fn try_get_result(&mut self) -> Result<ResultRegular, Error> {
        if self.get_and_clear_overrun() {
            return Err(Error::Overrun);
        } else if self.end_of_conversion() {
            return Ok(self.get_result_unchecked());
        }
        Err(Error::NotReady)
    }

    /// Reads and clears the current regular conversion result without checking
    /// `REOCF`.
    ///
    /// The conversion result is sign-extended from 24 to 32 bits and is not scaled.
    ///
    /// [`ResultRegular::pending`] is set if the regular conversion was delayed
    /// by an injected conversion.
    ///
    /// The returned data is only valid if `REOCF` was set before reading.
    ///
    /// # Note
    /// This path does not check or clear the overrun flag; use
    /// [`try_get_result`](Self::try_get_result) to propagate overruns.
    pub fn get_result_unchecked(&mut self) -> ResultRegular {
        let word = T::regs().flt(M::CHANNEL.index()).rdatar().read().0;
        ResultRegular::from_word(word)
    }

    /// Returns whether a regular conversion result is available.
    pub fn end_of_conversion(&self) -> bool {
        FilterRegs::<T, M>::end_of_regular_conversion()
    }

    /// Whether the regular overrun flag is set.
    pub fn overrun(&self) -> bool {
        FilterRegs::<T, M>::regular_overrun()
    }

    /// Clear the regular overrun flag.
    pub fn clear_overrun(&self) {
        FilterRegs::<T, M>::clear_regular_overrun();
    }

    /// Returns whether a regular conversion is currently in progress or pending.
    pub fn conversion_in_progress(&self) -> bool {
        FilterRegs::<T, M>::regular_conversion_in_progress()
    }

    /// Enables or disables continuous conversion mode.
    ///
    /// When enabled, the regular transceiver is converted repeatedly after each
    /// conversion request. Disabling it while a continuous conversion is in
    /// progress stops the conversion immediately.
    ///
    /// # Note
    /// Writing CR1 while continuous mode is enabled (RCONT=1) mid-conversion
    /// restarts the conversion.
    pub fn set_continuous(&mut self, enabled: bool) {
        FilterDisabled::<T, M>::set_continuous(enabled);
    }

    fn get_and_clear_overrun(&mut self) -> bool {
        let overrun = FilterRegs::<T, M>::regular_overrun();
        FilterRegs::<T, M>::clear_regular_overrun();
        overrun
    }
}

impl<'a, 'd, 't, T, M> FilterRegular<'a, 'd, 't, T, M, RegDma>
where
    T: Instance + FilterInterrupt<M>,
    M: FilterMarker + InstanceEvents<T>,
{
    /// Pointer to RDATAR, for custom DMA setups instead of `ring_buffered`.
    ///
    /// Reading clears the register, hence `&mut self`. Valid as long as the
    /// underlying `DfsdmCommon` stays enabled; no Rust lifetime ties to it.
    pub fn data_register(&mut self) -> *mut u32 {
        T::regs().flt(M::CHANNEL.index()).rdatar().as_ptr() as *mut u32
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
        let (slots, filterword) = Self::build_slots(transceivers);
        Self::set_channels(filterword);

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
    ///
    /// # Note
    /// Unlike the regular channel select, the injected select (JCHGR) takes
    /// effect immediately and resets any injected scan in progress.
    pub fn assign_transceivers<const N: usize>(&mut self, transceivers: [&'t dyn TransceiverTrait<T, Enabled>; N])
    where
        [(); N]: NonEmpty,
    {
        let (slots, filterword) = Self::build_slots(transceivers);
        Self::set_channels(filterword);
        self.injected = slots;
    }

    /// Builds the fixed-size injected-slot array plus the register bitmask
    /// from a caller-provided transceiver array of any lifetime.
    fn build_slots<'tcv, const N: usize>(
        transceivers: [&'tcv dyn TransceiverTrait<T, Enabled>; N],
    ) -> ([Option<&'tcv dyn TransceiverTrait<T, Enabled>>; 8], u8)
    where
        [(); N]: NonEmpty,
    {
        let filterword = filterword_of(&transceivers);

        let mut slots: [Option<&'tcv dyn TransceiverTrait<T, Enabled>>; 8] = [None; 8];
        for (i, tcv) in transceivers.iter().enumerate() {
            slots[i] = Some(*tcv);
        }

        (slots, filterword)
    }

    fn set_channels(channels: u8) {
        T::regs()
            .flt(M::CHANNEL.index())
            .jchgr()
            .write(|w| w.set_jchg(channels));
    }

    /// Start an injected conversion.
    ///
    /// # Note
    /// The request is ignored while an injected conversion is in progress
    /// (JCIP). An injected conversion preempts a running regular conversion
    /// (flagged via [`ResultRegular::pending`]).
    pub fn start_conversion(&mut self) {
        T::regs().flt(M::CHANNEL.index()).cr1().modify(|w| w.set_jswstart(true));
    }

    /// Await the next injected conversion result.
    ///
    /// Does not start a conversion: the conversion must already be running,
    /// started by an external trigger, or started via
    /// [`start_and_read`](Self::start_and_read). Resolves with the next
    /// [`ResultInjected`] once a conversion completes.
    ///
    /// # Note
    /// Like [`FilterRegular::read`], this hangs forever if the filter is
    /// starved (no data produced, or no trigger); see that method for the
    /// layered starvation detection.
    pub async fn read(&mut self) -> Result<ResultInjected, Error> {
        poll_fn(|cx| {
            FilterRegs::<T, M>::set_injected_end_of_conversion_interrupt(false);
            FilterRegs::<T, M>::set_injected_overrun_interrupt(false);

            T::state().injected_waker.register(cx.waker());
            match self.try_get_result() {
                Ok(result) => Poll::Ready(Ok(result)),
                Err(Error::Overrun) => Poll::Ready(Err(Error::Overrun)),
                Err(Error::NotReady) => {
                    FilterRegs::<T, M>::set_injected_end_of_conversion_interrupt(true);
                    FilterRegs::<T, M>::set_injected_overrun_interrupt(true);
                    Poll::Pending
                }
                Err(_) => unreachable!("Other errors invalid"),
            }
        })
        .await
    }

    /// Start an injected conversion and await its result.
    ///
    /// Equivalent to [`start_conversion`](Self::start_conversion) followed by
    /// [`read`](Self::read): the read future waits for the conversion it just
    /// launched.
    pub async fn start_and_read(&mut self) -> Result<ResultInjected, Error> {
        self.start_conversion();
        self.read().await
    }

    /// Attempts to read the current injected conversion result.
    ///
    /// Returns [`ResultInjected`] if `JEOCF` is set, [`Error::Overrun`] if an
    /// overrun occurred, or [`Error::NotReady`] if no conversion result is
    /// available.
    ///
    /// The conversion result is sign-extended from 24 to 32 bits and is not scaled.
    ///
    /// Reading the result clears the corresponding data register.
    pub fn try_get_result(&mut self) -> Result<ResultInjected, Error> {
        if self.get_and_clear_overrun() {
            return Err(Error::Overrun);
        } else if self.end_of_conversion() {
            return Ok(self.get_result_unchecked());
        }
        Err(Error::NotReady)
    }

    /// Reads and clears the current injected conversion result without checking
    /// `JEOCF`.
    ///
    /// The conversion result is sign-extended from 24 to 32 bits and is not scaled.
    ///
    /// The returned data is only valid if `JEOCF` was set before reading.
    ///
    /// # Note
    /// This path does not check or clear the overrun flag; use
    /// [`try_get_result`](Self::try_get_result) to propagate overruns.
    pub fn get_result_unchecked(&mut self) -> ResultInjected {
        let word = T::regs().flt(M::CHANNEL.index()).jdatar().read().0;
        ResultInjected::from_word(word)
    }

    /// Returns whether an injected conversion result is available.
    pub fn end_of_conversion(&self) -> bool {
        FilterRegs::<T, M>::end_of_injected_conversion()
    }

    /// Whether the injected overrun flag is set.
    pub fn overrun(&self) -> bool {
        FilterRegs::<T, M>::injected_overrun()
    }

    /// Clear the injected overrun flag.
    pub fn clear_overrun(&self) {
        FilterRegs::<T, M>::clear_injected_overun()
    }

    /// Returns whether an injected conversion is currently in progress or pending.
    pub fn conversion_in_progress(&self) -> bool {
        FilterRegs::<T, M>::injected_conversion_in_progress()
    }

    fn get_and_clear_overrun(&mut self) -> bool {
        let overrun = FilterRegs::<T, M>::injected_overrun();
        FilterRegs::<T, M>::clear_injected_overun();
        overrun
    }
}

impl<'a, 'd, 't, T, M> FilterInjected<'a, 'd, 't, T, M, InjDma>
where
    T: Instance + FilterInterrupt<M>,
    M: FilterMarker + InstanceEvents<T>,
{
    /// Pointer to JDATAR, for custom DMA setups instead of `ring_buffered`.
    ///
    /// Reading clears the register, hence `&mut self`. Valid as long as the
    /// underlying `DfsdmCommon` stays enabled; no Rust lifetime ties to it.
    pub fn data_register(&mut self) -> *mut u32 {
        T::regs().flt(M::CHANNEL.index()).jdatar().as_ptr() as *mut u32
    }
}

impl<'a, 'd, 't, T, M> FilterDma<T, M> for FilterRegular<'a, 'd, 't, T, M, RegDma>
where
    T: Instance + FilterInterrupt<M>,
    M: FilterMarker + InstanceEvents<T>,
{
    fn data_register(&mut self) -> *mut u32 {
        FilterRegular::data_register(self)
    }

    fn start_conversion(&mut self) {
        FilterRegular::start_conversion(self);
    }

    fn get_and_clear_overrun(&mut self) -> bool {
        FilterRegular::get_and_clear_overrun(self)
    }
}

impl<'a, 'd, 't, T, M> FilterDma<T, M> for FilterInjected<'a, 'd, 't, T, M, InjDma>
where
    T: Instance + FilterInterrupt<M>,
    M: FilterMarker + InstanceEvents<T>,
{
    fn data_register(&mut self) -> *mut u32 {
        FilterInjected::data_register(self)
    }

    fn start_conversion(&mut self) {
        FilterInjected::start_conversion(self);
    }

    fn get_and_clear_overrun(&mut self) -> bool {
        FilterInjected::get_and_clear_overrun(self)
    }
}

pub(crate) struct FilterRegs<T, M>(PhantomData<(T, M)>);

impl<T, M> FilterRegs<T, M>
where
    T: Instance + FilterInterrupt<M>,
    M: FilterMarker + InstanceEvents<T>,
{
    /// Enables or disables the filter (DFEN).
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

    pub(crate) fn regular_conversion_in_progress() -> bool {
        T::regs().flt(M::CHANNEL.index()).isr().read().rcip()
    }

    pub(crate) fn injected_conversion_in_progress() -> bool {
        T::regs().flt(M::CHANNEL.index()).isr().read().jcip()
    }

    /// Enables or disables regular overrun interrupts.
    pub(crate) fn set_regular_overrun_interrupt(enabled: bool) {
        // RMW'd from both ISR and thread (the ISR clears its own IE here) - cs is load-bearing.
        critical_section::with(|_cs| {
            T::regs()
                .flt(M::CHANNEL.index())
                .cr2()
                .modify(|w| w.set_rovrie(enabled));
        });
    }

    /// Enables or disables injected overrun interrupts.
    pub(crate) fn set_injected_overrun_interrupt(enabled: bool) {
        // RMW'd from both ISR and thread (the ISR clears its own IE here) - cs is load-bearing.
        critical_section::with(|_cs| {
            T::regs()
                .flt(M::CHANNEL.index())
                .cr2()
                .modify(|w| w.set_jovrie(enabled));
        });
    }

    pub(crate) fn regular_overrun() -> bool {
        T::regs().flt(M::CHANNEL.index()).isr().read().rovrf()
    }

    pub(crate) fn injected_overrun() -> bool {
        T::regs().flt(M::CHANNEL.index()).isr().read().jovrf()
    }

    pub(crate) fn clear_regular_overrun() {
        T::regs().flt(M::CHANNEL.index()).icr().modify(|w| w.set_clrrovrf(true));
    }

    pub(crate) fn clear_injected_overun() {
        T::regs().flt(M::CHANNEL.index()).icr().modify(|w| w.set_clrjovrf(true));
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
        if ShortCircuitDetector::<T>::channel_flags_masked() != 0u8 {
            ShortCircuitDetector::<T>::set_interrupt_enable(false);
            T::instance_state().short_circuit_waker.wake();
        }
        if ClockAbsenceDetector::<T>::channel_flags_masked() != 0u8 {
            ClockAbsenceDetector::<T>::set_interrupt_enable(false);
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
        if FilterRegs::<T, F>::end_of_injected_conversion() || FilterRegs::<T, F>::injected_overrun() {
            FilterRegs::<T, F>::set_injected_end_of_conversion_interrupt(false);
            FilterRegs::<T, F>::set_injected_overrun_interrupt(false);
            <T as FilterInterrupt<F>>::state().injected_waker.wake();
        }
        if FilterRegs::<T, F>::end_of_regular_conversion() || FilterRegs::<T, F>::regular_overrun() {
            FilterRegs::<T, F>::set_regular_end_of_conversion_interrupt(false);
            FilterRegs::<T, F>::set_regular_overrun_interrupt(false);
            <T as FilterInterrupt<F>>::state().regular_waker.wake();
        }
        if AnalogWatchdog::<T, F>::triggered() {
            AnalogWatchdog::<T, F>::set_interrupt_enable(false);
            <T as FilterInterrupt<F>>::state().watchdog_waker.wake();
        }

        // Instance logic (compiled out for Flt1..7)
        F::handle_instance_events();
    }
}

/// Builder for a [`Filter`]; binds it to `DfsdmCommon` on `build`.
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
        // 1. The `_irqs: impl Binding<...>` argument proves (at compile time) that
        //    `InterruptHandler<T, M>::on_interrupt` is wired to this IRQ line.
        // 2. The waker is initialized in `State::new()` (const, in a static) before
        //    any interrupt can fire.
        // 3. The NVIC unmask here is independent of the peripheral IE bits:
        //    no filter-level IE (REOCIE/JEOCIE/AWDIE/...) is set at this call site
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

// =============================================================================
// Interrupthandler
// =============================================================================

/// Per-filter interrupt binding: maps a filter marker to its interrupt type
/// and state.
pub trait FilterInterrupt<F: FilterMarker> {
    /// Interrupt type for this filter.
    type Interrupt: interrupt::typelevel::Interrupt;

    /// Filter-interrupt state.
    fn state() -> &'static State;
}
/// Instance-level interrupt handling for a filter marker. [`Flt0`] performs the
/// real handling; the other markers yield no-ops.
pub trait InstanceEvents<T: Instance> {
    /// Handles the instance-level events.
    ///
    /// # Safety
    ///
    /// Caller must be the interrupt handler for this filter.
    unsafe fn handle_instance_events();
}

// Implements empty InstanceEvents for provided Markers
macro_rules! impl_noop_instance_events {
    ($($flt:ident),*) => {
        $(
            impl<T: Instance> InstanceEvents<T> for $flt {
                #[inline(always)]
                unsafe fn handle_instance_events() {}
            }
        )*
    };
}

// Implement empty for all non Flt0 as Flt0 Handles instance-level events
impl_noop_instance_events!(Flt1, Flt2, Flt3, Flt4, Flt5, Flt6, Flt7);

// Ready bundles: one per filter-count capability, bundling the
// `FilterInterrupt<FltN>` chain for all filters the shape has. The chain
// matches the unconditional `foreach_interrupt!` binding in associations.rs.
macro_rules! define_dfsdm_ready {
    ($name:ident, [$($flt:ident),+ $(,)?]) => {
        /// IRQ readiness bundle: all the `FilterInterrupt`s a filter-count
        /// capability implies. Blanket-implemented for every `T` meeting
        /// the chain.
        pub trait $name: $(FilterInterrupt<$flt> +)* {}
        impl<T> $name for T where T: $(FilterInterrupt<$flt> +)* {}
    };
}

define_dfsdm_ready!(Flt1Ready, [Flt0]);
define_dfsdm_ready!(Flt2Ready, [Flt0, Flt1]);
define_dfsdm_ready!(Flt4Ready, [Flt0, Flt1, Flt2, Flt3]);
define_dfsdm_ready!(Flt6Ready, [Flt0, Flt1, Flt2, Flt3, Flt4, Flt5]);
define_dfsdm_ready!(Flt8Ready, [Flt0, Flt1, Flt2, Flt3, Flt4, Flt5, Flt6, Flt7]);

// =============================================================================
// Interrupt/FilterChannel state
// =============================================================================

/// State shared between a filter's interrupt handler and its filter object.
#[derive(Default)]
pub struct State {
    /// Waker for the injected requests
    pub injected_waker: AtomicWaker,
    /// Waker for the regular requests
    pub regular_waker: AtomicWaker,
    /// Waker for analog watchdog events
    pub watchdog_waker: AtomicWaker,
}

impl State {
    /// Instantiate a fresh `State`.
    ///
    /// This is `const` so it can initialize `static STATE` (see
    /// [`InstanceState::new`]); use [`Default`] where a non-`const` value is
    /// more convenient.
    pub const fn new() -> Self {
        Self {
            injected_waker: AtomicWaker::new(),
            regular_waker: AtomicWaker::new(),
            watchdog_waker: AtomicWaker::new(),
        }
    }
}

/// State shared between an instance's interrupt handler and its detectors.
#[derive(Default)]
pub struct InstanceState {
    /// Bitmask of transceivers whose short-circuit detector the driver has armed
    /// (aggregate SCDEN mirror; driver is the sole writer).
    pub short_circuit_armed: AtomicU8,

    /// Waker for short-circuit-detector events
    pub short_circuit_waker: AtomicWaker,

    /// Bitmask of transceivers whose clock-absence detector the driver has armed
    /// (aggregate CKABEN mirror; driver is the sole writer).
    pub clock_absence_armed: AtomicU8,

    /// Waker for clock-absence-detector events
    pub clock_absence_waker: AtomicWaker,
}

impl InstanceState {
    /// Instantiate a fresh `InstanceState`.
    ///
    /// This is `const` so it can initialize `static INSTANCE_STATE` (emitted
    /// by build.rs); use [`Default`] where a non-`const` value is more
    /// convenient.
    pub const fn new() -> Self {
        Self {
            short_circuit_armed: AtomicU8::new(0),
            short_circuit_waker: AtomicWaker::new(),
            clock_absence_armed: AtomicU8::new(0),
            clock_absence_waker: AtomicWaker::new(),
        }
    }
}

// =============================================================================
// Associate interrupts
// =============================================================================

// Implement single IRQ for a single filter
macro_rules! impl_dfsdm_filter_irq {
    ($inst:ident, $filter:ty, $irq:ident) => {
        impl FilterInterrupt<$filter> for crate::peripherals::$inst {
            type Interrupt = crate::interrupt::typelevel::$irq;

            fn state() -> &'static State {
                static STATE: State = State::new();
                &STATE
            }
        }
    };
}

// Bind every filter interrupt the chip actually exposes. The `foreach_interrupt!`
// catch-all skips FLTx rows a given chip doesn't have, so the filter count is
// irrelevant here - no per-variant dispatch.
foreach_interrupt! {
    ($inst:ident, dfsdm, $variant:ident, FLT0, $irq:ident) => {
        impl_dfsdm_filter_irq!($inst, Flt0, $irq);
    };
    ($inst:ident, dfsdm, $variant:ident, FLT1, $irq:ident) => {
        impl_dfsdm_filter_irq!($inst, Flt1, $irq);
    };
    ($inst:ident, dfsdm, $variant:ident, FLT2, $irq:ident) => {
        impl_dfsdm_filter_irq!($inst, Flt2, $irq);
    };
    ($inst:ident, dfsdm, $variant:ident, FLT3, $irq:ident) => {
        impl_dfsdm_filter_irq!($inst, Flt3, $irq);
    };
    ($inst:ident, dfsdm, $variant:ident, FLT4, $irq:ident) => {
        impl_dfsdm_filter_irq!($inst, Flt4, $irq);
    };
    ($inst:ident, dfsdm, $variant:ident, FLT5, $irq:ident) => {
        impl_dfsdm_filter_irq!($inst, Flt5, $irq);
    };
    ($inst:ident, dfsdm, $variant:ident, FLT6, $irq:ident) => {
        impl_dfsdm_filter_irq!($inst, Flt6, $irq);
    };
    ($inst:ident, dfsdm, $variant:ident, FLT7, $irq:ident) => {
        impl_dfsdm_filter_irq!($inst, Flt7, $irq);
    };
}

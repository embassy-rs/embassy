//! Instance-level and filter-level detectors: the analog watchdog, extremes,
//! short-circuit and clock-absence detectors.

use super::*;
use crate::dfsdm::filter::sign_extend_24;

/// Builds the [`Detectors`] pair.
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
    /// Creates a new detectors builder.
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
        // 1. The `_irqs: impl Binding<...>` argument proves (at compile time) that
        //    `InterruptHandler<T, Flt0>::on_interrupt` is wired to this IRQ line.
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
            <T as FilterInterrupt<Flt0>>::Interrupt::enable();
        }
        Detectors::new(common)
    }
}

// =============================================================================
// Interrupt/Event accessors filter
// =============================================================================

/// Analog watchdog configuration.
pub struct AnalogWatchdogConfig {
    /// Compare against the analog watchdog filter's output instead of the main
    /// filter output (AWFSEL).
    pub fastmode: bool,
    /// Break signals armed on the low threshold.
    pub low_break_signals: config::BreakSignals,
    /// Break signals armed on the high threshold.
    pub high_break_signals: config::BreakSignals,
    /// Low threshold.
    pub low_threshold: i32,
    /// High threshold.
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

/// Analog watchdog event.
pub enum AnalogWatchdogEvent {
    /// High threshold exceeded
    HighThreshold {
        /// bitmap of triggering transceivers.
        transceivers: u8,
    },
    /// Low threshold exceeded
    LowThreshold {
        /// bitmap of triggering transceivers.
        transceivers: u8,
    },
}
/// Analog watchdog of a filter.
pub struct AnalogWatchdog<'a, 'd, T, M>
where
    T: Instance + FilterInterrupt<M>,
    M: FilterMarker + InstanceEvents<T>,
{
    _instance_marker: PhantomData<(T, M)>,
    /// Keeps the [`DfsdmCommon`] borrow alive for `'a`.
    _common: PhantomData<&'a DfsdmCommon<'d, T, Enabled>>,
}

impl<'a, 'd, T, M> AnalogWatchdog<'a, 'd, T, M>
where
    T: Instance + FilterInterrupt<M>,
    M: FilterMarker + InstanceEvents<T>,
{
    pub(crate) fn new(_common: &'a DfsdmCommon<'d, T, Enabled>) -> Self {
        let mut new = Self {
            _instance_marker: PhantomData,
            _common: PhantomData,
        };

        new.configure(AnalogWatchdogConfig::default());
        new
    }

    /// Wait for an analog watchdog event.
    pub async fn wait_for_event(&mut self) -> AnalogWatchdogEvent {
        poll_fn(|cx| {
            Self::set_interrupt_enable(false);
            T::state().watchdog_waker.register(cx.waker());

            let high = Self::high_channels();
            let low = Self::low_channels();

            if high != 0 {
                Self::clear_high(high);
                return Poll::Ready(AnalogWatchdogEvent::HighThreshold { transceivers: high });
            }
            if low != 0 {
                Self::clear_low(low);
                return Poll::Ready(AnalogWatchdogEvent::LowThreshold { transceivers: low });
            }

            Self::set_interrupt_enable(true);
            Poll::Pending
        })
        .await
    }

    /// Apply a full configuration.
    pub fn configure(&mut self, config: AnalogWatchdogConfig) {
        self.enable_analog_watchdog_fastmode(config.fastmode);
        self.assign_low_to_break_signals(config.low_break_signals);
        self.assign_high_to_break_signals(config.high_break_signals);
        self.set_low_threshold(config.low_threshold);
        self.set_high_threshold(config.high_threshold);
    }

    /// Set the high threshold.
    pub fn set_high_threshold(&mut self, threshold: i32) {
        T::regs()
            .flt(M::CHANNEL.index())
            .awhtr()
            .modify(|w| w.set_awht(threshold as u32));
    }

    /// Set the low threshold.
    pub fn set_low_threshold(&mut self, threshold: i32) {
        T::regs()
            .flt(M::CHANNEL.index())
            .awltr()
            .modify(|w| w.set_awlt(threshold as u32));
    }

    /// Assign break signals to fire on the high threshold.
    ///
    /// # Note
    /// This routes a watchdog event to a DFSDM break wire (BKAWH); the
    /// receiving timer must separately map that wire to a break input (BRK).
    pub fn assign_high_to_break_signals(&mut self, break_signals: config::BreakSignals) {
        T::regs()
            .flt(M::CHANNEL.index())
            .awhtr()
            .modify(|w| w.set_bkawh(break_signals.bits()));
    }

    /// Assign break signals to fire on the low threshold.
    ///
    /// # Note
    /// This routes a watchdog event to a DFSDM break wire (BKAWL); the
    /// receiving timer must separately map that wire to a break input (BRK).
    pub fn assign_low_to_break_signals(&mut self, break_signals: config::BreakSignals) {
        T::regs()
            .flt(M::CHANNEL.index())
            .awltr()
            .modify(|w| w.set_bkawl(break_signals.bits()));
    }

    /// Enable or disable the watchdog filter as the comparison source (AWFSEL).
    ///
    /// # Note
    /// AWFSEL is per-channel and only meaningful in fast mode, where the
    /// watchdog compares against its own fast filter instead of the main filter
    /// output.
    pub fn enable_analog_watchdog_fastmode(&mut self, enabled: bool) {
        T::regs()
            .flt(M::CHANNEL.index())
            .cr1()
            .modify(|w| w.set_awfsel(enabled));
    }

    /// Assign provided transceivers to this analog watchdog
    pub fn assign_transceivers<const N: usize>(
        &mut self,
        // No borrow lifetime here as watchdog events are not awaited when the transceiver is off.
        // They're "errors", not results that waiting for might stall your program.
        transceivers: [&dyn TransceiverTrait<T, Enabled>; N],
    ) where
        [(); N]: NonEmpty,
    {
        let filterword = filterword_of(&transceivers);

        // thread-only writes, but full-register RMW on CR2 competes with the ISR's IE RMW - same cs discipline.
        critical_section::with(|_cs| {
            T::regs()
                .flt(M::CHANNEL.index())
                .cr2()
                .modify(|w| w.set_awdch(filterword));
        });
    }

    /// Whether the low-threshold flag is set for `channel`.
    pub fn channel_flag_low(&self, channel: TransceiverChannel) -> bool {
        self.flags_low().get_bit(channel.index())
    }

    /// Whether the high-threshold flag is set for `channel`.
    pub fn channel_flag_high(&self, channel: TransceiverChannel) -> bool {
        self.flags_high().get_bit(channel.index())
    }

    /// Low-threshold flag bitmap.
    pub fn flags_low(&self) -> u8 {
        Self::low_channels()
    }

    /// High-threshold flag bitmap.
    pub fn flags_high(&self) -> u8 {
        Self::high_channels()
    }

    /// Clear the low-threshold flag for `channel`.
    pub fn clear_channel_flags_low(&mut self, channel: TransceiverChannel) {
        Self::clear_low(1 << channel.index());
    }

    /// Clear the high-threshold flag for `channel`.
    pub fn clear_channel_flags_high(&mut self, channel: TransceiverChannel) {
        Self::clear_high(1 << channel.index());
    }

    /// Clear all low-threshold flags.
    pub fn clear_flags_low(&mut self) {
        Self::clear_low(0xFF);
    }

    /// Clear all high-threshold flags.
    pub fn clear_flags_high(&mut self) {
        Self::clear_high(0xFF);
    }

    /// Enables or disables analog watchdog interrupts.
    pub(crate) fn set_interrupt_enable(enabled: bool) {
        // RMW'd from both ISR and thread (the ISR clears its own IE here) - cs is load-bearing.
        critical_section::with(|_cs| {
            T::regs().flt(M::CHANNEL.index()).cr2().modify(|w| w.set_awdie(enabled));
        });
    }

    /// Returns whether the analog watchdog has been triggerd
    pub(crate) fn triggered() -> bool {
        T::regs().flt(M::CHANNEL.index()).isr().read().awdf()
    }

    /// Returns bitmap of channels who triggered the high threshold
    pub(crate) fn high_channels() -> u8 {
        T::regs().flt(M::CHANNEL.index()).awsr().read().awhtf()
    }

    /// Returns bitmap of channels who triggered the low threshold
    pub(crate) fn low_channels() -> u8 {
        T::regs().flt(M::CHANNEL.index()).awsr().read().awltf()
    }

    /// Clears the provided channels' analog watchdog flags
    pub(crate) fn clear_high(channels: u8) {
        T::regs()
            .flt(M::CHANNEL.index())
            .awsr()
            .modify(|w| w.set_awhtf(channels));
    }

    /// Clears the provided channels' analog watchdog flags
    pub(crate) fn clear_low(channels: u8) {
        T::regs()
            .flt(M::CHANNEL.index())
            .awsr()
            .modify(|w| w.set_awltf(channels));
    }
}

/// Extremes result.
pub struct ResultExtreme {
    /// Sign-extended 24-bit extreme value.
    pub data: i32,
    /// Transceiver it came from.
    pub channel: u8,
}

/// Extremes (min/max) detector of a filter.
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
        // No borrow lifetime here as watchdog events are not awaited when the transceiver is off.
        // They're "errors", not results that waiting for might stall your program.
        transceivers: [&dyn TransceiverTrait<T, Enabled>; N],
    ) where
        [(); N]: NonEmpty,
    {
        let filterword = filterword_of(&transceivers);

        // thread-only writes, but full-register RMW on CR2 competes with the ISR's IE RMW - same cs discipline.
        critical_section::with(|_cs| {
            T::regs()
                .flt(M::CHANNEL.index())
                .cr2()
                .modify(|w| w.set_exch(filterword));
        });
    }

    /// Reads the extremes detector maximum value and its corresponding channel.
    /// Reading this resets the register value to `0x800000` and clears the
    /// channel field.
    pub fn read_maxima(&mut self) -> ResultExtreme {
        let exmax = T::regs().flt(M::CHANNEL.index()).exmax().read();
        ResultExtreme {
            channel: exmax.exmaxch(),
            data: sign_extend_24(exmax.exmax()),
        }
    }

    /// Reads the extremes detector minimum value and its corresponding channel.
    /// Reading this resets the register value to `0x7FFFFF` and clears the
    /// channel field.
    pub fn read_minima(&mut self) -> ResultExtreme {
        let exmin = T::regs().flt(M::CHANNEL.index()).exmin().read();
        ResultExtreme {
            channel: exmin.exminch(),
            data: sign_extend_24(exmin.exmin()),
        }
    }
}

// =============================================================================
// Interrupt/Event accessors instance
// =============================================================================

/// Instance-level detectors.
pub struct Detectors<'a, 'd, T>
where
    T: Instance + FilterInterrupt<Flt0>,
{
    /// Short-circuit detector.
    pub short_circuit: ShortCircuitDetector<'a, 'd, T>,
    /// Clock-absence detector.
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
    /// Create an assignment: a transceiver and its short-circuit threshold.
    pub const fn new(transceiver: &'t dyn TransceiverTrait<T, Enabled>, threshold: u8) -> Self {
        Self { transceiver, threshold }
    }
}

/// Short-circuit detector: flags when a transceiver's input stays constant
/// (consecutive identical bits) for longer than the threshold, indicating a
/// stuck or short-circuited input.
pub struct ShortCircuitDetector<'a, 'd, T>
where
    T: Instance,
{
    /// Keeps the [`DfsdmCommon`] borrow alive for `'a`.
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
            Self::set_interrupt_enable(false);
            T::instance_state().short_circuit_waker.register(cx.waker());

            let channels = Self::channel_flags_masked();
            if channels != 0 {
                Self::clear_channels(channels);
                return Poll::Ready(channels);
            }

            Self::set_interrupt_enable(true);
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

        Self::set_channels(filterword_of(&tcv));
    }

    /// Unassigns the transceivers from the short-circuit-detector
    pub fn unassign_transceivers<const N: usize>(&mut self, transceivers: [&dyn TransceiverTrait<T, Enabled>; N])
    where
        [(); N]: NonEmpty,
    {
        Self::set_channels(Self::channel_word() & !filterword_of(&transceivers));
    }

    /// Assign break-signals for short-circuit-event of transceiver
    ///
    /// # Note
    /// This routes a short-circuit event to a DFSDM break wire (BKSCD); the
    /// receiving timer must separately map that wire to a break input (BRK).
    pub fn assign_break_signals(
        &mut self,
        transceiver: &dyn TransceiverTrait<T, Enabled>,
        signals: config::BreakSignals,
    ) {
        T::regs()
            .ch(transceiver.index())
            .awscdr()
            .modify(|w| w.set_bkscd(signals.bits()));
    }

    /// Set the short-circuit threshold for a transceiver.
    pub fn set_threshold(&mut self, transceiver: &dyn TransceiverTrait<T, Enabled>, threshold: u8) {
        T::regs()
            .ch(transceiver.index())
            .awscdr()
            .modify(|w| w.set_scdt(threshold));
    }

    /// Whether the short-circuit flag is set for `channel`.
    pub fn channel_flag(&self, channel: TransceiverChannel) -> bool {
        self.flags().get_bit(channel.index())
    }

    /// Clear the short-circuit flag for `channel`.
    pub fn clear_channel_flags(&mut self, channel: TransceiverChannel) {
        Self::clear_channels(1 << channel.index());
    }

    /// Short-circuit flag bitmap.
    pub fn flags(&self) -> u8 {
        Self::channel_flags_masked()
    }

    /// Clear all pending detector flags for currently armed channels.
    ///
    /// Use this after assigning transceivers to clear any startup residue
    /// before waiting for events.
    pub fn clear_flags(&mut self) {
        let armed = T::instance_state().short_circuit_armed.load(Ordering::Relaxed);
        Self::clear_channels(armed);
    }

    pub(crate) fn drop_transceiver(channel: TransceiverChannel) {
        let ch = channel.index();
        Self::set_channels(*Self::channel_word().set_bit(ch, false));
    }

    /// Aggregate CFGR1 bit-word for one detector kind, over the channels this
    /// instance actually has.
    fn channel_word() -> u8 {
        let count = <T::Transceivers as capability::TransceiverCount>::COUNT;
        (0..count).fold(0u8, |mut acc, y| {
            acc.set_bit(y as usize, T::regs().ch(y as usize).cfgr1().read().scden());
            acc
        })
    }

    /// Authority: make the registers match `mask` exactly, then refresh the armed cache.
    fn set_channels(mask: u8) {
        let mask = mask & channel_count_mask::<T>();
        for y in 0..<T::Transceivers as capability::TransceiverCount>::COUNT {
            let want = mask.get_bit(y as usize);
            T::regs().ch(y as usize).cfgr1().modify(|w| w.set_scden(want));
        }
        T::instance_state().short_circuit_armed.store(mask, Ordering::Relaxed);
    }

    /// Enables or disables short-circuit detector interrupts.
    pub(crate) fn set_interrupt_enable(enabled: bool) {
        // RMW'd from both ISR and thread (the ISR clears its own IE here) - cs is load-bearing.
        critical_section::with(|_cs| {
            T::regs().flt(0).cr2().modify(|w| w.set_scdie(enabled));
        });
    }

    /// Returns bitmap of channels who triggered the short-circuit-detector
    pub(crate) fn channel_flags() -> u8 {
        T::regs().flt(0).isr().read().scdf()
    }

    /// Returns bitmap of channels who triggered the short-circuit-detector and are armed
    pub(crate) fn channel_flags_masked() -> u8 {
        Self::channel_flags() & T::instance_state().short_circuit_armed.load(Ordering::Relaxed)
    }

    /// Clears the provided channel flags in the short-circuit-detector
    pub(crate) fn clear_channels(channels: u8) {
        T::regs().flt(0).icr().modify(|w| w.set_clrscdf(channels));
    }
}

/// Clock-absence detector.
pub struct ClockAbsenceDetector<'a, 'd, T>
where
    T: Instance,
{
    /// Keeps the [`DfsdmCommon`] borrow alive for `'a`.
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
            Self::set_interrupt_enable(false);
            T::instance_state().clock_absence_waker.register(cx.waker());

            let channels = Self::channel_flags_masked();

            if channels != 0 {
                Self::clear_channels(channels);
                return Poll::Ready(channels);
            }

            Self::set_interrupt_enable(true);
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
        Self::set_channels(filterword_of(&transceivers));
    }

    /// Unassigns the transceivers from the clock-absence-detector
    pub fn unassign_transceivers<const N: usize>(&mut self, transceivers: [&dyn TransceiverTrait<T, Enabled>; N])
    where
        [(); N]: NonEmpty,
    {
        Self::set_channels(Self::channel_word() & !filterword_of(&transceivers));
    }

    /// Whether the clock-absence flag is set for `channel`.
    pub fn channel_flag(&self, channel: TransceiverChannel) -> bool {
        self.flags().get_bit(channel.index())
    }

    /// Clear the clock-absence flag for `channel`.
    pub fn clear_channel_flags(&mut self, channel: TransceiverChannel) {
        Self::clear_channels(1 << channel.index());
    }

    /// Clock-absence flag bitmap.
    ///
    /// # Note
    /// The raw CKABF bits are held set while a channel is disabled or not yet
    /// synchronized, so they are masked here against the armed channel set.
    /// Call [`clear_flags`](Self::clear_flags) after assigning transceivers to
    /// drop startup residue.
    pub fn flags(&self) -> u8 {
        Self::channel_flags_masked()
    }

    /// Clear all pending detector flags for currently armed channels.
    ///
    /// Use this after assigning transceivers to clear any startup residue
    /// before waiting for events.
    pub fn clear_flags(&mut self) {
        let armed = T::instance_state().clock_absence_armed.load(Ordering::Relaxed);
        Self::clear_channels(armed);
    }

    pub(crate) fn drop_transceiver(channel: TransceiverChannel) {
        let ch = channel.index();
        Self::set_channels(*Self::channel_word().set_bit(ch, false));
    }

    /// Aggregate CFGR1 bit-word for one detector kind, over the channels this
    /// instance actually has.
    fn channel_word() -> u8 {
        let count = <T::Transceivers as capability::TransceiverCount>::COUNT;

        (0..count).fold(0u8, |mut acc, y| {
            acc.set_bit(y as usize, T::regs().ch(y as usize).cfgr1().read().ckaben());
            acc
        })
    }

    /// Authority: make the registers match `mask` exactly, then refresh the armed cache.
    fn set_channels(mask: u8) {
        let mask = mask & channel_count_mask::<T>();
        for y in 0..<T::Transceivers as capability::TransceiverCount>::COUNT {
            let want = mask.get_bit(y as usize);
            T::regs().ch(y as usize).cfgr1().modify(|w| w.set_ckaben(want));
        }
        T::instance_state().clock_absence_armed.store(mask, Ordering::Relaxed);
    }

    /// Enables or disables clock absence interrupts.
    pub(crate) fn set_interrupt_enable(enabled: bool) {
        // RMW'd from both ISR and thread (the ISR clears its own IE here) - cs is load-bearing.
        critical_section::with(|_cs| {
            T::regs().flt(0).cr2().modify(|w| w.set_ckabie(enabled));
        });
    }

    /// Returns bitmap of channels who triggered the clock-absence-detector
    pub(crate) fn channel_flags() -> u8 {
        T::regs().flt(0).isr().read().ckabf()
    }

    /// Returns bitmap of channels who triggered the clock-absence-detector and are armed
    pub(crate) fn channel_flags_masked() -> u8 {
        Self::channel_flags() & T::instance_state().clock_absence_armed.load(Ordering::Relaxed)
    }

    /// Try to clear the repective channels flag
    pub(crate) fn try_clear_channel_flag(channel: TransceiverChannel) -> bool {
        Self::clear_channels(1 << channel.index());
        !Self::channel_flags().get_bit(channel.index())
    }

    /// Clears the provided channel flags in the clock-absence-detector
    pub(crate) fn clear_channels(channels: u8) {
        T::regs().flt(0).icr().modify(|w| w.set_clrckabf(channels));
    }
}

/// filterword fold over a transceiver slice
pub(crate) fn filterword_of<T: Instance>(transceivers: &[&dyn TransceiverTrait<T, Enabled>]) -> u8 {
    transceivers.iter().fold(0u8, |mut acc, tcv| {
        acc.set_bit(tcv.index(), true);
        acc
    })
}

/// Valid-bits mask for this shape; every mask user passes gets intersected with it.
pub(crate) const fn channel_count_mask<T: Instance>() -> u8 {
    let count = <T::Transceivers as capability::TransceiverCount>::COUNT;
    ((1u16 << count) - 1) as u8
}

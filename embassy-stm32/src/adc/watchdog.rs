//! Analog watchdogs.

use core::future::poll_fn;
use core::marker::PhantomData;
use core::sync::atomic::{Ordering, compiler_fence};
use core::task::Poll;

use super::{Adc, AdcChannel, AdcRegs, BorrowedChannel, ConversionMode, Instance, SampleTimeOf};
use crate::atomic::AtomicClear;
use crate::mode::{Async, Mode};

/// Select which of the hardware analog watchdogs to use.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum WatchdogIndex {
    /// First analog watchdog (AWD1), available on every ADC.
    ///
    /// Monitors [`WatchdogChannels::All`] or [`WatchdogChannels::Single`].
    Awd1,
    /// Second analog watchdog (AWD2).
    ///
    /// Monitors [`WatchdogChannels::Single`] or [`WatchdogChannels::Channels`].
    Awd2,
    /// Third analog watchdog (AWD3).
    ///
    /// Monitors [`WatchdogChannels::Single`] or [`WatchdogChannels::Channels`].
    Awd3,
}

impl WatchdogIndex {
    pub(crate) fn index(self) -> usize {
        match self {
            WatchdogIndex::Awd1 => 0,
            WatchdogIndex::Awd2 => 1,
            WatchdogIndex::Awd3 => 2,
        }
    }
}

/// Channel selection passed into [`Adc::enable_watchdog`].
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum WatchdogChannels {
    /// Monitor all channels in the regular (and injected) sequence.
    ///
    /// Only valid with [`WatchdogIndex::Awd1`].
    All,
    /// Monitor a single specific channel.
    ///
    /// Valid for all watchdogs.
    Single(u8),
    /// Monitor a bitmask of channels (bit N = channel N).
    ///
    /// Only valid with [`WatchdogIndex::Awd2`] and [`WatchdogIndex::Awd3`].
    Channels(u32),
}

impl WatchdogChannels {
    /// Monitor `channel`.
    pub fn from_channel<'d, T>(channel: &impl AdcChannel<'d, T>) -> Self {
        Self::Single(channel.channel())
    }

    /// Also monitor `channel`.
    pub fn add_channel<'d, T>(self, channel: &impl AdcChannel<'d, T>) -> Self {
        WatchdogChannels::Channels(
            (match self {
                WatchdogChannels::All => panic!("cannot add channels to `All`"),
                WatchdogChannels::Single(ch) => 1 << ch,
                WatchdogChannels::Channels(ch) => ch,
            }) | 1 << channel.channel(),
        )
    }
}

/// A driver for an ADC analog watchdog.
///
/// Created by [`Adc::enable_watchdog`]. Does **not** borrow the [`Adc`]: you may hold this guard
/// while performing DMA or other ADC operations concurrently and call [`Self::wait`] to detect
/// when a monitored channel leaves the threshold window.
///
/// For self-contained single-pin monitoring that drives its own continuous conversion, use
/// [`Self::monitor`], which temporarily borrows the [`Adc`].
///
/// Dropping the guard disables the watchdog and its interrupt.
pub struct AnalogWatchdog<T: Instance, M: Mode> {
    index: usize,
    /// True when [`Self::monitor`] started a continuous conversion that must be stopped in Drop.
    stop_on_drop: bool,
    _marker: PhantomData<(T, M)>,
}

impl<'d, T: Instance, M: Mode> Adc<'d, T, M> {
    /// Enable an analog watchdog and return a guard.
    ///
    /// `watchdog` selects which of the hardware watchdogs to use. `channels` controls which ADC
    /// channels are monitored; see [`WatchdogChannels`] for which variants are valid for each
    /// watchdog. `low_threshold` and `high_threshold` are raw ADC counts in the **same space as
    /// the data register** for the currently configured resolution and oversampling. The
    /// watchdog fires when a sample falls **outside** `[low_threshold, high_threshold]`.
    ///
    /// Watchdogs 2 and 3 of 12-bit ADCs only compare the 8 most significant bits of a sample.
    ///
    /// Call [`AnalogWatchdog::wait`] to detect threshold crossings concurrently with other
    /// conversions, or [`AnalogWatchdog::monitor`] for self-contained single-pin monitoring
    /// (which temporarily borrows the ADC).
    ///
    /// # Panics
    ///
    /// Panics if `low_threshold > high_threshold`, if the ADC does not have the requested
    /// watchdog, or if a channel selection variant is used that is not supported by the chosen
    /// watchdog.
    #[must_use]
    pub fn enable_watchdog(
        &mut self,
        watchdog: WatchdogIndex,
        channels: WatchdogChannels,
        low_threshold: u32,
        high_threshold: u32,
    ) -> AnalogWatchdog<T, M> {
        assert!(
            low_threshold <= high_threshold,
            "low_threshold must be <= high_threshold"
        );
        let index = watchdog.index();
        assert!(index < T::Regs::AWD_COUNT, "this ADC has no watchdog {:?}", watchdog);

        T::regs().configure_awd(index, channels, low_threshold, high_threshold);
        AnalogWatchdog {
            index,
            stop_on_drop: false,
            _marker: PhantomData,
        }
    }
}

impl<T: Instance, M: Mode> AnalogWatchdog<T, M> {
    /// Whether the watchdog has fired since the last call (or since it was enabled). Clears the
    /// flag.
    pub fn is_triggered(&mut self) -> bool {
        T::regs().clear_awd_flag(self.index)
    }
}

impl<T: Instance> AnalogWatchdog<T, Async> {
    /// Wait for the watchdog to trigger.
    ///
    /// This method assumes conversions are already being performed externally (for example by DMA
    /// or another task running concurrently). For typical single-pin monitoring driven entirely
    /// by the watchdog driver, prefer [`Self::monitor`].
    pub async fn wait(&mut self) {
        self.start_awd();
        let index = self.index;

        poll_fn(|cx| {
            T::state().waker.register(cx.waker());

            if T::state().awd_triggered[index].clear() {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;
    }

    /// Continuously convert `channel` and return the first result that trips the analog watchdog.
    ///
    /// Thresholds and watchdog channel selection are configured in [`Adc::enable_watchdog`]. When
    /// using [`WatchdogChannels::Single`], pass the same physical channel here.
    ///
    /// This method takes exclusive access to the [`Adc`] for the duration of the operation
    /// because it stops any in-progress conversion and starts its own. For concurrent use
    /// with DMA, use [`Self::wait`] instead.
    ///
    /// # Cancel safety
    ///
    /// If this future is dropped before it resolves, the ongoing continuous conversion is stopped
    /// when the [`AnalogWatchdog`] guard is dropped.
    pub async fn monitor<'a>(
        &mut self,
        _adc: &mut Adc<'_, T, Async>,
        channel: impl BorrowedChannel<'a, T>,
        sample_time: SampleTimeOf<T>,
    ) -> u16 {
        let _scoped_wake_guard = <T as crate::rcc::SealedRccPeripheral>::RCC_INFO.wake_guard();

        let channel = channel.reborrow_adc();
        let r = T::regs();

        r.stop();
        r.configure_sequence(
            [((channel.channel(), channel.is_differential()), sample_time)].into_iter(),
            false,
        );
        r.enable();
        r.configure_dma(ConversionMode::NoDma);
        r.set_continuous(true);

        // Record that Drop must stop the ADC if this future is cancelled.
        self.stop_on_drop = true;

        self.start_awd();
        r.start();

        let index = self.index;
        let sample = poll_fn(|cx| {
            T::state().waker.register(cx.waker());
            compiler_fence(Ordering::SeqCst);

            if T::state().awd_triggered[index].clear() {
                Poll::Ready(unsafe { core::ptr::read_volatile(r.data()) })
            } else {
                Poll::Pending
            }
        })
        .await;

        r.stop();
        r.set_continuous(false);
        self.stop_on_drop = false;
        sample
    }

    fn start_awd(&mut self) {
        T::state().awd_triggered[self.index].store(false, Ordering::Release);
        T::regs().set_awd_interrupt(self.index, true);
    }
}

impl<T: Instance, M: Mode> Drop for AnalogWatchdog<T, M> {
    fn drop(&mut self) {
        T::regs().set_awd_interrupt(self.index, false);
        T::regs().disable_awd(self.index);

        // If monitor() started a continuous conversion that was not stopped normally (i.e. the
        // future was cancelled), stop the ADC and clear continuous mode now.
        if self.stop_on_drop {
            T::regs().stop();
            T::regs().set_continuous(false);
        }
    }
}

#[allow(unused)]
use super::SealedAdcChannel;

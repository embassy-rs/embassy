// This module is only instantiated for targets that have an ADC4 peripheral (STM32WBA, STM32U5).
// The threshold/channel register field names differ between the two families; see comments at each
// site.  If you add support for a new family, ensure the cfg guards below cover it.
#[cfg(not(any(stm32wba, stm32u5)))]
compile_error!("watchdog_adc4 is only valid for stm32wba and stm32u5 targets");

use core::future::poll_fn;
use core::marker::PhantomData;
use core::sync::atomic::{Ordering, compiler_fence};
use core::task::Poll;

use crate::adc::{Adc, AdcChannel, AdcRegs, ConversionMode, Instance};
use crate::interrupt::typelevel::Interrupt;
// Both families expose an `Exten` enum for the external-trigger-enable field, but under different
// names in the PAC.  Import it here under a single alias so the rest of the file is cfg-free.
#[cfg(stm32u5)]
use crate::pac::adc::vals::Adc4Exten as Exten;
#[cfg(stm32wba)]
use crate::pac::adc::vals::Exten;

/// Select which of the three hardware analog watchdogs to use.
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum WatchdogIndex {
    /// First analog watchdog (AWD1).
    ///
    /// Supports [`WatchdogChannels::All`] and [`WatchdogChannels::Single`] via CFGR1.
    Awd1,
    /// Second analog watchdog (AWD2).
    ///
    /// Supports [`WatchdogChannels::Single`] and [`WatchdogChannels::Channels`] via AWD2CR.
    Awd2,
    /// Third analog watchdog (AWD3).
    ///
    /// Supports [`WatchdogChannels::Single`] and [`WatchdogChannels::Channels`] via AWD3CR.
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
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum WatchdogChannels {
    /// Monitor all channels in the regular sequence.
    ///
    /// Only valid with [`WatchdogIndex::Awd1`].
    All,
    /// Monitor a single specific channel.
    ///
    /// Valid for all three watchdogs.
    Single(u8),
    /// Monitor a bitmask of channels (bit N = channel N).
    ///
    /// Only valid with [`WatchdogIndex::Awd2`] and [`WatchdogIndex::Awd3`].
    /// On STM32WBA: bits 0–13 (14 channels). On STM32U5: bits 0–23 (24 channels).
    Channels(u32),
}

/// A driver for an ADC analog watchdog.
///
/// Created by [`Adc::enable_watchdog`].  Does **not** borrow the [`Adc`] — you may hold this
/// guard while performing DMA or other ADC operations concurrently and call [`Self::wait`] to
/// detect when a monitored channel leaves the threshold window.
///
/// For self-contained single-pin monitoring that drives its own continuous conversion, use
/// [`Self::monitor`], which temporarily borrows the [`Adc`].
///
/// Dropping the guard disables the watchdog and its interrupt.
pub struct AnalogWatchdog<T: Instance<Regs = crate::pac::adc::Adc4>> {
    index: usize,
    /// True when [`Self::monitor`] started a continuous conversion that must be stopped in Drop.
    stop_on_drop: bool,
    _phantom: PhantomData<T>,
}

impl<T: Instance<Regs = crate::pac::adc::Adc4>> AnalogWatchdog<T> {
    pub(crate) fn new(index: usize) -> Self {
        Self {
            index,
            stop_on_drop: false,
            _phantom: PhantomData,
        }
    }

    /// Wait for the watchdog to trigger.
    ///
    /// The watchdog is configured in [`Adc::enable_watchdog`].
    ///
    /// This method assumes conversions are already being performed externally (for example by DMA
    /// or another task running concurrently).  For typical single-pin monitoring driven entirely
    /// by the watchdog driver, prefer [`Self::monitor`].
    pub async fn wait(&mut self) {
        self.start_awd();
        let index = self.index;

        poll_fn(|cx| {
            T::state().waker.register(cx.waker());

            if T::state().awd_triggered[index].load(Ordering::Acquire) {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;
    }

    /// Continuously convert `channel` and return the first result that trips the analog watchdog.
    ///
    /// Thresholds and watchdog channel selection are configured in [`Adc::enable_watchdog`].  When
    /// using [`WatchdogChannels::Single`], pass the same physical channel here.
    ///
    /// For AWD2/AWD3 with a [`WatchdogChannels::Channels`] bitmask, pass any one of the monitored
    /// channels here; the watchdog will fire when **any** of them leaves the threshold window.
    ///
    /// This method takes exclusive access to the [`Adc`] for the duration of the operation
    /// because it stops any in-progress conversion and starts its own.  For concurrent use
    /// with DMA, use [`Self::wait`] instead.
    ///
    /// # Cancel safety
    ///
    /// If this future is dropped before it resolves, the ongoing continuous conversion is stopped
    /// and continuous mode is cleared when the [`AnalogWatchdog`] guard is dropped.
    pub async fn monitor(
        &mut self,
        _adc: &mut Adc<'_, T>,
        channel: &mut impl AdcChannel<T>,
        sample_time: super::SampleTime,
    ) -> u16 {
        let _scoped_wake_guard = <T as crate::rcc::SealedRccPeripheral>::RCC_INFO.wake_guard();

        channel.setup();

        T::regs().stop(false);
        T::regs().configure_sequence([((channel.channel(), channel.is_differential()), sample_time)].into_iter());
        T::regs().enable();
        T::regs().configure_dma(ConversionMode::NoDma);

        T::regs().cfgr1().modify(|w| {
            w.set_cont(true);
            w.set_exten(Exten::Disabled);
        });

        // Record that Drop must stop the ADC if this future is cancelled.
        self.stop_on_drop = true;

        self.start_awd();
        T::regs().start();

        let index = self.index;
        let sample = poll_fn(|cx| {
            T::state().waker.register(cx.waker());
            compiler_fence(Ordering::SeqCst);

            if T::state().awd_triggered[index].load(Ordering::Acquire) {
                Poll::Ready(unsafe { core::ptr::read_volatile(T::regs().data()) })
            } else {
                Poll::Pending
            }
        })
        .await;

        // Normal completion: stop here so Drop does not do a redundant stop.
        T::regs().stop(false);
        T::regs().cfgr1().modify(|w| w.set_cont(false));
        self.stop_on_drop = false;
        sample
    }

    pub(crate) fn setup_awd(
        watchdog: WatchdogIndex,
        channels: WatchdogChannels,
        low_threshold: u16,
        high_threshold: u16,
    ) {
        match watchdog {
            WatchdogIndex::Awd1 => {
                // AWD1 threshold register field names differ between families:
                //   STM32WBA: lt1 / ht1 (named after the watchdog index)
                //   STM32U5:  lt3 / ht3 (the Adc4Awdtr struct reuses suffix "3" for all three
                //             watchdog threshold registers on that family — it is not the AWD3 field)
                T::regs().awd1tr().modify(|w| {
                    #[cfg(stm32wba)]
                    {
                        w.set_lt1(low_threshold);
                        w.set_ht1(high_threshold);
                    }
                    #[cfg(stm32u5)]
                    {
                        w.set_lt3(low_threshold);
                        w.set_ht3(high_threshold);
                    }
                });

                T::regs().cfgr1().modify(|w| {
                    w.set_awd1en(true);
                    #[cfg(stm32wba)]
                    {
                        use crate::pac::adc::vals::Awd1sgl;
                        match channels {
                            WatchdogChannels::Single(ch) => {
                                w.set_awd1sgl(Awd1sgl::SingleChannel);
                                w.set_awd1ch(ch);
                            }
                            WatchdogChannels::All => {
                                w.set_awd1sgl(Awd1sgl::AllChannels);
                            }
                            WatchdogChannels::Channels(_) => {
                                panic!(
                                    "WatchdogChannels::Channels bitmask is not supported for AWD1; use Single or All"
                                );
                            }
                        }
                    }
                    #[cfg(stm32u5)]
                    {
                        match channels {
                            WatchdogChannels::Single(ch) => {
                                w.set_awd1sgl(true);
                                w.set_awd1ch(ch);
                            }
                            WatchdogChannels::All => {
                                w.set_awd1sgl(false);
                            }
                            WatchdogChannels::Channels(_) => {
                                panic!(
                                    "WatchdogChannels::Channels bitmask is not supported for AWD1; use Single or All"
                                );
                            }
                        }
                    }
                });
            }

            WatchdogIndex::Awd2 => {
                // AWD2 threshold register field names:
                //   STM32WBA: lt2 / ht2  (Awd2tr struct, named after watchdog 2)
                //   STM32U5:  lt3 / ht3  (same Adc4Awdtr struct used for all three watchdogs;
                //             the suffix "3" is a PAC implementation detail, not the AWD index)
                T::regs().awd2tr().modify(|w| {
                    #[cfg(stm32wba)]
                    {
                        w.set_lt2(low_threshold);
                        w.set_ht2(high_threshold);
                    }
                    #[cfg(stm32u5)]
                    {
                        w.set_lt3(low_threshold);
                        w.set_ht3(high_threshold);
                    }
                });

                // AWD2/AWD3 use a dedicated channel-bitmask register (AWD2CR/AWD3CR) rather than
                // CFGR1 fields; setting any bit enables monitoring of that channel.
                T::regs().awd2cr().modify(|w| {
                    // Clear all channel bits first, then set the requested ones.
                    #[cfg(stm32wba)]
                    {
                        for n in 0..14usize {
                            w.set_awd2ch(n, false);
                        }
                        match channels {
                            WatchdogChannels::Single(ch) => w.set_awd2ch(ch as usize, true),
                            WatchdogChannels::Channels(mask) => {
                                for n in 0..14usize {
                                    if mask & (1 << n) != 0 {
                                        w.set_awd2ch(n, true);
                                    }
                                }
                            }
                            WatchdogChannels::All => {
                                panic!("WatchdogChannels::All is not supported for AWD2; use Channels(0x3FFF) to monitor all 14 channels");
                            }
                        }
                    }
                    #[cfg(stm32u5)]
                    {
                        for n in 0..24usize {
                            w.set_awdch(n, false);
                        }
                        match channels {
                            WatchdogChannels::Single(ch) => w.set_awdch(ch as usize, true),
                            WatchdogChannels::Channels(mask) => {
                                for n in 0..24usize {
                                    if mask & (1 << n) != 0 {
                                        w.set_awdch(n, true);
                                    }
                                }
                            }
                            WatchdogChannels::All => {
                                panic!("WatchdogChannels::All is not supported for AWD2; use Channels(0xFFFFFF) to monitor all 24 channels");
                            }
                        }
                    }
                });
            }

            WatchdogIndex::Awd3 => {
                // AWD3 threshold register: both WBA (Awd3tr) and U5 (Adc4Awdtr) happen to expose
                // the fields as lt3/ht3, so no cfg split is needed here.
                T::regs().awd3tr().modify(|w| {
                    w.set_lt3(low_threshold);
                    w.set_ht3(high_threshold);
                });

                T::regs().awd3cr().modify(|w| {
                    #[cfg(stm32wba)]
                    {
                        for n in 0..14usize {
                            w.set_awd3ch(n, false);
                        }
                        match channels {
                            WatchdogChannels::Single(ch) => w.set_awd3ch(ch as usize, true),
                            WatchdogChannels::Channels(mask) => {
                                for n in 0..14usize {
                                    if mask & (1 << n) != 0 {
                                        w.set_awd3ch(n, true);
                                    }
                                }
                            }
                            WatchdogChannels::All => {
                                panic!("WatchdogChannels::All is not supported for AWD3; use Channels(0x3FFF) to monitor all 14 channels");
                            }
                        }
                    }
                    #[cfg(stm32u5)]
                    {
                        for n in 0..24usize {
                            w.set_awdch(n, false);
                        }
                        match channels {
                            WatchdogChannels::Single(ch) => w.set_awdch(ch as usize, true),
                            WatchdogChannels::Channels(mask) => {
                                for n in 0..24usize {
                                    if mask & (1 << n) != 0 {
                                        w.set_awdch(n, true);
                                    }
                                }
                            }
                            WatchdogChannels::All => {
                                panic!("WatchdogChannels::All is not supported for AWD3; use Channels(0xFFFFFF) to monitor all 24 channels");
                            }
                        }
                    }
                });
            }
        }
    }

    fn start_awd(&mut self) {
        // Reset atomic flag, clear hardware ISR flag, enable NVIC + AWDIE.
        T::state().awd_triggered[self.index].store(false, Ordering::Release);
        T::regs().isr().write(|w| w.set_awd(self.index, true));
        T::Interrupt::unpend();
        unsafe {
            T::Interrupt::enable();
        }
        T::regs().ier().modify(|w| w.set_awdie(self.index, true));
    }
}

impl<T: Instance<Regs = crate::pac::adc::Adc4>> Drop for AnalogWatchdog<T> {
    fn drop(&mut self) {
        // Always disable the interrupt and watchdog enable/channel bits.
        T::regs().ier().modify(|w| w.set_awdie(self.index, false));

        match self.index {
            0 => T::regs().cfgr1().modify(|w| w.set_awd1en(false)),
            1 => T::regs().awd2cr().modify(|w| *w = Default::default()),
            2 => T::regs().awd3cr().modify(|w| *w = Default::default()),
            _ => {}
        }

        // If monitor() started a continuous conversion that was not stopped normally (i.e. the
        // future was cancelled), stop the ADC and clear continuous mode now.
        if self.stop_on_drop {
            T::regs().stop(false);
            T::regs().cfgr1().modify(|w| w.set_cont(false));
        }
    }
}

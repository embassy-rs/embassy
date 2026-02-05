//! ADC4 Analog Watchdog for STM32WBA/U5
//!
//! This module provides analog watchdog (AWD) functionality for ADC4 on STM32WBA
//! and STM32U5 families. Unlike ADCs with a single analog watchdog, ADC4 has three
//! independent watchdogs (AWD1, AWD2, AWD3) with different capabilities:
//!
//! - **AWD1**: Can monitor a single channel or all enabled channels
//! - **AWD2**: Can monitor multiple individually selected channels
//! - **AWD3**: Can monitor multiple individually selected channels
//!
//! # Features
//!
//! - Programmable 12-bit high and low thresholds
//! - Single channel or multi-channel monitoring
//! - Async API for waiting on threshold crossings
//! - Support for all three watchdog instances
//!
//! # Example
//!
//! ```rust,ignore
//! use embassy_stm32::adc::{Adc, SampleTime};
//! use embassy_stm32::adc::adc4::WatchdogChannels;
//!
//! let mut adc = Adc::new_adc4(p.ADC4);
//!
//! // Configure AWD1 to monitor channel 2 with thresholds
//! adc.init_watchdog_awd1(
//!     WatchdogChannels::Single(2),
//!     0,      // low threshold (12-bit ADC counts)
//!     3000,   // high threshold (target voltage in counts)
//! );
//!
//! // Wait for voltage to cross threshold
//! let value = adc.monitor_watchdog_awd1(SampleTime::CYCLES12_5).await;
//! info!("Threshold crossed at {} counts", value);
//! ```
//!
//! # Multi-Watchdog Example
//!
//! ```rust,ignore
//! // Use AWD1 for dynamic threshold (charging target)
//! adc.init_watchdog_awd1(WatchdogChannels::Single(2), 0, target_counts);
//!
//! // Use AWD2 for overvoltage protection (always active)
//! adc.init_watchdog_awd2(WatchdogChannels::Single(2), 0, max_safe_counts);
//!
//! // Race them - whichever triggers first
//! let result = select(
//!     adc.monitor_watchdog_awd1(sample_time),
//!     adc.monitor_watchdog_awd2(sample_time),
//! ).await;
//! ```

use core::future::poll_fn;
use core::task::Poll;

#[cfg(stm32wba)]
use crate::pac::adc::vals::{Awd1sgl, Cont, Res, SampleTime};
#[cfg(stm32u5)]
use crate::pac::adc::vals::{Adc4Awd1sgl as Awd1sgl, Adc4Res as Res, Adc4SampleTime as SampleTime};

use crate::adc::{Adc, Instance};
use crate::pac;
use embassy_sync::waitqueue::AtomicWaker;

/// Analog watchdog channel selection.
///
/// Specifies which ADC channels the watchdog should monitor.
#[derive(Clone, Copy, Debug)]
pub enum WatchdogChannels {
    /// Monitor a single channel identified by index (0-23)
    Single(u8),
    /// Monitor multiple channels identified by bitmask
    Multiple(u32),
}

impl WatchdogChannels {
    /// Create a watchdog configuration for a single channel.
    ///
    /// # Arguments
    /// * `channel` - Channel number (0-23 depending on MCU)
    pub fn single(channel: u8) -> Self {
        Self::Single(channel)
    }

    /// Create a watchdog configuration for multiple channels.
    ///
    /// # Arguments
    /// * `channels` - Bitmask of channels to monitor (bit N = channel N)
    pub fn multiple(channels: u32) -> Self {
        Self::Multiple(channels)
    }

    /// Add a channel to the watchdog configuration.
    pub fn add_channel(self, channel: u8) -> Self {
        WatchdogChannels::Multiple(
            (match self {
                WatchdogChannels::Single(ch) => 1 << ch,
                WatchdogChannels::Multiple(ch) => ch,
            }) | (1 << channel),
        )
    }
}

/// Static wakers for ADC4 watchdog instances
pub(crate) struct Adc4WatchdogState {
    pub awd1_waker: AtomicWaker,
    pub awd2_waker: AtomicWaker,
    pub awd3_waker: AtomicWaker,
}

impl Adc4WatchdogState {
    pub const fn new() -> Self {
        Self {
            awd1_waker: AtomicWaker::new(),
            awd2_waker: AtomicWaker::new(),
            awd3_waker: AtomicWaker::new(),
        }
    }
}

/// Global state for ADC4 watchdog wakers
pub(crate) static ADC4_WATCHDOG_STATE: Adc4WatchdogState = Adc4WatchdogState::new();

impl<'d, T: Instance<Regs = crate::pac::adc::Adc4>> Adc<'d, T> {
    // ========================================================================
    // AWD1 - Single channel or all channels
    // ========================================================================

    /// Configure analog watchdog 1 with specified channels and thresholds.
    ///
    /// AWD1 can monitor either a single channel or all enabled channels.
    /// For single channel mode, specify `WatchdogChannels::Single(ch)`.
    /// For all channels mode, specify `WatchdogChannels::Multiple(_)`.
    ///
    /// # Arguments
    /// * `channels` - Channel(s) to monitor
    /// * `low_threshold` - Lower threshold in ADC counts (12-bit, right-aligned)
    /// * `high_threshold` - Upper threshold in ADC counts (12-bit, right-aligned)
    ///
    /// # Threshold Format
    ///
    /// Thresholds are expressed in the same format as ADC results.
    /// For 12-bit resolution with right alignment, use values 0-4095.
    /// For lower resolutions, the appropriate LSBs should be zero.
    pub fn init_watchdog_awd1(
        &mut self,
        channels: WatchdogChannels,
        low_threshold: u16,
        high_threshold: u16,
    ) {
        Self::stop_awd();

        match channels {
            WatchdogChannels::Single(ch) => {
                T::regs().cfgr1().modify(|w| {
                    w.set_awd1ch(ch);
                    w.set_awd1sgl(Awd1sgl::SINGLE_CHANNEL);
                });
            }
            WatchdogChannels::Multiple(_) => {
                // AWD1 doesn't support arbitrary multi-channel, uses all channels
                T::regs().cfgr1().modify(|w| {
                    w.set_awd1ch(0);
                    w.set_awd1sgl(Awd1sgl::ALL_CHANNELS);
                });
            }
        }

        Self::set_awd1_thresholds_internal(low_threshold, high_threshold);
        Self::setup_awd1();
    }

    /// Monitor AWD1 and wait for threshold crossing.
    ///
    /// Starts continuous ADC conversion on the configured channels and waits
    /// asynchronously until the voltage crosses the threshold window.
    ///
    /// Returns the ADC value that triggered the watchdog.
    pub async fn monitor_watchdog_awd1(&mut self, sample_time: SampleTime) -> u16 {
        // Verify watchdog is configured
        assert!(
            T::regs().cfgr1().read().awd1en(),
            "AWD1 not enabled. Call init_watchdog_awd1() first."
        );

        T::regs().smpr().modify(|reg| reg.set_smp(0, sample_time));
        Self::start_awd1();

        let sample = poll_fn(|cx| {
            ADC4_WATCHDOG_STATE.awd1_waker.register(cx.waker());

            if T::regs().isr().read().awd(0) {
                Poll::Ready(T::regs().dr().read().0 as u16)
            } else {
                Poll::Pending
            }
        })
        .await;

        self.stop_watchdog_awd1();
        sample
    }

    /// Stop AWD1 monitoring and disable the watchdog.
    pub fn stop_watchdog_awd1(&mut self) {
        Self::stop_awd();
        Self::teardown_awd1();
    }

    /// Update AWD1 thresholds dynamically.
    ///
    /// Useful for changing target voltage without full reconfiguration.
    /// Can be called while ADC is running.
    pub fn set_awd1_thresholds(&mut self, low_threshold: u16, high_threshold: u16) {
        Self::set_awd1_thresholds_internal(low_threshold, high_threshold);
    }

    fn set_awd1_thresholds_internal(low_threshold: u16, high_threshold: u16) {
        // Validate threshold alignment for current resolution
        let threshold_mask = Self::get_threshold_mask();

        assert!(
            high_threshold & !threshold_mask == 0,
            "High threshold {:x} is invalid — only bits {:x} are allowed",
            high_threshold,
            threshold_mask
        );
        assert!(
            low_threshold & !threshold_mask == 0,
            "Low threshold {:x} is invalid — only bits {:x} are allowed",
            low_threshold,
            threshold_mask
        );

        T::regs().awd1tr().write(|w| {
            w.set_lt1(low_threshold & 0x0FFF);
            w.set_ht1(high_threshold & 0x0FFF);
        });
    }

    fn setup_awd1() {
        assert!(!T::regs().cr().read().adstart(), "Cannot configure AWD1 while conversion is running");
        T::regs().cfgr1().modify(|w| w.set_awd1en(true));
    }

    fn start_awd1() {
        // Clear AWD1 interrupt flag
        while T::regs().isr().read().awd(0) {
            T::regs().isr().modify(|w| w.set_awd(0, true));
        }

        // Enable AWD1 interrupt
        assert!(!T::regs().cr().read().adstart());
        T::regs().ier().modify(|w| {
            w.set_eocie(false);
            w.set_awdie(0, true);
        });

        // Start continuous conversion
        #[cfg(stm32wba)]
        T::regs().cfgr1().modify(|w| w.set_cont(Cont::CONTINUOUS));
        #[cfg(stm32u5)]
        T::regs().cfgr1().modify(|w| w.set_cont(true));

        T::regs().cr().modify(|w| w.set_adstart(true));
    }

    fn teardown_awd1() {
        T::regs().ier().modify(|w| w.set_awdie(0, false));

        // Clear AWD1 interrupt flag
        while T::regs().isr().read().awd(0) {
            T::regs().isr().modify(|w| w.set_awd(0, true));
        }

        T::regs().cfgr1().modify(|w| w.set_awd1en(false));
    }

    // ========================================================================
    // AWD2 - Multiple individually selected channels
    // ========================================================================

    /// Configure analog watchdog 2 with specified channels and thresholds.
    ///
    /// AWD2 can monitor multiple individually selected channels.
    ///
    /// # Arguments
    /// * `channels` - Channel(s) to monitor
    /// * `low_threshold` - Lower threshold in ADC counts (12-bit)
    /// * `high_threshold` - Upper threshold in ADC counts (12-bit)
    pub fn init_watchdog_awd2(
        &mut self,
        channels: WatchdogChannels,
        low_threshold: u16,
        high_threshold: u16,
    ) {
        let channel_mask = match channels {
            WatchdogChannels::Single(ch) => 1u32 << ch,
            WatchdogChannels::Multiple(mask) => mask,
        };

        // Configure AWD2 channel selection
        T::regs().awd2cr().write(|w| {
            for i in 0..24 {
                if (channel_mask & (1 << i)) != 0 {
                    w.set_awd2ch(i, true);
                }
            }
        });

        // Set thresholds
        T::regs().awd2tr().write(|w| {
            w.set_lt2(low_threshold & 0x0FFF);
            w.set_ht2(high_threshold & 0x0FFF);
        });
    }

    /// Monitor AWD2 and wait for threshold crossing.
    pub async fn monitor_watchdog_awd2(&mut self, sample_time: SampleTime) -> u16 {
        T::regs().smpr().modify(|reg| reg.set_smp(0, sample_time));

        // Clear and enable AWD2 interrupt
        T::regs().isr().modify(|w| w.set_awd(1, true));
        T::regs().ier().modify(|w| w.set_awdie(1, true));

        // Start continuous conversion
        #[cfg(stm32wba)]
        T::regs().cfgr1().modify(|w| w.set_cont(Cont::CONTINUOUS));
        #[cfg(stm32u5)]
        T::regs().cfgr1().modify(|w| w.set_cont(true));

        if !T::regs().cr().read().adstart() {
            T::regs().cr().modify(|w| w.set_adstart(true));
        }

        let sample = poll_fn(|cx| {
            ADC4_WATCHDOG_STATE.awd2_waker.register(cx.waker());

            if T::regs().isr().read().awd(1) {
                Poll::Ready(T::regs().dr().read().0 as u16)
            } else {
                Poll::Pending
            }
        })
        .await;

        self.stop_watchdog_awd2();
        sample
    }

    /// Stop AWD2 monitoring.
    pub fn stop_watchdog_awd2(&mut self) {
        Self::stop_awd();
        T::regs().ier().modify(|w| w.set_awdie(1, false));
        T::regs().isr().modify(|w| w.set_awd(1, true));
        T::regs().awd2cr().write(|w| {
            for i in 0..24 {
                w.set_awd2ch(i, false);
            }
        });
    }

    /// Update AWD2 thresholds dynamically.
    pub fn set_awd2_thresholds(&mut self, low_threshold: u16, high_threshold: u16) {
        T::regs().awd2tr().write(|w| {
            w.set_lt2(low_threshold & 0x0FFF);
            w.set_ht2(high_threshold & 0x0FFF);
        });
    }

    // ========================================================================
    // AWD3 - Multiple individually selected channels
    // ========================================================================

    /// Configure analog watchdog 3 with specified channels and thresholds.
    ///
    /// AWD3 can monitor multiple individually selected channels.
    ///
    /// # Arguments
    /// * `channels` - Channel(s) to monitor
    /// * `low_threshold` - Lower threshold in ADC counts (12-bit)
    /// * `high_threshold` - Upper threshold in ADC counts (12-bit)
    pub fn init_watchdog_awd3(
        &mut self,
        channels: WatchdogChannels,
        low_threshold: u16,
        high_threshold: u16,
    ) {
        let channel_mask = match channels {
            WatchdogChannels::Single(ch) => 1u32 << ch,
            WatchdogChannels::Multiple(mask) => mask,
        };

        // Configure AWD3 channel selection
        T::regs().awd3cr().write(|w| {
            for i in 0..24 {
                if (channel_mask & (1 << i)) != 0 {
                    w.set_awd3ch(i, true);
                }
            }
        });

        // Set thresholds
        T::regs().awd3tr().write(|w| {
            w.set_lt3(low_threshold & 0x0FFF);
            w.set_ht3(high_threshold & 0x0FFF);
        });
    }

    /// Monitor AWD3 and wait for threshold crossing.
    pub async fn monitor_watchdog_awd3(&mut self, sample_time: SampleTime) -> u16 {
        T::regs().smpr().modify(|reg| reg.set_smp(0, sample_time));

        // Clear and enable AWD3 interrupt
        T::regs().isr().modify(|w| w.set_awd(2, true));
        T::regs().ier().modify(|w| w.set_awdie(2, true));

        // Start continuous conversion
        #[cfg(stm32wba)]
        T::regs().cfgr1().modify(|w| w.set_cont(Cont::CONTINUOUS));
        #[cfg(stm32u5)]
        T::regs().cfgr1().modify(|w| w.set_cont(true));

        if !T::regs().cr().read().adstart() {
            T::regs().cr().modify(|w| w.set_adstart(true));
        }

        let sample = poll_fn(|cx| {
            ADC4_WATCHDOG_STATE.awd3_waker.register(cx.waker());

            if T::regs().isr().read().awd(2) {
                Poll::Ready(T::regs().dr().read().0 as u16)
            } else {
                Poll::Pending
            }
        })
        .await;

        self.stop_watchdog_awd3();
        sample
    }

    /// Stop AWD3 monitoring.
    pub fn stop_watchdog_awd3(&mut self) {
        Self::stop_awd();
        T::regs().ier().modify(|w| w.set_awdie(2, false));
        T::regs().isr().modify(|w| w.set_awd(2, true));
        T::regs().awd3cr().write(|w| {
            for i in 0..24 {
                w.set_awd3ch(i, false);
            }
        });
    }

    /// Update AWD3 thresholds dynamically.
    pub fn set_awd3_thresholds(&mut self, low_threshold: u16, high_threshold: u16) {
        T::regs().awd3tr().write(|w| {
            w.set_lt3(low_threshold & 0x0FFF);
            w.set_ht3(high_threshold & 0x0FFF);
        });
    }

    // ========================================================================
    // Common helpers
    // ========================================================================

    fn get_threshold_mask() -> u16 {
        match T::regs().cfgr1().read().res() {
            Res::BITS6 => 0x0FC0,  // 6 LSBs must be 0
            Res::BITS8 => 0x0FF0,  // 4 LSBs must be 0
            Res::BITS10 => 0x0FFC, // 2 LSBs must be 0
            Res::BITS12 => 0x0FFF,
            #[allow(unreachable_patterns)]
            _ => 0x0FFF,
        }
    }

    fn stop_awd() {
        // Stop conversion if running
        while T::regs().cr().read().addis() {}
        if T::regs().cr().read().adstart() {
            T::regs().cr().modify(|w| w.set_adstp(true));
            while T::regs().cr().read().adstp() {}
        }

        // Disable continuous mode
        #[cfg(stm32wba)]
        T::regs().cfgr1().modify(|w| w.set_cont(Cont::SINGLE));
        #[cfg(stm32u5)]
        T::regs().cfgr1().modify(|w| w.set_cont(false));

        // Disable all AWD interrupts
        T::regs().ier().modify(|w| {
            w.set_awdie(0, false);
            w.set_awdie(1, false);
            w.set_awdie(2, false);
        });

        // Clear all AWD flags
        T::regs().isr().modify(|w| {
            w.set_awd(0, true);
            w.set_awd(1, true);
            w.set_awd(2, true);
        });
    }
}

/// ADC4 interrupt handler for analog watchdogs.
///
/// This function should be called from the ADC4 global interrupt handler.
/// It checks all three AWD flags and wakes the appropriate tasks.
///
/// # Example Integration
///
/// ```rust,ignore
/// #[interrupt]
/// fn ADC4() {
///     embassy_stm32::adc::adc4::on_adc4_watchdog_interrupt();
///     // ... handle other ADC4 interrupts
/// }
/// ```
pub fn on_adc4_watchdog_interrupt() {
    let adc = pac::ADC4;
    let isr = adc.isr().read();

    // AWD1 (index 0)
    if isr.awd(0) {
        adc.isr().modify(|w| w.set_awd(0, true)); // Clear flag
        ADC4_WATCHDOG_STATE.awd1_waker.wake();
    }

    // AWD2 (index 1)
    if isr.awd(1) {
        adc.isr().modify(|w| w.set_awd(1, true));
        ADC4_WATCHDOG_STATE.awd2_waker.wake();
    }

    // AWD3 (index 2)
    if isr.awd(2) {
        adc.isr().modify(|w| w.set_awd(2, true));
        ADC4_WATCHDOG_STATE.awd3_waker.wake();
    }
}

/// Re-export for convenience
pub use WatchdogChannels as Adc4WatchdogChannels;

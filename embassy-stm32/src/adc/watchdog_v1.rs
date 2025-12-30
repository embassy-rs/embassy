use core::future::poll_fn;
use core::task::Poll;

use stm32_metapac::adc::vals::{Align, Awdsgl, Res, SampleTime};

use crate::adc::{Adc, AdcChannel, Instance};

/// This enum is passed into `Adc::init_watchdog` to specify the channels for the watchdog to monitor
pub enum WatchdogChannels {
    // Single channel identified by index
    Single(u8),
    // Multiple channels identified by mask
    Multiple(u16),
}

impl WatchdogChannels {
    pub fn from_channel<T>(channel: &impl AdcChannel<T>) -> Self {
        Self::Single(channel.channel())
    }

    pub fn add_channel<T>(self, channel: &impl AdcChannel<T>) -> Self {
        WatchdogChannels::Multiple(
            (match self {
                WatchdogChannels::Single(ch) => 1 << ch,
                WatchdogChannels::Multiple(ch) => ch,
            }) | 1 << channel.channel(),
        )
    }
}

impl<'d, T: Instance> Adc<'d, T> {
    /// Configure the analog window watchdog to monitor one or more ADC channels
    ///
    /// `high_threshold` and `low_threshold` are expressed in the same way as ADC results. The format
    /// depends on the values of CFGR1.ALIGN and CFGR1.RES.
    pub fn init_watchdog(&mut self, channels: WatchdogChannels, low_threshold: u16, high_threshold: u16) {
        Self::stop_awd();

        match channels {
            WatchdogChannels::Single(ch) => {
                T::regs().chselr().modify(|w| {
                    w.set_chsel_x(ch.into(), true);
                });
                T::regs().cfgr1().modify(|w| {
                    w.set_awdch(ch);
                    w.set_awdsgl(Awdsgl::SINGLE_CHANNEL)
                });
            }
            WatchdogChannels::Multiple(ch) => {
                T::regs().chselr().modify(|w| w.0 = ch.into());
                T::regs().cfgr1().modify(|w| {
                    w.set_awdch(0);
                    w.set_awdsgl(Awdsgl::ALL_CHANNELS)
                });
            }
        }

        Self::set_watchdog_thresholds(low_threshold, high_threshold);
        Self::setup_awd();
    }

    /// Monitor the voltage on the selected channels; return when it crosses the thresholds.
    ///
    /// ```rust,ignore
    /// // Wait for pin to go high
    /// adc.init_watchdog(WatchdogChannels::from_channel(&pin), 0, 0x07F);
    /// let v_high = adc.monitor_watchdog().await;
    /// info!("ADC sample is high {}", v_high);
    /// ```
    pub async fn monitor_watchdog(&mut self, sample_time: SampleTime) -> u16 {
        assert!(
            match T::regs().cfgr1().read().awdsgl() {
                Awdsgl::SINGLE_CHANNEL => T::regs().cfgr1().read().awdch() != 0,
                Awdsgl::ALL_CHANNELS => T::regs().cfgr1().read().awdch() == 0,
            },
            "`set_channel` should be called before `monitor`",
        );
        assert!(T::regs().chselr().read().0 != 0);
        T::regs().smpr().modify(|reg| reg.set_smp(sample_time.into()));
        Self::start_awd();

        let sample = poll_fn(|cx| {
            T::state().waker.register(cx.waker());

            if T::regs().isr().read().awd() {
                Poll::Ready(T::regs().dr().read().data())
            } else {
                Poll::Pending
            }
        })
        .await;

        self.stop_watchdog();
        sample
    }

    /// Stop monitoring the selected channels
    pub fn stop_watchdog(&mut self) {
        Self::stop_awd();
    }

    fn set_watchdog_thresholds(low_threshold: u16, high_threshold: u16) {
        // This function takes `high_threshold` and `low_threshold` in the same alignment and resolution
        // as ADC results, and programs them into ADC_DR. Because ADC_DR is always right-aligned on 12 bits,
        // some bit-shifting may be necessary. See more in table 47 §13.7.1 Analog Watchdog Comparison

        // Verify that the thresholds are in the correct bit positions according to alignment and resolution
        let threshold_mask = match (T::regs().cfgr1().read().align(), T::regs().cfgr1().read().res()) {
            (Align::LEFT, Res::BITS6) => 0x00FC,
            (Align::LEFT, Res::BITS8) => 0xFF00,
            (Align::LEFT, Res::BITS10) => 0xFFC0,
            (Align::LEFT, Res::BITS12) => 0xFFF0,
            (Align::RIGHT, Res::BITS6) => 0x003F,
            (Align::RIGHT, Res::BITS8) => 0x00FF,
            (Align::RIGHT, Res::BITS10) => 0x03FF,
            (Align::RIGHT, Res::BITS12) => 0x0FFF,
        };
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

        T::regs().tr().modify(|w| {
            w.set_lt(low_threshold << threshold_mask.leading_zeros() >> 4);
            w.set_ht(high_threshold << threshold_mask.leading_zeros() >> 4);
        })
    }

    fn setup_awd() {
        // Configure AWD
        assert!(!T::regs().cr().read().adstart());
        T::regs().cfgr1().modify(|w| w.set_awden(true));
    }

    fn start_awd() {
        // Clear AWD interrupt flag
        while T::regs().isr().read().awd() {
            T::regs().isr().modify(|regs| {
                regs.set_awd(true);
            })
        }

        // Enable AWD interrupt
        assert!(!T::regs().cr().read().adstart());
        T::regs().ier().modify(|w| {
            w.set_eocie(false);
            w.set_awdie(true)
        });

        // Start conversion
        T::regs().cfgr1().modify(|w| w.set_cont(true));
        T::regs().cr().modify(|w| w.set_adstart(true));
    }

    fn stop_awd() {
        // Stop conversion
        while T::regs().cr().read().addis() {}
        if T::regs().cr().read().adstart() {
            T::regs().cr().write(|x| x.set_adstp(true));
            while T::regs().cr().read().adstp() {}
        }
        T::regs().cfgr1().modify(|w| w.set_cont(false));

        // Disable AWD interrupt
        assert!(!T::regs().cr().read().adstart());
        T::regs().ier().modify(|w| w.set_awdie(false));

        // Clear AWD interrupt flag
        while T::regs().isr().read().awd() {
            T::regs().isr().modify(|regs| {
                regs.set_awd(true);
            })
        }
    }

    pub(crate) fn teardown_awd() {
        Self::stop_awd();
        T::regs().cfgr1().modify(|w| w.set_awden(false));
    }
}

use core::future::poll_fn;
use core::marker::PhantomData;
use core::task::Poll;

#[cfg(adc_l0)]
use stm32_metapac::adc::vals::Ckmode;

#[cfg(not(adc_l0))]
use super::Vbat;
use super::{Temperature, VrefInt, blocking_delay_us};
use crate::adc::{Adc, AdcChannel, Instance, Resolution, SampleTime};
use crate::interrupt::typelevel::Interrupt;
use crate::{Peri, interrupt, rcc};

mod watchdog_v1;
pub use watchdog_v1::WatchdogChannels;

pub const VDDA_CALIB_MV: u32 = 3300;
pub const VREF_INT: u32 = 1230;

/// Interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let isr = T::regs().isr().read();
        let ier = T::regs().ier().read();
        if ier.eocie() && isr.eoc() {
            // eocie is set during adc.read()
            T::regs().ier().modify(|w| w.set_eocie(false));
        } else if ier.awdie() && isr.awd() {
            // awdie is set during adc.monitor_watchdog()
            T::regs().cr().read().set_adstp(true);
            T::regs().ier().modify(|w| w.set_awdie(false));
        } else {
            return;
        }

        T::state().waker.wake();
    }
}

#[cfg(not(adc_l0))]
impl super::SealedSpecialConverter<super::Vbat> for crate::peripherals::ADC1 {
    const CHANNEL: u8 = 18;
}

impl super::SealedSpecialConverter<super::VrefInt> for crate::peripherals::ADC1 {
    const CHANNEL: u8 = 17;
}

#[cfg(adc_l0)]
impl super::SealedSpecialConverter<super::Temperature> for crate::peripherals::ADC1 {
    const CHANNEL: u8 = 18;
}

#[cfg(not(adc_l0))]
impl super::SealedSpecialConverter<super::Temperature> for crate::peripherals::ADC1 {
    const CHANNEL: u8 = 16;
}

impl<'d, T: Instance> Adc<'d, T> {
    pub fn new(
        adc: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
    ) -> Self {
        rcc::enable_and_reset_without_stop::<T>();

        // Delay 1μs when using HSI14 as the ADC clock.
        //
        // Table 57. ADC characteristics
        // tstab = 14 * 1/fadc
        blocking_delay_us(1);

        // set default PCKL/2 on L0s because HSI is disabled in the default clock config
        #[cfg(adc_l0)]
        T::regs().cfgr2().modify(|reg| reg.set_ckmode(Ckmode::PCLK_DIV2));

        // A.7.1 ADC calibration code example
        T::regs().cfgr1().modify(|reg| reg.set_dmaen(false));

        #[cfg(adc_l0)]
        let auto_off = T::regs().cfgr1().read().autoff();
        #[cfg(adc_l0)]
        T::regs().cfgr1().modify(|reg| reg.set_autoff(false));

        T::regs().cr().modify(|reg| reg.set_adcal(true));

        #[cfg(adc_l0)]
        while !T::regs().isr().read().eocal() {}

        #[cfg(not(adc_l0))]
        while T::regs().cr().read().adcal() {}

        #[cfg(adc_l0)]
        T::regs().cfgr1().modify(|reg| reg.set_autoff(auto_off));

        let s = Self { adc };

        s.enable();

        T::Interrupt::unpend();
        unsafe {
            T::Interrupt::enable();
        }

        s
    }

    fn enable(&self) {
        #[cfg(adc_l0)]
        if T::regs().cfgr1().read().autoff() {
            // In AUTOFF mode the ADC wakes automatically when conversion starts,
            // so waiting for ADRDY here can stall instead of helping.
            return;
        }

        // A.7.2 ADC enable sequence code example

        while T::regs().cr().read().addis() {}

        if !T::regs().cr().read().aden() {
            if T::regs().isr().read().adrdy() {
                T::regs().isr().modify(|reg| reg.set_adrdy(true));
            }
            T::regs().cr().modify(|reg| reg.set_aden(true));
            while !T::regs().isr().read().adrdy() {
                // ES0233, 2.4.3 ADEN bit cannot be set immediately after the ADC calibration
                // Workaround: keep setting ADEN until ADRDY goes high.
                T::regs().cr().modify(|reg| reg.set_aden(true));
            }
        }
    }

    /// Power down the ADC.
    ///
    /// This stops ADC operation and powers down ADC-specific circuitry.
    /// Later reads will enable the ADC again, but internal measurement paths
    /// such as VREFINT or temperature sensing may need to be re-enabled.
    pub fn power_down(&mut self) {
        self.stop_watchdog();
        Self::teardown_adc();
    }

    #[cfg(not(adc_l0))]
    pub fn enable_vbat(&self) -> Vbat {
        // SMP must be ≥ 56 ADC clock cycles when using HSI14.
        //
        // 6.3.20 Vbat monitoring characteristics
        // ts_vbat ≥ 4μs
        T::regs().ccr().modify(|reg| reg.set_vbaten(true));
        Vbat
    }

    pub fn enable_vref(&self) -> VrefInt {
        // Table 28. Embedded internal reference voltage
        // tstart = 10μs
        T::regs().ccr().modify(|reg| reg.set_vrefen(true));
        blocking_delay_us(10);
        VrefInt
    }

    pub fn enable_temperature(&self) -> Temperature {
        // SMP must be ≥ 56 ADC clock cycles when using HSI14.
        //
        // 6.3.19 Temperature sensor characteristics
        // tstart ≤ 10μs
        // ts_temp ≥ 4μs
        T::regs().ccr().modify(|reg| reg.set_tsen(true));
        blocking_delay_us(10);
        Temperature
    }

    #[cfg(adc_l0)]
    pub fn enable_auto_off(&self) {
        T::regs().cfgr1().modify(|reg| reg.set_autoff(true));
    }

    #[cfg(adc_l0)]
    pub fn disable_auto_off(&self) {
        T::regs().cfgr1().modify(|reg| reg.set_autoff(false));
    }

    pub fn set_resolution(&mut self, resolution: Resolution) {
        T::regs().cfgr1().modify(|reg| reg.set_res(resolution.into()));
    }

    #[cfg(adc_l0)]
    pub fn set_ckmode(&mut self, ckmode: Ckmode) {
        // set ADC clock mode
        T::regs().cfgr2().modify(|reg| reg.set_ckmode(ckmode));
    }

    pub async fn read(&mut self, channel: &mut impl AdcChannel<T>, sample_time: SampleTime) -> u16 {
        let _scoped_wake_guard = <T as crate::rcc::SealedRccPeripheral>::RCC_INFO.wake_guard();
        self.enable();

        let ch_num = channel.channel();
        channel.setup();

        // A.7.5 Single conversion sequence code example - Software trigger
        T::regs().chselr().write(|reg| reg.set_chsel_x(ch_num as usize, true));
        T::regs().smpr().modify(|reg| reg.set_smp(sample_time.into()));

        self.convert().await
    }

    async fn convert(&mut self) -> u16 {
        T::regs().isr().modify(|reg| {
            reg.set_eoc(true);
            reg.set_eosmp(true);
        });

        T::regs().ier().modify(|w| w.set_eocie(true));
        T::regs().cr().modify(|reg| reg.set_adstart(true));

        poll_fn(|cx| {
            T::state().waker.register(cx.waker());

            if T::regs().isr().read().eoc() {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;

        T::regs().dr().read().data()
    }

    fn teardown_adc() {
        // A.7.3 ADC disable code example
        if T::regs().cr().read().adstart() {
            T::regs().cr().modify(|reg| reg.set_adstp(true));
            while T::regs().cr().read().adstp() {}
        }

        if T::regs().cr().read().aden() {
            T::regs().cr().modify(|reg| reg.set_addis(true));
            while T::regs().cr().read().aden() {}
        }

        T::regs().ccr().modify(|reg| {
            reg.set_vrefen(false);
            reg.set_tsen(false);
            #[cfg(not(adc_l0))]
            reg.set_vbaten(false);
        });

        if T::regs().isr().read().adrdy() {
            T::regs().isr().modify(|reg| reg.set_adrdy(true));
        }

        #[cfg(adc_l0)]
        T::regs().cr().modify(|reg| reg.set_advregen(false));
    }
}

impl<'d, T: Instance> Drop for Adc<'d, T> {
    fn drop(&mut self) {
        self.stop_watchdog();
        Self::teardown_adc();
        Self::teardown_awd();

        <T as crate::rcc::SealedRccPeripheral>::RCC_INFO.disable_without_stop();
    }
}

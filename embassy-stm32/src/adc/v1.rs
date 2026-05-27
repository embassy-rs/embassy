use core::marker::PhantomData;

#[cfg(adc_l0)]
use stm32_metapac::adc::vals::Ckmode;
use stm32_metapac::adc::vals::Scandir;

#[cfg(not(adc_l0))]
use crate::adc::Vbat;
use crate::adc::{Adc, AdcRegs, ConversionMode, DefaultInstance, Resolution, SampleTime, Temperature, VrefInt};
use crate::interrupt::typelevel::Interrupt;
use crate::wait::block_for_us;
use crate::{Peri, interrupt, rcc};

mod watchdog_v1;
pub use watchdog_v1::WatchdogChannels;

pub const VDDA_CALIB_MV: u32 = 3300;
pub const VREF_INT: u32 = 1230;

/// Interrupt handler.
pub struct InterruptHandler<T: DefaultInstance> {
    _phantom: PhantomData<T>,
}

impl<T: DefaultInstance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
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
impl super::ConverterFor<super::Vbat> for crate::peripherals::ADC1 {
    const CHANNEL: u8 = 18;
}

impl super::ConverterFor<super::VrefInt> for crate::peripherals::ADC1 {
    const CHANNEL: u8 = 17;
}

#[cfg(adc_l0)]
impl super::ConverterFor<super::Temperature> for crate::peripherals::ADC1 {
    const CHANNEL: u8 = 18;
}

#[cfg(not(adc_l0))]
impl super::ConverterFor<super::Temperature> for crate::peripherals::ADC1 {
    const CHANNEL: u8 = 16;
}

impl AdcRegs for crate::pac::adc::Adc {
    fn data(&self) -> *mut u16 {
        crate::pac::adc::Adc::dr(*self).as_ptr() as *mut u16
    }

    fn enable(&self) {
        #[cfg(adc_l0)]
        if self.cfgr1().read().autoff() {
            // In AUTOFF mode the ADC wakes automatically when conversion starts,
            // so waiting for ADRDY here can stall instead of helping.
            return;
        }

        // A.7.2 ADC enable sequence code example
        while self.cr().read().addis() {}

        if !self.cr().read().aden() {
            if self.isr().read().adrdy() {
                self.isr().modify(|reg| reg.set_adrdy(true));
            }
            self.cr().modify(|reg| reg.set_aden(true));
            while !self.isr().read().adrdy() {
                // ES0233, 2.4.3 ADEN bit cannot be set immediately after the ADC calibration
                // Workaround: keep setting ADEN until ADRDY goes high.
                self.cr().modify(|reg| reg.set_aden(true));
            }
        }
    }

    fn start(&self) {
        self.isr().write(|reg| {
            reg.set_eoc(true);
            reg.set_eosmp(true);
        });

        // Begin ADC conversions
        self.cr().modify(|reg| reg.set_adstart(true));
    }

    fn stop(&self, _disable: bool) {
        // Stop conversion
        while self.cr().read().addis() {}

        // A.7.3 ADC disable code example
        if self.cr().read().adstart() {
            self.cr().modify(|reg| reg.set_adstp(true));
            while self.cr().read().adstp() {}
        }

        self.cfgr1().modify(|w| w.set_cont(false));

        // Disable AWD interrupt
        self.ier().modify(|w| w.set_awdie(false));

        // Clear AWD interrupt flag
        while self.isr().read().awd() {
            self.isr().modify(|regs| {
                regs.set_awd(true);
            })
        }

        self.cfgr1().modify(|w| w.set_awden(false));
    }

    fn wait_done(&self) -> bool {
        self.isr().read().eoc()
    }

    fn configure_dma(&self, conversion_mode: ConversionMode) {
        // Clear all interrupts
        self.isr().modify(|regs| {
            regs.set_eoc(false);
            regs.set_eosmp(true);
            regs.set_ovr(false);
        });

        self.ier().modify(|w| {
            // Enable interrupt for end of conversion
            w.set_eocie(true);
            // Enable interrupt for overrun
            w.set_ovrie(true);
        });

        self.cfgr1().modify(|w| {
            // Disable discontinuous mode
            w.set_discen(false);
            // Enable DMA mode
            w.set_dmaen(!matches!(conversion_mode, ConversionMode::NoDma));
            // DMA requests are issues as long as DMA=1 and data are converted.
            w.set_cont(false);
        });
    }

    fn configure_sequence(&self, sequence: impl ExactSizeIterator<Item = ((u8, bool), SampleTime)>) {
        let mut is_ordered_up = true;
        let mut is_ordered_down = true;

        let mut last_channel: u8 = 0;
        let mut sample_time: Self::SampleTime = SampleTime::Cycles15;

        self.chselr().write(|w| {
            for (i, ((channel, _), _sample_time)) in sequence.enumerate() {
                assert!(
                    sample_time == _sample_time || i == 0,
                    "F0/L0 only supports one sample time for the sequence."
                );

                sample_time = _sample_time;
                is_ordered_up = is_ordered_up && (channel > last_channel || i == 0);
                is_ordered_down = is_ordered_down && (channel < last_channel || i == 0);
                last_channel = channel;

                w.set_chsel_x(channel.into(), true);
            }
        });

        assert!(
            is_ordered_up || is_ordered_down,
            "F0/L0 channels must be passed in order.",
        );

        self.cfgr1().modify(|w| {
            w.set_scandir(if is_ordered_up {
                Scandir::Upward
            } else {
                Scandir::Backward
            })
        });
    }
}

impl<'d, T: DefaultInstance> Adc<'d, T> {
    pub fn new(
        adc: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
    ) -> Self {
        rcc::enable_and_reset::<T>();

        // Delay 1μs when using HSI14 as the ADC clock.
        //
        // Table 57. ADC characteristics
        // tstab = 14 * 1/fadc
        block_for_us(1);

        // set default PCKL/2 on L0s because HSI is disabled in the default clock config
        #[cfg(adc_l0)]
        T::regs().cfgr2().modify(|reg| reg.set_ckmode(Ckmode::PclkDiv2));

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

        T::regs().enable();

        T::Interrupt::unpend();
        unsafe {
            T::Interrupt::enable();
        }

        s
    }

    /// Power down the ADC.
    ///
    /// This stops ADC operation and powers down ADC-specific circuitry.
    /// Later reads will enable the ADC again, but internal measurement paths
    /// such as VREFINT or temperature sensing may need to be re-enabled.
    pub fn power_down(&mut self) {
        T::regs().stop(false);

        let r = T::regs();
        if r.cr().read().aden() {
            r.cr().modify(|reg| reg.set_addis(true));
            while r.cr().read().aden() {}
        }
    }

    #[cfg(not(adc_l0))]
    pub fn enable_vbat(&mut self) -> Vbat {
        // SMP must be ≥ 56 ADC clock cycles when using HSI14.
        //
        // 6.3.20 Vbat monitoring characteristics
        // ts_vbat ≥ 4μs
        T::regs().ccr().modify(|reg| reg.set_vbaten(true));
        Vbat
    }

    pub fn enable_vref(&mut self) -> VrefInt {
        // Table 28. Embedded internal reference voltage
        // tstart = 10μs
        T::regs().ccr().modify(|reg| reg.set_vrefen(true));
        block_for_us(10);
        VrefInt
    }

    pub fn enable_temperature(&mut self) -> Temperature {
        // SMP must be ≥ 56 ADC clock cycles when using HSI14.
        //
        // 6.3.19 Temperature sensor characteristics
        // tstart ≤ 10μs
        // ts_temp ≥ 4μs
        T::regs().ccr().modify(|reg| reg.set_tsen(true));
        block_for_us(10);
        Temperature
    }

    #[cfg(adc_l0)]
    pub fn enable_auto_off(&mut self) {
        T::regs().cfgr1().modify(|reg| reg.set_autoff(true));
    }

    #[cfg(adc_l0)]
    pub fn disable_auto_off(&mut self) {
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
}

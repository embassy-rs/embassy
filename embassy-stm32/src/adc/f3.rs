use core::future::poll_fn;
use core::marker::PhantomData;
use core::task::Poll;

use embassy_hal_internal::into_ref;
use embedded_hal_02::blocking::delay::DelayUs;

use crate::adc::{Adc, AdcPin, Instance, SampleTime};
use crate::interrupt::typelevel::Interrupt;
use crate::time::Hertz;
use crate::{interrupt, Peripheral};

pub const VDDA_CALIB_MV: u32 = 3300;
pub const ADC_MAX: u32 = (1 << 12) - 1;
pub const VREF_INT: u32 = 1230;

/// Interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        if T::regs().isr().read().eoc() {
            T::regs().ier().modify(|w| w.set_eocie(false));
        } else {
            return;
        }

        T::state().waker.wake();
    }
}

pub struct Vref;
impl<T: Instance> AdcPin<T> for Vref {}
impl<T: Instance> super::sealed::AdcPin<T> for Vref {
    fn channel(&self) -> u8 {
        18
    }
}

impl Vref {
    /// The value that vref would be if vdda was at 3300mv
    pub fn value(&self) -> u16 {
        crate::pac::VREFINTCAL.data().read().value()
    }
}

pub struct Temperature;
impl<T: Instance> AdcPin<T> for Temperature {}
impl<T: Instance> super::sealed::AdcPin<T> for Temperature {
    fn channel(&self) -> u8 {
        16
    }
}

impl<'d, T: Instance> Adc<'d, T> {
    pub fn new(
        adc: impl Peripheral<P = T> + 'd,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        delay: &mut impl DelayUs<u32>,
    ) -> Self {
        use crate::pac::adc::vals;

        into_ref!(adc);

        T::enable_and_reset();

        // Enable the adc regulator
        T::regs().cr().modify(|w| w.set_advregen(vals::Advregen::INTERMEDIATE));
        T::regs().cr().modify(|w| w.set_advregen(vals::Advregen::ENABLED));

        // Wait for the regulator to stabilize
        delay.delay_us(10);

        assert!(!T::regs().cr().read().aden());

        // Begin calibration
        T::regs().cr().modify(|w| w.set_adcaldif(false));
        T::regs().cr().modify(|w| w.set_adcal(true));

        while T::regs().cr().read().adcal() {}

        // Wait more than 4 clock cycles after adcal is cleared (RM0364 p. 223)
        delay.delay_us(1 + (6 * 1_000_000 / Self::freq().0));

        // Enable the adc
        T::regs().cr().modify(|w| w.set_aden(true));

        // Wait until the adc is ready
        while !T::regs().isr().read().adrdy() {}

        T::Interrupt::unpend();
        unsafe {
            T::Interrupt::enable();
        }

        Self {
            adc,
            sample_time: Default::default(),
        }
    }

    fn freq() -> Hertz {
        <T as crate::rcc::sealed::RccPeripheral>::frequency()
    }

    pub fn sample_time_for_us(&self, us: u32) -> SampleTime {
        match us * Self::freq().0 / 1_000_000 {
            0..=1 => SampleTime::Cycles1_5,
            2..=4 => SampleTime::Cycles4_5,
            5..=7 => SampleTime::Cycles7_5,
            8..=19 => SampleTime::Cycles19_5,
            20..=61 => SampleTime::Cycles61_5,
            62..=181 => SampleTime::Cycles181_5,
            _ => SampleTime::Cycles601_5,
        }
    }

    pub fn enable_vref(&self, _delay: &mut impl DelayUs<u32>) -> Vref {
        T::common_regs().ccr().modify(|w| w.set_vrefen(true));

        Vref {}
    }

    pub fn enable_temperature(&self) -> Temperature {
        T::common_regs().ccr().modify(|w| w.set_tsen(true));

        Temperature {}
    }

    pub fn set_sample_time(&mut self, sample_time: SampleTime) {
        self.sample_time = sample_time;
    }

    /// Perform a single conversion.
    async fn convert(&mut self) -> u16 {
        T::regs().isr().write(|_| {});
        T::regs().ier().modify(|w| w.set_eocie(true));
        T::regs().cr().modify(|w| w.set_adstart(true));

        poll_fn(|cx| {
            T::state().waker.register(cx.waker());

            if T::regs().isr().read().eoc() {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;

        T::regs().isr().write(|_| {});

        T::regs().dr().read().rdata()
    }

    pub async fn read(&mut self, pin: &mut impl AdcPin<T>) -> u16 {
        Self::set_channel_sample_time(pin.channel(), self.sample_time);

        // Configure the channel to sample
        T::regs().sqr1().write(|w| w.set_sq(0, pin.channel()));
        self.convert().await
    }

    fn set_channel_sample_time(ch: u8, sample_time: SampleTime) {
        let sample_time = sample_time.into();
        if ch <= 9 {
            T::regs().smpr1().modify(|reg| reg.set_smp(ch as _, sample_time));
        } else {
            T::regs().smpr2().modify(|reg| reg.set_smp((ch - 10) as _, sample_time));
        }
    }
}

impl<'d, T: Instance> Drop for Adc<'d, T> {
    fn drop(&mut self) {
        use crate::pac::adc::vals;

        T::regs().cr().modify(|w| w.set_adstp(true));

        while T::regs().cr().read().adstp() {}

        T::regs().cr().modify(|w| w.set_addis(true));

        while T::regs().cr().read().aden() {}

        // Disable the adc regulator
        T::regs().cr().modify(|w| w.set_advregen(vals::Advregen::INTERMEDIATE));
        T::regs().cr().modify(|w| w.set_advregen(vals::Advregen::DISABLED));

        T::disable();
    }
}

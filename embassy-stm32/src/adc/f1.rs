use core::future::poll_fn;
use core::marker::PhantomData;
use core::task::Poll;

use embassy_hal_internal::into_ref;
use embedded_hal_02::blocking::delay::DelayUs;

use crate::adc::{Adc, AdcPin, Instance, SampleTime};
use crate::time::Hertz;
use crate::{interrupt, Peripheral};

pub const VDDA_CALIB_MV: u32 = 3300;
pub const ADC_MAX: u32 = (1 << 12) - 1;
// No calibration data for F103, voltage should be 1.2v
pub const VREF_INT: u32 = 1200;

/// Interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        if T::regs().sr().read().eoc() {
            T::regs().cr1().modify(|w| w.set_eocie(false));
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
        17
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
    pub fn new(adc: impl Peripheral<P = T> + 'd, delay: &mut impl DelayUs<u32>) -> Self {
        into_ref!(adc);
        T::enable_and_reset();
        T::regs().cr2().modify(|reg| reg.set_adon(true));

        // 11.4: Before starting a calibration, the ADC must have been in power-on state (ADON bit = ‘1’)
        // for at least two ADC clock cycles
        delay.delay_us((1_000_000 * 2) / Self::freq().0 + 1);

        // Reset calibration
        T::regs().cr2().modify(|reg| reg.set_rstcal(true));
        while T::regs().cr2().read().rstcal() {
            // spin
        }

        // Calibrate
        T::regs().cr2().modify(|reg| reg.set_cal(true));
        while T::regs().cr2().read().cal() {
            // spin
        }

        // One cycle after calibration
        delay.delay_us((1_000_000) / Self::freq().0 + 1);

        Self {
            adc,
            sample_time: Default::default(),
        }
    }

    fn freq() -> Hertz {
        T::frequency()
    }

    pub fn sample_time_for_us(&self, us: u32) -> SampleTime {
        match us * Self::freq().0 / 1_000_000 {
            0..=1 => SampleTime::Cycles1_5,
            2..=7 => SampleTime::Cycles7_5,
            8..=13 => SampleTime::Cycles13_5,
            14..=28 => SampleTime::Cycles28_5,
            29..=41 => SampleTime::Cycles41_5,
            42..=55 => SampleTime::Cycles55_5,
            56..=71 => SampleTime::Cycles71_5,
            _ => SampleTime::Cycles239_5,
        }
    }

    pub fn enable_vref(&self, _delay: &mut impl DelayUs<u32>) -> Vref {
        T::regs().cr2().modify(|reg| {
            reg.set_tsvrefe(true);
        });
        Vref {}
    }

    pub fn enable_temperature(&self) -> Temperature {
        T::regs().cr2().modify(|reg| {
            reg.set_tsvrefe(true);
        });
        Temperature {}
    }

    pub fn set_sample_time(&mut self, sample_time: SampleTime) {
        self.sample_time = sample_time;
    }

    /// Perform a single conversion.
    async fn convert(&mut self) -> u16 {
        T::regs().cr2().modify(|reg| {
            reg.set_adon(true);
            reg.set_swstart(true);
        });
        T::regs().cr1().modify(|w| w.set_eocie(true));

        poll_fn(|cx| {
            T::state().waker.register(cx.waker());

            if !T::regs().cr2().read().swstart() && T::regs().sr().read().eoc() {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;

        T::regs().dr().read().0 as u16
    }

    pub async fn read(&mut self, pin: &mut impl AdcPin<T>) -> u16 {
        Self::set_channel_sample_time(pin.channel(), self.sample_time);
        T::regs().cr1().modify(|reg| {
            reg.set_scan(false);
            reg.set_discen(false);
        });
        T::regs().sqr1().modify(|reg| reg.set_l(0));

        T::regs().cr2().modify(|reg| {
            reg.set_cont(false);
            reg.set_exttrig(true);
            reg.set_swstart(false);
            reg.set_extsel(7); // SWSTART
        });

        // Configure the channel to sample
        T::regs().sqr3().write(|reg| reg.set_sq(0, pin.channel()));
        self.convert().await
    }

    fn set_channel_sample_time(ch: u8, sample_time: SampleTime) {
        let sample_time = sample_time.into();
        if ch <= 9 {
            T::regs().smpr2().modify(|reg| reg.set_smp(ch as _, sample_time));
        } else {
            T::regs().smpr1().modify(|reg| reg.set_smp((ch - 10) as _, sample_time));
        }
    }
}

impl<'d, T: Instance> Drop for Adc<'d, T> {
    fn drop(&mut self) {
        T::regs().cr2().modify(|reg| reg.set_adon(false));

        T::disable();
    }
}

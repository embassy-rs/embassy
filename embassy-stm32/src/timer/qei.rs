//! # Quadrature Encoder Interface
use crate::{
    gpio::PushPull,
    pac, rcc,
    timer::{CPin, General},
};

pub trait QeiExt: Sized + Instance {
    fn qei(
        self,
        pins: (
            impl Into<<Self as CPin<0>>::Ch<PushPull>>,
            impl Into<<Self as CPin<1>>::Ch<PushPull>>,
        ),
    ) -> Qei<Self>;
}

impl<TIM: Instance> QeiExt for TIM {
    fn qei(
        self,
        pins: (
            impl Into<<Self as CPin<0>>::Ch<PushPull>>,
            impl Into<<Self as CPin<1>>::Ch<PushPull>>,
        ),
    ) -> Qei<Self> {
        Qei::new(self, pins)
    }
}

/// Hardware quadrature encoder interface peripheral
pub struct Qei<TIM: Instance> {
    tim: TIM,
    pins: (
        <TIM as CPin<0>>::Ch<PushPull>,
        <TIM as CPin<1>>::Ch<PushPull>,
    ),
}

impl<TIM: Instance> Qei<TIM> {
    /// Configures a TIM peripheral as a quadrature encoder interface input
    pub fn new(
        mut tim: TIM,
        pins: (
            impl Into<<TIM as CPin<0>>::Ch<PushPull>>,
            impl Into<<TIM as CPin<1>>::Ch<PushPull>>,
        ),
    ) -> Self {
        // Enable and reset clock.
        unsafe {
            TIM::enable_unchecked();
            TIM::reset_unchecked();
        }

        let pins = (pins.0.into(), pins.1.into());
        tim.setup_qei();

        Qei { tim, pins }
    }

    /// Releases the TIM peripheral and QEI pins
    #[allow(clippy::type_complexity)]
    pub fn release(
        self,
    ) -> (
        TIM,
        (
            <TIM as CPin<0>>::Ch<PushPull>,
            <TIM as CPin<1>>::Ch<PushPull>,
        ),
    ) {
        (self.tim, self.pins)
    }

    /// Set current count number
    pub fn set_count(&mut self, value: TIM::Width) -> &mut Self {
        self.tim.write_count(value);
        self
    }
}

impl<TIM: Instance> embedded_hal::Qei for Qei<TIM> {
    type Count = TIM::Width;

    fn count(&self) -> Self::Count {
        self.tim.read_count()
    }

    fn direction(&self) -> embedded_hal::Direction {
        if self.tim.read_direction() {
            embedded_hal::Direction::Upcounting
        } else {
            embedded_hal::Direction::Downcounting
        }
    }
}

pub trait Instance: crate::Sealed + rcc::Enable + rcc::Reset + General + CPin<0> + CPin<1> {
    fn setup_qei(&mut self);

    fn read_direction(&self) -> bool;
}

macro_rules! hal {
    ($TIM:ty) => {
        impl Instance for $TIM {
            fn setup_qei(&mut self) {
                // Configure TxC1 and TxC2 as captures
                #[cfg(not(feature = "gpio-f410"))]
                self.ccmr1_input().write(|w| w.cc1s().ti1().cc2s().ti2());
                #[cfg(feature = "gpio-f410")]
                self.ccmr1_input()
                    .write(|w| unsafe { w.cc1s().bits(0b01).cc2s().bits(0b01) });
                // enable and configure to capture on rising edge
                self.ccer.write(|w| {
                    w.cc1e().set_bit().cc1p().clear_bit();
                    w.cc2e().set_bit().cc2p().clear_bit()
                });
                self.smcr.write(|w| w.sms().encoder_mode_3());
                self.set_auto_reload(<$TIM as General>::Width::MAX as u32)
                    .unwrap();
                self.cr1.write(|w| w.cen().set_bit());
            }

            fn read_direction(&self) -> bool {
                self.cr1.read().dir().bit_is_clear()
            }
        }
    };
}

#[cfg(feature = "tim1")]
hal! { pac::TIM1 }
#[cfg(feature = "tim2")]
hal! { pac::TIM2 }
#[cfg(feature = "tim3")]
hal! { pac::TIM3 }
#[cfg(feature = "tim4")]
hal! { pac::TIM4 }
#[cfg(feature = "tim5")]
hal! { pac::TIM5 }
#[cfg(feature = "tim8")]
hal! { pac::TIM8 }

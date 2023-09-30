use core::marker::PhantomData;

use embassy_hal_internal::{into_ref, PeripheralRef};

use super::*;
use crate::gpio::sealed::AFType;
use crate::gpio::AnyPin;
use crate::Peripheral;

pub enum Direction {
    Upcounting,
    Downcounting,
}

pub struct Ch1;
pub struct Ch2;

pub struct QeiPin<'d, Perip, Channel> {
    _pin: PeripheralRef<'d, AnyPin>,
    phantom: PhantomData<(Perip, Channel)>,
}

macro_rules! channel_impl {
    ($new_chx:ident, $channel:ident, $pin_trait:ident) => {
        impl<'d, Perip: CaptureCompare16bitInstance> QeiPin<'d, Perip, $channel> {
            pub fn $new_chx(pin: impl Peripheral<P = impl $pin_trait<Perip>> + 'd) -> Self {
                into_ref!(pin);
                critical_section::with(|_| {
                    pin.set_low();
                    pin.set_as_af(pin.af_num(), AFType::Input);
                    #[cfg(gpio_v2)]
                    pin.set_speed(crate::gpio::Speed::VeryHigh);
                });
                QeiPin {
                    _pin: pin.map_into(),
                    phantom: PhantomData,
                }
            }
        }
    };
}

channel_impl!(new_ch1, Ch1, Channel1Pin);
channel_impl!(new_ch2, Ch2, Channel2Pin);

pub struct Qei<'d, T> {
    _inner: PeripheralRef<'d, T>,
}

impl<'d, T: CaptureCompare16bitInstance> Qei<'d, T> {
    pub fn new(tim: impl Peripheral<P = T> + 'd, _ch1: QeiPin<'d, T, Ch1>, _ch2: QeiPin<'d, T, Ch2>) -> Self {
        Self::new_inner(tim)
    }

    fn new_inner(tim: impl Peripheral<P = T> + 'd) -> Self {
        into_ref!(tim);

        T::enable();
        <T as crate::rcc::sealed::RccPeripheral>::reset();

        // Configure TxC1 and TxC2 as captures
        T::regs_gp16().ccmr_input(0).modify(|w| {
            w.set_ccs(0, vals::CcmrInputCcs::TI4);
            w.set_ccs(1, vals::CcmrInputCcs::TI4);
        });

        // enable and configure to capture on rising edge
        T::regs_gp16().ccer().modify(|w| {
            w.set_cce(0, true);
            w.set_cce(1, true);

            w.set_ccp(0, false);
            w.set_ccp(1, false);
        });

        T::regs_gp16().smcr().modify(|w| {
            w.set_sms(vals::Sms::ENCODER_MODE_3);
        });

        T::regs_gp16().arr().modify(|w| w.set_arr(u16::MAX));
        T::regs_gp16().cr1().modify(|w| w.set_cen(true));

        Self { _inner: tim }
    }

    pub fn read_direction(&self) -> Direction {
        match T::regs_gp16().cr1().read().dir() {
            vals::Dir::DOWN => Direction::Downcounting,
            vals::Dir::UP => Direction::Upcounting,
        }
    }

    pub fn count(&self) -> u16 {
        T::regs_gp16().cnt().read().cnt()
    }
}
